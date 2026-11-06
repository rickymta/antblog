"""Kho người dùng giả trong bộ nhớ. Mật khẩu của lab là giá trị giả, chỉ để chạy thử cục bộ."""

from dataclasses import dataclass

from pwdlib import PasswordHash


@dataclass(frozen=True)
class UserRecord:
    user_id: str
    password_hash: str  # hash argon2id có muối; không bao giờ lưu mật khẩu gốc
    scopes: frozenset[str]


# (user_id, mật khẩu giả của lab, scope được phép cấp)
_SEED = [
    ("CU-001", "lab34-cu001-pass", {"customer"}),
    ("CU-002", "lab34-cu002-pass", {"customer"}),
    ("DR-001", "lab34-dr001-pass", {"driver"}),
    ("DP-001", "lab34-dp001-pass", {"dispatcher"}),
]


def build_users(hasher: PasswordHash) -> dict[str, UserRecord]:
    return {uid: UserRecord(uid, hasher.hash(pw), frozenset(scopes)) for uid, pw, scopes in _SEED}
