use crate::error::Error;
use crate::issue_tracker::IssueTracker;
use crate::Result;

use config_rs::Config as ConfigRs;

impl From<String> for IssueTracker {
    fn from(issue_tracker: String) -> Self {
        match &issue_tracker.to_lowercase()[..] {
            "github" => IssueTracker::Github,
            "bitbucketcloud" => IssueTracker::BitbucketCloud,
            "bitbucketserver" => IssueTracker::BitbucketServer,
            // "jira" => IssueTracker::Jira,
            _ => unimplemented!(),
        }
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub r#type: String,
    pub url: String,
    pub repository: String,
    pub username: String,
    pub token: String,
}

impl Config {
    fn is_valid(&self) -> Result<()> {
        match &self.r#type.to_lowercase()[..] {
            "github" | "bitbucketcloud" | "bitbucketserver" => Ok(()),
            unknown_type => Err(Error::UnknownConfigType(unknown_type.to_owned())),
        }
    }
}

pub fn load(path: Option<&str>) -> Result<Config> {
    let mut settings = ConfigRs::default();

    if let Some(path) = path {
        settings.merge(config::File::with_name(path))?;
    } else {
        settings.merge(config::File::with_name("fixme_settings"))?;
    }
    settings.merge(config::Environment::with_prefix("FIXME"))?;

    let cfg: Config = settings.try_into().map_err(Error::from)?;
    cfg.is_valid()?;

    Ok(cfg)
}
