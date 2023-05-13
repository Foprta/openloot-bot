use database;
use screener;

use regex::Regex;
use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "To subscribe to price drops, you need to send messages in the format similar to example. To add *multiple* items at once divide them by newline.

Example:
*gold_pass_0 4000*
*StygianMenace_Head 100.6*
*MysteryBox_EarlyAccess_TimeWarden 1550.7*

Also these commands are supported:"
)]
enum Command {
    #[command(description = "display this text")]
    Help,

    #[command(description = "show all my subscriptions")]
    ShowItems,

    #[command(description = "remove some subscriptions")]
    RemoveItems,

    #[command(description = "remove all subscriptions")]
    Clear,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Unable to parse ENV");

    let bot = Bot::from_env();

    let mut rx = screener::start_screening().await;

    tokio::task::spawn(async move {
        loop {
            rx.recv().await.unwrap();
            
            let mut db = database::Database::new();
        }
    });

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        let item_regex: Regex = Regex::new(r"^([A-Za-z_]+) (\d+(?:\.\d+)?)$").unwrap();

        let msg_text = match msg.text() {
            Some(text) => text,
            None => {
                return Ok(());
            }
        };

        let command = Command::parse(msg_text, "openloot_screener_bot");

        let sended_message = match command {
            Ok(command) => match command {
                Command::Help => {
                    bot.send_message(msg.chat.id, Command::descriptions().to_string())
                        .parse_mode(ParseMode::Markdown)
                        .await
                }
                _ => bot.send_dice(msg.chat.id).await,
            },
            Err(_) => {
                let message_lines: Vec<String> = msg_text
                    .lines()
                    .map(|line| line.trim().to_string())
                    .collect();

                let mut subscriptions: Vec<database::models::Subscription> = Vec::new();
                let mut not_parsed_lines: Vec<String> = Vec::new();
                let mut not_inserted_items: Vec<database::models::Subscription> = Vec::new();

                for line in message_lines.iter() {
                    let mut parsed_item = item_regex.captures_iter(line);

                    match parsed_item.next() {
                        Some(item) => {
                            let price: f64 = match item[2].parse() {
                                Ok(res) => res,
                                Err(_) => {
                                    not_parsed_lines.push(line.to_string());
                                    break;
                                }
                            };

                            let subscription = database::models::Subscription {
                                chat_id: msg.chat.id.to_string(),
                                price,
                                item_name: item[1].to_string(),
                                notificate: 1,
                            };

                            subscriptions.push(subscription);
                        }
                        None => {
                            not_parsed_lines.push(line.to_string());
                        }
                    }
                }

                if subscriptions.len().gt(&0) {
                    let mut db = database::Database::new();
                    not_inserted_items = db.insert_subscriptions(subscriptions);
                }

                let mut answer: String = String::new();

                if not_inserted_items.len().eq(&0) && not_parsed_lines.len().eq(&0) {
                    answer = "All items added sucessfully".to_string();
                } else {
                    let not_parsed_answer = not_parsed_lines.join("\n");

                    if not_parsed_lines.len().ne(&0) {
                        answer.push_str(&format!("Error parsing:\n{}", not_parsed_answer));
                    }

                    let not_inserted_answer = not_inserted_items
                        .iter()
                        .map(|item| format!("{} {}", item.item_name, item.price))
                        .collect::<Vec<String>>()
                        .join("\n");

                    if not_inserted_answer.len().ne(&0) {
                        answer.push_str(&format!(
                            "\n\nError inserting into database:\n{}",
                            not_inserted_answer
                        ));
                    }
                }

                bot.send_message(msg.chat.id, answer).await
            }
        };

        match sended_message {
            Err(err) => eprintln!("Error sending message: {}", err),
            _ => {}
        }

        Ok(())
    })
    .await;
}
