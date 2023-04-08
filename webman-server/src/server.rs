use super::config::{HOST, SYNC_NODES};
use chrono::NaiveDateTime;
use cookie::time::{Duration, OffsetDateTime};
use once_cell::sync::Lazy;
use rocket::{
    fairing::{self, AdHoc},
    fs::FileServer,
    http::{Cookie, CookieJar, Status},
    request::{FromRequest, Outcome, Request},
    response::status,
    serde::{json::Json, msgpack::MsgPack},
    Build, Rocket,
};
use rocket_db_pools::{sqlx, Connection, Database};
use std::sync::Mutex;
use webman_core::{
    browser::Browser,
    config,
    db::prelude as db,
    node::{self, Provider},
    resp::*,
    url::{self, tag, Filter, Url, UrlResult},
    Client,
};
type Result<T> = std::result::Result<T, rocket::response::Debug<anyhow::Error>>;

static TAG_LOG: Lazy<Mutex<tag::History>> = Lazy::new(|| Mutex::new(tag::History::default()));

#[derive(Database)]
#[database("webman")]

pub struct Pool(pub sqlx::PgPool);

/// there are two ways to pass the ApiKey request guard. The first way is to add api key to header "x-api-key", the second way is add the api key in the cookie "logged"
pub struct ApiKey<'r>(&'r str);

#[derive(Debug)]
pub enum ApiKeyError {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey<'r> {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // we first check if api key is set in the header, if not, cookie will be checked
        match req.headers().get_one("x-api-key") {
            Some(key) if key == config().api_key => Outcome::Success(ApiKey(key)),
            Some(_) => Outcome::Failure((Status::BadRequest, ApiKeyError::Invalid)),
            None => match req.cookies().get_private("logged") {
                Some(c) if c.value() == config().api_key => Outcome::Success(ApiKey("logged")),
                Some(_) => Outcome::Failure((Status::BadRequest, ApiKeyError::Invalid)),
                None => Outcome::Failure((Status::BadRequest, ApiKeyError::Missing)),
            },
        }
    }
}

/// to check whether the request guard ApiKey can be passed.
#[get("/ping")]
async fn auth_ping(_key: ApiKey<'_>) -> status::Accepted<&'static str> {
    status::Accepted(Some("login succeed"))
}

#[post("/login", data = "<auth>")]
async fn login(auth: Json<Auth>, cookies: &CookieJar<'_>) -> Result<&'static str> {
    if auth.api_key == config().api_key {
        let mut c = Cookie::new("logged", auth.api_key.to_owned());

        let mut exp = OffsetDateTime::now_utc();
        exp += Duration::weeks(12);
        c.set_expires(exp);

        cookies.add_private(c);
    } else {
        Err(anyhow::anyhow!("the api key is incorrect!"))?;
    }
    Ok("login succeed")
}

#[get("/provider/last_import_time?<name>")]
async fn last_import_time(
    mut pool: Connection<Pool>,
    name: &str,
    _key: ApiKey<'_>,
) -> Result<Json<NaiveDateTime>> {
    let time = db::last_import_time(&mut pool, name).await?;
    Ok(Json(time))
}

#[get("/urls/search")]
async fn search_by_query_without_key() -> Status {
    Status::Unauthorized
}

#[get("/urls/search?<query>&<limit>")]
async fn search_by_query(
    mut pool: Connection<Pool>,
    query: String,
    limit: i64,
    _key: ApiKey<'_>,
) -> Result<Json<Vec<UrlResult>>> {
    let f = Filter {
        limit,
        ..Filter::parse(query)
    };
    let res = db::fuzzy_search(&mut pool, f).await?;
    Ok(Json(res))
}

#[post("/urls/filter", data = "<filter>")]
async fn get_urls(
    mut pool: Connection<Pool>,
    filter: Json<url::Filter>,
    _key: ApiKey<'_>,
) -> Result<MsgPack<Vec<Url>>> {
    let mut filter = filter.into_inner();
    filter.init();
    let urls = db::get_urls(&mut pool, filter).await?;
    Ok(MsgPack(urls))
}

#[post("/urls/insert", data = "<insert>")]
async fn insert_urls(
    mut pool: Connection<Pool>,
    insert: MsgPack<UrlInsert>,
    _key: ApiKey<'_>,
) -> Result<Status> {
    let UrlInsert {
        name,
        browser,
        urls,
        last_import_time,
    } = insert.into_inner();
    info!("start to insert urls with length {}", urls.len());
    db::insert_urls(&mut pool, &name, browser, urls, last_import_time).await?;
    info!("urls successfull inserted!");
    Ok(Status::Ok)
}

#[post("/urls/insert_fake", data = "<insert>")]
async fn insert_fake_url(
    mut pool: Connection<Pool>,
    insert: Json<FakeUrl>,
    _key: ApiKey<'_>,
) -> Result<Status> {
    let UrlInsert {
        name,
        browser,
        urls,
        last_import_time,
    } = insert.into_inner().into();
    info!("start to insert fake url: {}", &urls[0].url);
    db::insert_urls(&mut pool, &name, browser, urls, last_import_time).await?;
    info!("fake url successfully inserted!");
    Ok(Status::Ok)
}

#[post("/urls/insert_json", data = "<insert>")]
async fn insert_urls_json(
    mut pool: Connection<Pool>,
    insert: Json<UrlInsert>,
    _key: ApiKey<'_>,
) -> Result<Status> {
    let UrlInsert {
        name,
        browser,
        urls,
        last_import_time,
    } = insert.into_inner();
    info!("start to insert urls with length {}", urls.len());
    db::insert_urls(&mut pool, &name, browser, urls, last_import_time).await?;
    info!("urls successfully inserted!");
    Ok(Status::Ok)
}

#[post("/urls/tag", data = "<tag_setter>")]
async fn set_tag(
    mut pool: Connection<Pool>,
    tag_setter: Json<UrlTagSetter>,
    _key: ApiKey<'_>,
) -> Result<Status> {
    info!("start to set tag as {:?}", &tag_setter);
    let UrlTagSetter { tag, urls } = tag_setter.into_inner();
    TAG_LOG.lock().unwrap().batch_insert(urls.clone(), tag);
    db::set_tag(&mut pool, urls, tag).await?;
    Ok(Status::Ok)
}

#[get("/memory/tag_log")]
async fn get_tag_log<'a>() -> Json<tag::History> {
    let log: &tag::History = &TAG_LOG.lock().unwrap();
    Json(log.to_owned())
}

#[post("/memory/tag_log", data = "<log>")]
async fn update_tag_log(mut pool: Connection<Pool>, log: Json<tag::History>) -> Result<Status> {
    let new_log = log.into_inner();
    let new_tags = {
        let mut cur_log = TAG_LOG.lock().unwrap();
        let new = cur_log.merge(&new_log);
        info!("tag_log is updated as {:?}", &cur_log);
        new
    };
    for (url, tag) in new_tags {
        db::set_tag(&mut pool, [url.to_string()], tag).await?;
    }
    Ok(Status::Ok)
}

#[get("/urls/tags")]
async fn get_all_tags(mut pool: Connection<Pool>, _key: ApiKey<'_>) -> Result<Json<url::Tags>> {
    let tags = db::get_all_tags(&mut pool).await?;
    Ok(Json(tags))
}

#[get("/provider/info")]
async fn providers(mut pool: Connection<Pool>, _key: ApiKey<'_>) -> Result<Json<Vec<Provider>>> {
    let p = db::all_providers(&mut pool).await?;
    Ok(Json(p))
}

#[get("/provider/browsers?<name>")]
async fn browsers(
    mut pool: Connection<Pool>,
    name: &str,
    _key: ApiKey<'_>,
) -> Result<Json<Vec<Browser>>> {
    let bs: Vec<Browser> = db::all_browsers(&mut pool, name).await?;
    Ok(Json(bs))
}

#[get("/sync?<remote>")]
async fn sync(remote: String, _key: ApiKey<'_>) -> Result<Status> {
    let c = Client::with_apikey(&config().api_key);
    let remote = node::Name::new(remote);
    c.sync_all(HOST.get().unwrap(), &remote).await?;
    Ok(Status::Ok)
}

#[get("/sync")]
async fn sync_all_nodes(_key: ApiKey<'_>) -> Result<Status> {
    let c = Client::with_apikey(&config().api_key);
    for node in SYNC_NODES.get().unwrap() {
        c.sync_all(HOST.get().unwrap(), &node.name).await?;
    }
    Ok(Status::Ok)
}

#[catch(422)]
fn error422() -> &'static str {
    "Error Data guard happened, please check the server log files"
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    if let Some(pool) = Pool::fetch(&rocket) {
        db::migrate(pool).await.unwrap();
        Ok(rocket)
    } else {
        Err(rocket)
    }
}

pub fn launch(fig: figment::Figment) -> Rocket<Build> {
    let react_location: String = fig
        .extract_inner("react_location")
        .expect("could not find react_location");
    rocket::custom(fig)
        .attach(Pool::init())
        .attach(AdHoc::try_on_ignite("DB migration", run_migrations))
        .mount(
            "/api",
            routes![
                browsers,
                last_import_time,
                get_urls,
                insert_urls,
                insert_urls_json,
                insert_fake_url,
                providers,
                set_tag,
                get_all_tags,
                get_tag_log,
                update_tag_log,
                search_by_query,
                sync,
                sync_all_nodes,
                search_by_query_without_key
            ],
        )
        .mount("/auth", routes![auth_ping, login,])
        .mount("/", FileServer::from(react_location))
        .register("/", catchers![error422])
}
