use diesel::prelude::*;

use crate::schema::items;
use crate::schema::subscriptions;

#[derive(Queryable, Insertable, AsChangeset)]
#[diesel(table_name = items)]
pub struct Item {
    pub id: String,
    pub collection: String,
    pub option_name: String,
    pub name: String,
    pub last_price: f64,
}

#[derive(Queryable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = subscriptions)]
pub struct Subscription {
    pub chat_id: String,
    pub item_name: String,
    pub price: f64,
    pub notificate: i32,
}