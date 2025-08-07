use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250722_172354_create_tbl_agent::TblAgent;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TblLlmTask::Table)
                    .if_not_exists()
                    .col(string(TblLlmTask::Id).primary_key())
                    .col(string(TblLlmTask::ReqAgentId))
                    .col(string(TblLlmTask::Model))
                    .col(string(TblLlmTask::Prompt))
                    .col(string(TblLlmTask::ReqContent))
                    .col(date_time(TblLlmTask::ReqPushAt).default(Expr::current_timestamp()))
                    .col(date_time_null(TblLlmTask::ReqPullAt))
                    .col(string_null(TblLlmTask::RspAgentId))
                    .col(string_null(TblLlmTask::RspContent))
                    .col(date_time_null(TblLlmTask::RspPushAt))
                    .col(date_time_null(TblLlmTask::RspPullAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(TblLlmTask::Table, TblLlmTask::ReqAgentId)
                            .to(TblAgent::Table, TblAgent::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TblLlmTask::Table, TblLlmTask::RspAgentId)
                            .to(TblAgent::Table, TblAgent::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TblLlmTask::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TblLlmTask {
    Table,
    Id,
    ReqAgentId,
    Model,      // 模型
    Prompt,     // 提示词
    ReqContent, // 任务问题内容
    ReqPushAt,  // 任务内容提交时间
    ReqPullAt,  // 任务接收时间，开始计算
    RspAgentId,
    RspContent, // 任务答案内容
    RspPushAt,  // 任务答案提交时间
    RspPullAt,  // 任务答案获取时间
}
