use actix_web::{get, HttpResponse, Responder};

#[utoipa::path(
    get,
    path = "/health-check",
    responses(
        (status = 200, description = "서버가 정상 동작 중", body = String)
    ),
    tag = "health check",
)]
#[get("/health-check")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}