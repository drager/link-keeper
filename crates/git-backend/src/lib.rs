use link_keeper::{
    backend::{AccessToken, Backend},
    Link, LinkKeeper,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Git {
    pub config: GitConfig,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct GitConfig {
    pub repository_path: PathBuf,
}

impl fmt::Display for Git {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.write_fmt(format_args!("Git"))
    }
}

impl Backend for Git {
    fn add(&self, link_keeper: &mut LinkKeeper) -> Result<(), ()> {
        dbg!("Adding Git backend");
        //link_keeper.add_backend(self);
        Ok(())
    }

    fn add_link(&self, link: &Link) -> Result<(), ()> {
        println!("Adding {:?} to {}", link, self);

        Ok(())
    }

    fn sign_in(&self, access_token: &AccessToken) -> Result<(), ()> {
        dbg!(access_token);
        Ok(())
    }

    fn sign_out(&self, access_token: &AccessToken) -> Result<(), ()> {
        Ok(())
    }

    fn get_toml_config(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(&self.config)
    }
}
