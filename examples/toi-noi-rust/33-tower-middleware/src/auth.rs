//! JWT HS256 và extractor `AuthUser`: xác thực nằm ở kiểu tham số của handler.

use std::marker::PhantomData;
use std::sync::Arc;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::problem::ApiError;

/// `aud` mà service sổ cái chấp nhận. Token cấp cho service khác bị từ chối.
pub const AUDIENCE: &str = "ledger-api";

/// Độ lệch đồng hồ cho phép khi kiểm `exp`. Mặc định của thư viện là 60 giây; ta hạ xuống 5.
pub const LEEWAY_SECS: u64 = 5;

/// Nội dung token. Thiếu bất kỳ trường nào là token hỏng.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Người gọi (tài khoản dịch vụ hoặc nhân viên).
    pub sub: String,
    /// Vai trò, ví dụ `["finance"]`.
    pub roles: Vec<String>,
    /// Đối tượng nhận token.
    pub aud: String,
    /// Hết hạn, giây kể từ epoch.
    pub exp: u64,
}

/// Vai trò nghiệp vụ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Kế toán/tài chính: ghi bút toán và đọc số dư.
    Finance,
    /// Chăm sóc khách hàng: chỉ đọc số dư.
    Cs,
}

impl Role {
    fn parse(raw: &str) -> Option<Role> {
        match raw {
            "finance" => Some(Role::Finance),
            "cs" => Some(Role::Cs),
            _ => None, // vai trò lạ không cấp quyền gì, nhưng cũng không làm hỏng token
        }
    }
}

/// Khoá và quy tắc kiểm tra token. Giữ sau `Arc`, dùng chung cho mọi request.
pub struct AuthConfig {
    encoding: EncodingKey,
    decoding: DecodingKey,
    validation: Validation,
}

impl AuthConfig {
    /// Khoá lab sinh tại chỗ mỗi lần chạy từ hai UUID v4: chỉ là giá trị lab, không phải secret.
    pub fn for_lab() -> AuthConfig {
        let secret = format!(
            "{}{}",
            uuid::Uuid::new_v4().simple(),
            uuid::Uuid::new_v4().simple()
        );
        AuthConfig::with_secret(secret.as_bytes())
    }

    /// Dựng từ khoá cho trước; kiểm tra: HS256, `exp`/`aud`/`sub` bắt buộc, `aud` phải khớp.
    pub fn with_secret(secret: &[u8]) -> AuthConfig {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = LEEWAY_SECS;
        validation.set_audience(&[AUDIENCE]);
        // Mặc định chỉ bắt buộc `exp`: token không có `aud` sẽ lọt qua nếu không liệt kê ở đây.
        validation.set_required_spec_claims(&["exp", "aud", "sub"]);
        AuthConfig {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
            validation,
        }
    }

    /// Ký token (chỉ lab dùng để tự cấp token cho test; service thật nhận token từ nơi khác).
    pub fn issue(&self, claims: &Claims) -> String {
        encode(&Header::new(Algorithm::HS256), claims, &self.encoding).expect("ký HS256 không lỗi")
    }

    /// Ký claims bất kỳ với thuật toán tuỳ chọn (lab dùng để dựng token hỏng có chủ đích).
    pub fn sign<T: Serialize>(&self, alg: Algorithm, claims: &T) -> String {
        encode(&Header::new(alg), claims, &self.encoding).expect("ký không lỗi")
    }

    /// Như [`AuthConfig::issue`] nhưng chọn thuật toán (để dựng token ký HS512).
    pub fn issue_alg(&self, alg: Algorithm, claims: &Claims) -> String {
        self.sign(alg, claims)
    }

    /// Kiểm chữ ký, thuật toán, `exp`, `aud`. Trả về lý do dạng chữ để ghi log (không chứa token).
    pub fn verify(&self, token: &str) -> Result<Claims, String> {
        decode::<Claims>(token, &self.decoding, &self.validation)
            .map(|data| data.claims)
            .map_err(|err| format!("{:?}", err.kind()))
    }
}

/// Giây hiện tại kể từ epoch, dùng để đặt `exp`.
pub fn now_secs() -> u64 {
    u64::try_from(jiff::Timestamp::now().as_second()).expect("sau 1970")
}

/// Người gọi đã xác thực. Handler nào khai báo tham số này thì bắt buộc có token hợp lệ.
#[derive(Debug, Clone)]
pub struct AuthUser {
    /// Giá trị `sub` của token.
    pub sub: String,
    roles: Vec<Role>,
}

impl AuthUser {
    /// Dựng từ claims đã kiểm (dùng cho test).
    pub fn from_claims(claims: &Claims) -> AuthUser {
        AuthUser {
            sub: claims.sub.clone(),
            roles: claims.roles.iter().filter_map(|r| Role::parse(r)).collect(),
        }
    }

    /// Phân quyền: 403 nếu người gọi không có vai trò nào trong `allowed`.
    pub fn require(&self, allowed: &'static [Role]) -> Result<(), ApiError> {
        if self.roles.iter().any(|r| allowed.contains(r)) {
            Ok(())
        } else {
            Err(ApiError::Forbidden { needs: allowed })
        }
    }
}

/// Chính sách quyền: danh sách vai trò được phép, khai báo ở kiểu.
pub trait Policy {
    /// Vai trò nào cũng đủ nếu nằm trong danh sách này.
    const ALLOWED: &'static [Role];
}

/// Người gọi đã xác thực VÀ có vai trò thuộc chính sách `P`. Kiểm quyền nằm trong extractor,
/// nên chạy trước extractor body: 403 đến trước lỗi parse JSON.
pub struct Authorized<P: Policy> {
    /// Người gọi.
    pub user: AuthUser,
    policy: PhantomData<P>,
}

impl<S, P> FromRequestParts<S> for Authorized<P>
where
    Arc<AuthConfig>: FromRef<S>,
    S: Send + Sync,
    P: Policy,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, ApiError> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        user.require(P::ALLOWED)?;
        Ok(Authorized {
            user,
            policy: PhantomData,
        })
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    Arc<AuthConfig>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, ApiError> {
        let config = Arc::<AuthConfig>::from_ref(state);
        let value = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or(ApiError::Unauthenticated)?
            .to_str()
            .map_err(|_| ApiError::Unauthenticated)?;
        // Scheme không phân biệt hoa thường (RFC 9110 mục 11.1).
        let token = match value.split_once(' ') {
            Some((scheme, token)) if scheme.eq_ignore_ascii_case("Bearer") => token.trim(),
            _ => return Err(ApiError::Unauthenticated),
        };
        match config.verify(token) {
            Ok(claims) => Ok(AuthUser::from_claims(&claims)),
            Err(reason) => {
                // Chỉ ghi loại lỗi. Không ghi token, không ghi claim.
                tracing::warn!(%reason, "từ chối token");
                Err(ApiError::Unauthenticated)
            }
        }
    }
}
