pub use sea_orm_migration::prelude::*;

mod m20230514_072524_create_items;
mod m20230514_073528_create_subscriptions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230514_072524_create_items::Migration),
            Box::new(m20230514_073528_create_subscriptions::Migration),
        ]
    }
}
