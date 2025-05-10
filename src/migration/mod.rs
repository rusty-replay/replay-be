pub use sea_orm_migration::prelude::*;

mod m20250420_000001_create_event_table;
mod m20250420_000002_create_user_table;
mod m20250420_000003_create_project_table;
mod m20250420_000004_create_issue_table;
mod m20250420_01_create_project_member_table;
mod m20250420_000005_create_transaction_table;
mod m20250420_000006_create_span_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250420_000002_create_user_table::Migration),
            Box::new(m20250420_000003_create_project_table::Migration),
            Box::new(m20250420_000004_create_issue_table::Migration),
            Box::new(m20250420_01_create_project_member_table::Migration),
            Box::new(m20250420_000001_create_event_table::Migration),
            Box::new(m20250420_000005_create_transaction_table::Migration),
            Box::new(m20250420_000006_create_span_table::Migration),
        ]
    }
}