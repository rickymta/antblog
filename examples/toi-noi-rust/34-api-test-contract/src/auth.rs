//! Xác thực JWT HS256 tối thiểu, dạng extractor như bài 33 nhưng rút gọn: dùng `scope` thay cho vai trò.
//! 401 khi không chứng minh được ai gọi, 403 khi biết người gọi nhưng thiếu quyền.

use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::problem::{ApiError, Problem};

/// Quyền đọc số dư.
pub const SCOPE_READ: &str = "ledger:read";
/// Quyền ghi bút toán.
pub const SCOPE_WRITE: &str = "ledger:write";

/// Nội dung token: ai gọi, hết hạn khi nào, có những quyền nào (cách nhau bằng dấu cách).
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: u64,
    scope: String,
}

/// Cấu hình kiểm tra token: khoá dùng chung HS256.
pub struct AuthConfig {
    key: DecodingKey,
    validation: Validation,
}

impl AuthConfig {
    /// Tạo từ khoá bí mật dùng chung. `leeway = 0`: token hết hạn là hết hạn ngay
    /// (mặc định của thư viện cho phép lệch 60 giây).
    pub fn new(secret: &[u8]) -> AuthConfig {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 0;
        AuthConfig {
            key: DecodingKey::from_secret(secret),
            validation,
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("đồng hồ sau năm 1970")
        .as_secs()
}

/// Ký một token cho lab và test. `ttl_secs` âm cho ra token đã hết hạn.
pub fn issue_token(secret: &[u8], sub: &str, scopes: &[&str], ttl_secs: i64) -> String {
    let exp = now_secs().saturating_add_signed(ttl_secs);
    let claims = Claims {
        sub: sub.to_string(),
        exp,
        scope: scopes.join(" "),
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .expect("ký HS256 với khoá byte không thể lỗi")
}

/// Người gọi đã xác thực. Là extractor: handler nào có tham số này thì không chạy nếu chưa qua cổng.
#[derive(Debug)]
pub struct Caller {
    /// Định danh người gọi (`sub`).
    pub sub: String,
    scopes: Vec<String>,
}

impl Caller {
    /// Yêu cầu một quyền; thiếu thì 403.
    pub fn require(&self, scope: &str, instance: &str) -> Result<(), ApiError> {
        if self.scopes.iter().any(|s| s == scope) {
            Ok(())
        } else {
            Err(Problem::forbidden(scope, instance).into())
        }
    }
}

impl FromRequestParts<AppState> for Caller {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Caller, ApiError> {
        // Mọi lý do đều cùng một thân 401: không cho người gọi biết token sai ở đâu (bài 33).
        let instance = parts.uri.path().to_string();
        let unauthorized = || ApiError::from(Problem::unauthenticated(&instance));
        let value = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or_else(unauthorized)?
            .to_str()
            .map_err(|_| unauthorized())?;
        // Scheme không phân biệt hoa thường (RFC 9110 mục 11.1).
        let token = match value.split_once(' ') {
            Some((scheme, token)) if scheme.eq_ignore_ascii_case("Bearer") => token.trim(),
            _ => return Err(unauthorized()),
        };
        let data = decode::<Claims>(token, &state.auth.key, &state.auth.validation)
            .map_err(|_| unauthorized())?;
        Ok(Caller {
            sub: data.claims.sub,
            scopes: data
                .claims
                .scope
                .split_whitespace()
                .map(String::from)
                .collect(),
        })
    }
}

/// Người gọi có quyền ghi. Kiểm quyền trong extractor, đặt trước extractor body, nên 403 đến trước
/// lỗi body: như `Authorized<P>` của bài 33.
#[derive(Debug)]
pub struct Writer(pub Caller);

impl FromRequestParts<AppState> for Writer {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Writer, ApiError> {
        let caller = Caller::from_request_parts(parts, state).await?;
        caller.require(SCOPE_WRITE, parts.uri.path())?;
        Ok(Writer(caller))
    }
}
