pub mod models;
mod schema;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv;

use crate::models::*;

pub struct Database {
    connection: SqliteConnection,
}

impl Database {
    pub fn new() -> Self {
        let database_path = dotenv::var("DATABASE_PATH").expect("DATABASE_PATH must be set");

        let connection = SqliteConnection::establish(&database_path).expect("Cannot connect to DB");

        Database { connection }
    }

    pub fn insert_items(&mut self, items: Vec<Item>) {
        for item in items.iter() {
            let insert_result = diesel::insert_into(schema::items::table)
                .values(item)
                .on_conflict(schema::items::id)
                .do_update()
                .set(item)
                .execute(&mut self.connection);

            if let Err(err) = insert_result {
                eprintln!("Could not insert item '{}'. Error: {}", item.name, err);
            }
        }
    }

    pub fn insert_subscriptions(&mut self, subscriptions: Vec<Subscription>) -> Vec<Subscription> {
        let mut not_inserted: Vec<Subscription> = Vec::new();

        for subscription in subscriptions.iter() {
            let insert_result = diesel::insert_into(schema::subscriptions::table)
                .values(subscription)
                .on_conflict((
                    schema::subscriptions::item_name,
                    schema::subscriptions::chat_id,
                ))
                .do_update()
                .set(subscription)
                .execute(&mut self.connection);

            if let Err(err) = insert_result {
                eprintln!(
                    "Could not insert subscription '{}'. Error: {}",
                    subscription.item_name, err
                );
                not_inserted.push(subscription.clone());
            }
        }

        return not_inserted;
    }

    pub fn get_items_no_notify(&mut self) -> Vec<Subscription> {
        use schema::subscriptions::dsl::*;
        
        let data = subscriptions.filter(notificate.eq(1)).load::<Subscription>(&mut self.connection);
        
        return Vec::new();
    }
}
