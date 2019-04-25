use git2::Remote;
use git2::{Repository, Signature};
use link_keeper::{
    backend::{AccessToken, Backend},
    Link, LinkKeeper,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Git {
    pub config: GitConfig,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct GitConfig {
    pub repository_path: PathBuf,
    pub push_on_add: bool,
}

impl fmt::Display for Git {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.write_fmt(format_args!("Git"))
    }
}

impl Backend for Git {
    fn add(&self, _link_keeper: &mut LinkKeeper) -> Result<(), failure::Error> {
        dbg!("Adding Git backend");
        //link_keeper.add_backend(self);
        Repository::init(&self.config.repository_path)?;

        Ok(())
    }

    fn add_link(&self, link: &Link) -> Result<(), failure::Error> {
        println!("Adding {:?} to {}", link, self);

        let repo = Repository::open(&self.config.repository_path)?;
        let comitter = Signature::now("Link keeper", "link_keeper@users.noreply.github.com")?;
        let tree_id = repo.index()?.write_tree()?;

        let parents = repo
            .head()
            .ok()
            .and_then(|head| head.target())
            .and_then(|parent| repo.find_commit(parent).ok());

        let parents = parents.iter().collect::<Vec<_>>();

        repo.commit(
            Some("HEAD"),
            &comitter,
            &comitter,
            &format!("Adding {:?}", link),
            &repo.find_tree(tree_id)?,
            parents.as_slice(),
        )?;

        let should_push_on_add = self.config.push_on_add;

        let remotes = repo.remotes()?;

        let remotes = remotes
            .iter()
            .filter_map(|remote| remote)
            .map(|remote| repo.find_remote(remote))
            .filter_map(|remote| remote.ok())
            .collect::<Vec<Remote>>();

        if should_push_on_add && remotes.is_empty() {
            // TODO: Use warn! macro
            println!("Warning! 'Push on add' is set to true but the remotes are empty. This means the push will be ignored...");
        }

        let git_credentials_callback =
            |user: &str, user_from_url: Option<&str>, _cred: git2::CredentialType| {
                println!("USER {:?}", user);
                println!("USER FROM URL {:?}", user_from_url);
                let key_path = std::path::Path::new("");
                git2::Cred::ssh_key(user_from_url.unwrap(), None, key_path, None)
            };

        let git_push_update_callback = |referance_name: &str, status_message: Option<&str>| {
            dbg!(referance_name);
            dbg!(status_message);
            Ok(())
        };

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(git_credentials_callback);
        callbacks.push_update_reference(git_push_update_callback);

        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // TODO: Push for every remote? Specify remote?
        let pushes = remotes
            .into_iter()
            .map(|mut remote| remote.push(&[], Some(&mut push_options)))
            .collect::<Vec<_>>();

        dbg!(&pushes);

        // TODO: Push after commit and let it be a setting

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
