use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    m20250723_025059_create_tbl_auth_user::TblAuthUser,
    m20250807_152429_create_tbl_auth_role::TblAuthRole,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TblAuthUserRole::Table)
                    .if_not_exists()
                    .col(integer(TblAuthUserRole::UserId))
                    .col(integer(TblAuthUserRole::RoleId))
                    .primary_key(
                        Index::create()
                            .col(TblAuthUserRole::UserId)
                            .col(TblAuthUserRole::RoleId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TblAuthUserRole::Table, TblAuthUserRole::UserId)
                            .to(TblAuthUser::Table, TblAuthUser::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TblAuthUserRole::Table, TblAuthUserRole::RoleId)
                            .to(TblAuthRole::Table, TblAuthRole::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TblAuthUserRole::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TblAuthUserRole {
    Table,
    UserId,
    RoleId,
}
