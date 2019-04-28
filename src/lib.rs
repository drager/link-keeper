use crate::backend::Backend;
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

pub mod backend;
pub mod file_handling;
pub mod raw_format;

#[derive(Debug, Serialize, Deserialize)]
struct Settings {
    config_path: PathBuf,
    config_file_name: String,
    raw_file_name: String,
}

impl Default for Settings {
    fn default() -> Self {
        let config_path = config_dir().expect("Failed to retrieve configuration directory");

        Self {
            config_path: config_path.join("link-keeper"),
            config_file_name: "link-keeper.toml".to_owned(),
            raw_file_name: "link_keeper.json".to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct LinkKeeper {
    activated_backends: Vec<Box<dyn Backend>>,
    settings: Settings,
}

impl LinkKeeper {
    pub fn register_backends<F>(&mut self, get_backends: F) -> Result<(), failure::Error>
    where
        F: FnOnce(&toml::Value) -> Option<Vec<Box<dyn Backend>>>,
    {
        let default_settings = Settings::default();

        let old_toml_config = Self::get_old_toml_config(
            &default_settings
                .config_path
                .join(&default_settings.config_file_name),
        )?;

        let backends = get_backends(&old_toml_config);

        self.activated_backends
            .append(&mut backends.unwrap_or_else(|| vec![]));

        Ok(())
    }

    pub fn new() -> Self {
        let default_settings = Settings::default();

        let old_toml_config = Self::get_old_toml_config(
            &default_settings
                .config_path
                .join(&default_settings.config_file_name),
        );

        let old_settings = old_toml_config
            .and_then(|config| config.try_into::<Settings>().map_err(|err| err.into()));

        dbg!(&old_settings);

        let link_keeper = LinkKeeper {
            activated_backends: vec![],
            settings: default_settings,
        };

        let full_config_path = link_keeper.full_config_path();

        if !Path::new(&link_keeper.settings.config_path).exists() {
            link_keeper.create_config_directory().expect(&format!(
                "Failed to create link keeper configuration directory at: {:?}",
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

    pub fn get_raw_file_name(&self) -> &String {
        &self.settings.raw_file_name
    }

    pub fn link_already_exists(&self, link: &str) -> Result<bool, io::Error> {
        Ok(false)
        /*if self.store.file_exists() {*/
        //let old_contents = self.store.read_data_from_file()?;

        //Ok(self.contains_link(&link, &old_contents))
        //} else {
        //Ok(false)
        /*}*/
    }

    // TODO: Should probably use failure and return Result<(), OwnError> instead
    pub fn add<'a>(&self, link: &'a str, category: Option<&'a str>) -> Result<(), failure::Error> {
        let new_link = Link {
            url: link.to_owned(),
            category: category.map(|cat| cat.to_owned()),
        };

        println!("Adding link to backends...");

        dbg!(&self.activated_backends);

        // TODO: Fail on every?
        // Option to abort on fail for any?
        let errors = self
            .activated_backends
            .iter()
            .map(|backend| backend.add_link(&new_link, &self))
            .filter(|result| result.is_err())
            .collect::<Vec<Result<(), failure::Error>>>();

        dbg!(errors);

        Ok(())
    }

    /// Get all the activated backends
    pub fn get_activated_backends(&self) -> Vec<&Box<dyn Backend>> {
        self.activated_backends
            .iter()
            .collect::<Vec<&Box<dyn Backend>>>()
    }

    pub fn add_backend(&mut self, backend: Box<dyn Backend>) -> Result<(), failure::Error> {
        backend.add(self)?;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
}

impl Link {
    pub fn new(url: String, category: Option<String>) -> Self {
        Link { url, category }
    }

    pub fn get_url(&self) -> String {
        self.url.to_owned()
    }

    pub fn get_category(&self) -> Option<String> {
        self.category.to_owned()
    }
}
