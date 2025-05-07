use actix_web::{post, web, HttpResponse, Responder};
use bytes::Bytes;
use prost::Message;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_proto::tonic::common::v1::any_value::Value as AnyValueKind;

#[post("/traces")]
async fn receive_traces(body: Bytes) -> impl Responder {
    // Protobuf 바이트를 디코딩
    let req = match ExportTraceServiceRequest::decode(body.as_ref()) {
        Ok(r) => r,
        Err(err) => {
            eprintln!("Failed to decode OTLP Protobuf: {}", err);
            return HttpResponse::BadRequest().body("invalid protobuf");
        }
    };

    for resource_spans in req.resource_spans {
        for scope_spans in resource_spans.scope_spans {
            for span in scope_spans.spans {
                let attrs: Vec<(String, Option<String>)> = span
                    .attributes
                    .into_iter()
                    .map(|kv| {
                        let val = kv
                            .value
                            .and_then(|any| any.value)
                            .and_then(|kind| match kind {
                                AnyValueKind::StringValue(s) => Some(s),
                                AnyValueKind::IntValue(i)    => Some(i.to_string()),
                                AnyValueKind::DoubleValue(d) => Some(d.to_string()),
                                AnyValueKind::BoolValue(b)   => Some(b.to_string()),
                                _ => None,
                            });
                        (kv.key, val)
                    })
                    .collect();

                println!(
                    "Span received: name={} start={} end={} attrs={:?}",
                    span.name,
                    span.start_time_unix_nano,
                    span.end_time_unix_nano,
                    attrs,
                );
            }
        }
    }

    HttpResponse::Ok().finish()
}
