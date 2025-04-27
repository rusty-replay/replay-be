use crate::model::auth::Claims;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, errors::Error as JwtError};
use std::env;
use actix_web::cookie::{Cookie, SameSite};
use jsonwebtoken::errors::ErrorKind;

pub struct JwtUtils;

pub enum TokenVerifyResult {
    Valid(Claims),
    Expired,
    Invalid,
}

impl JwtUtils {
    fn get_secret() -> String {
        env::var("JWT_SECRET").expect("JWT_SECRET must be set")
    }

    pub fn generate_token(user_id: i32, role: &str) -> Result<String, JwtError> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(1))
            .expect("유효한 시간을 생성할 수 없습니다")
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            role: role.to_string(),
            exp: expiration,
            iat: Utc::now().timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(Self::get_secret().as_bytes()),
        )
    }

    pub fn generate_refresh_token(user_id: i32) -> Result<String, JwtError> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(30))
            .expect("유효한 시간을 생성할 수 없습니다")
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            role: "refresh".to_string(),
            exp: expiration,
            iat: Utc::now().timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(Self::get_secret().as_bytes()),
        )
    }

    pub fn verify_token(token: &str) -> TokenVerifyResult {
        match decode::<Claims>(
            token,
            &DecodingKey::from_secret(Self::get_secret().as_bytes()),
            &Validation::default(),
        ) {
            Ok(data) => TokenVerifyResult::Valid(data.claims),
            Err(err) => match *err.kind() {
                ErrorKind::ExpiredSignature => TokenVerifyResult::Expired,
                _ => TokenVerifyResult::Invalid,
            },
        }
    }
}

pub fn build_access_token_cookie(token: &str) -> Cookie<'_> {
    Cookie::build("accessToken", token.to_string())
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::days(3))
        .finish()
}

pub fn build_refresh_token_cookie(token: &str) -> Cookie<'_> {
    Cookie::build("refreshToken", token.to_string())
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::days(60))
        .finish()
}