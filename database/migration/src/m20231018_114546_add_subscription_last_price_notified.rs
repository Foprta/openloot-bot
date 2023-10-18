use crate::m20230514_073528_create_subscriptions::Subscription;

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Subscription::Table)
                    .add_column(ColumnDef::new(Subscription::LastNotifiedPrice).double().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Subscription::Table)
                    .drop_column(Subscription::LastNotifiedPrice)
                    .to_owned(),
            )
            .await
    }
}


