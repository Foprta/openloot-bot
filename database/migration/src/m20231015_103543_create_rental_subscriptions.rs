use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RentalSubscription::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RentalSubscription::ChatId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RentalSubscription::ItemCollection)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RentalSubscription::ItemName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RentalSubscription::Price)
                            .double()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RentalSubscription::Notificate)
                            .boolean()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name("pk-rental_subsription")
                            .col(RentalSubscription::ChatId)
                            .col(RentalSubscription::ItemName)
                            .col(RentalSubscription::ItemCollection)
                            .primary(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RentalSubscription::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum RentalSubscription {
    Table,
    ChatId,
    ItemCollection,
    ItemName,
    Price,
    Notificate,
}
