use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TblHost::Table)
                    .if_not_exists()
                    .col(string(TblHost::AgentId).primary_key())
                    .col(string_null(TblHost::Name))
                    .col(string_null(TblHost::HostName))
                    .col(string_null(TblHost::OsVersion))
                    .col(string(TblHost::CpuArch))
                    .col(binary(TblHost::Content))
                    .col(date_time(TblHost::CreatedAt).default(Expr::current_timestamp()))
                    .col(date_time(TblHost::UpdatedAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TblHost::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TblHost {
    Table,
    AgentId,
    Name,
    HostName,
    OsVersion,
    CpuArch,
    Content,
    CreatedAt,
    UpdatedAt,
}
