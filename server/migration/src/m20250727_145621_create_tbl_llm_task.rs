use sea_orm_migration::{prelude::*, schema::*};

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
                    .col(string(TblLlmTask::Model))
                    .col(string(TblLlmTask::Prompt))
                    .col(string(TblLlmTask::ReqContent))
                    .col(date_time(TblLlmTask::ReqPushAt).default(Expr::current_timestamp()))
                    .col(date_time_null(TblLlmTask::ReqPullAt))
                    .col(string_null(TblLlmTask::RspContent))
                    .col(date_time_null(TblLlmTask::RspPushAt))
                    .col(date_time_null(TblLlmTask::RspPullAt))
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
    Model,      // 模型
    Prompt,     // 提示词
    ReqContent, // 任务问题内容
    ReqPushAt,  // 任务内容提交时间
    ReqPullAt,  // 任务接收时间，开始计算
    RspContent, // 任务答案内容
    RspPushAt,  // 任务答案提交时间
    RspPullAt,  // 任务答案获取时间
}
