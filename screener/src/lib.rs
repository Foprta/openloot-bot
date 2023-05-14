mod requests;

use database;

use requests::ItemsQuery;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::{task, time};

const PAGE_SIZE: u8 = 250;

pub async fn start_screening() -> Receiver<()> {
    let (tx, rx) = channel(1);

    task::spawn(async move {
        let database = database::Database::new().await;

        loop {
            let mut query = ItemsQuery {
                page: 1,
                page_size: PAGE_SIZE,
            };

            loop {
                let items: Vec<database::market_item::Model> = match get_items_models(&query).await
                {
                    Some(items) => items,
                    None => continue,
                };
                let items_len = items.len();

                database.insert_items(items).await;
                tx.send(()).await.expect("Channel broken");

                time::sleep(time::Duration::from_secs(3)).await;
                if items_len.lt(&250) {
                    break;
                }

                query.page += 1;
            }
        }
    });

    return rx;
}

async fn get_items_models(query: &ItemsQuery) -> Option<Vec<database::market_item::Model>> {
    let response = match requests::get_items(&query).await {
        Some(items) => items,
        None => return None,
    };

    let items: Vec<database::market_item::Model> = response
        .items
        .iter()
        .map(|item| database::market_item::Model {
            collection: item.metadata.collection.clone(),
            option_name: item.metadata.option_name.clone(),
            name: item.metadata.name.clone(),
            last_price: item.min_price,
        })
        .collect();

    return Some(items);
}
