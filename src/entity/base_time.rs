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
