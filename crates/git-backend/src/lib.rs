use git2::Commit;
use git2::Oid;
use git2::Remote;
use git2::{Repository, Signature};
use link_keeper::{
    backend::{AccessToken, Backend},
    raw_format, Link, LinkKeeper,
};
use markdown;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Git {
    pub config: GitConfig,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct GitConfig {
    pub repository_path: PathBuf,
    // Can be more sophisticated later, i.e choosing between multiple files
    pub file_name: String,
    pub push_on_add: bool,
}

impl fmt::Display for Git {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.write_fmt(format_args!("Git"))
    }
}

impl Git {
    fn write_to_markdown(&self, link: &Link) -> Result<(), failure::Error> {
        self.create_file()?;

        let formatted_data = if self.file_is_empty()? {
            self.format_data(&vec![link])?
        } else {
            let old_contents = self.read_data_from_file()?;
            let old_links = self.to_orginal_format(&old_contents)?;

            let mut old_links = old_links.iter().collect::<Vec<&Link>>();
            old_links.push(link);

            self.format_data(&old_links)?
        };

        self.write_to_file(formatted_data.as_bytes())?;

        Ok(())
    }

    fn format_data(&self, links: &Vec<&Link>) -> Result<String, failure::Error> {
        //dbg!(&links);
        let mut categorized_links: HashMap<Option<String>, Vec<&Link>> = HashMap::new();

        // Categorized so we have "category": vec[Link, Link];
        // Which will be used to render each link under the
        // right category.
        links.iter().for_each(|link| {
            categorized_links
                .entry(link.get_category())
                .or_insert(vec![])
                .push(link);
        });

        dbg!(&categorized_links);

        let formatted = categorized_links
            .iter()
            .fold("".to_owned(), |full_str, current_link| {
                println!("Current link!  {:?}", current_link);

                let category = if let Some(current_link) = current_link.0 {
                    format!("\n## {}\n", current_link)
                } else {
                    "".to_owned()
                };

                let sub_links = current_link
                    .1
                    .iter()
                    .map(|current_link| {
                        let url = current_link.get_url();
                        let url_format = format!("[{}]({})", url, url);

                        format!("{}\n\n", url_format)
                    })
                    .collect::<String>();

                format!("{}{}{}", full_str, category, sub_links)
            });

        Ok(formatted)
    }

    fn to_orginal_format(&self, contents: &str) -> Result<Vec<Link>, failure::Error> {
        let tokens = markdown::tokenize(&contents);

        let mut current_heading = None;

        let links = tokens
            .into_iter()
            .flat_map(|token| {
                match token {
                    markdown::Block::Header(span, _) => {
                        span.into_iter()
                            .flat_map(|first| {
                                match first {
                                    markdown::Span::Text(text) => {
                                        // Ok, we got a category if we see a header
                                        current_heading = Some(text);
                                        vec![]
                                    }
                                    _ => vec![],
                                }
                            })
                            .collect::<Vec<_>>()
                    }
                    markdown::Block::Paragraph(span) => span
                        .into_iter()
                        .filter_map(|line| match line {
                            markdown::Span::Link(link, _, _) => {
                                Some(Link::new(link, current_heading.to_owned()))
                            }
                            _ => None,
                        })
                        .collect::<Vec<_>>(),
                    _ => vec![],
                }
            })
            .collect::<Vec<_>>();

        Ok(links)
    }

    fn file_exists(&self) -> bool {
        let full_path = self.joined();

        full_path.exists()
    }

    fn create_file(&self) -> Result<(), failure::Error> {
        if !self.file_exists() {
            // TODO: Real logging
            dbg!("Creating file for git...");
            fs::File::create(self.joined())?;
        }

        Ok(())
    }

    fn joined(&self) -> PathBuf {
        self.config.repository_path.join(&self.config.file_name)
    }

    fn read_data_from_file(&self) -> Result<String, failure::Error> {
        let full_path = self.joined();
        let mut file = OpenOptions::new().read(true).open(full_path)?;

        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        Ok(buffer)
    }

    fn write_to_file(&self, contents: &[u8]) -> Result<(), failure::Error> {
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

    fn get_remotes<'a>(&'a self, repo: &'a Repository) -> Result<Vec<Remote>, failure::Error> {
        let remotes = repo.remotes()?;

        Ok(remotes
            .iter()
            .filter_map(|remote| remote)
            .map(move |remote| repo.find_remote(remote))
            .filter_map(|remote| remote.ok())
            .collect::<Vec<Remote>>())
    }

    fn get_parents<'a>(&'a self, repo: &'a Repository) -> Vec<Commit> {
        repo.head()
            .ok()
            .and_then(|head| head.target())
            .and_then(|parent| repo.find_commit(parent).ok())
            .into_iter()
            .collect::<Vec<Commit>>()
    }

    fn commit(
        &self,
        repo: &Repository,
        files: Vec<&String>,
        message: &str,
    ) -> Result<Oid, failure::Error> {
        let comitter = Signature::now("Link keeper", "link_keeper@users.noreply.github.com")?;

        let mut index = repo.index()?;

        // Add the file
        index.add_all(
            files.iter(),
            git2::IndexAddOption::DEFAULT,
            Some(&mut |path: &Path, _matched_spec: &[u8]| {
                println!("Add path: {:?}", path.display());
                // Returning zero will add the item to the index.
                0
            }),
        )?;

        // ... then write it!
        index.write()?;

        let parents = self.get_parents(repo);

        // Get the current tree id, this needs to be done
        // after we write the file above, otherwise the file will
        // just be added but not be part of the commit.
        let tree_id = index.write_tree()?;

        Ok(repo.commit(
            Some("HEAD"),
            &comitter,
            &comitter,
            &message,
            &repo.find_tree(tree_id)?,
            parents.iter().collect::<Vec<&Commit>>().as_slice(),
        )?)
    }

    fn git_credentials_callback(
        _user: &str,
        user_from_url: Option<&str>,
        _cred: git2::CredentialType,
    ) -> Result<git2::Cred, git2::Error> {
        git2::Cred::ssh_key_from_agent(
            user_from_url.ok_or(git2::Error::from_str("Failed to get user from url"))?,
        )
    }

    fn git_push_update_callback(
        _referance_name: &str,
        status_message: Option<&str>,
    ) -> Result<(), git2::Error> {
        dbg!(status_message);
        // If status_message is Some, then the push was rejected.
        Ok(())
    }

    fn push(&self, repo: &Repository, remotes: Vec<Remote>) -> Vec<Result<(), git2::Error>> {
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(Self::git_credentials_callback);
        callbacks.push_update_reference(Self::git_push_update_callback);

        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // TODO: Default to master, but read from config
        let branch_ref = repo
            .find_branch("master", git2::BranchType::Local)
            .map(|branch| branch.into_reference());

        // TODO: Can be array if read from config
        let reference_name = match branch_ref {
            Ok(ref reference) => reference.name(),
            Err(_) => None,
        };

        // TODO: Push for every remote? Specify remote?
        let pushes = remotes
            .into_iter()
            .map(|mut remote| {
                println!("Pushing to remote: {:?} {:?}", remote.url(), remote.name());

                let push = remote.push(
                    &[reference_name.unwrap_or_else(|| "")],
                    Some(&mut push_options),
                );

                match push {
                    Ok(_) => push,
                    Err(error) => {
                        // Ok, so we have trouble with SSH. Maybe a passphrase is required?
                        if error.class() == git2::ErrorClass::Ssh {
                            println!("Ssh error!");
                        }
                        Err(error)
                    }
                }
            })
            .collect::<Vec<_>>();

        pushes
    }
}

impl Backend for Git {
    fn add(&self, _link_keeper: &mut LinkKeeper) -> Result<(), failure::Error> {
        dbg!("Adding Git backend");
        Repository::init(&self.config.repository_path)?;

        Ok(())
    }

    fn add_link(&self, link: &Link, link_keeper: &LinkKeeper) -> Result<(), failure::Error> {
        println!("Adding {:?} to {}", link, self);

        self.write_to_markdown(&link)?;
        let repo_path = &self.config.repository_path;

        let repo = Repository::open(&repo_path)?;
        let raw_file = link_keeper.get_raw_file_name();

        // TODO: Should probably be moved.
        raw_format::add_to_raw(repo_path.to_owned(), raw_file.to_owned(), link.to_owned())?;

        let files_to_commit = vec![&self.config.file_name, &raw_file];

        let remotes = self.get_remotes(&repo)?;

        self.commit(
            &repo,
            files_to_commit,
            &format!(
                "Adding link with url: {} {}",
                link.get_url(),
                if link.get_category().is_some() {
                    format!("and category: {}", link.get_category().unwrap())
                } else {
                    "".to_string()
                }
            ),
        )?;

        if self.config.push_on_add && remotes.is_empty() {
            // TODO: Use warn! macro
            println!("Warning! 'Push on add' is set to true but the remotes are empty. This means the push will be ignored...");
        }

        // TODO: Let it be a setting
        let pushes = self.push(&repo, remotes);
        dbg!(&pushes);

        Ok(())
    }

    fn sign_in(&self, access_token: &AccessToken) -> Result<(), ()> {
        dbg!(access_token);
        Ok(())
    }

    fn sign_out(&self, _access_token: &AccessToken) -> Result<(), ()> {
        Ok(())
    }

    fn get_toml_config(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(&self.config)
    }
}
