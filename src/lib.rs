use crate::backend::{AvailableBackend, Backend, Git, GitConfig, Github, GithubConfig};
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

impl Default for Settings {
    fn default() -> Self {
        let config_path = config_dir().expect("Failed to retrieve configuration directory");

        Self {
            config_path: config_path.join("link-keeper"),
            config_file_name: "link-keeper.toml".to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct LinkKeeper<'a> {
    activated_backends: Vec<Box<dyn Backend>>,
    settings: Settings,
    store: Store<'a>,
}

fn get_old_backends(
    old_toml_config: &Result<toml::Value, failure::Error>,
) -> Option<Vec<Box<dyn Backend>>> {
    match old_toml_config {
        Ok(ref config) => config.as_table().and_then(|table| {
            table
                .get("backends")
                .and_then(|backends| backends.as_table())
                .map(|backends| {
                    backends
                        .into_iter()
                        .filter_map(|(key, value)| match AvailableBackend::from(key.as_str()) {
                            AvailableBackend::Git => Some(Box::new(Git {
                                config: value.clone().try_into::<GitConfig>().unwrap(),
                            })
                                as Box<dyn Backend>),
                            AvailableBackend::Github => Some(Box::new(Github {
                                config: value.clone().try_into::<GithubConfig>().unwrap(),
                            })
                                as Box<dyn Backend>),

                            _ => None,
                        })
                        .collect()
                })
        }),
        Err(_) => None,
    }
}

impl<'a> LinkKeeper<'a> {
    pub fn new() -> Self {
        let default_settings = Settings::default();

        let old_toml_config = Self::get_old_toml_config(
            &default_settings
                .config_path
                .join(&default_settings.config_file_name),
        );

        let old_backends = get_old_backends(&old_toml_config);

        dbg!(&old_backends);

        // TODO: Read from toml config before going further.
        let store = Store::new(
            env::current_dir().unwrap(),
            "link_keeper.json",
            &Format::Json,
        );

        let link_keeper = LinkKeeper {
            activated_backends: match old_backends {
                Some(old_backends) => old_backends,
                None => vec![],
            },
            settings: default_settings,
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

        println!("Adding link to backends...");

        dbg!(&self.activated_backends);

        // TODO: Fail on every?
        // Option to abort on fail for any?
        let errors = self
            .activated_backends
            .iter()
            .map(|backend| backend.add_link(&new_link))
            .filter(|result| result.is_err())
            .collect::<Vec<Result<(), ()>>>();

        dbg!(errors);

        // Always add to raw!
        self.add_to_raw(new_link)?;

        Ok(())
    }

    fn add_to_raw(&self, new_link: Link) -> Result<(), io::Error> {
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

    pub fn get_available_backends(&self) -> Vec<String> {
        vec![
            "Git".to_owned(),
            "Github".to_owned(),
            "GoogleDrive".to_owned(),
        ]
    }

    /// Get all the activated backends
    pub fn get_activated_backends(&self) -> Vec<&Box<dyn Backend>> {
        self.activated_backends
            .iter()
            .collect::<Vec<&Box<dyn Backend>>>()
    }

    pub fn add_backend(&mut self, backend: Box<dyn Backend>) -> Result<(), failure::Error> {
        self.activated_backends.push(backend);

        self.create_toml_string()
            .map(|toml_string| self.write_to_config(&toml_string))??;

        Ok(())
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

    fn get_old_toml_config(path: &Path) -> Result<toml::Value, failure::Error> {
        let mut file = File::open(path)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(toml::from_str::<toml::Value>(&contents)?)
    }

    fn create_toml_string(&self) -> Result<String, toml::ser::Error> {
        let merge_tomls = |config: Result<String, toml::ser::Error>, backend: &Box<dyn Backend>| {
            config
                .map(|toml_string| {
                    format!(
                        "\n[backends.{}]\n{}",
                        backend.to_string().to_lowercase().replace(" ", "_"),
                        toml_string
                    )
                })
                .unwrap_or_else(|_| "".to_owned())
        };
        dbg!(&self.activated_backends);

        let backend_config_string = self
            .activated_backends
            .iter()
            .map(|backend| merge_tomls(backend.get_toml_config(), backend))
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
pub struct Link<'a> {
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
