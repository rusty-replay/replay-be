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
                let start = Utc
                    .timestamp_opt(
                        (span.start_time_unix_nano / 1_000_000_000) as i64,
                        (span.start_time_unix_nano % 1_000_000_000) as u32,
                    )
                    .unwrap();
                let end = Utc
                    .timestamp_opt(
                        (span.end_time_unix_nano / 1_000_000_000) as i64,
                        (span.end_time_unix_nano % 1_000_000_000) as u32,
                    )
                    .unwrap();

                let mut http = std::collections::HashMap::new();
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
    let mut by_trace: std::collections::HashMap<_, Vec<_>> = Default::default();
    for s in all_spans {
        by_trace.entry(s.0.clone()).or_default().push(s);
    }

    let txn = db.begin().await?;

    for (trace_id, spans) in by_trace {
        let start_ts = spans.iter().map(|s| s.4).min().unwrap();
        let end_ts = spans.iter().map(|s| s.5).max().unwrap();
        let duration = (end_ts.timestamp_millis() - start_ts.timestamp_millis()) as i32;

        let tx_model = transaction::ActiveModel {
            project_id: Set(/* TODO: API key → project_id 매핑 */ 1),
            trace_id: Set(trace_id.clone()),
            name: Set("unnamed".into()),
            start_timestamp: Set(start_ts),
            end_timestamp: Set(end_ts),
            duration_ms: Set(duration),
            environment: Set("production".into()),
            status: Set("ok".into()),
            tags: Set(None),
            ..Default::default()
        };
        let tx_inserted = tx_model.insert(&txn).await?;

        for (_trace, span_id, parent, name, start, end, http) in spans {
            let span_model = span::ActiveModel {
                transaction_id: Set(tx_inserted.id),
                span_id: Set(span_id),
                parent_span_id: Set(Some(parent)),
                name: Set(name),
                start_timestamp: Set(start),
                end_timestamp: Set(end),
                duration_ms: Set((end.timestamp_millis() - start.timestamp_millis()) as i32),
                http_method: Set(http.get("http.method").cloned()),
                http_url: Set(http.get("http.url").cloned()),
                http_status_code: Set(http.get("http.status_code").and_then(|v| v.parse().ok())),
                http_status_text: Set(http.get("http.status_text").cloned()),
                http_response_content_length: Set(http.get("http.response_content_length").and_then(|v| v.parse().ok())),
                http_host: Set(http.get("http.host").cloned()),
                http_scheme: Set(http.get("http.scheme").cloned()),
                http_user_agent: Set(http.get("http.user_agent").cloned()),
                attributes: Set(Some(serde_json::to_value(http).unwrap())),
                ..Default::default()
            };
            span_model.insert(&txn).await?;
        }
    }

    txn.commit().await?;

    Ok(HttpResponse::Ok().finish())
}