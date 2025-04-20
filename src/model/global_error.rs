use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    // 인증 관련 에러
    InvalidCredentials,
    TokenExpired,
    InvalidToken,
    Unauthorized,

    // 리소스 관련 에러
    ResourceNotFound,
    UserNotFound,
    ProjectNotFound,
    IssueNotFound,

    // 유효성 검사 에러
    ValidationFailed,
    InvalidInput,

    // 데이터베이스 에러
    DatabaseError,
    DuplicateEntry,

    // 서버 에러
    InternalServerError,

    // 요청 관련 에러
    RateLimitExceeded,
    BadRequest,
}

impl ErrorCode {
    pub fn message(&self) -> &'static str {
        match self {
            ErrorCode::InvalidCredentials => "Invalid email or password",
            ErrorCode::TokenExpired => "Token has expired",
            ErrorCode::InvalidToken => "Invalid token",
            ErrorCode::Unauthorized => "Unauthorized access",

            ErrorCode::ResourceNotFound => "Resource not found",
            ErrorCode::UserNotFound => "User not found",
            ErrorCode::ProjectNotFound => "Project not found",
            ErrorCode::IssueNotFound => "Issue not found",

            ErrorCode::ValidationFailed => "Validation failed",
            ErrorCode::InvalidInput => "Invalid input data",

            ErrorCode::DatabaseError => "Database error occurred",
            ErrorCode::DuplicateEntry => "Resource already exists",

            ErrorCode::InternalServerError => "Internal server error",

            ErrorCode::RateLimitExceeded => "Too many requests",
            ErrorCode::BadRequest => "Bad request",
        }
    }

    pub fn status_code(&self) -> u16 {
        match self {
            ErrorCode::InvalidCredentials => 401,
            ErrorCode::TokenExpired => 401,
            ErrorCode::InvalidToken => 401,
            ErrorCode::Unauthorized => 401,

            ErrorCode::ResourceNotFound => 404,
            ErrorCode::UserNotFound => 404,
            ErrorCode::ProjectNotFound => 404,
            ErrorCode::IssueNotFound => 404,

            ErrorCode::ValidationFailed => 400,
            ErrorCode::InvalidInput => 400,

            ErrorCode::DatabaseError => 500,
            ErrorCode::DuplicateEntry => 409,

            ErrorCode::InternalServerError => 500,

            ErrorCode::RateLimitExceeded => 429,
            ErrorCode::BadRequest => 400,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}