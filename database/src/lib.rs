pub mod entities;

use dotenv;
use sea_orm::*;

use entities::prelude::*;
pub use entities::{market_item, subscription};

#[derive(Clone)]
pub struct Database {
    connection: sea_orm::DatabaseConnection,
}

impl Database {
    pub async fn new() -> Self {
        let database_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");

        println!("{}", database_url);

        let connection = sea_orm::Database::connect(database_url)
            .await
            .expect("Cound not connect to Database");

        Database { connection }
    }

    pub async fn insert_items(&self, items: Vec<market_item::Model>) {
        if items.len().eq(&0) {
            return;
        }

        let active_models: Vec<market_item::ActiveModel> = items
            .iter()
            .cloned()
            .map(|model| model.into_active_model())
            .collect();

        let result = MarketItem::insert_many(active_models)
            .on_conflict(
                sea_query::OnConflict::columns([
                    market_item::Column::Collection,
                    market_item::Column::OptionName,
                ])
                .update_column(market_item::Column::LastPrice)
                .to_owned(),
            )
            .exec(&self.connection)
            .await;

        match result {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error inserting Items: {}", err);
            }
        }
    }

    pub async fn insert_subscriptions(&self, subscriptions: Vec<subscription::Model>) {
        if subscriptions.len().eq(&0) {
            return;
        }

        let active_models: Vec<subscription::ActiveModel> = subscriptions
            .iter()
            .cloned()
            .map(|model| model.into_active_model())
            .collect();

        let result = Subscription::insert_many(active_models)
            .on_conflict(
                sea_query::OnConflict::columns([
                    subscription::Column::ChatId,
                    subscription::Column::ItemName,
                ])
                .update_columns([
                    subscription::Column::Price,
                    subscription::Column::Notificate,
                ])
                .to_owned(),
            )
            .exec(&self.connection)
            .await;

        match result {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error inserting Subscriptions: {}", err);
            }
        }
    }

    pub async fn get_subscriptions_to_notificate(&self) -> Vec<(subscription::Model, Option<market_item::Model>)> {
        let result = Subscription::find()
            .filter(subscription::Column::Notificate.eq(true))
            .find_also_related(market_item::Entity)
            .all(&self.connection).await.unwrap();
        
        return result;
    }
}
