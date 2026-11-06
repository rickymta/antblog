"""Phát và kiểm token JWT; dependency `require_scope`. Không tự viết mật mã: ký và kiểm chữ ký
do PyJWT làm, băm mật khẩu do pwdlib (argon2id) làm."""

import logging
from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from typing import Annotated, Any

import jwt
from fastapi import Depends, HTTPException, Request, Security
from fastapi.security import OAuth2PasswordBearer, SecurityScopes

from app.settings import Settings

log = logging.getLogger("toinoi.auth")

ALGORITHM = "HS256"
SCOPES = {
    "customer": "Khách: báo giá, xem chuyến của mình",
    "driver": "Tài xế: xem khả năng nhận chuyến",
    "dispatcher": "Điều phối: xem hàng đợi và mọi chuyến",
}
# tokenUrl chỉ để Swagger UI biết nơi lấy token; nó không phải lúc nào cũng khớp với prefix thật.
# auto_error=False: tự trả 401 theo hợp đồng của ta (problem+json, tiếng Việt) thay vì chuỗi mặc định.
oauth2_scheme = OAuth2PasswordBearer(tokenUrl="/v1/auth/token", scopes=SCOPES, auto_error=False)

# Claim bắt buộc: thiếu một claim là token bị từ chối, không phải bỏ qua.
REQUIRED_CLAIMS = ["exp", "iat", "sub", "iss", "aud"]


class TokenError(Exception):
    """Token không dùng được. `reason` chỉ để log; client luôn nhận cùng một thông báo 401."""

    def __init__(self, reason: str) -> None:
        super().__init__(reason)
        self.reason = reason


@dataclass(frozen=True)
class Principal:
    user_id: str
    scopes: frozenset[str]


class TokenService:
    def __init__(self, settings: Settings) -> None:
        self._key = settings.jwt_secret.get_secret_value()
        self._issuer = settings.jwt_issuer
        self._audience = settings.jwt_audience
        self.ttl_seconds = settings.access_token_ttl_seconds

    def issue(self, user_id: str, scopes: frozenset[str], now: datetime | None = None) -> str:
        now = now or datetime.now(UTC)
        claims = {
            "iss": self._issuer,
            "aud": self._audience,
            "sub": user_id,
            "iat": now,
            "exp": now + timedelta(seconds=self.ttl_seconds),
            "scope": " ".join(sorted(scopes)),  # RFC 6749: scope là chuỗi cách nhau bởi dấu cách
        }
        return jwt.encode(claims, self._key, algorithm=ALGORITHM)

    def decode(self, token: str) -> Principal:
        try:
            # algorithms=[...] luôn truyền tường minh: token tự khai alg=none hay alg khác bị từ chối.
            claims: dict[str, Any] = jwt.decode(
                token,
                self._key,
                algorithms=[ALGORITHM],
                audience=self._audience,
                issuer=self._issuer,
                options={"require": REQUIRED_CLAIMS},
            )
        except jwt.ExpiredSignatureError as exc:
            raise TokenError("expired") from exc
        except jwt.InvalidSignatureError as exc:
            raise TokenError("bad_signature") from exc
        except jwt.InvalidAudienceError as exc:
            raise TokenError("bad_audience") from exc
        except jwt.InvalidIssuerError as exc:
            raise TokenError("bad_issuer") from exc
        except jwt.InvalidAlgorithmError as exc:
            raise TokenError("bad_algorithm") from exc
        except jwt.MissingRequiredClaimError as exc:
            raise TokenError(f"missing_claim:{exc.claim}") from exc
        except jwt.InvalidTokenError as exc:
            raise TokenError("malformed") from exc
        scope = claims.get("scope", "")
        if not isinstance(scope, str) or not isinstance(claims["sub"], str):
            # RFC 6749: scope là chuỗi cách nhau bằng dấu cách; dạng khác (ví dụ danh sách) là token lạ.
            raise TokenError("malformed")
        return Principal(claims["sub"], frozenset(scope.split()))


def _bearer_challenge(**params: str) -> dict[str, str]:
    # RFC 6750 mục 3: `Bearer error="invalid_token"`; các tham số cách nhau bằng dấu phẩy.
    inner = ", ".join(f'{k}="{v}"' for k, v in params.items())
    return {"WWW-Authenticate": f"Bearer {inner}" if inner else "Bearer"}


async def current_principal(
    request: Request,
    security_scopes: SecurityScopes,
    token: Annotated[str | None, Depends(oauth2_scheme)],
) -> Principal:
    """401 khi không chứng minh được danh tính; 403 khi biết là ai nhưng không đủ scope."""
    if token is None:
        # RFC 6750: chưa gửi credential thì chỉ thách thức "Bearer", chưa có mã lỗi.
        raise HTTPException(status_code=401, detail="cần đăng nhập", headers=_bearer_challenge())
    tokens: TokenService = request.app.state.tokens
    try:
        principal = tokens.decode(token)
    except TokenError as exc:
        # Không log chính token; log kèm request_id để nối với response client nhận.
        log.warning("auth.token.rejected reason=%s request_id=%s", exc.reason, request.state.request_id)
        raise HTTPException(
            status_code=401,
            detail="token không hợp lệ hoặc đã hết hạn",
            headers=_bearer_challenge(error="invalid_token"),
        ) from exc
    missing = [s for s in security_scopes.scopes if s not in principal.scopes]
    if missing:
        log.warning("auth.scope.denied user=%s missing=%s request_id=%s",
                    principal.user_id, ",".join(missing), request.state.request_id)
        raise HTTPException(
            status_code=403,
            detail=f"thiếu scope: {' '.join(missing)}",
            headers=_bearer_challenge(error="insufficient_scope", scope=" ".join(security_scopes.scopes)),
        )
    request.state.user_id = principal.user_id
    return principal


def require_scope(*scopes: str):  # noqa: ANN201 - trả về đối tượng Security của FastAPI
    """`Annotated[Principal, require_scope("dispatcher")]`: yêu cầu đủ mọi scope được liệt kê."""
    return Security(current_principal, scopes=list(scopes))


Authenticated = Annotated[Principal, Security(current_principal)]
