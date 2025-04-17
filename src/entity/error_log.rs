use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "error_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub message: String,
    #[sea_orm(column_type = "Text")]
    pub stacktrace: String,
    pub app_version: String,
    pub timestamp: String,
    pub group_hash: String,
    pub replay: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}