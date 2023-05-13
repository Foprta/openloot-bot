// @generated automatically by Diesel CLI.

diesel::table! {
    items (id) {
        id -> Text,
        collection -> Text,
        option_name -> Text,
        name -> Text,
        last_price -> Double,
    }
}

diesel::table! {
    subscriptions (chat_id, item_name) {
        chat_id -> Text,
        item_name -> Text,
        price -> Double,
        notificate -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    items,
    subscriptions,
);
