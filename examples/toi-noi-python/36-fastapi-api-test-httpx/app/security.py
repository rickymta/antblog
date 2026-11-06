"""Kiểm token JWT và `require_scope`, theo lab 34 (OAuth2PasswordBearer + Security để scope hiện trong OpenAPI).
Lab này chỉ KIỂM token, không cấp token: test tự ký token bằng jwt.encode. Không tự viết mật mã: PyJWT làm."""

import logging
from dataclasses import dataclass
from typing import Annotated, Any

import jwt
from fastapi import Depends, HTTPException, Request, Security
from fastapi.security import OAuth2PasswordBearer, SecurityScopes

from app.settings import Settings

log = logging.getLogger("toinoi.auth")

ALGORITHM = "HS256"
SCOPES = {
    "customer": "Khách: báo giá, xem chuyến của mình",
    "driver": "Tài xế: xem chuyến được giao",
    "dispatcher": "Điều phối: xem mọi chuyến, hủy chuyến",
}
# tokenUrl chỉ để Swagger UI biết nơi lấy token (lab 34 cấp token; lab này không có endpoint đó).
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

    def decode(self, token: str) -> Principal:
        try:
            # algorithms=[...] luôn truyền tường minh: token tự khai alg=none hay alg khác bị từ chối.
            claims: dict[str, Any] = jwt.decode(token, self._key, algorithms=[ALGORITHM],
                                                audience=self._audience, issuer=self._issuer,
                                                options={"require": REQUIRED_CLAIMS})
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
            raise TokenError("malformed")  # RFC 6749: scope là chuỗi cách nhau bằng dấu cách; list là token lạ
        return Principal(claims["sub"], frozenset(scope.split()))


def get_token_service(request: Request) -> TokenService:
    """Khớp nối để test thay bộ kiểm token bằng bản giả mà vẫn chạy kiểm scope (xem tests/test_auth.py)."""
    return request.app.state.tokens


def _bearer_challenge(**params: str) -> dict[str, str]:
    # RFC 6750 mục 3: `Bearer error="invalid_token"`; các tham số cách nhau bằng dấu phẩy.
    inner = ", ".join(f'{k}="{v}"' for k, v in params.items())
    return {"WWW-Authenticate": f"Bearer {inner}" if inner else "Bearer"}


async def current_principal(
    request: Request,
    security_scopes: SecurityScopes,
    token: Annotated[str | None, Depends(oauth2_scheme)],
    tokens: Annotated[TokenService, Depends(get_token_service)],
) -> Principal:
    """401 khi không chứng minh được danh tính; 403 khi biết là ai nhưng thiếu scope."""
    if token is None:
        raise HTTPException(status_code=401, detail="cần đăng nhập", headers=_bearer_challenge())
    try:
        principal = tokens.decode(token)
    except TokenError as exc:
        # Không log chính token; log kèm request_id để nối với response client nhận.
        log.warning("auth.token.rejected reason=%s request_id=%s", exc.reason, request.state.request_id)
        raise HTTPException(status_code=401, detail="token không hợp lệ hoặc đã hết hạn",
                            headers=_bearer_challenge(error="invalid_token")) from exc
    missing = [s for s in security_scopes.scopes if s not in principal.scopes]
    if missing:
        log.warning("auth.scope.denied user=%s missing=%s request_id=%s",
                    principal.user_id, ",".join(missing), request.state.request_id)
        raise HTTPException(status_code=403, detail=f"thiếu scope: {' '.join(missing)}",
                            headers=_bearer_challenge(error="insufficient_scope",
                                                      scope=" ".join(security_scopes.scopes)))
    return principal


def require_scope(*scopes: str):  # noqa: ANN201 - trả về đối tượng Security của FastAPI
    """`Annotated[Principal, require_scope("dispatcher")]`: yêu cầu ĐỦ MỌI scope được liệt kê (như lab 34)."""
    return Security(current_principal, scopes=list(scopes))


Authenticated = Annotated[Principal, Security(current_principal)]
