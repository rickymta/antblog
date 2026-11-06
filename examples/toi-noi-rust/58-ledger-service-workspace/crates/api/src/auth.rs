//! JWT HS256 cơ bản (bài 33): chữ ký, thuật toán, `exp`, `aud`; phân quyền theo vai trò.
//! Dùng chung cho HTTP (extractor) và gRPC (interceptor).

use std::marker::PhantomData;
use std::sync::Arc;

use axum::extract::{FromRef, FromRequestParts, OriginalUri};
use axum::http::StatusCode;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::problem::{ApiError, Problem};

/// `aud` mà service sổ cái chấp nhận. Token cấp cho service khác bị từ chối.
pub const AUDIENCE: &str = "ledger-api";

/// Độ lệch đồng hồ cho phép khi kiểm `exp`. Mặc định của thư viện là 60 giây; ta hạ xuống 5.
pub const LEEWAY_SECS: u64 = 5;

/// Độ dài tối thiểu của khóa HS256, tính theo byte (bằng độ dài đầu ra của SHA-256).
pub const MIN_SECRET_BYTES: usize = 32;

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

/// Khóa và quy tắc kiểm tra token. Giữ sau `Arc`, dùng chung cho mọi request.
pub struct AuthConfig {
    encoding: EncodingKey,
    decoding: DecodingKey,
    validation: Validation,
}

impl AuthConfig {
    /// Dựng từ khóa cho trước; kiểm tra: HS256, `exp`/`aud`/`sub` bắt buộc, `aud` phải khớp.
    ///
    /// # Errors
    /// Khóa ngắn hơn [`MIN_SECRET_BYTES`].
    pub fn with_secret(secret: &[u8]) -> Result<AuthConfig, String> {
        if secret.len() < MIN_SECRET_BYTES {
            return Err(format!("khóa JWT cần ít nhất {MIN_SECRET_BYTES} byte"));
        }
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = LEEWAY_SECS;
        validation.set_audience(&[AUDIENCE]);
        // Mặc định chỉ bắt buộc `exp`: token không có `aud` sẽ lọt qua nếu không liệt kê ở đây.
        validation.set_required_spec_claims(&["exp", "aud", "sub"]);
        Ok(AuthConfig {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
            validation,
        })
    }

    /// Ký token (chỉ lab và test dùng để tự cấp; service thật nhận token từ nơi khác).
    pub fn issue(&self, claims: &Claims) -> String {
        self.sign(Algorithm::HS256, claims)
    }

    /// Ký claims với thuật toán tuỳ chọn (test dùng để dựng token hỏng có chủ đích).
    pub fn sign<T: Serialize>(&self, alg: Algorithm, claims: &T) -> String {
        encode(&Header::new(alg), claims, &self.encoding).expect("ký không lỗi")
    }

    /// Kiểm chữ ký, thuật toán, `exp`, `aud`. Trả về lý do dạng chữ để ghi log (không chứa token).
    ///
    /// # Errors
    /// Tên loại lỗi của `jsonwebtoken`, không kèm token hay claim.
    pub fn verify(&self, token: &str) -> Result<Claims, String> {
        decode::<Claims>(token, &self.decoding, &self.validation)
            .map(|data| data.claims)
            .map_err(|err| format!("{:?}", err.kind()))
    }
}

/// Giây hiện tại kể từ epoch, dùng để đặt `exp` khi lab và test tự cấp token.
/// Thư viện JWT kiểm `exp` bằng đồng hồ thật, nên chỗ này không đi qua `Clock`.
#[allow(clippy::disallowed_methods)]
pub fn now_secs() -> u64 {
    u64::try_from(jiff::Timestamp::now().as_second()).expect("sau 1970")
}

/// Người gọi đã xác thực.
#[derive(Debug, Clone)]
pub struct AuthUser {
    /// Giá trị `sub` của token.
    pub sub: String,
    roles: Vec<Role>,
}

impl AuthUser {
    /// Dựng từ claims đã kiểm.
    pub fn from_claims(claims: &Claims) -> AuthUser {
        AuthUser {
            sub: claims.sub.clone(),
            roles: claims.roles.iter().filter_map(|r| Role::parse(r)).collect(),
        }
    }

    /// Có ít nhất một vai trò trong `allowed` không.
    pub fn has_any(&self, allowed: &[Role]) -> bool {
        self.roles.iter().any(|r| allowed.contains(r))
    }
}

/// Tách token khỏi giá trị header `Authorization`. Scheme không phân biệt hoa thường (RFC 9110).
pub fn bearer_token(header_value: &str) -> Option<&str> {
    match header_value.split_once(' ') {
        Some((scheme, token)) if scheme.eq_ignore_ascii_case("Bearer") => Some(token.trim()),
        _ => None,
    }
}

/// Chính sách quyền: danh sách vai trò được phép, khai báo ở kiểu.
pub trait Policy {
    /// Vai trò nào cũng đủ nếu nằm trong danh sách này.
    const ALLOWED: &'static [Role];
}

/// Ghi: chỉ `finance`.
pub struct Writers;
impl Policy for Writers {
    const ALLOWED: &'static [Role] = &[Role::Finance];
}

/// Đọc: `finance` và `cs`.
pub struct Readers;
impl Policy for Readers {
    const ALLOWED: &'static [Role] = &[Role::Finance, Role::Cs];
}

/// Người gọi đã xác thực VÀ có vai trò thuộc chính sách `P`. Kiểm quyền nằm trong extractor,
/// nên chạy trước extractor body: 401 và 403 đến trước lỗi parse JSON.
pub struct Authorized<P: Policy> {
    /// Người gọi.
    pub user: AuthUser,
    policy: PhantomData<P>,
}

fn problem(parts: &Parts, status: StatusCode, slug: &str, title: &str, detail: String) -> ApiError {
    let instance = parts
        .extensions
        .get::<OriginalUri>()
        .map_or_else(|| parts.uri.path().to_string(), |o| o.0.path().to_string());
    Problem::new(status, slug, title, detail, &instance).into()
}

impl<S, P> FromRequestParts<S> for Authorized<P>
where
    Arc<AuthConfig>: FromRef<S>,
    S: Send + Sync,
    P: Policy,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, ApiError> {
        let config = Arc::<AuthConfig>::from_ref(state);
        let unauthenticated = |parts: &Parts| {
            problem(
                parts,
                StatusCode::UNAUTHORIZED,
                "unauthenticated",
                "Chưa xác thực",
                "thiếu hoặc không hợp lệ: Authorization Bearer token".to_string(),
            )
        };
        let Some(token) = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(bearer_token)
        else {
            return Err(unauthenticated(parts));
        };
        let claims = match config.verify(token) {
            Ok(claims) => claims,
            Err(reason) => {
                // Chỉ ghi loại lỗi. Không ghi token, không ghi claim.
                tracing::warn!(%reason, "từ chối token");
                return Err(unauthenticated(parts));
            }
        };
        let user = AuthUser::from_claims(&claims);
        if !user.has_any(P::ALLOWED) {
            return Err(problem(
                parts,
                StatusCode::FORBIDDEN,
                "forbidden",
                "Không đủ quyền",
                format!("cần một trong các vai trò: {:?}", P::ALLOWED),
            ));
        }
        Ok(Authorized {
            user,
            policy: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> AuthConfig {
        AuthConfig::with_secret(&[7u8; 32]).unwrap()
    }

    fn claims(roles: &[&str], aud: &str, exp: u64) -> Claims {
        Claims {
            sub: "svc-payment".to_string(),
            roles: roles.iter().map(|r| (*r).to_string()).collect(),
            aud: aud.to_string(),
            exp,
        }
    }

    #[test]
    fn valid_token_roundtrips_and_bad_ones_are_rejected() {
        let auth = config();
        let exp = now_secs() + 60;
        let good = auth.issue(&claims(&["finance"], AUDIENCE, exp));
        assert_eq!(auth.verify(&good).unwrap().sub, "svc-payment");
        // Sai audience, hết hạn, thuật toán khác, rác: đều bị từ chối.
        assert!(
            auth.verify(&auth.issue(&claims(&["finance"], "khac", exp)))
                .is_err()
        );
        assert!(
            auth.verify(&auth.issue(&claims(&["finance"], AUDIENCE, now_secs() - 3600)))
                .is_err()
        );
        let hs512 = auth.sign(Algorithm::HS512, &claims(&["finance"], AUDIENCE, exp));
        assert!(auth.verify(&hs512).is_err());
        assert!(auth.verify("khong.phai.jwt").is_err());
    }

    #[test]
    fn short_secret_is_refused() {
        assert!(AuthConfig::with_secret(b"ngan").is_err());
    }

    #[test]
    fn roles_decide_what_is_allowed() {
        let finance = AuthUser::from_claims(&claims(&["finance"], AUDIENCE, 0));
        let cs = AuthUser::from_claims(&claims(&["cs", "la-hoang"], AUDIENCE, 0));
        assert!(finance.has_any(Writers::ALLOWED) && finance.has_any(Readers::ALLOWED));
        assert!(!cs.has_any(Writers::ALLOWED) && cs.has_any(Readers::ALLOWED));
    }

    #[test]
    fn bearer_scheme_is_case_insensitive() {
        assert_eq!(bearer_token("bearer abc"), Some("abc"));
        assert_eq!(bearer_token("Basic abc"), None);
        assert_eq!(bearer_token("Bearer"), None);
    }
}
