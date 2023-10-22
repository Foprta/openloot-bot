mod requests;

use tokio::sync::mpsc::Sender;

use database;

use requests::ItemsQuery;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::task;

const PAGE_SIZE: u8 = 250;

pub async fn start_screening() -> Receiver<()> {
    let (tx, rx) = channel(1);

    let tx_clone = tx.clone();
    start_page_parsing(1, tx_clone);
    let tx_clone = tx.clone();
    start_page_parsing(2, tx_clone);
    let tx_clone = tx.clone();
    start_page_parsing(3, tx_clone);

    return rx;
}

fn start_page_parsing(page_size: u16, tx: Sender<()>) {
    task::spawn(async move {
        let database = database::Database::new().await;

        let query = ItemsQuery {
            page: page_size,
            page_size: PAGE_SIZE,
        };

        loop {
            let items: Vec<database::market_item::Model> = match get_items_models(&query).await {
                Some(items) => items,
                None => continue,
            };

            database.insert_items(items).await;
            tx.send(()).await.expect("Channel broken");

            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
    });
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
