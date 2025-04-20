use actix_web::{get, post, web, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, QueryFilter, ColumnTrait, TransactionTrait};
use sea_query::Condition;
use crate::model::global_error::{AppError, ErrorCode};
use crate::entity::user::{self, Entity as UserEntity};
use crate::model::auth::{RegisterRequest, LoginRequest, AuthResponse, RefreshTokenRequest, UserResponse, Claims};
use crate::auth::jwt::JwtUtils;

#[post("/auth/register")]
pub async fn register(
    body: web::Json<RegisterRequest>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    let txn = db.begin().await
        .map_err(|e| AppError::with_detail(ErrorCode::DatabaseError, format!("트랜잭션 시작 실패: {}", e)))?;

    let existing_user = UserEntity::find()
        .filter(
            Condition::any()
                .add(user::Column::Username.eq(&body.username))
                .add(user::Column::Email.eq(&body.email))
        )
        .one(&txn)
        .await
        .map_err(|e| AppError::with_detail(ErrorCode::DatabaseError, format!("사용자 조회 실패: {}", e)))?;

    if existing_user.is_some() {
        let _ = txn.rollback().await;
        return Err(AppError::new(ErrorCode::DuplicateAccountEmail));
    }

    let hashed_password = hash(&body.password, DEFAULT_COST)
        .map_err(|_| AppError::new(ErrorCode::InternalError))?;

    let new_user = user::ActiveModel {
        username: Set(body.username.clone()),
        email: Set(body.email.clone()),
        password: Set(hashed_password),
        role: Set("user".to_string()),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };

    let user = new_user.insert(&txn).await
        .map_err(|e| AppError::with_detail(ErrorCode::DatabaseError, format!("사용자 생성 실패: {}", e)))?;

    let token = JwtUtils::generate_token(user.id, &user.role)
        .map_err(|_| AppError::new(ErrorCode::TokenGenerationFailed))?;
    let r_token = JwtUtils::generate_refresh_token(user.id)
        .map_err(|_| AppError::new(ErrorCode::TokenGenerationFailed))?;

    txn.commit().await
        .map_err(|e| AppError::with_detail(ErrorCode::DatabaseError, format!("트랜잭션 커밋 실패: {}", e)))?;

    Ok(HttpResponse::Created().json(AuthResponse {
        token,
        refresh_token: r_token,
        user_id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
    }))
}

#[post("/auth/login")]
pub async fn login(
    body: web::Json<LoginRequest>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    let txn = db.begin().await
        .map_err(|e| AppError::with_detail(ErrorCode::DatabaseError, format!("트랜잭션 시작 실패: {}", e)))?;

    let user_option = UserEntity::find()
        .filter(user::Column::Email.eq(&body.email))
        .one(&txn)
        .await
        .map_err(|e| AppError::with_detail(ErrorCode::DatabaseError, format!("사용자 조회 실패: {}", e)))?;

    let user = match user_option {
        Some(user) => user,
        None => return Err(AppError::new(ErrorCode::InvalidEmailPwd)),
    };

    let is_valid = verify(&body.password, &user.password)
        .map_err(|_| AppError::new(ErrorCode::InternalError))?;

    if !is_valid {
        return Err(AppError::new(ErrorCode::InvalidEmailPwd));
    }

    let token = JwtUtils::generate_token(user.id, &user.role)
        .map_err(|_| AppError::new(ErrorCode::TokenGenerationFailed))?;

    let r_token = JwtUtils::generate_refresh_token(user.id)
        .map_err(|_| AppError::new(ErrorCode::TokenGenerationFailed))?;

    txn.commit().await
        .map_err(|e| AppError::with_detail(ErrorCode::DatabaseError, format!("트랜잭션 커밋 실패: {}", e)))?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        refresh_token: r_token,
        user_id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
    }))
}

#[post("/auth/refresh")]
pub async fn refresh_token(
    body: web::Json<RefreshTokenRequest>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, AppError> {
    let claims = JwtUtils::verify_token(&body.refresh_token)
        .map_err(|_| AppError::new(ErrorCode::InvalidRefreshToken))?;

    if claims.role != "refresh" {
        return Err(AppError::new(ErrorCode::NotRefreshToken));
    }

    let user_id = claims.sub.parse::<i32>()
        .map_err(|_| AppError::new(ErrorCode::InternalError))?;

    let user_option = UserEntity::find_by_id(user_id)
        .one(db.get_ref())
        .await
        .map_err(|e| AppError::with_detail(ErrorCode::DatabaseError, format!("사용자 조회 실패: {}", e)))?;

    let user = match user_option {
        Some(user) => user,
        None => return Err(AppError::new(ErrorCode::MemberNotFound)),
    };

    let new_token = JwtUtils::generate_token(user.id, &user.role)
        .map_err(|_| AppError::new(ErrorCode::TokenGenerationFailed))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": new_token
    })))
}

#[get("/auth/me")]
pub async fn get_me(
    db: web::Data<DatabaseConnection>,
    req_claims: web::ReqData<Claims>,
) -> Result<HttpResponse, AppError> {
    let txn = db.begin().await
        .map_err(|e| AppError::with_detail(ErrorCode::DatabaseError, format!("트랜잭션 시작 실패: {}", e)))?;

    let claims = req_claims.into_inner();

    let user_id = claims.sub.parse::<i32>()
        .map_err(|_| AppError::new(ErrorCode::InternalError))?;

    let user_options = UserEntity::find_by_id(user_id)
        .one(&txn)
        .await
        .map_err(|e| AppError::with_detail(ErrorCode::DatabaseError, format!("사용자 조회 실패: {}", e)))?;

    let user = match user_options {
        Some(user) => user,
        None => return Err(AppError::new(ErrorCode::MemberNotFound)),
    };

    txn.commit().await
        .map_err(|e| AppError::with_detail(ErrorCode::DatabaseError, format!("트랜잭션 커밋 실패: {}", e)))?;

    Ok(HttpResponse::Ok().json(UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
        created_at: user.created_at.into(),
    }))
}