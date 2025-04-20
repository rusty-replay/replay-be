
use actix_web::{get, post, web, HttpResponse, Responder};
use sea_orm::{EntityTrait, Set, ActiveModelTrait, QueryOrder};
use crate::entity::error_log::{self, ActiveModel, Entity as ErrorEntity};
use crate::model::error::{ErrorReportRequest, ErrorReportResponse};
use sha2::{Sha256, Digest};

fn calculate_group_hash(message: &str, stack: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(message);
    hasher.update(stack);
    let result = hasher.finalize();
    format!("{:x}", result)
}

#[get("/health-check")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[post("/errors")]
pub async fn report_error(
    body: web::Json<ErrorReportRequest>,
    db: web::Data<sea_orm::DatabaseConnection>,
) -> impl Responder {
    let group_hash = calculate_group_hash(&body.message, &body.stacktrace);

    let new_log = ActiveModel {
        message: Set(body.message.clone()),
        stacktrace: Set(body.stacktrace.clone()),
        app_version: Set(body.app_version.clone()),
        timestamp: Set(body.timestamp.clone()),
        group_hash: Set(group_hash.clone()),
        replay: Set(body.replay.clone().into()),
        ..Default::default()
    };

    let inserted = new_log.insert(db.get_ref()).await.unwrap();

    HttpResponse::Ok().json(ErrorReportResponse {
        id: inserted.id,
        message: inserted.message,
        stacktrace: inserted.stacktrace,
        app_version: inserted.app_version,
        timestamp: inserted.timestamp,
        group_hash,
        replay: inserted.replay,
    })
}

#[get("/errors")]
pub async fn list_errors(
    db: web::Data<sea_orm::DatabaseConnection>,
) -> impl Responder {
    let logs = ErrorEntity::find()
        .order_by_desc(error_log::Column::Id)
        .all(db.get_ref())
        .await
        .unwrap();

    let response: Vec<ErrorReportResponse> = logs
        .into_iter()
        .map(|l| ErrorReportResponse {
            id: l.id,
            message: l.message,
            stacktrace: l.stacktrace,
            app_version: l.app_version,
            timestamp: l.timestamp,
            group_hash: l.group_hash,
            replay: l.replay,
        })
        .collect();

    HttpResponse::Ok().json(response)
}

#[get("/errors/{id}")]
pub async fn get_error(
    db: web::Data<sea_orm::DatabaseConnection>,
    path: web::Path<i32>,
) -> impl Responder {
    let id = path.into_inner();
    if let Some(l) = ErrorEntity::find_by_id(id)
        .one(db.get_ref())
        .await
        .unwrap()
    {
        HttpResponse::Ok().json(ErrorReportResponse {
            id: l.id,
            message: l.message,
            stacktrace: l.stacktrace,
            app_version: l.app_version,
            timestamp: l.timestamp,
            group_hash: l.group_hash,
            replay: l.replay,
        })
    } else {
        HttpResponse::NotFound().body("Not Found")
    }
}
