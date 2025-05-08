use actix_web::{post, web, HttpResponse, Responder};
use bytes::Bytes;
use prost::Message;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_proto::tonic::common::v1::any_value::Value as AnyValueKind;

#[post("/traces")]
async fn receive_traces(body: Bytes) -> impl Responder {
    let req = match ExportTraceServiceRequest::decode(body.as_ref()) {
        Ok(r) => r,
        Err(err) => {
            eprintln!("Failed to decode OTLP Protobuf: {}", err);
            return HttpResponse::BadRequest().body("invalid protobuf");
        }
    };

    for resource_spans in req.resource_spans {
        for scope_spans in resource_spans.scope_spans {
            for span in &scope_spans.spans {
                let attrs: Vec<(String, Option<String>)> = span
                    .attributes
                    .iter()
                    .map(|kv| {
                        let val = kv
                            .value
                            .as_ref()
                            .and_then(|any| any.value.as_ref())
                            .and_then(|kind| match kind {
                                AnyValueKind::StringValue(s) => Some(s.clone()),
                                AnyValueKind::IntValue(i)    => Some(i.to_string()),
                                AnyValueKind::DoubleValue(d) => Some(d.to_string()),
                                AnyValueKind::BoolValue(b)   => Some(b.to_string()),
                                _ => None,
                            });
                        (kv.key.clone(), val)
                    })
                    .collect();

                println!("Span attrs: {:?}", attrs);
                println!("Full span:\n{:#?}", span);
            }
        }
    }

    HttpResponse::Ok().finish()
}