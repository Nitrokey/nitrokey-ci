use std::collections::HashMap;

use anyhow::{Context as _, Result};
use octocrab::{
    commits::PullRequestTarget,
    models::{issues::Comment, pulls::PullRequest, IssueState},
    Octocrab,
};

use crate::webhook_command_utils::BotCommand;

pub struct GitHubApi {
    octocrab: Octocrab,
    owner: String,
    repo: String,
}

impl GitHubApi {
    pub fn init(owner: &str, repo: &str, api_token: String) -> Result<GitHubApi> {
        let octocrab = Octocrab::builder()
            .personal_token(api_token)
            .build()
            .context("failed to create API client")?;
        Ok(GitHubApi {
            octocrab,
            owner: owner.to_owned(),
            repo: repo.to_owned(),
        })
    }

    pub async fn find_pull_request(&self, commit: &str) -> Result<PullRequest> {
        let mut pull_requests = self
            .octocrab
            .commits(self.owner.clone(), self.repo.clone())
            .associated_pull_requests(PullRequestTarget::Sha(commit.to_owned()))
            .send()
            .await
            .context("failed to fetch matching pull requests")?
            .items;
        pull_requests.retain(|pr| pr.state == Some(IssueState::Open));
        anyhow::ensure!(
            pull_requests.len() <= 1,
            "multiple open pull request found for commit {} in repository {}/{}",
            commit,
            self.owner,
            self.repo,
        );
        pull_requests.pop().with_context(|| {
            format!(
                "no open pull request found for commit {} in repository {}/{}",
                commit, self.owner, self.repo
            )
        })
    }

    pub async fn find_comment(
        &self,
        pull_request: &PullRequest,
        filter: &str,
    ) -> Result<Option<Comment>> {
        let mut comments = self
            .octocrab
            .issues(self.owner.clone(), self.repo.clone())
            .list_comments(pull_request.number)
            .send()
            .await
            .context("failed to fetch pull request comments")?
            .items;
        comments.retain(|comment| {
            comment
                .body
                .as_ref()
                .map(|body| body.starts_with(filter))
                .unwrap_or_default()
        });
        anyhow::ensure!(
            comments.len() <= 1,
            "multiple match comments found for pull request #{} in repository {}/{}",
            pull_request.number,
            self.owner,
            self.repo,
        );
        Ok(comments.pop())
    }

    pub async fn create_comment(&self, pull_request: &PullRequest, body: &str) -> Result<Comment> {
        self.octocrab
            .issues(self.owner.clone(), self.repo.clone())
            .create_comment(pull_request.number, body)
            .await
            .with_context(|| {
                format!(
                    "failed to add comment to pull request #{} in repo {}/{}",
                    pull_request.number, self.owner, self.repo
                )
            })
    }

    pub async fn update_comment(&self, comment: &Comment, body: &str) -> Result<()> {
        self.octocrab
            .issues(self.owner.clone(), self.repo.clone())
            .update_comment(comment.id, body)
            .await
            .with_context(|| {
                format!(
                    "failed to update comment #{} in repo {}/{}",
                    comment.id, self.owner, self.repo
                )
            })?;
        Ok(())
    }

    pub async fn get_pull_request_by_id(&self, id: u64) -> Result<PullRequest> {
        self.octocrab
            .pulls(self.owner.to_owned(), self.repo.to_owned())
            .get(id)
            .await
            .context("Error: Failed to fetch matching pull request")
    }
}

pub struct GitLabAPI {
    token: String,
    instance: String,
    namespace: String,
    repo: String,
}

impl GitLabAPI {
    pub fn init(token: &str, instance: &str, namespace: &str, repo: &str) -> Self {
        GitLabAPI {
            token: token.to_string(),
            instance: instance.to_string(),
            namespace: namespace.to_string(),
            repo: repo.to_string(),
        }
    }

    async fn trigger_pipeline_with_variables(
        &self,
        branch: &str,
        variables: HashMap<&str, &str>,
    ) -> Result<String> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://{}/api/v4/projects/{}%2F{}/trigger/pipeline?token={}&ref={}",
            self.instance, self.namespace, self.repo, self.token, branch
        );

        let form: Vec<(String, String)> = variables
            .iter()
            .map(|(k, v)| (format!("variables[{k}]"), v.to_string()))
            .collect();
        let res = client.post(url).form(&form).send().await?;
        res.text()
            .await
            .context("failed to get gitlab api response")
    }

    pub async fn trigger_pipeline_with_command(
        &self,
        branch: &str,
        command: &BotCommand,
        comment_id: &str,
    ) -> Result<String> {
        let mut variables = HashMap::new();
        variables.insert("COMMAND_BOT", command.bot.as_str());
        variables.insert("COMMAND", command.command.as_str());
        variables.insert("COMMAND_ARGS", command.args.as_str());
        variables.insert("COMMENT_ID", comment_id);

        self.trigger_pipeline_with_variables(branch, variables)
            .await
    }
}
