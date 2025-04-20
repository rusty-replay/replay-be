use actix_web::{get, post, web, HttpResponse, Responder, http::header};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, QueryFilter, ColumnTrait};
use sea_query::Condition;
use crate::entity::user::{self, Entity as UserEntity};
use crate::model::auth::{RegisterRequest, LoginRequest, AuthResponse, RefreshTokenRequest, UserResponse, Claims};
use crate::auth::jwt::JwtUtils;

#[post("/auth/register")]
pub async fn register(
    body: web::Json<RegisterRequest>,
    db: web::Data<DatabaseConnection>,
) -> impl Responder {
    let existing_user = UserEntity::find()
        .filter(
            Condition::any()
                .add(user::Column::Username.eq(&body.username))
                .add(user::Column::Email.eq(&body.email))
        )
        .one(db.get_ref())
        .await;

    match existing_user {
        Ok(Some(_)) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Username or email already exists"
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error"
            }));
        }
        _ => {}
    }

    let hashed_password = match hash(&body.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Error hashing password"
            }));
        }
    };

    let new_user = user::ActiveModel {
        username: Set(body.username.clone()),
        email: Set(body.email.clone()),
        password: Set(hashed_password),
        role: Set("user".to_string()),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };

    let user_result = new_user.insert(db.get_ref()).await;
    match user_result {
        Ok(user) => {
            let token = JwtUtils::generate_token(user.id, &user.role).unwrap();
            let r_token = JwtUtils::generate_refresh_token(user.id).unwrap();

            HttpResponse::Created().json(AuthResponse {
                token,
                refresh_token: r_token,
                user_id: user.id,
                username: user.username,
                email: user.email,
                role: user.role,
            })
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create user"
            }))
        }
    }
}

#[post("/auth/login")]
pub async fn login(
    body: web::Json<LoginRequest>,
    db: web::Data<DatabaseConnection>,
) -> impl Responder {
    let txn = db.begin().await?;

    let user_result = UserEntity::find()
        .filter(user::Column::Email.eq(&body.email))
        .one(db.get_ref())
        .await;

    let user = match user_result {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid credentials"
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error"
            }));
        }
    };

    let is_valid = match verify(&body.password, &user.password) {
        Ok(valid) => valid,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Error verifying password"
            }));
        }
    };

    if !is_valid {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid credentials"
        }));
    }

    let token = JwtUtils::generate_token(user.id, &user.role).unwrap();
    let r_token = JwtUtils::generate_refresh_token(user.id).unwrap();

    HttpResponse::Ok().json(AuthResponse {
        token,
        refresh_token: r_token,
        user_id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
    })
}

#[post("/auth/refresh")]
pub async fn refresh_token(
    body: web::Json<RefreshTokenRequest>,
    db: web::Data<DatabaseConnection>,
) -> impl Responder {
    let claims = match JwtUtils::verify_token(&body.refresh_token) {
        Ok(claims) => claims,
        Err(_) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid refresh token"
            }));
        }
    };

    if claims.role != "refresh" {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Not a refresh token"
        }));
    }

    let user_id = match claims.sub.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Invalid user ID in token"
            }));
        }
    };

    let user_result = UserEntity::find_by_id(user_id).one(db.get_ref()).await;
    let user = match user_result {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "User not found"
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error"
            }));
        }
    };

    let new_token = JwtUtils::generate_token(user.id, &user.role).unwrap();

    HttpResponse::Ok().json(serde_json::json!({
        "token": new_token
    }))
}

#[get("/auth/me")]
pub async fn get_me(
    db: web::Data<DatabaseConnection>,
    req_claims: web::ReqData<Claims>,
) -> impl Responder {
    let claims = req_claims.into_inner();

    let user_id = match claims.sub.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Invalid user ID in token"
            }));
        }
    };

    let user_result = UserEntity::find_by_id(user_id).one(db.get_ref()).await;
    match user_result {
        Ok(Some(user)) => {
            HttpResponse::Ok().json(UserResponse {
                id: user.id,
                username: user.username,
                email: user.email,
                role: user.role,
                created_at: user.created_at.into(),
            })
        }
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "User not found"
            }))
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error"
            }))
        }
    }
}
