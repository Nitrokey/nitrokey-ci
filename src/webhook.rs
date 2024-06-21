use std::{fs, path::PathBuf};

use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;

#[derive(PartialEq, Debug)]
pub struct Repository {
    pub owner: String,
    pub name: String,
}

pub struct Webhook {
    pub action: String,
    pub comment: String,
    pub comment_id: String,
    pub author: String,
    pub author_association: String,
    pub issue_number: u64,
    pub repo: Repository,
}

#[derive(Deserialize)]
struct WebhookRaw {
    action: String,
    comment: WebhookComment,
    issue: WebhookIssue,
    repository: WebhookRepository,
}

#[derive(Deserialize)]
struct WebhookComment {
    body: String,
    id: u64,
    author_association: String,
    user: WebhookCommentUser,
}

#[derive(Deserialize)]
struct WebhookCommentUser {
    login: String,
}

#[derive(Deserialize)]
struct WebhookIssue {
    number: u64,
}

#[derive(Deserialize)]
struct WebhookRepository {
    owner: WebhookRepositoryOwner,
    name: String,
}

#[derive(Deserialize)]
struct WebhookRepositoryOwner {
    login: String,
}

impl Webhook {
    pub fn parse(file: &PathBuf) -> Result<Self> {
        let text = fs::read_to_string(file).with_context(|| {
            format!(
                "Error: Failed to read issue_comment data from file '{}'",
                &file.display()
            )
        })?;
        Self::parse_from_str(&text)
    }

    pub fn parse_from_str(text: &str) -> Result<Self> {
        let raw: WebhookRaw = serde_json::from_str(text)
            .with_context(|| "Error: Failed to read json data from issue_comment data")?;
        let action = raw.action;
        let comment = raw.comment.body;
        let comment_id = raw.comment.id.to_string();
        let author = raw.comment.user.login;
        let author_association = raw.comment.author_association;

        let issue_number = raw.issue.number;

        let owner = raw.repository.owner.login;
        let repo = raw.repository.name;
        let repo = Repository { owner, name: repo };

        Ok(Self {
            action,
            comment,
            comment_id,
            author,
            author_association,
            issue_number,
            repo,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WEBHOOK_DATA: &str = include_str!("../test/webhook_test_data.json");

    #[test]
    fn test_parse_webhook() {
        let webhook = Webhook::parse_from_str(WEBHOOK_DATA).unwrap();
        assert_eq!(webhook.action, "created".to_string());
        assert_eq!(
            webhook.comment,
            "@bot test_command test2\r\n@bot command2 3\r\nthis is a reponame".to_string()
        );
        assert_eq!(webhook.comment_id, "4242424242".to_string());
        assert_eq!(webhook.author, "authorlogin".to_string());
        assert_eq!(webhook.author_association, "OWNER".to_string());
        assert_eq!(webhook.issue_number, 3);
        assert_eq!(
            webhook.repo,
            Repository {
                owner: "namespace".to_string(),
                name: "reponame".to_string()
            }
        );
    }
}
