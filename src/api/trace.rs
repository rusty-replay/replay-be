use std::collections::HashMap;
use crate::entity::{span, transaction};
use crate::model::global_error::{AppError, ErrorCode};
use actix_web::{post, web, HttpResponse};
use chrono::{DateTime, TimeZone, Utc};
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_proto::tonic::common::v1::any_value::Value as AnyValueKind;
use prost::Message;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionTrait};

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

    // collect span
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

    // trace_id 별로 그룹화
    let mut by_trace: HashMap<_, Vec<_>> = Default::default();
    for s in all_spans {
        by_trace.entry(s.0.clone()).or_default().push(s);
    }

    let txn = db.begin().await?;

    for (trace_id, spans) in by_trace {
        let default_time = Utc::now();
        let start_ts = spans.iter()
            .map(|s| &s.4)
            .min()
            .unwrap_or(&default_time);

        let end_ts = spans.iter()
            .map(|s| &s.5)
            .max()
            .unwrap_or(&default_time);

        let tx_active = transaction::ActiveModel::new(
            1,
            trace_id.clone(),
            "unnamed",
            *start_ts,
            *end_ts,
            "production",
            "ok",
            None,
        );

        let tx_inserted: transaction::Model = tx_active.insert(&txn).await?;

        for (_trace, span_id, parent, name, start, end, http) in spans {
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
                Some(serde_json::to_value(&http).unwrap()),
            );

            // let span_model: span::Model = span_active.insert(&txn).await?;
             span_active.insert(&txn).await?;
        }
    }
    txn.commit().await?;
    Ok(HttpResponse::Ok().finish())
}