use crate::webhook::Webhook;

#[derive(PartialEq, Debug)]
pub struct BotCommand {
    pub command: String,
    pub args: String,
    pub bot: String,
}

pub fn extract_commands(webhook: &Webhook, bots: &Vec<String>) -> Vec<BotCommand> {
    // prevent bots from triggering commands potentially creating loops
    if bots.contains(&webhook.author) {
        println!("{} cannot trigger commands", webhook.author);
        return vec![];
    }

    // only continue if comment was not deleted
    if webhook.action.eq("deleted") {
        println!("Exiting: comment deleted");
        return vec![];
    }

    // ensure author of command has sufficient rights
    let rights = &webhook.author_association;
    if !(rights.eq("OWNER") || rights.eq("COLLABORATOR")) {
        println!("permission denied: {}", rights);
        return vec![];
    }

    parse_commands(&webhook.comment, bots)
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
