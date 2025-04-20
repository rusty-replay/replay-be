mod db;
mod api;
mod model;
mod entity;
mod auth;
mod migration;

use actix_cors::Cors;
use actix_web::{App, HttpServer};
use actix_web::http::header;
use actix_web::web::{scope, Data};
use db::init_db;
use dotenv::dotenv;
use sea_orm::{Schema, DatabaseBackend, ConnectionTrait, Statement};
use sea_query::MysqlQueryBuilder;
use tracing_log::log::info;
use tracing_subscriber::EnvFilter;
use entity::{error_log, user};
use rusty_replay::telemetry::{get_subscriber, init_subscriber};
use crate::auth::AuthMiddleware;
use crate::migration::{Migrator, MigratorTrait};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber(
        "rusty_replay".into(),
        "info,sqlx=debug".into(),
        std::io::stdout
    );
    init_subscriber(subscriber);

    info!("애플리케이션 시작 중...");

    dotenv().ok();
    info!("환경 변수 로드 완료");

    let db = init_db().await?;
    info!("데이터베이스 마이그레이션 실행 중...");
    Migrator::up(&db, None).await?;
    info!("마이그레이션 완료");

    let db_data = Data::new(db);

    info!("서버 시작 중: http://127.0.0.1:8080");
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            // .allowed_headers(vec![header::CONTENT_TYPE])
            .allowed_headers(vec![header::CONTENT_TYPE, header::AUTHORIZATION])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(db_data.clone())
            .service(api::health_check)
            .service(api::register)
            .service(api::login)
            .service(api::refresh_token)
            .service(
                scope("/api")
                    .wrap(AuthMiddleware)
                    .service(api::get_me)
                    .service(api::report_error)
                    .service(api::list_errors)
                    .service(api::get_error))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;

    Ok(())
}