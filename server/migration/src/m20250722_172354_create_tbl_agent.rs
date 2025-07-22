use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TblAgent::Table)
                    .if_not_exists()
                    .col(string(TblAgent::Id).primary_key())
                    .col(string(TblAgent::Version))
                    .col(string(TblAgent::State))
                    .col(string(TblAgent::Token))
                    .col(date_time(TblAgent::CreatedAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TblAgent::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TblAgent {
    Table,
    Id,
    Version,
    State,
    Token,
    CreatedAt,
}
