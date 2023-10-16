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
    pub min_price: f64,
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

    let url = format!("https://proxy.scrapeops.io/v1/?api_key={}&url=https%3A%2F%2Fapi.openloot.com%2Fv2%2Fmarket%2Flistings%3FgameId%3D56a149cf-f146-487a-8a1c-58dc9ff3a15c%26onSale%3Dtrue%26page%3D{}%26pageSize%3D{}%26sort%3Dname%253Aasc&country=us&device_type=desktop&premium=true", dotenv::var("PROXY_API_KEY").expect("PROXY_API_KEY must be set"), query.page, query.page_size);

    let req = client.get(url);

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

    let data: ItemsResponse = match serde_json::from_str(&res) {
        Ok(result) => result,
        Err(err) => {
            eprintln!("Error parsing ItemsResponse: {}", err);
            return None;
        }
    };

    return Some(data);
}
