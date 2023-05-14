use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MarketItem::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(MarketItem::Collection).string().not_null())
                    .col(ColumnDef::new(MarketItem::OptionName).string().not_null())
                    .col(ColumnDef::new(MarketItem::Name).string().not_null())
                    .col(ColumnDef::new(MarketItem::LastPrice).double().not_null())
                    .primary_key(
                        Index::create()
                            .name("pk-market_item")
                            .col(MarketItem::Collection)
                            .col(MarketItem::OptionName)
                            .primary(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MarketItem::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum MarketItem {
    Table,
    Collection,
    OptionName,
    Name,
    LastPrice,
}
