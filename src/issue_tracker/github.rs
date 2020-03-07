use async_std::task;
use github_rs::client::{Executor, Github};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Issue, IssueTrackerClient};
use crate::error::Error;
use crate::Result;

pub struct GithubClient {
    url: String,
    username: String,
    token: String,
    repository_slug: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct IssueParameters {
    title: String,
    body: String,
    assignees: Vec<String>,
}

impl GithubClient {
    pub fn new(url: String, username: String, token: String, repository_slug: String) -> Self {
        Self {
            url,
            username,
            token,
            repository_slug,
        }
    }
}

#[async_trait]
impl IssueTrackerClient for GithubClient {
    async fn create_issue(&self, issue: Issue) -> Result<Issue> {
        let token = self.token.clone();
        let repository_slug = self.repository_slug.clone();

        let (_, status_code, json_resp) = task::spawn_blocking(move || {
            let issues_endpoint = format!("repos/{}/issues", repository_slug);
            let client = Github::new(token)?;
            let mut assignees = vec![];
            if let Some(assignee) = issue.assignee {
                assignees.push(assignee);
            }

            let mut issue_params = IssueParameters {
                title: issue.title,
                body: issue.details,
                assignees,
            };
            let resp = client
                .post(&issue_params)
                .custom_endpoint(&issues_endpoint)
                .execute::<Value>();

            if resp.is_err() {
                issue_params.assignees = vec![];
                client
                    .post(&issue_params)
                    .custom_endpoint(&issues_endpoint)
                    .execute::<Value>()
            } else {
                resp
            }
        })
        .await?;

        if status_code.as_u16() >= 400 {
            return Err(Error::HttpError {
                status_code: status_code.as_u16(),
                body: format!("{:#?}", json_resp),
            });
        }

        if json_resp.is_none() {
            return Ok(Issue::default());
        }
        let json_resp = json_resp.unwrap_or_default();
        let mut assignee = None;
        if let Some(assignees) = json_resp["assignees"].as_array() {
            if !assignees.is_empty() {
                assignee = Some(
                    assignees.get(0).unwrap()["login"]
                        .as_str()
                        .unwrap_or_default()
                        .to_owned(),
                );
            }
        }

        Ok(Issue {
            title: json_resp["title"].as_str().unwrap_or_default().to_owned(),
            details: json_resp["body"].as_str().unwrap_or_default().to_owned(),
            status: json_resp["state"].as_str().unwrap_or_default().to_owned(),
            r#ref: Some(format!(
                "{}",
                json_resp["number"].as_i64().unwrap_or_default()
            )),
            assignee,
        })
    }

    fn get_issue_url(&self, issue: &Issue) -> Option<String> {
        issue
            .r#ref
            .as_ref()
            .map(|reference| format!("{}/{}/issues/{}", self.url, self.repository_slug, reference))
    }

    // fn get_issue(&self, issue_ref: String) -> Result<Issue> {
    //     let issues_endpoint = format!("repos/{}/issues/{}", self.repository_slug, issue_ref);
    //     let client = Github::new(&self.token)?;

    //     let (_, status_code, json_resp) = client
    //         .get()
    //         .custom_endpoint(&issues_endpoint)
    //         .execute::<Value>()?;

    //     if status_code.as_u16() >= 400 {
    //         return Err(Error::HttpError {
    //             status_code: status_code.as_u16(),
    //             body: format!("{:#?}", json_resp),
    //         });
    //     }

    //     if json_resp.is_none() {
    //         return Ok(Issue::default());
    //     }
    //     let json_resp = json_resp.unwrap_or_default();

    //     Ok(Issue {
    //         title: json_resp["title"].as_str().unwrap_or_default().to_owned(),
    //         details: json_resp["body"].as_str().unwrap_or_default().to_owned(),
    //         status: json_resp["state"].as_str().unwrap_or_default().to_owned(),
    //         r#ref: Some(format!(
    //             "{}",
    //             json_resp["number"].as_i64().unwrap_or_default()
    //         )),
    //     })
    // }
}
