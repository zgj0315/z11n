use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TblAuthUser::Table)
                    .if_not_exists()
                    .col(pk_auto(TblAuthUser::Id))
                    .col(string(TblAuthUser::Username))
                    .col(string(TblAuthUser::Password))
                    .col(date_time(TblAuthUser::CreatedAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TblAuthUser::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TblAuthUser {
    Table,
    Id,
    Username,
    Password,
    CreatedAt,
}
