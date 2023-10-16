use curl::easy::Easy;
use serde::Deserialize;
use serde_json;
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

    let mut json = Vec::new();

    let mut easy = Easy::new();
    easy.proxy_username(&dotenv::var("PROXY_USER").expect("PROXY_USER must be set")).unwrap();
    easy.proxy_password(&dotenv::var("PROXY_PASSWORD").expect("PROXY_PASSWORD must be set")).unwrap();
    easy.proxy(&dotenv::var("PROXY_URL").expect("PROXY_URL must be set")).unwrap();
    easy.useragent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36").unwrap();
    easy.proxy_port(10000).unwrap();
    easy.get(true).unwrap();

    let encoded_query = encode(query);
    easy.url(&format!("https://api.openloot.com/v2/market/rentals?gameId=56a149cf-f146-487a-8a1c-58dc9ff3a15c&page=1&pageSize=100&sort=price%3Aasc&q={}", encoded_query)).unwrap();

    {
        let mut transfer = easy.transfer();
        transfer
            .write_function(|data| {
                json.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }

    let result: ItemsResponse = match serde_json::from_slice(&json) {
        Ok(res) => res,
        Err(err) => {
            eprintln!("Error parsing ItemsResponse: {}", err);
            return None;
        }
    };

    return Some(result);
}
