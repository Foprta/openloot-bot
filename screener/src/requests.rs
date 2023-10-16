use reqwest;
use serde::Deserialize;
use urlencoding::encode;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ItemsResponse {
    pub items: Vec<Item>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub content: Vec<ContentItem>,
    pub price: String,
    pub id: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContentItem {
    pub metadata: ItemMetadata,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemMetadata {
    pub collection: String,
    pub name: String,
    pub option_name: String,
}

pub async fn get_rental(query: &String) -> Option<ItemsResponse> {
    println!("Getting rentals for '{}'", query);

    let proxy = reqwest::Proxy::https(format!(
        "http://{}:{}@{}",
        &dotenv::var("PROXY_USER").expect("PROXY_USER must be set"),
        &dotenv::var("PROXY_PASSWORD").expect("PROXY_PASSWORD must be set"),
        &dotenv::var("PROXY_URL").expect("PROXY_URL must be set")
    ))
    .unwrap();
    let client = reqwest::ClientBuilder::new()
        .proxy(proxy)
        .danger_accept_invalid_hostnames(true)
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let encoded_query = encode(query);
    let request_url = format!("https://api.openloot.com/v2/market/rentals?gameId=56a149cf-f146-487a-8a1c-58dc9ff3a15c&page=1&pageSize=100&sort=price%3Aasc&q={}", encoded_query);

    let req = client.get(request_url);

    let res = match req.send().await {
        Ok(result) => result,
        Err(err) => {
            eprintln!("Error getting items: {}", err);
            return None;
        }
    };

    let result: ItemsResponse = match res.json().await {
        Ok(res) => res,
        Err(err) => {
            eprintln!("Error parsing ItemsResponse: {}", err);
            return None;
        }
    };

    return Some(result);
}
