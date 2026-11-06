//! Interceptor phía server: đọc `authorization` và `x-request-id`, ghi `grpc-timeout` đã thấy.

use std::time::Duration;

use tonic::metadata::MetadataValue;
use tonic::{Request, Status};
use uuid::Uuid;

/// Token của lab. Giá trị giả để chạy thử, không phải secret thật.
pub const LAB_TOKEN: &str = "lab-token-khong-phai-bi-mat";

/// Thông tin interceptor rút từ header và nhét vào `extensions` của request.
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// `x-request-id` của client; nếu thiếu, server tự sinh.
    pub request_id: String,
    /// Giá trị thô của header `grpc-timeout` (ví dụ `100m`), nếu client có đặt deadline.
    pub grpc_timeout: Option<String>,
}

impl RequestContext {
    /// Ngân sách thời gian cho handler: `grpc-timeout` trừ một khoảng dự trữ để kịp trả lời.
    pub fn budget(&self) -> Option<Duration> {
        let total = parse_grpc_timeout(self.grpc_timeout.as_deref()?)?;
        Some(total.saturating_sub(RESERVE))
    }
}

/// Dự trữ để handler tự trả `DEADLINE_EXCEEDED` trước khi tầng timeout của tonic cắt ngang bằng `CANCELLED`.
pub const RESERVE: Duration = Duration::from_millis(25);

/// Đọc `grpc-timeout` theo đặc tả gRPC over HTTP/2: tối đa 8 chữ số rồi một đơn vị `H M S m u n`.
pub fn parse_grpc_timeout(raw: &str) -> Option<Duration> {
    let unit = raw.chars().last()?;
    let digits = &raw[..raw.len() - unit.len_utf8()];
    if digits.is_empty() || digits.len() > 8 {
        return None;
    }
    let n: u64 = digits.parse().ok()?;
    match unit {
        'H' => Some(Duration::from_secs(n * 3600)),
        'M' => Some(Duration::from_secs(n * 60)),
        'S' => Some(Duration::from_secs(n)),
        'm' => Some(Duration::from_millis(n)),
        'u' => Some(Duration::from_micros(n)),
        'n' => Some(Duration::from_nanos(n)),
        _ => None,
    }
}

/// Dựng interceptor kiểm token `Bearer <token>`. Closure `FnMut(Request<()>) -> Result<Request<()>, Status>`
/// tự động là `tonic::service::Interceptor`; phải `Clone` vì tonic nhân bản service cho mỗi kết nối.
pub fn auth_interceptor(
    token: &str,
) -> impl FnMut(Request<()>) -> Result<Request<()>, Status> + Clone + use<> {
    let expected = format!("Bearer {token}");
    move |mut req: Request<()>| {
        match req
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
        {
            Some(v) if v == expected => {}
            Some(_) => return Err(Status::unauthenticated("token không đúng")),
            None => return Err(Status::unauthenticated("thiếu header authorization")),
        }
        let header = |name: &str| {
            req.metadata()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
        };
        let ctx = RequestContext {
            request_id: header("x-request-id").unwrap_or_else(|| Uuid::new_v4().to_string()),
            grpc_timeout: header("grpc-timeout"),
        };
        req.extensions_mut().insert(ctx);
        Ok(req)
    }
}

/// Giá trị metadata ASCII từ chuỗi; chuỗi có ký tự cấm trong header thì trả `internal`.
pub fn ascii(value: &str) -> Result<MetadataValue<tonic::metadata::Ascii>, Status> {
    value
        .parse()
        .map_err(|_| Status::internal("giá trị metadata không hợp lệ"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request_with(headers: &[(&'static str, &str)]) -> Request<()> {
        let mut req = Request::new(());
        for (name, value) in headers {
            req.metadata_mut().insert(*name, value.parse().unwrap());
        }
        req
    }

    #[test]
    fn parses_every_grpc_timeout_unit() {
        assert_eq!(parse_grpc_timeout("2H"), Some(Duration::from_secs(7200)));
        assert_eq!(parse_grpc_timeout("3M"), Some(Duration::from_secs(180)));
        assert_eq!(parse_grpc_timeout("5S"), Some(Duration::from_secs(5)));
        assert_eq!(parse_grpc_timeout("100m"), Some(Duration::from_millis(100)));
        // Đây là dạng tonic client gửi cho 100 ms.
        assert_eq!(
            parse_grpc_timeout("100000u"),
            Some(Duration::from_millis(100))
        );
        assert_eq!(parse_grpc_timeout("7n"), Some(Duration::from_nanos(7)));
    }

    #[test]
    fn rejects_malformed_grpc_timeout() {
        for raw in ["", "m", "100", "100x", "123456789m", "-5S", "1.5S"] {
            assert_eq!(parse_grpc_timeout(raw), None, "{raw:?}");
        }
    }

    #[test]
    fn budget_keeps_a_reserve() {
        let ctx = |raw: &str| RequestContext {
            request_id: String::new(),
            grpc_timeout: Some(raw.to_owned()),
        };
        assert_eq!(ctx("300m").budget(), Some(Duration::from_millis(275)));
        assert_eq!(ctx("10m").budget(), Some(Duration::ZERO));
        assert_eq!(
            RequestContext {
                request_id: String::new(),
                grpc_timeout: None
            }
            .budget(),
            None
        );
    }

    #[test]
    fn rejects_missing_or_wrong_token() {
        let mut check = auth_interceptor("t");
        let missing = check(request_with(&[])).unwrap_err();
        let wrong = check(request_with(&[("authorization", "Bearer x")])).unwrap_err();
        let no_scheme = check(request_with(&[("authorization", "t")])).unwrap_err();
        for status in [missing, wrong, no_scheme] {
            assert_eq!(status.code(), tonic::Code::Unauthenticated);
        }
    }

    #[test]
    fn keeps_request_id_and_generates_one_when_absent() {
        let mut check = auth_interceptor("t");
        let given = check(request_with(&[
            ("authorization", "Bearer t"),
            ("x-request-id", "abc"),
            ("grpc-timeout", "5S"),
        ]))
        .unwrap();
        let ctx = given.extensions().get::<RequestContext>().unwrap();
        assert_eq!(ctx.request_id, "abc");
        assert_eq!(ctx.grpc_timeout.as_deref(), Some("5S"));

        let generated = check(request_with(&[("authorization", "Bearer t")])).unwrap();
        let ctx = generated.extensions().get::<RequestContext>().unwrap();
        assert_eq!(ctx.request_id.len(), 36, "UUID dạng chuỗi");
        assert_eq!(ctx.grpc_timeout, None);
    }
}
