use sea_orm_migration::{prelude::*, schema::*};

use crate::idents::I;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(I::Entry)
                    .col(pk_auto(I::Id))
                    .col(string(I::Data))
                    .col(date_time(I::AddedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(I::Entry).to_owned())
            .await
    }
}
