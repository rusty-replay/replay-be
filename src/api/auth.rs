use actix_web::{get, post, web, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, QueryFilter, ColumnTrait, TransactionTrait};
use sea_query::Condition;
use crate::model::global_error::{AppError, ErrorCode, ValidationFieldError};
use crate::entity::user::{self, Entity as UserEntity};
use crate::model::auth::{RegisterRequest, LoginRequest, UserResponse};
use crate::auth::jwt::{build_access_token_cookie, build_refresh_token_cookie, JwtUtils, TokenVerifyResult};

#[post("/auth/register")]
pub async fn register(
    body: web::Json<RegisterRequest>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    validate_register_request(&body.username, &body.email, &body.password)?;

    let txn = db.begin().await?;

    let existing_user = UserEntity::find()
        .filter(
            Condition::any()
                .add(user::Column::Username.eq(&body.username))
                .add(user::Column::Email.eq(&body.email))
        )
        .one(&txn)
        .await?;

    if existing_user.is_some() {
        txn.rollback().await.ok();
        return Err(AppError::bad_request(ErrorCode::DuplicateAccountEmail));
    }

    let hashed_password = hash(&body.password, DEFAULT_COST)
        .map_err(|_| AppError::internal_error(ErrorCode::InternalError))?;

    let new_user = user::ActiveModel {
        username: Set(body.username.clone()),
        email: Set(body.email.clone()),
        password: Set(hashed_password),
        role: Set("user".to_string()),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };

    let user = new_user.insert(&txn).await?;

    let access_token = JwtUtils::generate_token(user.id, &user.role)?;
    let refresh_token_str = JwtUtils::generate_refresh_token(user.id)?;

    txn.commit().await?;

    Ok(HttpResponse::Created()
        .cookie(build_access_token_cookie(&access_token))
        .cookie(build_refresh_token_cookie(&refresh_token_str))
        .finish()
    )
}

#[post("/auth/login")]
pub async fn login(
    body: web::Json<LoginRequest>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    validate_login_request(&body.email, &body.password)?;

    let txn = db.begin().await?;

    let user = UserEntity::find()
        .filter(user::Column::Email.eq(&body.email))
        .one(&txn)
        .await?
        .ok_or_else(|| AppError::bad_request(ErrorCode::InvalidEmailPwd))?;

    let is_valid = verify(&body.password, &user.password)
        .map_err(|_| AppError::internal_error(ErrorCode::InternalError))?;

    if !is_valid {
        return Err(AppError::bad_request(ErrorCode::InvalidEmailPwd));
    }

    let access_token = JwtUtils::generate_token(user.id, &user.role)?;
    let refresh_token_str = JwtUtils::generate_refresh_token(user.id)?;

    txn.commit().await?;

    Ok(HttpResponse::Ok()
        .cookie(build_access_token_cookie(&access_token))
        .cookie(build_refresh_token_cookie(&refresh_token_str))
        .finish()
    )
}

#[post("/auth/refresh")]
pub async fn refresh_token(
    req: actix_web::HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    let refresh_token_cookie = req.cookie("refreshToken")
        .ok_or_else(|| AppError::unauthorized(ErrorCode::InvalidAuthToken))?;

    match JwtUtils::verify_token(refresh_token_cookie.value()) {
        TokenVerifyResult::Valid(claims) => {
            if claims.role != "refresh" {
                return Err(AppError::unauthorized(ErrorCode::NotRefreshToken));
            }

            let user_id = claims.sub.parse::<i32>()
                .map_err(|_| AppError::internal_error(ErrorCode::InternalError))?;

            let user = UserEntity::find_by_id(user_id)
                .one(db.get_ref())
                .await?
                .ok_or_else(|| AppError::not_found(ErrorCode::MemberNotFound))?;

            let new_access_token = JwtUtils::generate_token(user.id, &user.role)?;

            Ok(HttpResponse::Ok()
                .cookie(build_access_token_cookie(&new_access_token))
                .finish())
        }
        TokenVerifyResult::Expired | TokenVerifyResult::Invalid => {
            Err(AppError::unauthorized(ErrorCode::InvalidRefreshToken))
        }
    }
}

#[get("/auth/me")]
pub async fn get_me(
    db: web::Data<DatabaseConnection>,
    user_id: web::ReqData<i32>,
) -> Result<HttpResponse, AppError> {
    let txn = db.begin().await?;

    let user = UserEntity::find_by_id(*user_id)
        .one(&txn)
        .await?
        .ok_or_else(|| AppError::not_found(ErrorCode::MemberNotFound))?;

    txn.commit().await?;

    Ok(HttpResponse::Ok().json(UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
        created_at: user.created_at.into(),
    }))
}


fn validate_login_request(email: &str, password: &str) -> Result<(), AppError> {
    let mut errors = Vec::new();

    if email.trim().is_empty() {
        errors.push(ValidationFieldError {
            field: "email".to_string(),
            message: "이메일은 필수입니다.".to_string(),
        });
    }

    if password.len() < 8 {
        errors.push(ValidationFieldError {
            field: "password".to_string(),
            message: "비밀번호는 최소 8자 이상이어야 합니다.".to_string(),
        });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::ValidationError(errors))
    }
}

fn validate_register_request(username: &str, email: &str, password: &str) -> Result<(), AppError> {
    let mut errors = Vec::new();

    if username.trim().is_empty() {
        errors.push(ValidationFieldError {
            field: "username".to_string(),
            message: "사용자명은 필수입니다.".to_string(),
        });
    }

    if email.trim().is_empty() {
        errors.push(ValidationFieldError {
            field: "email".to_string(),
            message: "이메일은 필수입니다.".to_string(),
        });
    } else if !email.contains('@') {
        errors.push(ValidationFieldError {
            field: "email".to_string(),
            message: "유효한 이메일 형식이 아닙니다.".to_string(),
        });
    }

    if password.len() < 8 {
        errors.push(ValidationFieldError {
            field: "password".to_string(),
            message: "비밀번호는 최소 8자 이상이어야 합니다.".to_string(),
        });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::ValidationError(errors))
    }
}

