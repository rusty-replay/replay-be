mod db;
mod api;
mod model;
mod entity;
mod auth;
mod migration;
mod util;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use actix_web::http::header;
use actix_web::middleware::from_fn;
use actix_web::web::{scope, Data, JsonConfig};
use db::init_db;
use dotenv::dotenv;
use sea_orm::{Schema, DatabaseBackend, ConnectionTrait, Statement};
use sea_query::MysqlQueryBuilder;
use tracing_log::log::info;
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_actix_web::AppExt;
use utoipa_swagger_ui::SwaggerUi;
use entity::{event, user};
use rusty_replay::telemetry::{get_subscriber, init_subscriber};
use crate::auth::{auth_middleware};
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

    info!("서버 시작 중: http://127.0.0.1:8081");
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://localhost:3001")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
                header::ORIGIN,
                header::ACCESS_CONTROL_REQUEST_METHOD,
                header::ACCESS_CONTROL_REQUEST_HEADERS
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .app_data(JsonConfig::default().limit(10 * 1024 * 1024))
            .wrap(cors)
            .app_data(db_data.clone())
            .service(api::health_check::health_check)
            .service(api::register)
            .service(api::login)
            .service(api::refresh_token)

            .service(api::report_batch_events)
            .service(api::report_event)
            .service(api::receive_traces)
            .service(
                scope("/api")
                    .wrap(from_fn(auth_middleware))
                    .service(api::get_me)

                    .service(api::create_project)
                    .service(api::update_project)
                    .service(api::list_user_projects)
                    .service(api::get_project)
                    .service(api::delete_project)
                    .service(api::get_project_users)

                    .service(api::get_project_events)
                    .service(api::list_project_events)
                    .service(api::set_priority)
                    .service(api::set_assignee)
                    .service(api::set_event_status)

                    .service(api::get_transactions)
                    .service(api::get_transaction_spans)
            )
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", ApiDoc::openapi()))
    })
        .bind(("127.0.0.1", 8081))?
        .run()
        .await?;

    Ok(())
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::health_check::health_check,
        crate::api::auth::register,
        crate::api::auth::login,
        crate::api::auth::refresh_token,
        crate::api::auth::get_me,

        crate::api::project::create_project,
        crate::api::project::update_project,
        crate::api::project::list_user_projects,
        crate::api::project::get_project,
        crate::api::project::delete_project,
        crate::api::project::get_project_users,

        crate::api::event::report_event,
        crate::api::event::report_batch_events,
        crate::api::event::get_project_events,
        crate::api::event::list_project_events,
        crate::api::event::set_priority,
        crate::api::event::set_assignee,
        crate::api::event::set_event_status,

        crate::api::trace::receive_traces,
        crate::api::trace::get_transaction_spans,
        crate::api::trace::get_transactions,
    ),
)]
struct ApiDoc;
