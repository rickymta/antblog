"""Hàng đợi người duyệt và nhật ký quyết định trên PostgreSQL, schema `lab61`.

Quy tắc được ép ở CƠ SỞ DỮ LIỆU (ràng buộc và trigger), không chỉ trong mã Python, để một đoạn SQL viết vội cũng không phá được:
  - `confirmed_fraud` và `cleared` chỉ do người duyệt đã đăng ký và đang hoạt động (`lab61.reviewer`, ở đây chỉ có RV-001) chốt, và chỉ từ
    `needs_review`; dịch vụ chấm điểm `svc:fraud-scoring` chỉ được ghi sự kiện `flagged`. (Không dùng approved/rejected: ở bài 70 `approved`
    là hồ sơ KYC đạt, dễ lẫn.) `confirmed_fraud`: người duyệt xác nhận chuyến gian lận; `cleared`: bác đề xuất, chuyến bình thường;
  - đề xuất mới chỉ được INSERT ở trạng thái `needs_review`; đã chốt thì không sửa, không xóa;
  - `decision_log` chỉ ghi thêm (không UPDATE, DELETE, TRUNCATE) và mỗi dòng phải KHỚP hàng đợi: cùng chuyến, phiên bản mô hình, điểm; sự
    kiện chốt phải đúng với trạng thái, người chốt và thời điểm chốt hiện có trong hàng đợi (nên không ghi được log «giả mạo»);
  - cùng một chuyến chấm bởi cùng một phiên bản mô hình chỉ vào hàng đợi một lần (chạy lại không nhân đôi).
Những thứ lab KHÔNG chặn được: người có quyền chủ bảng hoặc superuser vẫn `DISABLE TRIGGER`, `DROP TABLE` hay sửa dữ liệu được, nên
«không ai viết lại được» chỉ đúng với người không có quyền đó. Production: vai trò (role) riêng cho dịch vụ chấm điểm chỉ có INSERT,
REVOKE các quyền còn lại, và lưu bản sao nhật ký ra nơi ngoài tầm với của người quản trị cơ sở dữ liệu (kho chỉ ghi thêm).
"""

import json
import os
from dataclasses import dataclass
from datetime import datetime
from typing import Any

import psycopg
from psycopg.rows import dict_row

from pii import scrub, scrub_json

SCHEMA = "lab61"
DEFAULT_PG_URL = "postgresql://toinoi:toinoi@localhost:54329/toinoi"
PG_URL = os.environ.get("TOINOI_PG_URL", DEFAULT_PG_URL)
COMPOSE = "docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait"
REVIEWER_ID = "RV-001"
SYSTEM_ACTOR = "svc:fraud-scoring"
DECISIONS = ("confirmed_fraud", "cleared")
RULE = "expected_cost: p * (amount_vnd + 20000) > 8000"

DDL = [
    f"CREATE SCHEMA {SCHEMA}",
    f"""CREATE TABLE {SCHEMA}.reviewer (
        reviewer_id text PRIMARY KEY CHECK (reviewer_id ~ '^RV-[0-9]{{3}}$'),
        display_name text NOT NULL,
        active boolean NOT NULL DEFAULT true)""",
    f"""CREATE TABLE {SCHEMA}.review_queue (
        queue_id bigserial PRIMARY KEY,
        trip_id text NOT NULL,
        model_version text NOT NULL,
        rules_version integer NOT NULL,
        score numeric(9,8) NOT NULL CHECK (score >= 0 AND score <= 1),
        amount_vnd bigint NOT NULL CHECK (amount_vnd >= 0),
        priority numeric(14,2) NOT NULL,
        reasons jsonb NOT NULL,
        status text NOT NULL DEFAULT 'needs_review' CHECK (status IN ('needs_review', 'confirmed_fraud', 'cleared')),
        enqueued_at timestamptz NOT NULL,
        due_at timestamptz NOT NULL,
        decided_at timestamptz,
        decided_by text REFERENCES {SCHEMA}.reviewer (reviewer_id),
        decision_note text,
        UNIQUE (trip_id, model_version),
        CHECK (due_at > enqueued_at),
        CHECK (decided_at IS NULL OR decided_at >= enqueued_at),
        CHECK ((status = 'needs_review') = (decided_by IS NULL)),
        CHECK ((decided_by IS NULL) = (decided_at IS NULL)))""",
    f"CREATE INDEX review_queue_open ON {SCHEMA}.review_queue (priority DESC) WHERE status = 'needs_review'",
    f"""CREATE TABLE {SCHEMA}.decision_log (
        log_id bigserial PRIMARY KEY,
        at timestamptz NOT NULL,
        event text NOT NULL CHECK (event IN ('flagged', 'confirmed_fraud', 'cleared')),
        queue_id bigint NOT NULL REFERENCES {SCHEMA}.review_queue (queue_id),
        trip_id text NOT NULL,
        actor text NOT NULL,
        model_version text NOT NULL,
        rules_version integer NOT NULL,
        score numeric(9,8) NOT NULL,
        rule text,
        reasons jsonb,
        note text,
        UNIQUE (queue_id, event),
        CHECK (event <> 'flagged' OR actor = '{SYSTEM_ACTOR}'),
        CHECK (event = 'flagged' OR actor ~ '^RV-[0-9]{{3}}$'))""",
    f"""CREATE FUNCTION {SCHEMA}.guard_queue() RETURNS trigger LANGUAGE plpgsql AS $$
        BEGIN
            IF TG_OP = 'DELETE' THEN
                RAISE EXCEPTION 'review_queue không cho xóa dòng';
            END IF;
            IF TG_OP = 'INSERT' THEN
                IF NEW.status <> 'needs_review' THEN
                    RAISE EXCEPTION 'đề xuất mới chỉ được vào ở needs_review, không phải %', NEW.status;
                END IF;
                RETURN NEW;
            END IF;
            IF OLD.status <> 'needs_review' THEN
                RAISE EXCEPTION 'quyết định % đã chốt, không sửa', OLD.queue_id;
            END IF;
            IF (NEW.trip_id, NEW.model_version, NEW.rules_version, NEW.score, NEW.amount_vnd, NEW.priority, NEW.reasons, NEW.enqueued_at, NEW.due_at)
               IS DISTINCT FROM (OLD.trip_id, OLD.model_version, OLD.rules_version, OLD.score, OLD.amount_vnd, OLD.priority, OLD.reasons, OLD.enqueued_at, OLD.due_at) THEN
                RAISE EXCEPTION 'không đổi nội dung đề xuất của dịch vụ chấm điểm';
            END IF;
            IF NEW.decided_by IS NOT NULL AND NOT EXISTS (SELECT 1 FROM {SCHEMA}.reviewer WHERE reviewer_id = NEW.decided_by AND active) THEN
                RAISE EXCEPTION 'người duyệt % không hoạt động', NEW.decided_by;
            END IF;
            RETURN NEW;
        END $$""",
    f"CREATE TRIGGER guard_queue BEFORE INSERT OR UPDATE OR DELETE ON {SCHEMA}.review_queue FOR EACH ROW EXECUTE FUNCTION {SCHEMA}.guard_queue()",
    f"""CREATE FUNCTION {SCHEMA}.guard_log() RETURNS trigger LANGUAGE plpgsql AS $$
        BEGIN
            RAISE EXCEPTION 'decision_log chỉ ghi thêm (% bị chặn)', TG_OP;
        END $$""",
    f"CREATE TRIGGER guard_log BEFORE UPDATE OR DELETE ON {SCHEMA}.decision_log FOR EACH ROW EXECUTE FUNCTION {SCHEMA}.guard_log()",
    f"CREATE TRIGGER guard_log_truncate BEFORE TRUNCATE ON {SCHEMA}.decision_log FOR EACH STATEMENT EXECUTE FUNCTION {SCHEMA}.guard_log()",
    f"""CREATE FUNCTION {SCHEMA}.log_matches_queue() RETURNS trigger LANGUAGE plpgsql AS $$
        DECLARE q {SCHEMA}.review_queue%ROWTYPE;
        BEGIN
            SELECT * INTO q FROM {SCHEMA}.review_queue WHERE queue_id = NEW.queue_id;
            IF (NEW.trip_id, NEW.model_version, NEW.rules_version, NEW.score)
               IS DISTINCT FROM (q.trip_id, q.model_version, q.rules_version, q.score) THEN
                RAISE EXCEPTION 'log không khớp đề xuất % trong hàng đợi (chuyến, phiên bản, điểm)', NEW.queue_id;
            END IF;
            IF NEW.event = 'flagged' THEN
                IF q.status <> 'needs_review' OR NEW.at <> q.enqueued_at THEN
                    RAISE EXCEPTION 'sự kiện flagged phải ghi lúc vào hàng đợi, khi đề xuất còn needs_review';
                END IF;
            ELSIF (q.status, q.decided_by, q.decided_at) IS DISTINCT FROM (NEW.event, NEW.actor, NEW.at) THEN
                RAISE EXCEPTION 'sự kiện % không khớp quyết định đang có trong hàng đợi (trạng thái, người chốt, thời điểm)', NEW.event;
            END IF;
            RETURN NEW;
        END $$""",
    f"CREATE TRIGGER log_matches_queue BEFORE INSERT ON {SCHEMA}.decision_log FOR EACH ROW EXECUTE FUNCTION {SCHEMA}.log_matches_queue()",
]


class AlreadyDecided(RuntimeError):
    pass


@dataclass(frozen=True)
class Flag:
    """Một đề xuất gắn cờ do dịch vụ chấm điểm tạo ra."""

    trip_id: str
    model_version: str
    rules_version: int
    score: float
    amount_vnd: int
    priority: float  # chi phí kỳ vọng của bỏ sót: p * (tiền chuyến + xử lý); việc xếp hàng theo đây
    reasons: dict[str, Any]
    enqueued_at: datetime
    due_at: datetime


def connect() -> psycopg.Connection:
    return psycopg.connect(PG_URL, autocommit=True, row_factory=dict_row, connect_timeout=5)


class ReviewStore:
    def __init__(self, conn: psycopg.Connection) -> None:
        self.conn = conn

    def create_schema(self) -> None:
        self.drop_schema()
        for stmt in DDL:
            self.conn.execute(stmt)
        self.conn.execute(f"INSERT INTO {SCHEMA}.reviewer (reviewer_id, display_name) VALUES (%s, %s)", (REVIEWER_ID, "Người duyệt 1"))

    def drop_schema(self) -> None:
        self.conn.execute(f"DROP SCHEMA IF EXISTS {SCHEMA} CASCADE")

    def schema_exists(self) -> bool:
        row = self.conn.execute("SELECT count(*) AS n FROM information_schema.schemata WHERE schema_name = %s", (SCHEMA,)).fetchone()
        return bool(row["n"])

    def enqueue(self, flag: Flag) -> int | None:
        """Thêm đề xuất và sự kiện `flagged` trong MỘT giao dịch. Trả None nếu (chuyến, phiên bản mô hình) đã có.

        Mọi trường văn bản đi qua `scrub` (kể cả mã chuyến, tên phiên bản và reasons); trường số không chứa văn bản nên không cần."""
        reasons = json.dumps(scrub_json(flag.reasons), ensure_ascii=False)
        trip_id, model_version = scrub(flag.trip_id), scrub(flag.model_version)
        with self.conn.transaction():
            row = self.conn.execute(
                f"""INSERT INTO {SCHEMA}.review_queue
                    (trip_id, model_version, rules_version, score, amount_vnd, priority, reasons, enqueued_at, due_at)
                    VALUES (%s, %s, %s, %s, %s, %s, %s::jsonb, %s, %s)
                    ON CONFLICT (trip_id, model_version) DO NOTHING RETURNING queue_id""",
                (trip_id, model_version, flag.rules_version, flag.score, flag.amount_vnd, flag.priority, reasons, flag.enqueued_at, flag.due_at),
            ).fetchone()
            if row is None:
                return None
            self.conn.execute(
                f"""INSERT INTO {SCHEMA}.decision_log
                    (at, event, queue_id, trip_id, actor, model_version, rules_version, score, rule, reasons)
                    VALUES (%s, 'flagged', %s, %s, %s, %s, %s, %s, %s, %s::jsonb)""",
                (flag.enqueued_at, row["queue_id"], trip_id, SYSTEM_ACTOR, model_version, flag.rules_version, flag.score, RULE, reasons),
            )
            return row["queue_id"]

    def decide(self, queue_id: int, reviewer_id: str, decision: str, note: str, at: datetime) -> None:
        """Người duyệt chốt `confirmed_fraud` hoặc `cleared`. Đề xuất đã chốt thì AlreadyDecided (hai người cùng bấm: một người thua)."""
        if decision not in DECISIONS:
            raise ValueError(f"quyết định không hợp lệ: {decision}")
        note = scrub(note)
        with self.conn.transaction():
            row = self.conn.execute(
                f"""UPDATE {SCHEMA}.review_queue
                    SET status = %s, decided_by = %s, decided_at = %s, decision_note = %s
                    WHERE queue_id = %s AND status = 'needs_review'
                    RETURNING trip_id, model_version, rules_version, score""",
                (decision, reviewer_id, at, note, queue_id),
            ).fetchone()
            if row is None:
                raise AlreadyDecided(f"đề xuất {queue_id} không còn ở needs_review")
            self.conn.execute(
                f"""INSERT INTO {SCHEMA}.decision_log
                    (at, event, queue_id, trip_id, actor, model_version, rules_version, score, note)
                    VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s)""",
                (at, decision, queue_id, row["trip_id"], reviewer_id, row["model_version"], row["rules_version"], row["score"], note),
            )
