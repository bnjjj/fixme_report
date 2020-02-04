mod github;

use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use crate::config::Config;
use crate::parser::Annotation;
use crate::Result;

use async_trait::async_trait;

pub use github::GithubClient;

pub enum IssueTracker {
    Github,
    BitbucketCloud,
    BitbucketServer,
    // Jira,
}

pub struct Templates {
    pub todo: Option<PathBuf>,
    pub fixme: Option<PathBuf>,
}

#[derive(Clone, Debug, Default)]
pub struct Issue {
    pub r#ref: Option<String>,
    pub title: String,
    pub details: String,
    pub status: String,
}

impl Issue {
    pub fn new(title: String, details: String) -> Self {
        Self {
            r#ref: None,
            title,
            details,
            status: String::from("open"),
        }
    }

    pub fn from(annotation: Annotation, templates: &Templates) -> Result<Issue> {
        let issue = match annotation {
            Annotation::FixMe(comment) => {
                let title = if comment.details.len() > 60 {
                    format!("Fixme: {}...", &comment.details[..60])
                } else {
                    format!("Fixme: {}", &comment.details)
                };

                let details = if let Some(template_file) = &templates.fixme {
                    let template = read_file(template_file)?;
                    comment.render(&template)?
                } else {
                    format!(
                        r#"+ Filename: {:?}
+ Line: {}
+ Comment: {}"#,
                        comment.file, comment.line, comment.details
                    )
                };

                Issue::new(title, details)
            }
            Annotation::Todo(comment) => {
                let title = if comment.details.len() > 60 {
                    format!("Todo: {}...", &comment.details[..60])
                } else {
                    format!("Todo: {}", &comment.details)
                };

                let details = if let Some(template_file) = &templates.todo {
                    let template = read_file(template_file)?;
                    comment.render(&template)?
                } else {
                    format!(
                        r#"+ Filename: {:?}
+ Line: {}
+ Comment: {}"#,
                        comment.file, comment.line, comment.details
                    )
                };

                Issue::new(title, details)
            }
        };

        Ok(issue)
    }
}

fn read_file(path: &PathBuf) -> Result<String> {
    let mut buffer = String::new();
    let mut file = File::open(path)?;

    file.read_to_string(&mut buffer)?;

    Ok(buffer)
}

#[async_trait]
pub trait IssueTrackerClient {
    async fn create_issue(&self, issue: Issue) -> Result<Issue>;
    fn get_issue_url(&self, issue: &Issue) -> Option<String>;
    // async fn get_issue(&self, issue_ref: String) -> Result<Issue>;
    // fn open_issue(&self, issue: Issue) -> Result<Issue, std::io::Error>;
    // fn delete_issue(&self, issue: Issue) -> Result<(), std::io::Error>;
}

pub fn new(config: Config) -> Box<dyn IssueTrackerClient> {
    Box::new(GithubClient::new(
        config.url,
        config.username,
        config.token,
        config.repository,
    ))
}
