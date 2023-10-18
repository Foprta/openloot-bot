use crate::m20230514_072524_create_items::MarketItem;

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Subscription::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Subscription::ChatId).string().not_null())
                    .col(
                        ColumnDef::new(Subscription::ItemCollection)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Subscription::ItemName).string().not_null())
                    .col(ColumnDef::new(Subscription::Price).double().not_null())
                    .col(
                        ColumnDef::new(Subscription::Notificate)
                            .boolean()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name("pk-subsription")
                            .col(Subscription::ChatId)
                            .col(Subscription::ItemName)
                            .primary(),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk-subscription-market_item")
                            .from_tbl(Subscription::Table)
                            .from_col(Subscription::ItemName)
                            .to_tbl(MarketItem::Table)
                            .to_col(MarketItem::OptionName)
                            .from_col(Subscription::ItemCollection)
                            .to_col(MarketItem::Collection),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Subscription::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Subscription {
    Table,
    ChatId,
    ItemCollection,
    ItemName,
    Price,
    LastNotifiedPrice,
    Notificate,
}
