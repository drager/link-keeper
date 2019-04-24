use link_keeper::{
    backend::{AccessToken, Backend},
    Link, LinkKeeper,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct Github {
    pub config: GithubConfig,
}

impl fmt::Display for Github {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.write_fmt(format_args!("Github"))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct GithubConfig {
    pub access_token: AccessToken,
}

impl Backend for Github {
    fn add(&self, _link_keeper: &mut LinkKeeper) -> Result<(), failure::Error> {
        Ok(())
        //self.sign_in(&self.config.access_token)

        //.map(move |_user| link_keeper.add_backend(AvailableBackend::Github(self.config)))
    }

    //fn get(&self) -> Self {
    //Github {config: GithubConfig {access_token: AccessToken("")}
    /*}*/

    fn sign_in(&self, access_token: &AccessToken) -> Result<(), ()> {
        dbg!(access_token);
        Ok(())
    }

    fn sign_out(&self, _access_token: &AccessToken) -> Result<(), ()> {
        Ok(())
    }

    fn add_link(&self, link: &Link) -> Result<(), failure::Error> {
        println!("Adding {:?} to {}", link, self);
        Ok(())
    }

    fn get_toml_config(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(&self.config)
    }
}
