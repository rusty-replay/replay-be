use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // 400 BAD REQUEST
    ValidationError,
    DuplicateAccountEmail,
    InvalidPassword,
    InvalidEmailPwd,
    NotRefreshToken,
    InvalidRefreshToken,

    // 401 UNAUTHORIZED
    AuthenticationFailed,
    ExpiredAuthToken,
    InvalidAuthToken,

    // 403 FORBIDDEN
    NotEnoughPermission,

    // 404 NOT FOUND
    MemberNotFound,
    GroupNotFound,
    ProjectNotFound,

    // 500 SERVER ERRORS
    DatabaseError,
    InternalError,
    TokenGenerationFailed,
}

impl ErrorCode {
    pub fn message(&self) -> &'static str {
        match self {
            ErrorCode::ValidationError => "유효성 검증에 실패했습니다",
            ErrorCode::DuplicateAccountEmail => "이미 등록된 이메일입니다. 로그인해주세요",
            ErrorCode::InvalidPassword => "비밀번호는 최소 8자 이상이어야 합니다",
            ErrorCode::InvalidEmailPwd => "잘못된 자격 증명입니다",
            ErrorCode::NotRefreshToken => "잘못된 리프레시 토큰입니다",
            ErrorCode::InvalidRefreshToken => "리프레시 토큰이 유효하지 않습니다",

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
        }
    }

    pub fn status_code(&self) -> actix_web::http::StatusCode {
        use actix_web::http::StatusCode;

        match self {
            ErrorCode::InvalidRefreshToken |
            ErrorCode::NotRefreshToken |
            ErrorCode::InvalidEmailPwd |
            ErrorCode::ValidationError |
            ErrorCode::DuplicateAccountEmail |
            ErrorCode::InvalidPassword => StatusCode::BAD_REQUEST,

            ErrorCode::AuthenticationFailed |
            ErrorCode::ExpiredAuthToken |
            ErrorCode::InvalidAuthToken => StatusCode::UNAUTHORIZED,

            ErrorCode::NotEnoughPermission => StatusCode::FORBIDDEN,

            ErrorCode::ProjectNotFound |
            ErrorCode::MemberNotFound |
            ErrorCode::GroupNotFound => StatusCode::NOT_FOUND,

            ErrorCode::DatabaseError |
            ErrorCode::InternalError |
            ErrorCode::TokenGenerationFailed => StatusCode::INTERNAL_SERVER_ERROR,
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
    #[error("{0}")]
    ApiError(ErrorCode, Option<String>),
}

impl AppError {
    pub fn new(code: ErrorCode) -> Self {
        AppError::ApiError(code, None)
    }

    pub fn with_detail(code: ErrorCode, detail: String) -> Self {
        AppError::ApiError(code, Some(detail))
    }
}

#[derive(serde::Serialize)]
struct ErrorResponse {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::ApiError(code, detail) => {
                let response = ErrorResponse {
                    code: format!("{:?}", code),
                    message: code.message().to_string(),
                    detail: detail.clone(),
                };

                HttpResponse::build(code.status_code())
                    .json(response)
            }
        }
    }
}