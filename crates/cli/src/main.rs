use clap::{App, Arg, SubCommand};
use console::style;
use dialoguer::{Confirmation, Input, PasswordInput, Select};
use link_keeper::{
    backend::{AccessToken, Backend},
    LinkKeeper,
};
use link_keeper_git_backend::{Git, GitConfig};
use link_keeper_github_backend::{Github, GithubConfig};
use std::env;
use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum AvailableBackend {
    Git,
    Github,
}

impl fmt::Display for AvailableBackend {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            AvailableBackend::Git => fmt.write_fmt(format_args!("Git")),
            AvailableBackend::Github => fmt.write_fmt(format_args!("Github")),
        }
    }
}

impl From<usize> for AvailableBackend {
    fn from(num: usize) -> Self {
        match num {
            0 => AvailableBackend::Git,
            1 => AvailableBackend::Github,
            _ => panic!("Unknown"),
        }
    }
}

impl From<&str> for AvailableBackend {
    fn from(string: &str) -> Self {
        match string {
            "git" => AvailableBackend::Git,
            "github" => AvailableBackend::Github,
            _ => panic!("Unknown"),
        }
    }
}

fn get_old_backends(old_toml_config: &toml::Value) -> Option<Vec<Box<dyn Backend>>> {
    old_toml_config.as_table().and_then(|table| {
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
                    })
                    .collect()
            })
    })
}

const PKG_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
const PKG_NAME: Option<&'static str> = option_env!("CARGO_PKG_NAME");

fn main() -> Result<(), io::Error> {
    let add_command = "add";
    let add_link_command = "link";
    let category_link_command = "category";
    let backend_command = "backend";
    let backend_add_command = "add";

    // TODO: Add configure subcommand
    // It should also be run automatically if no config file is found
    let mut keeper = LinkKeeper::new();

    keeper.register_backends(&get_old_backends).unwrap();

    let matches = App::new(PKG_NAME.unwrap_or_else(|| "link-keeper"))
        .version(PKG_VERSION.unwrap_or_else(|| "0.1.0"))
        .author("Jesper Håkansson. <jesper@jesperh.se>")
        .about("Keep your links stored.")
        .subcommand(
            SubCommand::with_name(backend_command)
                .about("Backend subcommand, add and remove backends")
                .subcommand(
                    SubCommand::with_name(backend_add_command)
                        .about("Add a backend to store links at"),
                ),
        )
        .subcommand(
            SubCommand::with_name(add_command)
                .arg(
                    Arg::with_name(add_link_command)
                        .help("The link to be stored. For example: https://github.com/drager/link-keeper")
                        .required(true),
                )
                .arg(
                    Arg::with_name(category_link_command)
                        .short("c")
                        .long(category_link_command)
                        .takes_value(true)
                        .help("Attach a category to your link"),
                )
                .about("Store a link at the given backend"),
        )
        .get_matches();

    if let Some(backend_matches) = matches.subcommand_matches(backend_command) {
        if let Some(_backend_add_matches) = backend_matches.subcommand_matches(backend_add_command)
        {
            // TODO: Filter away already added backends.
            let available_backends = [AvailableBackend::Git, AvailableBackend::Github];
            let selected_backend = AvailableBackend::from(
                Select::new()
                    .with_prompt("Choose to add one of the following backends")
                    .items(&available_backends)
                    .default(0)
                    .interact()?,
            );

            match selected_backend {
                AvailableBackend::Github => {
                    let access_token: String = PasswordInput::new()
                        .with_prompt(&format!("Your {} access token", selected_backend))
                        .interact()?;

                    keeper
                        .add_backend(Box::new(Github {
                            config: GithubConfig {
                                access_token: AccessToken(access_token),
                            },
                        }))
                        .unwrap();
                }
                AvailableBackend::Git => {
                    let current_dir: String =
                        env::current_dir().unwrap().to_str().unwrap().to_owned();
                    let repository_path: String = Input::new()
                        .with_prompt(&format!(
                            "In what repository should the links be stored? (default: {:?})",
                            current_dir
                        ))
                        .default(current_dir)
                        .show_default(false)
                        .interact()?;

                    let default_file_name = "README.md".to_owned();

                    let file_name: String = Input::new()
                        .with_prompt(&format!(
                            "... and what name would you like the file to have? (default: {:?})",
                            default_file_name
                        ))
                        .default(default_file_name)
                        .show_default(false)
                        .interact()?;

                    let push_on_add: bool = Input::new()
                        .with_prompt(&format!(
                            "Should link keeper automatically push when adding a link? (default: {:?})",
                            true
                        ))
                        .default(true)
                        .show_default(false)
                        .interact()?;

                    keeper
                        .add_backend(Box::new(Git {
                            config: GitConfig {
                                repository_path: PathBuf::from(repository_path),
                                file_name,
                                push_on_add,
                            },
                        }))
                        .unwrap();
                }
            }
        }
    }

    let _ = matches
        .subcommand_matches(add_command)
        .and_then(|add_matches| {
            let category = add_matches.value_of(category_link_command);

            add_matches.value_of(add_link_command).map(|new_link| {
                if keeper.get_activated_backends().is_empty() {
                    eprintln!(
                        "{}{}",
                        style("warning").yellow().bold(),
                        style(format!(": No backend activated...\n")).bold(),
                    );
                }

                // TODO: Real error handling
                if keeper.link_already_exists(new_link).unwrap() {
                    eprintln!(
                        "{}{}",
                        style("warning").yellow().bold(),
                        style(format!(": Link already exists\n")).bold(),
                    );

                    if Confirmation::new()
                        .with_text("Do you want to add it anyway?")
                        .interact()
                        .unwrap()
                    {
                        keeper.add(new_link, category).unwrap();
                    }
                } else {
                    keeper.add(new_link, category).unwrap();
                }
            })
        });
    Ok(())
}
