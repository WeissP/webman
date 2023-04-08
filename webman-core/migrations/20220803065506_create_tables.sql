-- Add migration script here
CREATE TYPE tag AS ENUM ('normal', 'saved', 'favorite', 'readlater');
CREATE TYPE privacy AS ENUM ('normal', 'private');
CREATE TYPE browser AS ENUM ('chromium', 'chrome', 'safari', 'firefox');

CREATE TABLE providers 
  (
    id SMALLSERIAL NOT NULL PRIMARY KEY,
    provider_name varchar(16) NOT NULL UNIQUE,
    last_import_time timestamp NOT NULL default '1970-01-01 00:00:00'
  );

CREATE TABLE urls 
  (
    id SERIAL NOT NULL PRIMARY KEY,
    url TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    tag tag NOT NULL DEFAULT 'normal',
    privacy privacy NOT NULL DEFAULT 'normal'
  );

CREATE TABLE visits 
  (
    url_id INTEGER NOT NULL,
    provider_id  SMALLINT NOT NULL,
    browser_type  browser NOT NULL,
    visit_count INTEGER NOT NULL,
    last_visit_time  timestamp NOT NULL default '1970-01-01 00:00:00',
    FOREIGN KEY(provider_id) REFERENCES providers(id),
    FOREIGN KEY(url_id) REFERENCES urls(id),
    PRIMARY KEY(url_id, browser_type, provider_id)
  );


