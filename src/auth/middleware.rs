use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    Error, HttpMessage,
};
use std::future::{Future, Ready, ready};
use std::pin::Pin;
use std::task::{Context, Poll};

use super::jwt::JwtUtils;

pub struct AuthMiddleware;

// 미들웨어 팩토리
impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = req.headers().get("Authorization");

        let auth_result = match auth_header {
            Some(header_value) => {
                let auth_str = header_value.to_str().unwrap_or("");
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
                    match JwtUtils::verify_token(token) {
                        Ok(claims) => {
                            req.extensions_mut().insert(claims);
                            Ok(())
                        }
                        Err(_) => Err(ErrorUnauthorized("Invalid token")),
                    }
                } else {
                    Err(ErrorUnauthorized("Invalid Authorization header format"))
                }
            }
            None => Err(ErrorUnauthorized("Authorization header missing")),
        };

        let fut = self.service.call(req);
        Box::pin(async move {
            match auth_result {
                Ok(_) => fut.await,
                Err(e) => Err(e),
            }
        })
    }
}