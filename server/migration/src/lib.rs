pub use sea_orm_migration::prelude::*;

mod m20250722_172354_create_tbl_agent;
mod m20250723_025059_create_tbl_auth_user;
mod m20250723_080947_create_tbl_host;
mod m20250727_145621_create_tbl_llm_task;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250722_172354_create_tbl_agent::Migration),
            Box::new(m20250723_025059_create_tbl_auth_user::Migration),
            Box::new(m20250723_080947_create_tbl_host::Migration),
            Box::new(m20250727_145621_create_tbl_llm_task::Migration),
        ]
    }
}
