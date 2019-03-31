use crate::{Link, LinkKeeper};
use chrono;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Serialize)]
pub enum AvailableBackend {
    Git,
    Github,
    GoogleDrive,
}

impl fmt::Display for AvailableBackend {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            AvailableBackend::Git => fmt.write_fmt(format_args!("Git")),
            AvailableBackend::Github => fmt.write_fmt(format_args!("Github")),
            AvailableBackend::GoogleDrive => fmt.write_fmt(format_args!("Google drive")),
        }
    }
}

impl From<usize> for AvailableBackend {
    fn from(num: usize) -> Self {
        match num {
            0 => AvailableBackend::Git,
            1 => AvailableBackend::Github,
            2 => AvailableBackend::GoogleDrive,
            _ => panic!("Unknown"),
        }
    }
}

impl From<&str> for AvailableBackend {
    fn from(string: &str) -> Self {
        match string {
            "git" => AvailableBackend::Git,
            "github" => AvailableBackend::Github,
            "google_drive" => AvailableBackend::GoogleDrive,
            _ => panic!("Unknown"),
        }
    }
}

#[derive(Debug)]
pub struct Data {
    url: String,
    name: String,
    date_time: chrono::DateTime<chrono::offset::Utc>,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct AccessToken(pub String);

pub trait Backend: fmt::Debug + fmt::Display {
    // TODO: Better return types!
    fn add(&self, link_keeper: &mut LinkKeeper) -> Result<(), ()>;
    fn sign_in(&self, access_token: &AccessToken) -> Result<(), ()>;
    fn sign_out(&self, access_token: &AccessToken) -> Result<(), ()>;
    fn add_link(&self, link: &Link) -> Result<(), ()>;
    fn get_toml_config(&self) -> Result<String, toml::ser::Error>;
    //fn send(&self, data: &Data) -> Result<Response, ()>;
    //fn get();
    //fn get_all();
}

// TODO: Each backend should be put in it's own crate.
impl Backend for Github {
    fn add(&self, link_keeper: &mut LinkKeeper) -> Result<(), ()> {
        self.sign_in(&self.config.access_token)
        //.map(move |_user| link_keeper.add_backend(AvailableBackend::Github(self.config)))
    }

    fn sign_in(&self, access_token: &AccessToken) -> Result<(), ()> {
        dbg!(access_token);
        Ok(())
    }

    fn sign_out(&self, access_token: &AccessToken) -> Result<(), ()> {
        Ok(())
    }

    fn add_link(&self, link: &Link) -> Result<(), ()> {
        println!("Adding {:?} to {}", link, self);
        Ok(())
    }

    fn get_toml_config(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(&self.config)
    }
}

#[derive(Debug, Serialize)]
pub struct GoogleDrive {
    pub config: GoogleConfig,
}

impl fmt::Display for GoogleDrive {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.write_fmt(format_args!("Google drive"))
    }
}

#[derive(Debug, Serialize, PartialEq, Default)]
pub struct GoogleConfig {
    pub access_token: AccessToken,
}

/*impl Backend for GoogleDrive {*/
//fn add(&self, link_keeper: &mut LinkKeeper) -> Result<(), ()> {
//self.sign_in(&self.config.access_token)
////.map(|_user| link_keeper.add_backend(AvailableBackend::GoogleDrive(self.config)))
//}

//fn add_link(&self, link: &Link) -> Result<(), ()> {
//Ok(())
//}

//fn sign_in(&self, access_token: &AccessToken) -> Result<(), ()> {
//dbg!(access_token);
//Ok(())
//}

//fn sign_out(&self, access_token: &AccessToken) -> Result<(), ()> {
//Ok(())
//}

//fn get_config(&self) -> Box<std::any::Any> {
//Box::new(self.config)
//}

//fn send(&self, data: &Data) -> Result<Response, ()> {
////Ok(Response)
//}
//}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct GitConfig {
    pub repository_path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Git {
    pub config: GitConfig,
}

impl fmt::Display for Git {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.write_fmt(format_args!("Git"))
    }
}

// TODO: Each backend should be put in it's own crate.
impl Backend for Git {
    fn add(&self, link_keeper: &mut LinkKeeper) -> Result<(), ()> {
        dbg!("Adding Git backend");
        //link_keeper.add_backend(self);
        Ok(())
    }

    fn add_link(&self, link: &Link) -> Result<(), ()> {
        println!("Adding {:?} to {}", link, self);
        Err(())
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
