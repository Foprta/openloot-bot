use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ItemsResponse {
    pub items: Vec<Item>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
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

    let response_body = format!("Response: {:?}", &res);

    let data: ItemsResponse = match res.json().await {
        Ok(result) => result,
        Err(err) => {
            eprintln!(
                "Error parsing ItemsResponse: {}\nResponse was: {}",
                err, response_body
            );
            return None;
        }
    };

    return Some(data);
}
