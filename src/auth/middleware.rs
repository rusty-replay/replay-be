use actix_web::{
    Error, HttpMessage,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    body::MessageBody,
    http::Method,
    HttpResponse,
};
use actix_web::body::BoxBody;
use actix_web::ResponseError;
use crate::model::global_error::{AppError, ErrorCode};
use super::jwt::{build_access_token_cookie, JwtUtils, TokenVerifyResult};

pub async fn auth_middleware(
    mut req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    if req.method() == Method::OPTIONS {
        return Ok(req.into_response(HttpResponse::Ok().finish().map_into_boxed_body()));
    }

    if let Some(access_cookie) = req.cookie("accessToken") {
        let token = access_cookie.value();

        match JwtUtils::verify_token(token) {
            TokenVerifyResult::Valid(claims) => {
                req.extensions_mut().insert(claims.clone());
                if let Ok(user_id) = claims.sub.parse::<i32>() {
                    req.extensions_mut().insert(user_id);
                }
                return next.call(req).await;
            }
            TokenVerifyResult::Expired => {
                if let Some(refresh_cookie) = req.cookie("refreshToken") {
                    let refresh_token = refresh_cookie.value();
                    return match JwtUtils::verify_token(refresh_token) {
                        TokenVerifyResult::Valid(refresh_claims) if refresh_claims.role == "refresh" => {
                            let user_id = refresh_claims.sub.parse::<i32>()
                                .map_err(|_| AppError::internal_error(ErrorCode::InternalError))?;

                            let new_access_token = JwtUtils::generate_token(user_id, "user")
                                .map_err(|_| AppError::internal_error(ErrorCode::InternalError))?;

                            let mut res = next.call(req).await?;
                            res.response_mut()
                                .add_cookie(&build_access_token_cookie(&new_access_token))
                                .ok();
                            Ok(res)
                        }
                        TokenVerifyResult::Expired => {
                            // Response를 직접 만들어서 반환
                            // 안 그러면 custom error(AppError)로 처리 안 됨
                            let resp = AppError::unauthorized(ErrorCode::ExpiredRefreshToken).error_response();
                            return Ok(req.into_response(resp.map_into_boxed_body()));
                        }
                        _ => {
                            let resp = AppError::unauthorized(ErrorCode::InvalidRefreshToken).error_response();
                            return Ok(req.into_response(resp.map_into_boxed_body()));
                        }
                    }
                }
                let resp = AppError::unauthorized(ErrorCode::ExpiredAuthToken).error_response();
                return Ok(req.into_response(resp.map_into_boxed_body()));
            }
            TokenVerifyResult::Invalid => {
                let resp = AppError::unauthorized(ErrorCode::InvalidAuthToken).error_response();
                return Ok(req.into_response(resp.map_into_boxed_body()));
            }
        }
    } else {
        let resp = AppError::unauthorized(ErrorCode::InvalidAuthToken).error_response();
        return Ok(req.into_response(resp.map_into_boxed_body()));
    }
}
