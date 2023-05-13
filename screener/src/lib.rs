mod requests;

use database;

use requests::ItemsQuery;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::{task, time};

const PAGE_SIZE: u8 = 250;

pub async fn start_screening() -> Receiver<()> {
    let (tx, rx) = channel(1);

    task::spawn(async move {
        let mut database = database::Database::new();

        loop {
            let mut query = ItemsQuery {
                page: 1,
                page_size: PAGE_SIZE,
            };

            loop {
                let items: Vec<database::models::Item> = match get_items_models(&query).await {
                    Some(items) => items,
                    None => continue,
                };
                let items_len = items.len();

                database.insert_items(items);
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

async fn get_items_models(query: &ItemsQuery) -> Option<Vec<database::models::Item>> {
    let response = match requests::get_items(&query).await {
        Some(items) => items,
        None => return None,
    };

    let items: Vec<database::models::Item> = response
        .items
        .iter()
        .map(|item| database::models::Item {
            id: item.id.clone(),
            collection: item.metadata.collection.clone(),
            option_name: item.metadata.option_name.clone(),
            name: item.metadata.name.clone(),
            last_price: item.min_price,
        })
        .collect();

    return Some(items);
}
