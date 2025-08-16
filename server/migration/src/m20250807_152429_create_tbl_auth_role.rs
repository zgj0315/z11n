use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TblAuthRole::Table)
                    .if_not_exists()
                    .col(pk_auto(TblAuthRole::Id))
                    .col(string(TblAuthRole::Name).unique_key())
                    .col(binary(TblAuthRole::Apis))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TblAuthRole::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum TblAuthRole {
    Table,
    Id,
    Name,
    Apis,
}
