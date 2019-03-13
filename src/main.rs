use clap::{App, Arg, SubCommand};
use console::style;
use dialoguer::{Confirmation, PasswordInput, Select};
use link_keeper::{
    backend::{AccessToken, AvailableBackend},
    LinkKeeper,
};
use std::io;

const PKG_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
const PKG_NAME: Option<&'static str> = option_env!("CARGO_PKG_NAME");

fn main() -> Result<(), io::Error> {
    let add_command = "add";
    let add_link_command = "link";
    let backend_command = "backend";
    let backend_add_command = "add";

    let mut keeper = LinkKeeper::new();

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
                .about("Store a link at the given backend"),
        )
        .get_matches();

    if let Some(backend_matches) = matches.subcommand_matches(backend_command) {
        if let Some(_backend_add_matches) = backend_matches.subcommand_matches(backend_add_command)
        {
            let available_backends = keeper.get_available_backends();
            let selected_backend = AvailableBackend::from(
                Select::new()
                    .with_prompt("Choose to add one of the following backends")
                    .items(&available_backends)
                    .default(0)
                    .interact()?,
            );

            match selected_backend {
                AvailableBackend::Github(_) | AvailableBackend::GoogleDrive(_) => {
                    let access_token: String = PasswordInput::new()
                        .with_prompt(&format!("Your {} access token", selected_backend))
                        .interact()?;

                    keeper
                        .init_backend(&selected_backend, AccessToken(access_token))
                        .unwrap();
                }
            }
        }
    }

    let _ = matches
        .subcommand_matches(add_command)
        .and_then(|add_matches| {
            add_matches.value_of(add_link_command).map(|new_link| {
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
                        keeper.add(new_link, None).unwrap();
                    }
                } else {
                    keeper.add(new_link, None).unwrap();
                }
            })
        });
    Ok(())
}
