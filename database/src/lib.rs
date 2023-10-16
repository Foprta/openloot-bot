pub mod entities;

use dotenv;
use sea_orm::*;

use entities::prelude::*;
pub use entities::rental_subscription;

#[derive(Clone)]
pub struct Database {
    connection: sea_orm::DatabaseConnection,
}

impl Database {
    pub async fn new() -> Self {
        let database_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let connection = sea_orm::Database::connect(database_url)
            .await
            .expect("Cound not connect to Database");

        Database { connection }
    }

    pub async fn insert_subscriptions(&self, subscriptions: Vec<rental_subscription::Model>) {
        if subscriptions.len().eq(&0) {
            return;
        }

        let active_models: Vec<rental_subscription::ActiveModel> = subscriptions
            .iter()
            .cloned()
            .map(|model| model.into_active_model())
            .collect();

        let result = RentalSubscription::insert_many(active_models)
            .on_conflict(
                sea_query::OnConflict::columns([
                    rental_subscription::Column::ChatId,
                    rental_subscription::Column::ItemName,
                    rental_subscription::Column::ItemCollection,
                ])
                .update_columns([
                    rental_subscription::Column::Price,
                    rental_subscription::Column::Notificate,
                ])
                .to_owned(),
            )
            .exec(&self.connection)
            .await;

        match result {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error inserting Rental Subscriptions: {}", err);
            }
        }
    }

    pub async fn get_subscriptions_to_notificate(&self) -> Vec<rental_subscription::Model> {
        let result = RentalSubscription::find()
            .filter(rental_subscription::Column::Notificate.eq(true))
            .all(&self.connection)
            .await
            .unwrap();

        return result;
    }
}
