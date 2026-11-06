"""Bài 47: lưu kết quả OCR vào MongoDB: mặt nạ PII, hash có khóa, TTL retention, audit, migration schema.

Chạy demo:  uv run --project <lab> <lab>/main.py
Verify:     uv run --project <lab> <lab>/main.py --verify

Cần MongoDB trong hạ tầng dùng chung:
    docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
Lab đụng vào ba collection `lab47_ocr_documents`, `lab47_ocr_raw`, `lab47_audit_log` (xóa chúng khi xong) VÀ, ở phần TTL,
tạm đặt tham số `ttlMonitorSleepSecs` của CẢ mongod về 1 giây (mặc định 60) rồi trả lại: mọi lab khác dùng chung container
cũng bị quét nhanh trong lúc đó. Lab bắt SIGTERM để trả lại; nếu bị SIGKILL, lần chạy sau tự phát hiện và trả về 60.

Dữ liệu là tổng hợp: CCCD giả bắt đầu 099, GPLX giả bắt đầu 99, tên giả. Khóa băm dưới đây là giá trị CỦA LAB, ai cũng đọc
được trong mã nguồn: production lấy từ secret manager và đặt tách khỏi cơ sở dữ liệu. Kết quả OCR là đầu vào cho
người duyệt, không tự động là quyết định xác minh.
"""

import asyncio
import hashlib
import hmac
import itertools
import json
import os
import random
import signal
import sys
import time
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Any

from pydantic import ValidationError
from pymongo import AsyncMongoClient
from pymongo.asynchronous.database import AsyncDatabase
from pymongo.errors import PyMongoError, WriteError

from app import documents as docs
from app.access import RevealDenied, lookup_by_number, view_document
from app.audit import AUDIT_RETENTION, Actor, MongoAuditSink
from app.documents import AUDIT, DOCS, RAW, RAW_RETENTION
from app.pii import ID_FIELD, PiiKey, load_pii_key, mask_digits, pii_hash
from app.retention import fast_ttl_monitor, wait_until_gone
from synthetic_card import make_card_fields

NEWER_MODEL = "tesseract-5.6.0+vie+eng/psm6"  # giả định một bản Tesseract mới hơn, chỉ để thử nhiều phiên bản mô hình
# Giá trị CỦA LAB (>= 32 ký tự). Đặt qua biến môi trường để `load_pii_key()` đọc như production; thiếu biến thì báo lỗi.
LAB_PII_HASH_KEY = "lab47-khoa-bam-chi-dung-trong-lab-khong-phai-bi-mat"
os.environ.setdefault("TOINOI_PII_HASH_KEY", LAB_PII_HASH_KEY)
os.environ.setdefault("TOINOI_PII_KEY_ID", "lab-1")

MONGO_URL = os.environ.get("TOINOI_MONGO_URL", "mongodb://localhost:27019/toinoi")
COMPOSE = "docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait"
FIXTURE = Path(__file__).resolve().parent / "fixtures" / "job_result_cccd.json"
REVIEWER = Actor("DP-001", frozenset({"kyc:view", "kyc:reveal"}))
VIEWER = Actor("DP-002", frozenset({"kyc:view"}))  # không có kyc:reveal
NO_SCOPE = Actor("DP-003", frozenset())


class Report:
    """Gom các ca kiểm. Verify in từng ca; demo chỉ gom số ca đạt để in một dòng cuối."""

    def __init__(self, verbose: bool) -> None:
        self.verbose = verbose
        self.results: list[tuple[str, bool]] = []

    def check(self, name: str, ok: bool, detail: str = "") -> None:
        self.results.append((name, ok))
        if self.verbose:
            print(f"  {'đạt ' if ok else 'SAI '} {name}" + (f"  [{detail}]" if detail and not ok else ""))


def say(report: Report, *parts: object) -> None:
    if not report.verbose:
        print(*parts)


def show(doc: dict[str, Any]) -> str:
    """In document gọn: ObjectId và datetime thành chuỗi."""
    return json.dumps({k: v for k, v in doc.items() if k != "_id"}, ensure_ascii=False, indent=2, default=str)


def dump(doc: object) -> str:
    return json.dumps(doc, ensure_ascii=False, default=str)


def sha(text: str) -> str:
    return hashlib.sha256(text.encode()).hexdigest()


def load_average() -> str:
    one, five, fifteen = os.getloadavg()
    return f"load average 1/5/15 phút: {one:.2f} {five:.2f} {fifteen:.2f}"


# ---------------------------------------------------------------- dữ liệu tổng hợp


def fixture_output() -> dict[str, Any]:
    """Kết quả job của bài 46 (thẻ giả seed 7, có `image_sha256`), lưu nguyên làm dữ liệu thử."""
    return json.loads(FIXTURE.read_text(encoding="utf-8"))


def cccd_output(seed: int, kyc_id: str, *, model: str = "demo-ocr-1.0", confidence: float = 0.95) -> dict[str, Any]:
    """Kết quả OCR có hình dạng như job của bài 46, nhưng nội dung là đáp án của thẻ giả (không chạy Tesseract)."""
    fields = make_card_fields(seed)
    return {"ok": True, "kyc_id": kyc_id, "document_type": "cccd", "fields": fields,
            "field_confidence": {k: confidence for k in fields}, "confidence": confidence, "needs_review": False,
            "model_version": model, "raw_text": "\n".join(fields.values())}


def gplx_output(kyc_id: str) -> dict[str, Any]:
    rng = random.Random(kyc_id)
    fields = {"license_number": "99" + "".join(rng.choices("0123456789", k=10)), "full_name": "NGUYEN VAN A",
              "license_class": "B2", "expiry_date": "2031-05-20"}
    return {"ok": True, "kyc_id": kyc_id, "document_type": "gplx", "fields": fields,
            "field_confidence": {k: 0.9 for k in fields}, "confidence": 0.9, "needs_review": False,
            "model_version": "demo-ocr-1.0", "raw_text": "\n".join(fields.values())}


def legacy_v2(seed: int, kyc_id: str, created_at: datetime) -> dict[str, Any]:
    """Document v2 của bài 26: số giấy tờ nằm rõ trong fields, chưa có chính sách PII."""
    fields = make_card_fields(seed)
    return {"kyc_id": kyc_id, "document_type": "cccd", "schema_version": 2, "model_version": "demo-ocr-0.9",
            "fields": {"id_number": fields["id_number"], "full_name": fields["full_name"],
                       "date_of_birth": docs.to_iso_date(fields["date_of_birth"]), "sex": fields["sex"],
                       "hometown_city": fields["hometown_city"]},
            "field_confidence": {k: 0.9 for k in ("id_number", "full_name", "date_of_birth", "sex", "hometown_city")},
            "confidence": 0.9, "created_at": created_at}


def brute_force(prefix: str, target: str, *, key: bytes | None) -> tuple[str | None, float]:
    """Thử 10^6 số còn lại (đã biết `099` + giới tính/thế kỷ + 2 số năm, vì chúng suy ra từ ngày sinh và giới tính
    đang lưu rõ). key=None: SHA-256 trơn; có key: HMAC-SHA-256 như pii_hash."""
    started = time.perf_counter()
    for tail in range(1_000_000):
        number = f"{prefix}{tail:06d}"
        message = f"cccd\x1f{number}".encode() if key else number.encode()
        digest = hmac.digest(key, message, "sha256").hex() if key else hashlib.sha256(message).hexdigest()
        if digest == target:
            return number, time.perf_counter() - started
    return None, time.perf_counter() - started


def candidates_from_mask(masked: str, date_of_birth: str, sex: str) -> list[str]:
    """Mọi số CCCD 12 chữ số khớp mặt nạ khi kẻ tấn công biết thêm ngày sinh và giới tính (đang lưu rõ trong document):
    chữ số thứ 4 là giới tính và thế kỷ, chữ số thứ 5 và 6 là hai số cuối của năm sinh."""
    year = int(date_of_birth[:4])
    known = {3: str((0 if year < 2000 else 2) + (0 if sex == "Nam" else 1)), 4: f"{year % 100:02d}"[0], 5: f"{year % 100:02d}"[1]}
    free = [i for i, c in enumerate(masked) if c == "*" and i not in known]
    out = []
    for digits in itertools.product("0123456789", repeat=len(free)):
        chars = list(masked)
        for i, d in known.items():
            chars[i] = d
        for i, d in zip(free, digits, strict=True):
            chars[i] = d
        out.append("".join(chars))
    return out


# ---------------------------------------------------------------- các nhóm ca


async def part_pii(report: Report, pii_key: PiiKey) -> None:
    say(report, "== 1. Che số và băm có khóa ==")
    number = "099078777631"
    masked = mask_digits(number)
    say(report, f"mask_digits({number!r}) -> {masked!r}")
    report.check("mask 12 chữ số: 3 đầu + 7 sao + 2 cuối, giữ độ dài", masked == "099*******31" and len(masked) == 12)
    bad = []
    for value in ("0990", "09907877763x", "٠٩٩٠٧٨٧٧٧٦٣١", ""):
        try:
            mask_digits(value)
            bad.append(value)
        except ValueError:
            pass
    report.check("mask từ chối chuỗi ngắn, chữ lẫn số, chữ số Arabic-Indic, chuỗi rỗng", not bad, str(bad))

    fixture = fixture_output()["fields"]
    dob = docs.to_iso_date(fixture["date_of_birth"])
    candidates = candidates_from_mask(mask_digits(fixture["id_number"]), dob, fixture["sex"])
    say(report, f"biết mặt nạ {mask_digits(fixture['id_number'])} + ngày sinh {dob} + giới tính {fixture['sex']}: "
                f"còn {len(candidates)} số CCCD ứng viên")
    report.check("mặt nạ cộng ngày sinh và giới tính còn 10.000 ứng viên chứa số thật (không phải 100 như mặt nạ 4+4)",
                 len(candidates) == 10_000 and fixture["id_number"] in candidates and len(set(candidates)) == 10_000)

    digest = pii_hash(number, "cccd", pii_key)
    say(report, f"pii_hash(...) = {digest[:16]}... ({len(digest)} ký tự hex), khóa {pii_key!r}")
    other = PiiKey("lab-2", b"mot-khoa-khac-hoan-toan-0123456789-abcdef")
    report.check("băm: cùng đầu vào ra cùng giá trị; đổi loại giấy tờ hoặc khóa thì khác",
                 digest == pii_hash(number, "cccd", pii_key) and digest != pii_hash(number, "gplx", pii_key)
                 and digest != pii_hash(number, "cccd", other))
    report.check("repr của PiiKey không lộ khóa", LAB_PII_HASH_KEY not in repr(pii_key) and "<ẩn>" in repr(pii_key))

    leaked = False
    saved = {k: os.environ.pop(k) for k in ("TOINOI_PII_HASH_KEY", "TOINOI_PII_KEY_ID") if k in os.environ}
    try:
        try:
            load_pii_key()
            missing = "không lỗi"
        except ValidationError as exc:
            missing = "ValidationError"
            leaked = LAB_PII_HASH_KEY in str(exc)
        os.environ["TOINOI_PII_HASH_KEY"] = "ngan-qua-chua-du-32-ky-tu"
        try:
            load_pii_key()
            short = "không lỗi"
        except ValidationError as exc:
            short = "ValidationError"
            leaked = leaked or "ngan-qua" in str(exc)
    finally:
        os.environ.update(saved)
    report.check("thiếu khóa hoặc khóa dưới 32 ký tự: ValidationError, không có mặc định, lỗi không in lại giá trị",
                 missing == short == "ValidationError" and not leaked, f"{missing}, {short}")

    # Thí nghiệm: số CCCD chỉ còn 10^6 khả năng khi đã biết giới tính và năm sinh, nên băm trơn không che được gì.
    target_number = fixture["id_number"]  # 099090159083: nam, 1990
    prefix = target_number[:6]
    plain = hashlib.sha256(target_number.encode()).hexdigest()
    found, took = brute_force(prefix, plain, key=None)
    say(report, f"SHA-256 trơn, biết 6 số đầu: dò lại được {found} sau {took:.2f} giây (tối đa 10^6 lần thử)")
    report.check("SHA-256 không khóa của số CCCD bị dò ra bằng 10^6 lần thử", found == target_number)
    keyed = pii_hash(target_number, "cccd", pii_key)
    found2, took2 = brute_force(prefix, keyed, key=None)
    say(report, f"cùng bộ dò nhưng giá trị là HMAC có khóa mà kẻ tấn công không có: {found2} sau {took2:.2f} giây")
    report.check("kẻ tấn công có dữ liệu mà không có khóa: không dò ra", found2 is None)
    found3, took3 = brute_force(prefix, keyed, key=pii_key.key)
    say(report, f"nếu lộ cả khóa lẫn dữ liệu: dò ra {found3} sau {took3:.2f} giây")
    report.check("lộ cả khóa lẫn dữ liệu thì dò ra như thường: khóa phải ở nơi khác cơ sở dữ liệu", found3 == target_number)


async def part_save(report: Report, db: AsyncDatabase[dict[str, Any]], pii_key: PiiKey) -> None:
    say(report, "\n== 2. Lưu kết quả OCR của bài 46 ==")
    output = fixture_output()
    image_sha = output["image_sha256"]
    now = datetime.now(UTC).replace(microsecond=0)  # giờ thật: ngày cố định trong quá khứ xa sẽ bị TTL xóa mất
    result = await docs.save_ocr_result(db, output, image_key="kyc/KYC-0007/9f2c.png", image_sha256=image_sha,
                                        key=pii_key, now=now)
    stored = await db[DOCS].find_one({"kyc_id": "KYC-0007"})
    raw = await db[RAW].find_one({"kyc_id": "KYC-0007"})
    assert stored is not None and raw is not None
    say(report, show(stored))
    say(report, "raw tương ứng:", show({k: v for k, v in raw.items() if k != "raw_text"}))
    plain_id = output["fields"]["id_number"]
    report.check("document mới: schema_version 3, created True", stored["schema_version"] == 3 and result.created)
    report.check("số giấy tờ trong document chỉ còn dạng che: 099*******83",
                 stored["fields"]["id_number_masked"] == "099*******83" and "id_number" not in stored["fields"])
    report.check("toàn bộ document không chứa số giấy tờ rõ ở bất kỳ trường nào", plain_id not in dump(stored))
    report.check("ngày sinh dd/mm/yyyy trên thẻ -> ISO; confidence là nhỏ nhất; mã mô hình và hash ảnh được giữ",
                 stored["fields"]["date_of_birth"] == "1990-03-13" and stored["confidence"] == 0.952
                 and stored["model_version"] == output["model_version"] and stored["image_sha256"] == image_sha)
    report.check("raw: số rõ + văn bản thô + khóa ảnh, expire_at là kiểu date, đúng created_at + 30 ngày",
                 raw["plain"] == {"id_number": plain_id} and raw["image_key"] == "kyc/KYC-0007/9f2c.png"
                 and isinstance(raw["expire_at"], datetime) and raw["expire_at"] - raw["created_at"] == RAW_RETENTION
                 and stored["raw_id"] == raw["_id"])

    codes = []
    base = {k: v for k, v in stored.items() if k != "_id"} | {"model_version": "demo-ocr-bad"}
    for name, candidate in (
        ("số rõ trong fields", {**base, "fields": {**base["fields"], "id_number": plain_id}}),
        ("trường lạ trong fields", {**base, "fields": {**base["fields"], "so_cccd": plain_id}}),
        ("trường lạ ngoài fields", {**base, "raw_text": plain_id}),
        ("hash rỗng", {**base, "fields": {**base["fields"], "id_number_hash": ""}}),
        ("mặt nạ sai dạng", {**base, "fields": {**base["fields"], "id_number_masked": "099-83"}}),
        ("thiếu pii_key_id", {k: v for k, v in base.items() if k != "pii_key_id"}),
    ):
        try:
            await db[DOCS].insert_one(candidate)
            codes.append(None)
            await db[DOCS].delete_one({"kyc_id": "KYC-0007", "model_version": "demo-ocr-bad"})  # nếu validator hỏng, dọn để các ca sau không lệch
        except WriteError as exc:
            codes.append(exc.code)
    say(report, "validator từ chối (mã lỗi):", codes)
    report.check("validator v3 từ chối số rõ, trường lạ (trong và ngoài fields), hash rỗng, mặt nạ sai dạng, thiếu pii_key_id (mã 121)",
                 codes == [121] * 6, str(codes))

    # Raw hết hạn rồi lưu lại cùng kết quả: không được tạo raw mới trong khi document vẫn trỏ raw cũ.
    await db[RAW].delete_one({"_id": stored["raw_id"]})
    again = await docs.save_ocr_result(db, output, image_key="kyc/KYC-0007/9f2c.png", image_sha256=image_sha,
                                       key=pii_key, now=now + timedelta(hours=1))
    report.check("lưu lại cùng kết quả sau khi raw đã hết hạn: không thêm document, không dựng lại raw",
                 not again.created and again.document["_id"] == stored["_id"]
                 and await db[RAW].count_documents({"kyc_id": "KYC-0007"}) == 0)
    await db[RAW].insert_one(raw)  # trả raw về để các ca sau dùng

    newer = await docs.save_ocr_result(db, {**output, "model_version": NEWER_MODEL}, image_key="kyc/KYC-0007/9f2c.png",
                                       image_sha256=image_sha, key=pii_key, now=now + timedelta(days=1))
    retake = await docs.save_ocr_result(db, output, image_key="kyc/KYC-0007/ab34.png", image_sha256=sha("ảnh chụp lại"),
                                        key=pii_key, now=now + timedelta(days=2))
    latest = await db[DOCS].find_one({"kyc_id": "KYC-0007"}, sort=[("created_at", -1), ("_id", -1)])
    say(report, f"mô hình mới: created={newer.created}; chụp lại ảnh cùng mô hình: created={retake.created}; "
                f"tổng {await db[DOCS].count_documents({'kyc_id': 'KYC-0007'})} document")
    report.check("mô hình mới là document mới, bản cũ còn", newer.created)
    report.check("chụp lại ảnh (hash khác) với cùng mô hình: document mới, không mất kết quả của ảnh trước; bản mới nhất là bản chụp lại",
                 retake.created and await db[DOCS].count_documents({"kyc_id": "KYC-0007"}) == 3
                 and latest is not None and latest["image_sha256"] == sha("ảnh chụp lại"))

    gplx = gplx_output("KYC-0008")
    saved = await docs.save_ocr_result(db, gplx, image_key="kyc/KYC-0008/ab12.png", image_sha256=sha("gplx-0008"),
                                       key=pii_key, now=now)
    report.check("GPLX cùng cách: license_number_masked theo mẫu, không còn license_number",
                 saved.document["fields"]["license_number_masked"] == mask_digits(gplx["fields"]["license_number"])
                 and "license_number" not in saved.document["fields"] and gplx["fields"]["license_number"] not in dump(saved.document))

    # Đầu vào xấu: lưu như kết quả cần người duyệt, không bỏ, không để lại raw mồ côi.
    missing_number = await docs.save_ocr_result(
        db, {**output, "kyc_id": "KYC-0009", "fields": {**output["fields"], "id_number": None}}, image_key="x",
        image_sha256=sha("thiếu số"), key=pii_key, now=now)
    bad_input = await docs.save_ocr_result(
        db, {**output, "kyc_id": "KYC-0010", "fields": {**output["fields"], "id_number": "09909015908", "date_of_birth": "31/02/1990"}},
        image_key="y", image_sha256=sha("số 11 chữ số"), key=pii_key, now=now)
    say(report, "thiếu số ->", dump({k: missing_number.document[k] for k in ("needs_review", "missing_fields")}),
        "| số 11 chữ số và ngày 31/02 ->", dump({k: bad_input.document[k] for k in ("needs_review", "missing_fields")}))
    report.check("kết quả không đọc ra số giấy tờ: vẫn được lưu, needs_review true, missing_fields ghi rõ, số che và hash là null",
                 missing_number.created and missing_number.document["needs_review"] is True
                 and missing_number.document["missing_fields"] == ["id_number"]
                 and missing_number.document["fields"]["id_number_masked"] is None)
    report.check("số 11 chữ số và ngày 31/02/1990: lưu với needs_review, thiếu id_number và date_of_birth, không ném lỗi",
                 bad_input.created and bad_input.document["missing_fields"] == ["date_of_birth", "id_number"]
                 and bad_input.document["fields"]["date_of_birth"] is None)
    referenced = {d["raw_id"] async for d in db[DOCS].find({}, {"raw_id": 1})}
    orphans = [r["kyc_id"] async for r in db[RAW].find({}, {"kyc_id": 1}) if r["_id"] not in referenced]
    report.check("không có raw mồ côi sau các lần lưu (kể cả đầu vào xấu)", not orphans, str(orphans))


async def part_lookup(report: Report, db: AsyncDatabase[dict[str, Any]], audit: MongoAuditSink, pii_key: PiiKey) -> None:
    say(report, "\n== 3. Đối chiếu theo giá trị băm ==")
    for n in range(1, 201):  # 200 hồ sơ để explain có ý nghĩa hơn một collection chỉ vài dòng
        await docs.save_ocr_result(db, cccd_output(100 + n, f"KYC-{1000 + n}"), image_key=f"kyc/{n}.png",
                                   image_sha256=sha(f"card-{n}"), key=pii_key)
    shared = fixture_output()["fields"]["id_number"]  # cùng số CCCD đứng tên hai hồ sơ khác nhau
    await docs.save_ocr_result(db, {**fixture_output(), "kyc_id": "KYC-2001"}, image_key="kyc/dup.png",
                               image_sha256=sha("dup"), key=pii_key)
    hits = await lookup_by_number(db, audit, [pii_key], number=shared, document_type="cccd", actor=REVIEWER)
    say(report, "tra số", mask_digits(shared), "->", sorted(h["kyc_id"] for h in hits), "(KYC-0007 có ba document)")
    say(report, "dòng trả về:", dump(hits[0]))
    report.check("cùng số CCCD ở hai hồ sơ: tra theo hash thấy cả hai hồ sơ (bốn document, vì KYC-0007 có ba)",
                 sorted({h["kyc_id"] for h in hits}) == ["KYC-0007", "KYC-2001"] and len(hits) == 4, str(hits))
    report.check("kết quả đối chiếu chỉ có dạng che", shared not in dump(hits) and "id_number_hash" not in dump(hits))
    report.check("số không có trong hệ thống: 0 kết quả",
                 await lookup_by_number(db, audit, [pii_key], number="099000000000", document_type="cccd", actor=REVIEWER) == [])
    explain = await db[DOCS].find({"document_type": "cccd", "pii_key_id": pii_key.key_id,
                                   "fields.id_number_hash": pii_hash(shared, "cccd", pii_key)}).explain()
    stats = explain["executionStats"]
    plan = explain["queryPlanner"]["winningPlan"]
    plan = plan.get("queryPlan", plan)
    while "inputStage" in plan:
        plan = plan["inputStage"]
    say(report, f"explain: {plan['stage']} trên {plan.get('indexName')}, kiểm {stats['totalDocsExamined']} document, trả {stats['nReturned']}")
    report.check("tra theo hash dùng index ix_id_hash (IXSCAN), chỉ kiểm document trả về",
                 plan["stage"] == "IXSCAN" and plan.get("indexName") == "ix_id_hash"
                 and stats["totalDocsExamined"] == stats["nReturned"], f"{plan}")
    rotated = PiiKey("lab-2", b"khoa-moi-sau-khi-xoay-vong-0123456789-abcdef")
    report.check("sau khi xoay khóa, tra bằng khóa mới một mình không thấy bản ghi cũ; phải giữ khóa cũ trong danh sách để tra",
                 await lookup_by_number(db, audit, [rotated], number=shared, document_type="cccd", actor=REVIEWER) == []
                 and len(await lookup_by_number(db, audit, [rotated, pii_key], number=shared, document_type="cccd", actor=REVIEWER)) == 4)
    try:
        await lookup_by_number(db, audit, [pii_key], number=shared, document_type="cccd", actor=NO_SCOPE)
        denied = "được phép"
    except PermissionError:
        denied = "PermissionError"
    report.check("tra số không có scope kyc:view: PermissionError (và ghi lookup_denied)", denied == "PermissionError")


async def part_retention(report: Report, client: AsyncMongoClient[dict[str, Any]], db: AsyncDatabase[dict[str, Any]]) -> None:
    say(report, "\n== 4. Retention: TTL index ==")
    indexes = await db[RAW].index_information()
    audit_indexes = await db[AUDIT].index_information()
    say(report, "index TTL của raw:", indexes["ttl_raw_expire_at"].get("expireAfterSeconds"), "giây sau expire_at")
    report.check("raw có TTL index trên expire_at với expireAfterSeconds = 0",
                 indexes["ttl_raw_expire_at"]["key"] == [("expire_at", 1)] and indexes["ttl_raw_expire_at"]["expireAfterSeconds"] == 0)
    report.check("audit có TTL index riêng, dài hơn raw",
                 audit_indexes["ttl_audit_expire_at"]["expireAfterSeconds"] == 0 and AUDIT_RETENTION > RAW_RETENTION)

    now = datetime.now(UTC).replace(microsecond=0)

    def raw_doc(kyc: str, expire_at: object) -> dict[str, Any]:
        return {"kyc_id": kyc, "document_type": "cccd", "model_version": "ttl-test", "image_sha256": None,
                "image_key": None, "raw_text": "x", "plain": {"id_number": "099000000001"}, "created_at": now,
                "expire_at": expire_at}

    async with fast_ttl_monitor(client, 1) as fast:  # đặt tham số TRƯỚC khi chèn, để đo từ lúc chèn là công bằng
        timeout = 20 if fast else 130
        say(report, f"ttlMonitorSleepSecs tạm đặt 1 giây: {fast}")
        inserted = time.monotonic()
        await db[RAW].insert_many([
            raw_doc("KYC-9001", now - timedelta(seconds=5)),  # A: đã quá hạn, kiểu date: sẽ bị xóa
            raw_doc("KYC-9002", now + timedelta(days=1)),  # B: còn hạn
            raw_doc("KYC-9003", now + timedelta(seconds=3)),  # C: sắp hết hạn nhưng sẽ bị gỡ expire_at (giữ lại vì lý do pháp lý)
            raw_doc("KYC-9004", (now - timedelta(days=1)).isoformat()),  # D: quá hạn nhưng là chuỗi, không phải date
            raw_doc("KYC-9005", now + timedelta(seconds=3)),  # E: đối chứng của C, không gỡ gì
        ])
        await db[RAW].update_one({"kyc_id": "KYC-9003"}, {"$unset": {"expire_at": ""}})
        gone_a = await wait_until_gone(db[RAW], {"kyc_id": "KYC-9001"}, timeout, since=inserted)
        gone_e = await wait_until_gone(db[RAW], {"kyc_id": "KYC-9005"}, timeout, since=inserted)
    survivors = sorted([d["kyc_id"] async for d in db[RAW].find({"kyc_id": {"$in": [f"KYC-900{i}" for i in range(1, 6)]}})])
    say(report, f"tính từ lúc chèn: A (đã quá hạn, kiểu date) biến mất sau {gone_a and round(gone_a, 1)} giây; E (hết hạn sau 3 giây) sau {gone_e and round(gone_e, 1)} giây")
    say(report, "còn lại:", survivors, "(B còn hạn, C đã gỡ expire_at, D có expire_at là chuỗi)")
    report.check("document quá hạn (kiểu date) bị TTL xóa", gone_a is not None)
    report.check("document còn hạn, document đã gỡ expire_at (giữ lại) và document có expire_at kiểu chuỗi đều KHÔNG bị xóa",
                 survivors == ["KYC-9002", "KYC-9003", "KYC-9004"], str(survivors))
    report.check("đối chứng: document hết hạn sau 3 giây cũng bị xóa, nên việc C còn lại là do gỡ expire_at", gone_e is not None)


async def part_audit(report: Report, db: AsyncDatabase[dict[str, Any]], audit: MongoAuditSink, pii_key: PiiKey) -> None:
    say(report, "\n== 5. Audit: ai xem gì ==")
    shared = fixture_output()["fields"]["id_number"]

    v = await view_document(db, audit, pii_key, kyc_id="KYC-0007", document_type="cccd", actor=VIEWER)
    report.check("xem mặc định: số đã che, không có số rõ", v.document["fields"]["id_number_masked"] == "099*******83"
                 and v.revealed is None and shared not in dump(v.document))

    revealed = await view_document(db, audit, pii_key, kyc_id="KYC-2001", document_type="cccd", actor=REVIEWER,
                                   reveal=True, reason="đối chiếu hồ sơ trùng số CCCD")
    report.check("xem số rõ có scope kyc:reveal và lý do: trả số rõ từ raw", revealed.revealed == {"id_number": shared})

    denied: list[str] = []
    for label, actor, reason in (("thiếu scope", VIEWER, "đối chiếu hồ sơ trùng số CCCD"), ("thiếu lý do", REVIEWER, "xem"),
                                 ("không có lý do", REVIEWER, None)):
        try:
            await view_document(db, audit, pii_key, kyc_id="KYC-2001", document_type="cccd", actor=actor, reveal=True, reason=reason)
            denied.append(f"{label}: được phép")
        except RevealDenied:
            pass
    report.check("xem số rõ thiếu scope hoặc lý do ngắn/trống: RevealDenied, không trả gì", not denied, str(denied))

    class BrokenAudit:
        async def record(self, **kwargs: object) -> None:
            raise OSError("kho audit không ghi được")

    try:
        await view_document(db, BrokenAudit(), pii_key, kyc_id="KYC-2001", document_type="cccd", actor=REVIEWER,
                            reveal=True, reason="đối chiếu hồ sơ trùng số CCCD")
        outcome = "đã trả dữ liệu"
    except OSError:
        outcome = "OSError"
    report.check("audit không ghi được: ngoại lệ nổi lên, không có dữ liệu nào được trả (fail closed)", outcome == "OSError")

    rows = await db[AUDIT].find({}, {"_id": 0}).sort([("at", 1), ("_id", 1)]).to_list(length=None)
    say(report, "nhật ký audit (không in expire_at):")
    for row in rows:
        say(report, " ", {k: v for k, v in row.items() if k != "expire_at"} | {"at": row["at"].isoformat()})
    expected = ["lookup"] * 4 + ["lookup_denied", "view", "reveal", "reveal_denied", "reveal_denied", "reveal_denied"]
    report.check("audit ghi đủ 10 dòng theo thứ tự: 4 lookup, 1 lookup_denied, view, reveal, 3 reveal_denied",
                 [r["action"] for r in rows] == expected, str([r["action"] for r in rows]))
    reveal_row = next(r for r in rows if r["action"] == "reveal")
    report.check("dòng audit có actor, kyc_id, lý do; hết hạn sau 400 ngày; không chứa số giấy tờ (rõ hay che)",
                 reveal_row["actor"] == "DP-001" and reveal_row["kyc_id"] == "KYC-2001" and reveal_row["reason"]
                 and reveal_row["expire_at"] - reveal_row["at"] == AUDIT_RETENTION
                 and shared not in dump(rows) and "099*******" not in dump(rows))

    # Hết hạn lưu bản gốc: xem mặc định vẫn được, xin số rõ nhận raw_expired
    latest = await db[DOCS].find_one({"kyc_id": "KYC-0007", "document_type": "cccd"}, sort=[("created_at", -1), ("_id", -1)])
    assert latest is not None
    await db[RAW].delete_one({"_id": latest["raw_id"]})  # raw của bản mới nhất
    gone = await view_document(db, audit, pii_key, kyc_id="KYC-0007", document_type="cccd", actor=REVIEWER,
                               reveal=True, reason="kiểm tra sau khi hết hạn lưu")
    report.check("bản gốc đã hết hạn: vẫn xem được dạng che; xin số rõ nhận raw_expired thay vì lỗi",
                 gone.raw_expired and gone.revealed is None and gone.document["fields"]["id_number_masked"] == "099*******83")


async def part_migration(report: Report, client: AsyncMongoClient[dict[str, Any]], db: AsyncDatabase[dict[str, Any]],
                         audit: MongoAuditSink, pii_key: PiiKey) -> None:
    say(report, "\n== 6. Migration schema v2 (bài 26) -> v3 ==")
    now = datetime.now(UTC).replace(microsecond=0)
    ages = [60, 45, 40, 5, 4, 3]  # ba document cũ hơn 30 ngày (raw đã hết hạn), ba document mới
    legacy = [legacy_v2(500 + i, f"KYC-{3000 + i}", now - timedelta(days=age)) for i, age in enumerate(ages)]
    await db[DOCS].insert_many(legacy)
    plain_ids = [d["fields"]["id_number"] for d in legacy]
    groups = {g["_id"]: g["n"] async for g in await db[DOCS].aggregate([{"$group": {"_id": "$schema_version", "n": {"$sum": 1}}}])}
    say(report, "theo schema_version trước khi migrate:", dict(sorted(groups.items())))
    leaked_before = await db[DOCS].count_documents({"fields.id_number": {"$exists": True}})
    say(report, "document còn số rõ trong fields:", leaked_before)
    report.check("trước migrate: 6 document v2 của bài 26 mang số rõ (validator đang cho phép v2)", groups.get(2) == 6 and leaked_before == 6, str(groups))

    old = await db[DOCS].find_one({"kyc_id": "KYC-3000"})
    assert old is not None
    upgraded = docs.normalize_to_v3(old, pii_key)
    say(report, "đọc v2 như v3 (upgrade on read):", dump(upgraded["fields"]))
    report.check("upgrade on read: v2 đọc ra hình dạng v3 đã che, số rõ không qua tầng trên; DB chưa đổi",
                 upgraded["schema_version"] == 3 and plain_ids[0] not in dump(upgraded)
                 and upgraded["fields"]["id_number_masked"] == mask_digits(plain_ids[0])
                 and (await db[DOCS].find_one({"kyc_id": "KYC-3000"}))["schema_version"] == 2)  # type: ignore[index]
    mixed = await view_document(db, audit, pii_key, kyc_id="KYC-3000", document_type="cccd", actor=VIEWER)
    report.check("view_document trên document v2 chưa migrate: vẫn trả dạng che", plain_ids[0] not in dump(mixed.document))
    legacy_reveal = await view_document(db, audit, pii_key, kyc_id="KYC-3000", document_type="cccd", actor=REVIEWER,
                                        reveal=True, reason="đối chiếu hồ sơ cũ chưa migrate")
    report.check("reveal trên document v2 chưa migrate: số rõ còn trong document nên được trả, không báo raw_expired sai",
                 legacy_reveal.revealed == {"id_number": plain_ids[0]} and not legacy_reveal.raw_expired)
    try:
        docs.normalize_to_v3({**old, "schema_version": 1}, pii_key)
        outcome = "không lỗi"
    except ValueError:
        outcome = "ValueError"
    report.check("document v1 của bài 26 chưa qua migration v1 sang v2: ValueError rõ ràng, không đoán", outcome == "ValueError")

    first, second = await asyncio.gather(docs.migrate_v2_to_v3(db, pii_key, batch=2), docs.migrate_v2_to_v3(db, pii_key, batch=2))
    say(report, f"hai tác vụ migrate chạy đồng thời trong một tiến trình: sửa {first} và {second} document")
    groups_after = {g["_id"]: g["n"] async for g in await db[DOCS].aggregate([{"$group": {"_id": "$schema_version", "n": {"$sum": 1}}}])}
    report.check("hai tác vụ migrate đồng thời: tổng số document được sửa đúng 6, không bản nào sửa lần hai",
                 first + second == 6, f"{first}+{second}")
    leaked = [i for i in plain_ids if i in dump(await db[DOCS].find({}).to_list(length=None))]
    report.check("sau migrate: không còn v2, không còn trường id_number, không số rõ nào trong cả collection",
                 groups_after.get(2) is None and await db[DOCS].count_documents({"fields.id_number": {"$exists": True}}) == 0
                 and not leaked, f"{groups_after}, rò {len(leaked)}")
    migrated = await db[DOCS].find_one({"kyc_id": "KYC-3000"})
    assert migrated is not None
    report.check("document đã migrate bằng đúng kết quả của upgrade on read (che, băm, cờ needs_review)",
                 migrated["fields"] == upgraded["fields"] and migrated["needs_review"] is False and migrated["pii_key_id"] == pii_key.key_id)

    raws = {d["kyc_id"]: d async for d in db[RAW].find({"kyc_id": {"$in": [d["kyc_id"] for d in legacy]}})}
    say(report, "raw tạo ra khi migrate:", {k: v["expire_at"].date().isoformat() for k, v in sorted(raws.items())})
    report.check("migrate đưa số rõ vào raw với expire_at = created_at + 30 ngày (không kéo dài thời hạn lưu cũ)",
                 len(raws) == 6 and all(d["expire_at"] == d["created_at"] + RAW_RETENTION for d in raws.values()))
    expired_now = await db[RAW].count_documents({"expire_at": {"$lt": now}, "migrated_from": 2})
    say(report, "trong đó đã quá hạn ngay lúc migrate (sẽ bị TTL dọn):", expired_now)
    report.check("ba document cũ hơn 30 ngày sinh ra raw đã quá hạn; ba document mới thì chưa", expired_now == 3, str(expired_now))

    async with fast_ttl_monitor(client, 1):
        gone_old = await wait_until_gone(db[RAW], {"migrated_from": 2, "expire_at": {"$lt": now}}, 20)
    recent = await db[RAW].count_documents({"migrated_from": 2})
    say(report, f"TTL dọn raw đã quá hạn sau {gone_old and round(gone_old, 1)} giây; raw còn lại: {recent}")
    report.check("TTL dọn ba raw quá hạn do migrate tạo ra; ba raw còn hạn ở lại", gone_old is not None and recent == 3, str(recent))

    remaining = await db[DOCS].count_documents({"schema_version": {"$lt": 3}})
    report.check("trước khi siết validator: không còn document nào dưới v3", remaining == 0, str(remaining))
    if remaining == 0:
        await db.command({"collMod": DOCS, "validator": docs.documents_validator(allow_legacy=False)})
    try:
        await db[DOCS].insert_one({k: v for k, v in legacy[0].items() if k != "_id"} | {"model_version": "demo-ocr-0.8"})
        code = None
    except WriteError as exc:
        code = exc.code
    say(report, "sau collMod siết validator, chèn v2 -> WriteError", code)
    report.check("siết validator chỉ còn v3: ghi v2 (số rõ) bị từ chối (mã 121)", code == 121)


async def run(verbose: bool) -> int:
    report = Report(verbose)
    client: AsyncMongoClient[dict[str, Any]] = AsyncMongoClient(MONGO_URL, tz_aware=True, serverSelectionTimeoutMS=2000)
    try:
        await client.admin.command("ping")
    except PyMongoError:
        await client.close()
        print(f"FAIL: không nối được MongoDB tại {MONGO_URL} ({COMPOSE})")
        return 1
    # SIGTERM huỷ tác vụ chính để `finally` trả lại ttlMonitorSleepSecs và dọn collection.
    asyncio.get_running_loop().add_signal_handler(signal.SIGTERM, asyncio.current_task().cancel)  # type: ignore[union-attr]
    db = client.get_default_database("toinoi")
    pii_key = load_pii_key()
    audit = MongoAuditSink(db[AUDIT])
    try:
        await docs.reset_and_setup(db)
        await part_pii(report, pii_key)
        await part_save(report, db, pii_key)
        await part_lookup(report, db, audit, pii_key)
        await part_retention(report, client, db)
        await part_audit(report, db, audit, pii_key)
        await part_migration(report, client, db, audit, pii_key)
    finally:
        for name in (DOCS, RAW, AUDIT):
            await db[name].drop()  # dọn dẹp
        await client.close()
    passed = sum(ok for _, ok in report.results)
    total = len(report.results)
    if passed == total:
        print(f"OK {passed}/{total}" if verbose else f"\nĐã dọn ba collection lab47. Kiểm chứng: {passed}/{total} ca đạt.")
        return 0
    for name, ok in report.results:
        if not ok:
            print(f"SAI: {name}")
    print(f"FAIL {passed}/{total}")
    return 1


def main() -> None:
    verbose = "--verify" in sys.argv[1:]
    print(f"MongoDB {MONGO_URL}; khóa băm của lab (không dùng ngoài lab); {load_average()}" if verbose else load_average())
    sys.exit(asyncio.run(run(verbose)))


if __name__ == "__main__":
    main()
