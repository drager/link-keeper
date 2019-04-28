use crate::{Link, LinkKeeper};
use chrono;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug)]
pub struct Data {
    url: String,
    name: String,
    date_time: chrono::DateTime<chrono::offset::Utc>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct AccessToken(pub String);

pub trait Backend: fmt::Debug + fmt::Display {
    // TODO: Better return types!
    fn add(&self, link_keeper: &mut LinkKeeper) -> Result<(), failure::Error>;
    fn sign_in(&self, access_token: &AccessToken) -> Result<(), ()>;
    fn sign_out(&self, access_token: &AccessToken) -> Result<(), ()>;
    fn add_link(&self, link: &Link, link_keeper: &LinkKeeper) -> Result<(), failure::Error>;
    fn get_toml_config(&self) -> Result<String, toml::ser::Error>;
    //fn send(&self, data: &Data) -> Result<Response, ()>;
    //fn get();
    //fn get_all();
}
