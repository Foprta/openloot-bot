mod requests;

use database;

use tokio::sync::mpsc::{channel, Receiver};
use tokio::{task, time};

#[derive(Debug)]
pub struct RentalNotification {
    pub item_id: String,
    pub price: String,
    pub subscription: database::rental_subscription::Model,
}

pub async fn start_screening() -> Receiver<RentalNotification> {
    let (tx, rx) = channel(1);

    task::spawn(async move {
        let database = database::Database::new().await;

        loop {
            let rental_subscriptions = database.get_subscriptions_to_notificate().await;

            for subscription in rental_subscriptions {
                if subscription.price.eq(&0.0) {
                    continue;
                }

                if let Some(rental) = requests::get_rental(&subscription.item_name).await {
                    for rental_item in rental.items {
                        if rental_item.price.parse::<f64>().unwrap() <= subscription.price {
                            if rental_item.content.iter().any(|i| {
                                i.metadata
                                    .name
                                    .eq_ignore_ascii_case(&subscription.item_name)
                            }) {
                                tx.send(RentalNotification {
                                    subscription: subscription.clone(),
                                    item_id: rental_item.id,
                                    price: rental_item.price,
                                })
                                .await
                                .expect("Channel broken");
                            }
                        } else {
                            // if any item in price sorted is higher than wanted price - all other is higher too
                            break;
                        }
                    }
                }
            }

            time::sleep(time::Duration::from_secs(5)).await;
        }
    });

    return rx;
}
