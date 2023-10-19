use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use database;

use async_once::AsyncOnce;
use lazy_static::lazy_static;
use regex::Regex;
use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands, RequestError};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "
min_percent_drop min_profit
30               100
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
    minimum_percent_drop: f64,
    minimum_profit: f64,
}

struct Notification {
    chat_id: String,
    message: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Unable to parse ENV");
    let bot = Bot::new(
        dotenv::var("TELOXIDE_TOKEN_FLIPPER").expect("TELOXIDE_TOKEN_FLIPPER must be set"),
    );
    let subscriptions = Arc::new(Mutex::new(HashMap::<String, UserSubscription>::new()));

    tokio::task::spawn({
        let mut average_prices: HashMap<String, f64> = HashMap::new();
        let mut last_notified_prices: HashMap<String, HashMap<String, f64>> = HashMap::new();
        let subscriptions_cloned = Arc::clone(&subscriptions);
        let bot = bot.clone();

        async move {
            let db = DATABASE.get().await;

            loop {
                let market_items = db.get_market_items().await;
                let mut notifications: Vec<Notification> = Vec::new();

                for item in market_items.iter() {
                    let item_average_price = average_prices
                        .get(&item.option_name)
                        .unwrap_or(&item.last_price);

                    let subscriptions_cloned = subscriptions_cloned.lock().unwrap();

                    for (subscription_chat_id, subscription) in subscriptions_cloned.iter() {
                        let profit = item_average_price - item.last_price;
                        let percent_drop = &100.0 * (1.0 - item.last_price / item_average_price);

                        let already_notified = {
                            if let Some(chat_notifications) =
                                last_notified_prices.get(subscription_chat_id)
                            {
                                if let Some(last_notified_price) =
                                    chat_notifications.get(&item.option_name)
                                {
                                    last_notified_price.eq(&item.last_price)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        };

                        if (profit >= subscription.minimum_profit)
                            & (percent_drop >= subscription.minimum_percent_drop)
                        {
                            if already_notified.eq(&false) {
                                let message = format!(
                                    "
Current price: {}
Wanted price: {}

https://openloot.com/items/{}/{}
            ",
                                    item.last_price,
                                    item_average_price,
                                    item.collection,
                                    item.option_name
                                );

                                notifications.push(Notification {
                                    chat_id: subscription_chat_id.clone(),
                                    message,
                                });

                                if let Some(chat_notifications) =
                                    last_notified_prices.get_mut(subscription_chat_id)
                                {
                                    chat_notifications.insert(item.option_name.clone(), item.last_price);
                                } else {
                                    let mut chat_notifications = HashMap::new();
                                    chat_notifications.insert(item.option_name.clone(), item.last_price);
                                    last_notified_prices.insert(subscription_chat_id.clone(), chat_notifications);
                                }
                            }
                        } else if let Some(chat_notifications) =
                            last_notified_prices.get_mut(subscription_chat_id)
                        {
                            chat_notifications.remove(&item.option_name);
                        }
                    }

                    let new_average_price =
                        item_average_price + (item.last_price - item_average_price) / 5000.0;
                    average_prices.insert(item.option_name.to_string(), new_average_price);
                }

                for notification in notifications {
                    if let Err(err) = bot
                        .send_message(notification.chat_id, notification.message)
                        .await
                    {
                        eprintln!("Error sending mesage: {}", err);
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }
    });

    // Receiver
    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let subscriptions = Arc::clone(&subscriptions);
        async move { answer(bot, msg, subscriptions).await }
    })
    .await;
}

async fn answer(
    bot: Bot,
    msg: Message,
    subscriptions: Arc<Mutex<HashMap<String, UserSubscription>>>,
) -> Result<(), RequestError> {
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
        },
        Err(_) => {
            let message_lines: Vec<String> = msg_text
                .lines()
                .map(|line| line.trim().to_string())
                .collect();

            let mut answer = String::from("Done");

            for line in message_lines.iter() {
                let mut parsed_item = item_regex.captures_iter(line);

                match parsed_item.next() {
                    Some(item) => {
                        let min_percent_drop: f64 = match item[1].parse() {
                            Ok(res) => res,
                            Err(_) => {
                                answer = String::from("Cannot parse your message");
                                break;
                            }
                        };

                        let min_price_drop: f64 = match item[2].parse() {
                            Ok(res) => res,
                            Err(_) => {
                                answer = String::from("Cannot parse your message");
                                break;
                            }
                        };

                        subscriptions.lock().unwrap().insert(
                            msg.chat.id.to_string(),
                            UserSubscription {
                                minimum_percent_drop: min_percent_drop,
                                minimum_profit: min_price_drop,
                            },
                        );
                    }
                    None => {}
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
}
