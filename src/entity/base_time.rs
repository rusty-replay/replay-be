use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

pub trait BaseTimeFields {
    fn created_at(&self) -> &DateTime<Utc>;
    fn updated_at(&self) -> &Option<DateTime<Utc>>;
    fn deleted_at(&self) -> &Option<DateTime<Utc>>;
    fn deleted_by(&self) -> &Option<i64>;

    fn is_deleted(&self) -> bool {
        self.deleted_at().is_some()
    }

    fn is_deleted_and_expired(&self) -> bool {
        self.deleted_at().map_or(false, |deleted_at| {
            deleted_at.to_owned() + chrono::Duration::days(30) < Utc::now()
        })
    }
}

#[async_trait::async_trait]
pub trait ActiveModelTimeBehavior {
    fn set_created_at(&mut self, dt: DateTime<Utc>);
    fn set_updated_at(&mut self, dt: DateTime<Utc>);
    fn set_deleted(&mut self, by: i64, dt: DateTime<Utc>);
    fn clear_deleted(&mut self);

    async fn before_save_common<C: ConnectionTrait>(
        mut self,
        _db: &C,
        insert: bool,
    ) -> Result<Self, DbErr>
    where
        Self: Sized,
    {
        let now = Utc::now();
        if insert {
            self.set_created_at(now);
        } else {
            self.set_updated_at(now);
        }
        Ok(self)
    }

    fn soft_delete(&mut self, deleted_by: i64) {
        let now = Utc::now();
        self.set_deleted(deleted_by, now);
    }

    fn restore(&mut self) {
        self.clear_deleted();
    }
}
