use crate::backend::{AccessToken, AvailableBackend, Backend, Github, GoogleDrive};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

pub mod backend;

pub struct MyQuery;

#[derive(Debug, Serialize, Deserialize)]
struct Settings {
    config_path: PathBuf,
    config_file_name: String,
}

#[derive(Debug, Serialize)]
pub struct LinkKeeper<'a> {
    activated_backends: Vec<AvailableBackend>,
    settings: Settings,
    store: Store<'a>,
}

impl<'a> LinkKeeper<'a> {
    pub fn new() -> Self {
        let config_path = config_dir().expect("Failed to retrieve configuration directory");

        // TODO: Read from toml config before going further.
        let store = Store::new(
            env::current_dir().unwrap(),
            "link_keeper.json",
            &Format::Json,
        );

        let link_keeper = LinkKeeper {
            activated_backends: vec![],
            settings: Settings {
                config_path: config_path.join("link-keeper"),
                config_file_name: "link-keeper.toml".to_owned(),
            },
            store,
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

    pub fn link_already_exists(&self, link: &str) -> Result<bool, io::Error> {
        if self.store.file_exists() {
            let old_contents = self.store.read_data_from_file()?;

            Ok(self.contains_link(&link, &old_contents))
        } else {
            Ok(false)
        }
    }

    // TODO: Should probably use failure and return Result<(), OwnError> instead
    pub fn add(&self, link: &'a str, category: Option<&'a str>) -> Result<(), io::Error> {
        let new_link = Link { link, category };

        self.store.create_file()?;

        let formatted_data = self.store.format_data(&vec![new_link])?;

        let formatted_data = if self.store.file_is_empty()? {
            formatted_data
        } else {
            let old_contents = self.store.read_data_from_file()?;

            let mut old_contents_as_orginal = self.store.to_orginal_format(&old_contents)?;

            old_contents_as_orginal.append(&mut self.store.to_orginal_format(&formatted_data)?);

            self.store.format_data(&old_contents_as_orginal)?
        };

        self.store.write_to_file(formatted_data.as_bytes())?;

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

    fn contains_link(&self, new_link: &str, old_contents: &str) -> bool {
        old_contents.contains(new_link)
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

#[derive(Debug, Serialize)]
enum Format {
    Json,
    Markdown,
}

#[derive(Debug, Serialize)]
struct Store<'a> {
    path: PathBuf,
    file_name: &'a str,
    format: &'a Format,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Link<'a> {
    link: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<&'a str>,
}

impl<'a> Store<'a> {
    fn new(path: PathBuf, file_name: &'a str, format: &'a Format) -> Self {
        Store {
            path,
            file_name,
            format,
        }
    }

    fn format_data(&self, links: &'a Vec<Link>) -> Result<String, serde_json::error::Error> {
        let formatted = match self.format {
            Format::Json => serde_json::to_string(links)?,
            Format::Markdown => "".to_owned(),
        };

        Ok(formatted)
    }

    fn to_orginal_format(&self, contents: &'a str) -> Result<Vec<Link>, serde_json::error::Error> {
        let formatted = match self.format {
            Format::Json => serde_json::from_str::<_>(contents)?,
            Format::Markdown => unimplemented!(),
        };

        Ok(formatted)
    }

    fn file_exists(&self) -> bool {
        let full_path = self.joined();

        full_path.exists()
    }

    fn create_file(&self) -> Result<(), io::Error> {
        if !self.file_exists() {
            // TODO: Real logging
            dbg!("Creating file...");
            fs::File::create(self.joined())?;
        }

        Ok(())
    }

    fn joined(&self) -> PathBuf {
        self.path.join(&self.file_name)
    }

    fn read_data_from_file(&self) -> Result<String, io::Error> {
        let full_path = self.joined();
        let mut file = OpenOptions::new().read(true).open(full_path)?;

        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        Ok(buffer)
    }

    fn write_to_file(&self, contents: &[u8]) -> Result<(), io::Error> {
        let full_path = self.joined();

        // TODO: Improvement, use .append(true) instead of write.
        let mut file = OpenOptions::new().write(true).open(full_path)?;

        file.write_all(contents)?;

        Ok(())
    }

    fn file_is_empty(&self) -> Result<bool, io::Error> {
        let full_path = self.joined();

        let mut file = File::open(full_path)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        Ok(contents.is_empty())
    }
}
