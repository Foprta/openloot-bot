use database;
use screener;

use async_once::AsyncOnce;
use lazy_static::lazy_static;
use regex::Regex;
use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands, RequestError};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "
To subscribe to rental price drops, you need to send messages in the format similar to example. To add *multiple* items at once divide them by newline.

Example:
*rent 'Lunar Time Warden' 10*
*rent 'Shrouded Armory' 2*
*rent 'Bug Buster' 10*

Also these commands are supported:
    "
)]
enum Command {
    #[command(description = "display this text")]
    Help,
}

lazy_static! {
    static ref DATABASE: AsyncOnce<database::Database> =
        AsyncOnce::new(async { database::Database::new().await });
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Unable to parse ENV");
    let bot = Bot::from_env();
    let db = DATABASE.get().await;

    let mut rx = screener::start_screening().await;

    // Notifier
    tokio::task::spawn({
        let bot = bot.clone();

        async move {
            loop {
                if let Some(data) = rx.recv().await {
                    let message = format!(
                        "
{}

Current price: {}
Wanted price: {}

https://openloot.com/marketplace/rentals/{}
            ",
                        data.subscription.item_name,
                        data.price,
                        data.subscription.price,
                        data.item_id
                    );

                    bot.send_message(data.subscription.chat_id, message)
                        .await
                        .expect("Cannt send message");
                };
            }
        }
    });

    // Reciever
    teloxide::repl(bot, |bot: Bot, msg: Message| async {
        let db = DATABASE.get().await;

        answer(bot, msg, db).await
    })
    .await;
}

async fn answer(bot: Bot, msg: Message, db: &database::Database) -> Result<(), RequestError> {
    let item_regex: Regex = Regex::new(r"^rent '(.*)' (\d+(?:\.\d+)?)$").unwrap();

    let msg_text = match msg.text() {
        Some(text) => text,
        None => {
            return Ok(());
        }
    };

    let command = Command::parse(msg_text, "openloot_test_bot");

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

            let mut subscriptions: Vec<database::rental_subscription::Model> = Vec::new();

            for line in message_lines.iter() {
                let mut parsed_item = item_regex.captures_iter(line);

                match parsed_item.next() {
                    Some(item) => {
                        let price: f64 = match item[2].parse() {
                            Ok(res) => res,
                            Err(_) => {
                                break;
                            }
                        };

                        let subscription = database::rental_subscription::Model {
                            chat_id: msg.chat.id.to_string(),
                            price,
                            item_collection: "BT0".to_string(),
                            item_name: item[1].to_string(),
                            notificate: true,
                        };

                        subscriptions.push(subscription);
                    }
                    None => {}
                }
            }

            if subscriptions.len().gt(&0) {
                db.insert_subscriptions(subscriptions).await;
            }

            let answer: String = String::from("Done");

            bot.send_message(msg.chat.id, answer).await
        }
    };

    match sended_message {
        Err(err) => eprintln!("Error sending message: {}", err),
        _ => {}
    }

    Ok(())
}
