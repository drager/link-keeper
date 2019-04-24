use git2::{Repository, Signature, Tree};
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
}

impl fmt::Display for Git {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.write_fmt(format_args!("Git"))
    }
}

impl Backend for Git {
    fn add(&self, link_keeper: &mut LinkKeeper) -> Result<(), failure::Error> {
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

        // TODO: Push after commit and let it be a setting

        Ok(())
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
