use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ItemsResponse {
    pub items: Vec<Item>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub metadata: ItemMetadata,
    pub min_price: f32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemMetadata {
    pub collection: String,
    pub name: String,
    pub option_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemsQuery {
    pub page: u16,
    pub page_size: u8,
}

pub async fn get_items(query: &ItemsQuery) -> Option<ItemsResponse> {
    println!("Getting items on page {}", query.page);

    let client = reqwest::Client::new();

    let req = client
        .get("https://openloot.com/api/v2/market/listings?sort=name%3Adesc")
        .query(query);

    let res = match req.send().await {
        Ok(result) => result,
        Err(err) => {
            eprintln!("Error getting items: {}", err);
            return None;
        }
    };

    let res = match res.text().await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading response: {}", e);
            return None;
        }
    };

    println!("{}", res.len());

    let data: ItemsResponse = match serde_json::from_str(&res) {
        Ok(result) => result,
        Err(err) => {
            eprintln!("Error parsing ItemsResponse: {}", err);
            return None;
        }
    };

    return Some(data);
}
