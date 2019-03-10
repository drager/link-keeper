use chrono;
use dirs::config_dir;
use graphql_client::{GraphQLQuery, Response};
use serde::Serialize;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub struct MyQuery;

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

#[derive(Debug, Serialize)]
struct Settings {
    config_path: PathBuf,
    config_file_name: String,
}

#[derive(Debug, Serialize)]
pub struct LinkKeeper {
    activated_backends: Vec<AvailableBackend>,
    settings: Settings,
}

impl LinkKeeper {
    pub fn new() -> Self {
        let config_path = config_dir().expect("Failed to retrieve configuration directory");

        let link_keeper = LinkKeeper {
            activated_backends: vec![],
            settings: Settings {
                config_path: config_path.join("link-keeper"),
                config_file_name: "link-keeper.toml".to_owned(),
            },
        };

        let full_config_path = link_keeper.full_config_path();

        if !Path::new(&link_keeper.settings.config_path).exists() {
            link_keeper.create_config_directory().expect(&format!(
                "Failed to create link keeper confugration directory at: {:?}",
                link_keeper.settings.config_path
            ));
        }

        if !full_config_path.exists() {
            link_keeper.create_config_file().expect(&format!(
                "Failed to create configuration file at: {:?}",
                full_config_path
            ))
        }

        link_keeper
    }

    /// Convience function to get the full path to the configuration file
    fn full_config_path(&self) -> PathBuf {
        self.settings
            .config_path
            .join(&self.settings.config_file_name)
    }

    fn create_config_directory(&self) -> io::Result<()> {
        fs::create_dir(&self.settings.config_path)?;

        Ok(())
    }

    fn create_config_file(&self) -> io::Result<()> {
        File::create(
            &self
                .settings
                .config_path
                .join(&self.settings.config_file_name),
        )?;

        Ok(())
    }

    pub fn get_available_backends(&self) -> Vec<Box<Backend>> {
        vec![Box::new(Github), Box::new(GoogleDrive)]
    }

    pub fn init_backend(
        &mut self,
        backend: &AvailableBackend,
        access_token: AccessToken,
    ) -> Result<(), ()> {
        match backend {
            AvailableBackend::Github(_) => Github.add(self, access_token),
            AvailableBackend::GoogleDrive(_) => GoogleDrive.add(self, access_token),
        }
    }

    fn add_backend(&mut self, backend: AvailableBackend) {
        if !self.activated_backends.contains(&backend) {
            self.activated_backends.push(backend);
            // TODO: Should use `?` here and use `failure` for errors.
            self.create_toml_string()
                .map(|toml_string| self.write_to_config(&toml_string));
        }
    }

    fn create_toml_string(&self) -> Result<String, toml::ser::Error> {
        let merge_tomls = |config: Result<String, toml::ser::Error>, backend: &AvailableBackend| {
            config
                .map(|toml_string| {
                    format!(
                        "\n[{}]\n{}",
                        backend.to_string().to_lowercase().replace(" ", "_"),
                        toml_string
                    )
                })
                .unwrap_or_else(|_| "".to_owned())
        };

        let backend_config_string = self
            .activated_backends
            .iter()
            .map(|backend| match backend {
                AvailableBackend::Github(config) => merge_tomls(toml::to_string(config), backend),
                AvailableBackend::GoogleDrive(config) => {
                    merge_tomls(toml::to_string(config), backend)
                }
            })
            .fold("".to_owned(), |prev, curr| format!("{}{}", prev, curr));

        toml::to_string(&self.settings)
            .map(|toml_string| format!("{}{}", toml_string, backend_config_string))
    }

    fn write_to_config(&self, toml_string: &str) -> Result<(), io::Error> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.full_config_path())?;

        file.write_all(toml_string.as_bytes())?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Data {
    url: String,
    name: String,
    date_time: chrono::DateTime<chrono::offset::Utc>,
}

#[derive(Debug)]
struct Github;

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
struct GoogleDrive;

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
