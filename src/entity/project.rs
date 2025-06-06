use chrono::{Utc, DateTime};
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

use crate::model::project::ProjectCreateRequest;
use crate::entity::base_time::{BaseTimeFields, ActiveModelTimeBehavior};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub api_key: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::event::Entity")]
    Event,
}

impl Related<super::event::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Event.def()
    }
}

impl BaseTimeFields for Model {
    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
    fn updated_at(&self) -> &Option<DateTime<Utc>> {
        &self.updated_at
    }
    fn deleted_at(&self) -> &Option<DateTime<Utc>> {
        &self.deleted_at
    }
    fn deleted_by(&self) -> &Option<i64> {
        &self.deleted_by
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C: ConnectionTrait>(self, db: &C, insert: bool) -> Result<Self, DbErr> {
        self.before_save_common(db, insert).await
    }
}

impl ActiveModelTimeBehavior for ActiveModel {
    fn set_created_at(&mut self, dt: DateTime<Utc>) {
        self.created_at = Set(dt);
    }

    fn set_updated_at(&mut self, dt: DateTime<Utc>) {
        self.updated_at = Set(Some(dt));
    }

    fn set_deleted(&mut self, by: i64, dt: DateTime<Utc>) {
        self.deleted_at = Set(Some(dt));
        self.deleted_by = Set(Some(by));
    }

    fn clear_deleted(&mut self) {
        self.deleted_at = Set(None);
        self.deleted_by = Set(None);
    }
}

impl ActiveModel {
    pub fn from_request(request: ProjectCreateRequest) -> Self {
        Self {
            name: Set(request.name),
            description: Set(request.description),
            ..Default::default()
        }
    }

    pub fn soft_delete(&mut self, deleted_by: i64) {
        let now = Utc::now();
        self.set_deleted(deleted_by, now);
    }

    pub fn restore(&mut self) {
        self.clear_deleted();
    }
}
