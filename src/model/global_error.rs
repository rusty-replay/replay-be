use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use thiserror::Error;
use std::fmt;
use jsonwebtoken::errors::ErrorKind;
use sea_orm::DbErr;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidAssignee,
    ValidationError,
    DuplicateAccountEmail,
    InvalidPassword,
    InvalidEmailPwd,
    NotRefreshToken,
    InvalidRefreshToken,
    InvalidApiKey,

    AuthenticationFailed,
    ExpiredAuthToken,
    InvalidAuthToken,

    NotEnoughPermission,

    MemberNotFound,
    GroupNotFound,
    ProjectNotFound,
    ErrorLogNotFound,

    DatabaseError,
    InternalError,
    TokenGenerationFailed,
    JwtInvalidToken,
    JwtExpiredToken,
    ExpiredRefreshToken,
    MissingField,

}

impl ErrorCode {
    pub fn message(&self) -> &'static str {
        match self {
            ErrorCode::InvalidAssignee => "assignee를 찾을 수 없습니다",
            ErrorCode::MissingField => "필수 요청값이 누락되었습니다",
            ErrorCode::ValidationError => "요청값 유효성 검사에 실패했습니다",
            ErrorCode::DuplicateAccountEmail => "이미 등록된 이메일입니다. 로그인해주세요",
            ErrorCode::InvalidPassword => "비밀번호는 최소 8자 이상이어야 합니다",
            ErrorCode::InvalidEmailPwd => "잘못된 자격 증명입니다",
            ErrorCode::NotRefreshToken => "잘못된 리프레시 토큰입니다",
            ErrorCode::InvalidRefreshToken => "리프레시 토큰이 유효하지 않습니다",
            ErrorCode::InvalidApiKey => "유효하지 않은 API 키입니다",
            ErrorCode::ErrorLogNotFound => "유효하지 않은 에러 로그 ID입니다",
            ErrorCode::AuthenticationFailed => "인증에 실패했습니다",
            ErrorCode::ExpiredAuthToken => "로그인 토큰이 만료되었습니다",
            ErrorCode::InvalidAuthToken => "유효하지 않은 로그인 토큰입니다",
            ErrorCode::NotEnoughPermission => "권한이 부족합니다",
            ErrorCode::MemberNotFound => "사용자를 찾을 수 없습니다",
            ErrorCode::GroupNotFound => "유효하지 않은 그룹 ID입니다",
            ErrorCode::ProjectNotFound => "유효하지 않은 프로젝트 ID입니다",
            ErrorCode::DatabaseError => "데이터베이스 오류가 발생했습니다",
            ErrorCode::InternalError => "내부 서버 오류가 발생했습니다",
            ErrorCode::TokenGenerationFailed => "토큰 생성에 실패했습니다",
            ErrorCode::JwtInvalidToken => "JWT 토큰이 유효하지 않습니다",
            ErrorCode::JwtExpiredToken => "JWT 토큰이 만료되었습니다",
            ErrorCode::ExpiredRefreshToken => "refreshToken이 만료되었습니다",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Bad Request: {0:?}")]
    BadRequest(ErrorCode),

    #[error("Unauthorized: {0:?}")]
    Unauthorized(ErrorCode),

    #[error("Forbidden: {0:?}")]
    Forbidden(ErrorCode),

    #[error("Not Found: {0:?}")]
    NotFound(ErrorCode),

    #[error("Internal Server Error: {0:?}")]
    InternalServerError(ErrorCode),

    #[error("Validation Error")]
    ValidationError(Vec<ValidationFieldError>),
}

impl AppError {
    pub fn bad_request(code: ErrorCode) -> Self {
        Self::BadRequest(code)
    }

    pub fn unauthorized(code: ErrorCode) -> Self {
        Self::Unauthorized(code)
    }

    pub fn forbidden(code: ErrorCode) -> Self {
        Self::Forbidden(code)
    }

    pub fn not_found(code: ErrorCode) -> Self {
        Self::NotFound(code)
    }

    pub fn internal_error(code: ErrorCode) -> Self {
        Self::InternalServerError(code)
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ValidationFieldError {
    pub field: String,
    pub message: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
enum ErrorResponse {
    General {
        #[serde(rename = "errorCode")]
        error_code: String,
        message: String,
    },
    Validation {
        code: String,
        message: String,
        errors: Vec<ValidationFieldError>,
    },
}

// DbErr를 AppError로 변환
impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        let err_str = err.to_string();
        if err_str.contains("Duplicate entry") {
            AppError::bad_request(ErrorCode::DuplicateAccountEmail)
        } else if err_str.contains("Record not found") {
            AppError::not_found(ErrorCode::ErrorLogNotFound)
        } else {
            AppError::internal_error(ErrorCode::DatabaseError)
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        match err.kind() {
            ErrorKind::ExpiredSignature => AppError::unauthorized(ErrorCode::JwtExpiredToken),
            _ => AppError::unauthorized(ErrorCode::JwtInvalidToken),
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::ValidationError(errors) => {
                HttpResponse::BadRequest().json(ErrorResponse::Validation {
                    code: "ValidationError".to_string(),
                    message: ErrorCode::ValidationError.message().to_string(),
                    errors: errors.clone(),
                })
            }
            _ => {
                let code = match self {
                    AppError::BadRequest(c)
                    | AppError::Unauthorized(c)
                    | AppError::Forbidden(c)
                    | AppError::NotFound(c)
                    | AppError::InternalServerError(c) => c,
                    _ => unreachable!(),
                };
                HttpResponse::build(self.status_code()).json(ErrorResponse::General {
                    error_code: format!("{:?}", code),
                    message: code.message().to_string(),
                })
            }
        }
    }
}
