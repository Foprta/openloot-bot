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
To subscribe to price drops, you need to send messages in the format similar to example. To add *multiple* items at once divide them by newline.

Example:
*gold_pass_0 4000*
*StygianMenace_Head 100.6*
*MysteryBox_EarlyAccess_TimeWarden 1550.7*

Also these commands are supported:
    "
)]
enum Command {
    #[command(description = "display this text")]
    Help,
    //    #[command(description = "show all my subscriptions")]
    //    ShowItems,
    //
    //    #[command(description = "remove some subscriptions")]
    //    RemoveItems,
    //
    //    #[command(description = "remove all subscriptions")]
    //    Clear,
}

lazy_static! {
    static ref DATABASE: AsyncOnce<database::Database> =
        AsyncOnce::new(async { database::Database::new().await });
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Unable to parse ENV");
    let bot = Bot::new(
        dotenv::var("TELOXIDE_TOKEN_SCREENER").expect("TELOXIDE_TOKEN_SCREENER must be set"),
    );

    let mut rx = screener::start_screening().await;

    // Notifier
    tokio::task::spawn({
        let db = DATABASE.get().await;

        let bot = bot.clone();

        async move {
            loop {
                rx.recv().await.unwrap();

                send_notifications(&bot, &db).await;
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

async fn send_notifications(bot: &Bot, db: &database::Database) {
    let notifications = db.get_subscriptions_to_notificate().await;

    let mut sent_notifications: Vec<database::subscription::Model> = Vec::new();

    for notification in notifications.iter() {
        let mut updated_notification = notification.0.clone();

        let item_data = match notification.1.clone() {
            Some(data) => data,
            None => continue,
        };

        if let Some(last_notified_price) = notification.0.last_notified_price {
            if item_data.last_price.eq(&last_notified_price) {
                continue;
            }
        }

        if item_data.last_price > notification.0.price {
            updated_notification.last_notified_price = None;
            sent_notifications.push(updated_notification);
            continue;
        }

        let message = format!(
            "
Current price: {}
Wanted price: {}

https://openloot.com/items/{}/{}
            ",
            item_data.last_price,
            notification.0.price.clone(),
            item_data.collection,
            item_data.option_name
        );

        match bot
            .send_message(notification.0.chat_id.clone(), message)
            .await
        {
            Ok(_) => {
                updated_notification.last_notified_price = Some(item_data.last_price);
                sent_notifications.push(updated_notification);
            },
            Err(err) => {
                eprintln!("{}", err);
            }
        }
    }

    db.insert_subscriptions(sent_notifications).await;
}

async fn answer(bot: Bot, msg: Message, db: &database::Database) -> Result<(), RequestError> {
    let item_regex: Regex = Regex::new(r"^([A-Za-z0-9_]+) (\d+(?:\.\d+)?)$").unwrap();

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

            let mut subscriptions: Vec<database::subscription::Model> = Vec::new();

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

                        let subscription = database::subscription::Model {
                            chat_id: msg.chat.id.to_string(),
                            price,
                            item_collection: "BT0".to_string(),
                            item_name: item[1].to_string(),
                            notificate: true,
                            last_notified_price: None,
                        };

                        subscriptions.push(subscription);
                    }
                    None => {}
                }
            }

            let answer: String;
            if subscriptions.len().gt(&0) {
                answer = String::from("Done");
                db.insert_subscriptions(subscriptions).await;
            } else {
                answer = String::from("Cannot parse your message");
            }

            bot.send_message(msg.chat.id, answer).await
        }
    };

    match sended_message {
        Err(err) => eprintln!("Error sending message: {}", err),
        _ => {}
    }

    Ok(())
}
