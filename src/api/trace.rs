use crate::entity::{span, transaction};
use crate::model::common::PaginationResponse;
use crate::model::global_error::{AppError, ErrorCode};
use crate::model::span::{SpanResponse, TransactionListQuery, TransactionWithSpansResponse};
use crate::model::transaction::TransactionResponse;
use actix_web::{get, post, web, HttpResponse};
use chrono::{DateTime, TimeZone, Utc};
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_proto::tonic::common::v1::any_value::Value as AnyValueKind;
use prost::Message;
use rand::{rng, Rng};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait};
use serde::Deserialize;
use std::collections::HashMap;
use crate::model::transaction::TraceRequest;

pub fn generate_mixed_id() -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rng();

    (0..8)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

// AnyValueKind → String
fn any_to_string(kind: &AnyValueKind) -> Option<String> {
    match kind {
        AnyValueKind::StringValue(s) => Some(s.clone()),
        AnyValueKind::IntValue(i) => Some(i.to_string()),
        AnyValueKind::DoubleValue(d) => Some(d.to_string()),
        AnyValueKind::BoolValue(b) => Some(b.to_string()),
        _ => None,
    }
}

pub fn calculate_duration(start: &DateTime<Utc>, end: &DateTime<Utc>) -> i32 {
    (end.timestamp_millis() - start.timestamp_millis()) as i32
}

fn format_utc(nano: u64) -> Result<DateTime<Utc>, &'static str> {
    Utc.timestamp_opt(
        (nano / 1_000_000_000) as i64,
        (nano % 1_000_000_000) as u32,
    )
        .single()
        .ok_or("Invalid timestamp")
}

#[utoipa::path(
    post,
    path = "/api/trace",
    request_body = TraceRequest,
    responses(
        (status = 200, description = "Traces received successfully"),
        (status = 400, description = "Invalid request", body = AppError),
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Trace",
    summary = "trace 받기"
)]
#[post("/traces")]
pub async fn receive_traces(
    body: web::Bytes,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    let req = ExportTraceServiceRequest::decode(body.as_ref())
        .map_err(|e| {
            log::error!("OTLP decode error: {}", e);
            AppError::bad_request(ErrorCode::InvalidEvent)
        })?;

    println!("OTLP traces: {:?}", req);

    let mut all_spans = Vec::new();
    for rs in req.resource_spans {
        for ss in rs.scope_spans {
            for span in ss.spans {
                let trace_id = hex::encode(span.trace_id);
                let start = match format_utc(span.start_time_unix_nano) {
                    Ok(ts) => ts,
                    Err(e) => {
                        log::warn!("Invalid start timestamp for span {}: {}", span.name, e);
                        continue;
                    }
                };

                let end = match format_utc(span.end_time_unix_nano) {
                    Ok(ts) => ts,
                    Err(e) => {
                        log::warn!("Invalid end timestamp for span {}: {}", span.name, e);
                        continue;
                    }
                };

                let mut http = HashMap::new();
                for kv in span.attributes {
                    if let Some(val) = kv.value.and_then(|any| any.value).and_then(|k| any_to_string(&k)) {
                        http.insert(kv.key, val);
                    }
                }
                all_spans.push((
                    trace_id,
                    span.span_id.clone(),
                    span.parent_span_id.clone(),
                    span.name.clone(),
                    start,
                    end,
                    http,
                ));
            }
        }
    }

    if all_spans.is_empty() {
        return Ok(HttpResponse::NoContent().finish());
    }

    let txn = db.begin().await?;

    let default_time = Utc::now();
    let start_ts = all_spans.iter()
        .map(|s| &s.4)
        .min()
        .unwrap_or(&default_time);

    let end_ts = all_spans.iter()
        .map(|s| &s.5)
        .max()
        .unwrap_or(&default_time);

    let trace_id = generate_mixed_id();
    let tx_active = transaction::ActiveModel::new(
        1,
        trace_id.clone(),
        "unified_transaction",
        *start_ts,
        *end_ts,
        "production",
        "ok",
        None,
    );

    let tx_inserted: transaction::Model = tx_active.insert(&txn).await?;

    for (orig_trace_id, span_id, parent, name, start, end, http) in all_spans {
        let mut http_with_orig_trace = http.clone();
        http_with_orig_trace.insert("original_trace_id".to_string(), orig_trace_id);

        let span_active = span::ActiveModel::new(
            tx_inserted.id,
            span_id.clone(),
            Some(parent),
            name.clone(),
            start,
            end,
            http.get("http.method").cloned(),
            http.get("http.url").cloned(),
            http.get("http.status_code").and_then(|v| v.parse().ok()),
            http.get("http.status_text").cloned(),
            http.get("http.response_content_length").and_then(|v| v.parse().ok()),
            http.get("http.host").cloned(),
            http.get("http.scheme").cloned(),
            http.get("http.user_agent").cloned(),
            Some(serde_json::to_value(&http_with_orig_trace).unwrap_or_default()),
        );

        span_active.insert(&txn).await?;
    }

    txn.commit().await?;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    get,
    path = "/api/transactions",
    summary = "transaction 가져오기",
    params(
        ("page" = i32, Query, description = "Page number", example = 1),
        ("size" = i32, Query, description = "Page size", example = 10),
    ),
    responses(
        (status = 200, description = "Transactions retrieved successfully", body = PaginationResponse<TransactionResponse>),
        (status = 400, description = "Invalid request", body = AppError),
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Trace"
)]
#[get("/transactions")]
pub async fn get_transactions(
    query: web::Query<TransactionListQuery>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    let page = query.page;
    let size = query.size;

    let offset = ((page - 1) * size) as u64;

    let total = transaction::Entity::find()
        .count(db.as_ref())
        .await?;

    let transactions = transaction::Entity::find()
        .order_by_desc(transaction::Column::StartTimestamp)
        .offset(offset)
        .limit(size as u64)
        .all(db.as_ref())
        .await?;

    let transaction_responses: Vec<TransactionResponse> = transactions
        .into_iter()
        .map(TransactionResponse::from)
        .collect();

    let response = PaginationResponse::new(
        transaction_responses,
        page,
        size,
        total as i64,
    );

    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/api/transactions/{id}/spans",
    summary = "transaction spans 가져오기",
    params(
        ("id" = i32, Path, description = "Transaction ID"),
    ),
    responses(
        (status = 200, description = "Transaction spans retrieved successfully", body = TransactionWithSpansResponse),
        (status = 404, description = "Transaction not found", body = AppError),
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Trace"
)]
#[get("/transactions/{id}/spans")]
pub async fn get_transaction_spans(
    path: web::Path<String>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    let trace_id = path.into_inner();

    let transaction = transaction::Entity::find()
        .filter(transaction::Column::TraceId.eq(trace_id.clone()))
        .one(db.as_ref())
        .await?
        .ok_or_else(|| AppError::not_found(ErrorCode::TransactionNotFound))?;

    let spans = span::Entity::find()
        .filter(span::Column::TransactionId.eq(transaction.id))
        .order_by(span::Column::StartTimestamp, Order::Asc)
        .all(db.as_ref())
        .await?;

    let transaction_response = TransactionResponse::from(transaction);
    let span_responses: Vec<SpanResponse> = spans
        .into_iter()
        .map(SpanResponse::from)
        .collect();

    let response = TransactionWithSpansResponse {
        transaction: transaction_response,
        spans: span_responses,
    };

    Ok(HttpResponse::Ok().json(response))
}