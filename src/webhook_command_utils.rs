use crate::{
    api::{GitHubApi, RepoPermission},
    webhook::Webhook,
};
use anyhow::{bail, Result};

#[derive(PartialEq, Debug)]
pub struct BotCommand {
    pub command: String,
    pub args: String,
    pub bot: String,
}

pub async fn extract_commands(
    webhook: &Webhook,
    bots: &Vec<String>,
    api: &GitHubApi,
) -> Result<Vec<BotCommand>> {
    // prevent bots from triggering commands potentially creating loops
    if bots.contains(&webhook.author) {
        bail!("{} cannot trigger commands", webhook.author);
    }

    // only continue if comment was not deleted

    if webhook.action.eq("deleted") {
        bail!("Exiting: comment deleted");
    }

    // ensure author of command has sufficient rights
    let perm = api.get_user_permission(&webhook.author).await?;
    if perm < RepoPermission::Maintain {
        bail!("Insufficient permissions: {:?} ({})", perm, webhook.author);
    }
    Ok(parse_commands(&webhook.comment, bots))
}

fn parse_commands(text: &str, bots: &Vec<String>) -> Vec<BotCommand> {
    let mut bot_commands: Vec<BotCommand> = Vec::new();
    for bot in bots {
        let atbot = &format!("@{bot} ");
        text.lines()
            .flat_map(|line| line.strip_prefix(atbot))
            .map(|line| line.split_once(' ').unwrap_or((line, "")))
            .map(|com| BotCommand {
                command: com.0.to_owned(),
                args: com.1.to_owned(),
                bot: bot.to_owned(),
            })
            .for_each(|command| bot_commands.push(command));
    }

    bot_commands
}

#[cfg(test)]
mod tests {
    use super::BotCommand;

    use super::parse_commands;
    #[test]
    fn test_parse_commands() {
        assert_eq!(
            parse_commands(
                "@bot test_command test2\r\n@bot2 command2 3 4\r\n@bot2 command\r\nthis is a comment",
                &vec!["bot".to_string(), "bot2".to_string()]
            ),
            vec![
                BotCommand {
                    command: "test_command".to_string(),
                    args: "test2".to_string(),
                    bot: "bot".to_string(),
                },
                BotCommand {
                    command: "command2".to_string(),
                    args: "3 4".to_string(),
                    bot: "bot2".to_string()
                },
                BotCommand {
                    command: "command".to_string(),
                    args: "".to_string(),
                    bot: "bot2".to_string()
                },
            ]
        );
    }
}
