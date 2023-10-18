use std::collections::HashMap;

use database::{self, subscription};

use std::sync::Arc;
use async_once::AsyncOnce;
use lazy_static::lazy_static;
use regex::Regex;
use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands, RequestError};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "
percent min_profit
30      100
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

struct UserSubscription {
    chat_id: String,
    percent_drop: f64,
    minimum_profit: f64,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Unable to parse ENV");
    let bot = Bot::new(dotenv::var("TELOXIDE_TOKEN_FLIPPER").expect("TELOXIDE_TOKEN_FLIPPER must be set"));
    let subscriptions: Arc<Vec<UserSubscription>> = Arc::new(Vec::new());

    let subscriptions_clone = Arc::clone(&subscriptions);
    tokio::task::spawn(async move {
        let mut average_prices: HashMap<&str, f64> = HashMap::new();

        let db = DATABASE.get().await;

        for item in db.get_market_items().await.iter() {
            let item_average_price = average_prices.entry(&item.option_name).or_insert(item.last_price);

            for subscription in subscriptions_clone.iter() {
                let minimum_profit = *item_average_price - item.last_price;
                let percent_drop = &100.0 * item.last_price / *item_average_price;

                if (minimum_profit >= subscription.minimum_profit) & (percent_drop >= subscription.percent_drop) {
                    println!("{} {} {} {}", minimum_profit,subscription.minimum_profit,percent_drop,subscription.percent_drop );
                }
            }
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    });

    // Reciever
    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        let db = DATABASE.get().await;

        answer(bot, msg, *subscriptions).await
    })
    .await;
}

async fn send_notifications(bot: &Bot, subscriptions: Vec<UserSubscription>) {
    let notifications = db.get_subscriptions_to_notificate().await;

    let mut sent_notifications: Vec<database::subscription::Model> = Vec::new();

    for notification in notifications.iter() {
        let item_data = match notification.1.clone() {
            Some(data) => data,
            None => continue,
        };

        if item_data.last_price > notification.0.price {
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
            Ok(_) => {}
            Err(err) => {
                eprintln!("{}", err);
            }
        }
    }

    db.insert_subscriptions(sent_notifications).await;
}

async fn answer(bot: Bot, msg: Message, db: &database::Database) -> Result<(), RequestError> {
    let item_regex: Regex = Regex::new(r"^(\d+(?:\.\d+)?) (\d+(?:\.\d+)?)$").unwrap();

    let msg_text = match msg.text() {
        Some(text) => text,
        None => {
            return Ok(());
        }
    };

    let command = Command::parse(msg_text, "openloot_flipper_bot");

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
