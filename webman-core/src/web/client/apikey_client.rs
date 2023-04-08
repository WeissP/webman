use super::{client_trait::ClientTrait, Client};
use crate::node::Node;

use async_trait::async_trait;

use reqwest::RequestBuilder;

pub struct ApiKeyClient {
    c: reqwest::Client,
    api_key: String,
}

impl Client {
    pub fn with_apikey(api_key: &str) -> Self {
        Client(Box::new(ApiKeyClient {
            c: reqwest::Client::new(),
            api_key: api_key.to_owned(),
        }))
    }
}

#[async_trait]
impl ClientTrait for ApiKeyClient {
    fn post(&self, node: &Node, end_point: &str) -> RequestBuilder {
        self.c
            .post(node.api_url(end_point))
            .header("x-api-key", &self.api_key)
    }
    fn get(&self, node: &Node, end_point: &str) -> RequestBuilder {
        self.c
            .get(node.api_url(end_point))
            .header("x-api-key", &self.api_key)
    }
}
