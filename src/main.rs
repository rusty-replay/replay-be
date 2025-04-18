mod db;
mod api;
mod model;
mod entity;

use actix_cors::Cors;
use actix_web::{App, HttpServer};
use actix_web::http::header;
use actix_web::web::Data;
use db::init_db;
use dotenv::dotenv;
use sea_orm::{Schema, DatabaseBackend, ConnectionTrait, Statement};
use sea_query::MysqlQueryBuilder;
use tracing_subscriber::EnvFilter;
use entity::error_log;
use rusty_replay::telemetry::{get_subscriber, init_subscriber};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("rusty_replay".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    dotenv().ok();
    let db = init_db().await?;

    let schema = Schema::new(DatabaseBackend::MySql);
    let create_table_stmt = schema
        .create_table_from_entity(error_log::Entity)
        .if_not_exists()
        .to_string(MysqlQueryBuilder);

    db.execute(Statement::from_string(
        DatabaseBackend::MySql,
        create_table_stmt,
    ))
        .await?;

    let db_data = Data::new(db);

    HttpServer::new(move || {

        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allowed_headers(vec![header::CONTENT_TYPE])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(db_data.clone())
            .service(api::health_check)
            .service(api::report_error)
            .service(api::list_errors)
            .service(api::get_error)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;



    Ok(())
}