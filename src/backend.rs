use crate::LinkKeeper;
use chrono;
use serde::Serialize;
use std::fmt;

#[derive(Debug, PartialEq, Serialize)]
pub enum AvailableBackend {
    Github(GithubConfig),
    GoogleDrive(GoogleConfig),
}

impl fmt::Display for AvailableBackend {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            AvailableBackend::Github(_) => fmt.write_fmt(format_args!("Github")),
            AvailableBackend::GoogleDrive(_) => fmt.write_fmt(format_args!("Google drive")),
        }
    }
}

impl From<usize> for AvailableBackend {
    fn from(num: usize) -> Self {
        match num {
            0 => AvailableBackend::Github(GithubConfig::default()),
            1 => AvailableBackend::GoogleDrive(GoogleConfig::default()),
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

#[derive(Debug)]
pub struct Github;

impl fmt::Display for Github {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.write_fmt(format_args!("Github"))
    }
}

#[derive(Debug, Serialize, PartialEq, Default)]
pub struct GithubConfig {
    access_token: AccessToken,
}

#[derive(Debug, Default, Serialize, PartialEq)]
pub struct AccessToken(pub String);

pub trait Backend: fmt::Debug + fmt::Display {
    fn add(&self, link_keeper: &mut LinkKeeper, access_token: AccessToken) -> Result<(), ()>;
    fn sign_in(&self, access_token: &AccessToken) -> Result<(), ()>;
    fn sign_out(&self, access_token: &AccessToken) -> Result<(), ()>;
    //fn send(&self, data: &Data) -> Result<Response, ()>;
    //fn get();
    //fn get_all();
}

impl Backend for Github {
    fn add(&self, link_keeper: &mut LinkKeeper, access_token: AccessToken) -> Result<(), ()> {
        self.sign_in(&access_token).map(|_user| {
            link_keeper.add_backend(AvailableBackend::Github(GithubConfig { access_token }))
        })
    }

    fn sign_in(&self, access_token: &AccessToken) -> Result<(), ()> {
        dbg!(access_token);
        Ok(())
    }

    fn sign_out(&self, access_token: &AccessToken) -> Result<(), ()> {
        Ok(())
    }

    /*fn send(&self, data: &Data) -> Result<Response, ()> {*/
    //Ok(Response)
    /*}*/
}

#[derive(Debug)]
pub struct GoogleDrive;

impl fmt::Display for GoogleDrive {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.write_fmt(format_args!("Google drive"))
    }
}

#[derive(Debug, Serialize, PartialEq, Default)]
pub struct GoogleConfig {
    access_token: AccessToken,
}

impl Backend for GoogleDrive {
    fn add(&self, link_keeper: &mut LinkKeeper, access_token: AccessToken) -> Result<(), ()> {
        self.sign_in(&access_token).map(|_user| {
            link_keeper.add_backend(AvailableBackend::GoogleDrive(GoogleConfig { access_token }))
        })
    }

    fn sign_in(&self, access_token: &AccessToken) -> Result<(), ()> {
        dbg!(access_token);
        Ok(())
    }

    fn sign_out(&self, access_token: &AccessToken) -> Result<(), ()> {
        Ok(())
    }

    /*fn send(&self, data: &Data) -> Result<Response, ()> {*/
    //Ok(Response)
    /*}*/
}
