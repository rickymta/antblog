"""Bài 15: pydantic v2 - validate hồ sơ tài xế và cấu hình ứng dụng.

Chạy:   uv run --project <lab> <lab>/main.py
Verify: uv run --project <lab> <lab>/main.py --verify
"""

import json
import os
import sys
import tempfile
from contextlib import AbstractContextManager
from decimal import Decimal
from pathlib import Path
from unittest.mock import patch

from pydantic import ConfigDict, ValidationError

from models import (
    DriverProfile,
    DriverRecord,
    FrozenDriverProfile,
    QuoteRequest,
    ServiceKind,
    StrictQuoteBody,
    StrictQuoteRequest,
    error_report,
    sample_payload,
    updated,
)
from settings import ENV_PREFIX, AppSettings, settings_error_report


def clean_env(**overrides: str) -> AbstractContextManager[object]:
    """Môi trường tạm: bỏ mọi TOINOI_* đang có trên máy, rồi thêm các giá trị ghi đè."""
    env = {k: v for k, v in os.environ.items() if not k.startswith(ENV_PREFIX)}
    env.update(overrides)
    return patch.dict(os.environ, env, clear=True)


def show(title: str) -> None:
    print(f"\n== {title} ==")


def demo_dataclass_vs_model() -> None:
    show("dataclass nhận mọi thứ, BaseModel thì không")
    record = DriverRecord(driver_id=123, full_name=None, phone="abc")  # type: ignore[arg-type]
    print("dataclass:", record)
    try:
        DriverProfile.model_validate(
            sample_payload(driver_id=123, full_name=None, phone="abc")
        )
    except ValidationError as exc:
        print(json.dumps(error_report(exc), ensure_ascii=False, indent=2))


def demo_valid_profile() -> None:
    show("payload hợp lệ: chuẩn hóa và xuất ra")
    profile = DriverProfile.model_validate(
        sample_payload(full_name="  Nguyễn   Văn A ", phone="0900.000.001")
    )
    print("repr      :", repr(profile))
    print("phone     :", profile.phone)
    print("model_dump đầy đủ:", profile.model_dump())
    print("model_dump(exclude):", profile.model_dump(exclude={"id_number", "license_number"}))
    print("json      :", profile.model_dump_json(exclude={"id_number", "license_number"}))
    legacy = DriverProfile.model_validate(
        {k: v for k, v in sample_payload().items() if k != "full_name"}
        | {"fullName": "Trần Thị B"}
    )
    print("alias     :", legacy.full_name)


def demo_errors() -> None:
    show("nhiều lỗi cùng lúc, có cấu trúc")
    bad = sample_payload(
        driver_id="DR-1",
        id_number="12345",
        license_number="990000000001",
        phone="12",
        city="Huế",
        extra_field=1,
    )
    try:
        DriverProfile.model_validate(bad)
    except ValidationError as exc:
        print(f"{exc.error_count()} lỗi")
        print(json.dumps(error_report(exc), ensure_ascii=False, indent=2))
    try:
        DriverProfile.model_validate(
            sample_payload(license_number="099000000001")
        )
    except ValidationError as exc:
        print("model_validator:", error_report(exc))


def demo_json_and_strict() -> None:
    show("JSON, ép kiểu lax, strict và từ chối float")
    for raw in ('{"distance_km": "2.5"}', '{"distance_km": 2, "service": "food"}'):
        req = QuoteRequest.model_validate_json(raw)
        print(f"{raw} -> {req!r}")
    for raw in ('{"distance_km": 2.5}', '{"distance_km": "100.5"}', '{"distance_km": "abc"}',
                '{"service": "taxi"}'):
        try:
            QuoteRequest.model_validate_json(raw)
        except ValidationError as exc:
            print(f"{raw} -> {error_report(exc)}")
    req = QuoteRequest.model_validate_json('{"distance_km": "2.5", "service": "food"}')
    print("model_dump         :", req.model_dump())
    print("model_dump(json)   :", req.model_dump(mode="json"))
    try:
        StrictQuoteBody.model_validate({"distance_km": "2.5"})
    except ValidationError as exc:
        print("strict, python mode, chuỗi:", error_report(exc))
    print("strict, python mode, Decimal:", StrictQuoteBody.model_validate({"distance_km": Decimal("2.5")}))
    print("strict, JSON, chuỗi        :", StrictQuoteBody.model_validate_json('{"distance_km": "2.5"}'))
    try:
        StrictQuoteRequest.model_validate_json('{"distance_km": "2.5"}')
    except ValidationError as exc:
        print("strict + validator before, JSON, chuỗi:", error_report(exc))


def demo_validate_assignment() -> None:
    show("gán lại thuộc tính: validate_assignment, giới hạn và frozen")
    profile = DriverProfile.model_validate(sample_payload())
    try:
        profile.phone = "abc"
    except ValidationError as exc:
        print("lỗi ở trường  :", error_report(exc))
    print("phone vẫn là  :", profile.phone)
    try:
        profile.license_number = profile.id_number
    except ValidationError as exc:
        print("lỗi cấp model :", error_report(exc))
    print("nhưng license_number đã thành:", profile.license_number)
    frozen = FrozenDriverProfile.model_validate(sample_payload())
    try:
        frozen.phone = "0911111111"
    except ValidationError as exc:
        print("frozen        :", error_report(exc))
    fresh = updated(frozen, phone="0911 111 111")
    print("bản mới       :", fresh.phone, "| bản cũ:", frozen.phone)
    try:
        updated(frozen, license_number=frozen.id_number)
    except ValidationError as exc:
        print("bản mới sai   :", error_report(exc), "| bản cũ giữ nguyên:", frozen.license_number)


def demo_errors_leak() -> None:
    show("lỗi không làm rò số giấy tờ")
    try:
        DriverProfile.model_validate(sample_payload(id_number="099000000"))
    except ValidationError as exc:
        print(str(exc))
        print("errors()[0]['input']:", exc.errors(include_url=False)[0]["input"])


def demo_settings() -> None:
    show("BaseSettings: mặc định, env, .env, tham số")
    with clean_env():
        print("mặc định        :", AppSettings())
    with clean_env(
        TOINOI_QUOTE_TTL_SECONDS="600", TOINOI_DEBUG="yes", TOINOI_DEFAULT_CITY="Đà Nẵng"
    ):
        s = AppSettings()
        print("từ env          :", s.quote_ttl_seconds, s.debug, s.default_city)
        print("tham số thắng env:", AppSettings(quote_ttl_seconds=45).quote_ttl_seconds)
    with tempfile.TemporaryDirectory() as tmp:
        dotenv = Path(tmp) / ".env"
        dotenv.write_text(
            "TOINOI_QUOTE_TTL_SECONDS=120\nTOINOI_DEFAULT_CITY=TP.HCM\n", encoding="utf-8"
        )
        with clean_env(TOINOI_QUOTE_TTL_SECONDS="600"):
            s = AppSettings(_env_file=dotenv)
            print("env thắng .env  :", s.quote_ttl_seconds, "| .env cho city:", s.default_city)
    with clean_env(TOINOI_PG_URL="postgresql://toinoi:toinoi@db.noi-bo:5432/toinoi"):
        s = AppSettings()
        print("repr SecretStr  :", repr(s.pg_url))
        print("giá trị thật    :", s.pg_url.get_secret_value())
    with clean_env(TOINOI_QUOTE_TTL_SECONDS="5", TOINOI_DEBUG="maybe", TOINOI_DEFAULT_CITY="Huế"):
        try:
            AppSettings()
        except ValidationError as exc:
            print("cấu hình sai, dừng khi khởi động:")
            for line in settings_error_report(exc):
                print("  " + line)


def verify() -> None:
    """So các kết quả kỳ vọng đã in trong bài; sai thì in FAIL và thoát với mã 1."""
    failures: list[str] = []
    total = 0

    def check(name: str, got: object, expected: object) -> None:
        nonlocal total
        total += 1
        if got != expected:
            failures.append(name)
            print(f"SAI: {name}: nhận {got!r}, kỳ vọng {expected!r}")

    def codes(payload: dict[str, object]) -> list[tuple[str, str]]:
        try:
            DriverProfile.model_validate(payload)
        except ValidationError as exc:
            return [(e["field"], e["code"]) for e in error_report(exc)]
        return []

    # dataclass không kiểm
    check("dataclass nhận rác", DriverRecord(driver_id=123, full_name=None, phone="abc").driver_id, 123)  # type: ignore[arg-type]

    # hồ sơ hợp lệ + chuẩn hóa
    profile = DriverProfile.model_validate(sample_payload(full_name="  Nguyễn   Văn A ", phone="0900.000.001"))
    check("chuẩn hóa tên", profile.full_name, "Nguyễn Văn A")
    check("chuẩn hóa phone", profile.phone, "0900000001")
    check("repr ẩn số giấy tờ", "099000000001" in repr(profile), False)
    check("repr=False không ẩn khỏi model_dump", profile.model_dump()["id_number"], "099000000001")
    check("model_dump loại giấy tờ", sorted(profile.model_dump(exclude={"id_number", "license_number"})),
          ["city", "driver_id", "full_name", "phone"])
    check("alias fullName", DriverProfile.model_validate(
        {k: v for k, v in sample_payload().items() if k != "full_name"} | {"fullName": "Trần Thị B"}
    ).full_name, "Trần Thị B")

    # lỗi có cấu trúc
    check("driver_id sai", codes(sample_payload(driver_id="DR-1")), [("driver_id", "string_pattern_mismatch")])
    check("CCCD ngắn", codes(sample_payload(id_number="12345")), [("id_number", "digits_12")])
    check("GPLX có chữ", codes(sample_payload(license_number="99abc")), [("license_number", "digits_12")])
    check("phone sai", codes(sample_payload(phone="12")), [("phone", "phone_format")])
    check("thành phố lạ", codes(sample_payload(city="Huế")), [("city", "literal_error")])
    check("khóa lạ", codes(sample_payload(extra_field=1)), [("extra_field", "extra_forbidden")])
    check("thiếu khóa", codes({k: v for k, v in sample_payload().items() if k != "phone"}), [("phone", "missing")])
    check("driver_id kiểu số", codes(sample_payload(driver_id=123)), [("driver_id", "string_type")])
    check("hai giấy tờ trùng", codes(sample_payload(license_number="099000000001")), [("", "documents_same")])
    check("nhiều lỗi", len(codes(sample_payload(driver_id="x", id_number="1", phone="1", city="?"))), 4)
    try:
        profile.phone = "abc"
        assigned = True
    except ValidationError:
        assigned = False
    check("validate_assignment", (assigned, profile.phone), (False, "0900000001"))
    check("model_validator chỉ chạy khi các trường đã đạt",
          codes(sample_payload(id_number="099000000001", license_number="099000000001", phone="x")),
          [("phone", "phone_format")])

    # giới hạn: model_validator(after) lỗi khi gán -> báo lỗi nhưng object ĐÃ đổi
    mutable = DriverProfile.model_validate(sample_payload())
    try:
        mutable.license_number = mutable.id_number
        raised = False
    except ValidationError as exc:
        raised = [e["code"] for e in error_report(exc)] == ["documents_same"]
    check("gán vi phạm quy tắc chéo trường: có lỗi", raised, True)
    check("... nhưng object đã bị đổi", mutable.license_number, mutable.id_number)
    frozen = FrozenDriverProfile.model_validate(sample_payload())
    try:
        frozen.phone = "0911111111"
        frozen_codes: list[str] = []
    except ValidationError as exc:
        frozen_codes = [e["code"] for e in error_report(exc)]
    check("frozen chặn gán", (frozen_codes, frozen.phone), (["frozen_instance"], "0900000001"))
    fresh = updated(frozen, phone="0911 111 111")
    check("updated dựng bản mới", (fresh.phone, frozen.phone, fresh is frozen), ("0911111111", "0900000001", False))
    try:
        updated(frozen, license_number=frozen.id_number)
        updated_codes: list[str] = []
    except ValidationError as exc:
        updated_codes = [e["code"] for e in error_report(exc)]
    check("updated kiểm lại cả quy tắc chéo", (updated_codes, frozen.license_number), (["documents_same"], "990000000001"))

    # chữ số Unicode không lọt qua [0-9]
    arabic = "٠٩٩٠٠٠٠٠٠٠٠١"  # 12 chữ số Ả Rập-Ấn Độ
    check("CCCD chữ số Ả Rập bị từ chối", codes(sample_payload(id_number=arabic)), [("id_number", "digits_12")])
    check("driver_id chữ số Ả Rập bị từ chối", codes(sample_payload(driver_id="DR-٠٠١")), [("driver_id", "string_pattern_mismatch")])
    check("phone chữ số Ả Rập bị từ chối", codes(sample_payload(phone="٠٩٠٠٠٠٠٠٠١")), [("phone", "phone_format")])

    # hide_input_in_errors: chuỗi lỗi không chứa input, errors() vẫn có
    try:
        DriverProfile.model_validate(sample_payload(id_number="099000000"))
        text, raw_input = "", None
    except ValidationError as exc:
        text, raw_input = str(exc), exc.errors(include_url=False)[0]["input"]
    check("str(ValidationError) không in input_value", ("input_value" in text, "099000000" in text), (False, False))
    check("errors() vẫn giữ khóa input", raw_input, "099000000")
    try:
        DriverProfile.model_validate(sample_payload(license_number="099000000001", id_number="099000000001"))
        model_level = ""
    except ValidationError as exc:
        model_level = str(exc)
    check("lỗi cấp model không in payload", "099000000001" in model_level, False)

    class Loose(DriverProfile):  # cùng quy tắc, nhưng không kiểm khi gán lại
        model_config = ConfigDict(validate_assignment=False)

    loose = Loose.model_validate(sample_payload())
    loose.city = "Huế"  # type: ignore[assignment]
    check("không validate_assignment: gán sai lọt qua", loose.city, "Huế")

    # QuoteRequest: JSON, lax, strict
    check("JSON chuỗi -> Decimal", QuoteRequest.model_validate_json('{"distance_km": "2.5"}').distance_km, Decimal("2.5"))
    check("JSON số nguyên -> Decimal", QuoteRequest.model_validate_json('{"distance_km": 2, "service": "food"}').service, ServiceKind.FOOD)
    check("model_dump giữ Decimal và StrEnum", QuoteRequest(distance_km=Decimal("2.5"), service=ServiceKind.FOOD).model_dump(),
          {"distance_km": Decimal("2.5"), "service": ServiceKind.FOOD})
    check("model_dump(mode='json') ra chuỗi", QuoteRequest(distance_km=Decimal("2.5"), service=ServiceKind.FOOD).model_dump(mode="json"),
          {"distance_km": "2.5", "service": "food"})
    for raw, code in [('{"distance_km": 2.5}', "float_not_allowed"), ('{"distance_km": "100.5"}', "less_than_equal"),
                      ('{"distance_km": "abc"}', "decimal_parsing"), ('{"service": "taxi"}', "missing"),
                      ('{"distance_km": "1.0001"}', "decimal_max_places")]:
        try:
            QuoteRequest.model_validate_json(raw)
            got: list[str] = []
        except ValidationError as exc:
            got = [e["code"] for e in error_report(exc)]
        check(f"quote {raw}", code in got, True)
    try:
        StrictQuoteBody.model_validate({"distance_km": "2.5"})
        strict_ok = True
    except ValidationError:
        strict_ok = False
    check("strict từ chối chuỗi (python mode)", strict_ok, False)
    check("strict, JSON, chuỗi vẫn được nhận", StrictQuoteBody.model_validate_json('{"distance_km": "2.5"}').distance_km, Decimal("2.5"))
    try:
        QuoteRequest.model_validate({"distance_km": 2.5})
        python_float = "không lỗi"
    except ValidationError as exc:
        python_float = error_report(exc)[0]["code"]
    check("float Python bị từ chối", python_float, "float_not_allowed")
    check("strict nhận Decimal", StrictQuoteBody.model_validate({"distance_km": Decimal("2.5")}).distance_km, Decimal("2.5"))

    # settings
    with clean_env():
        d = AppSettings()
        check("settings mặc định", (d.quote_ttl_seconds, d.debug, d.default_city, d.max_distance_km),
              (300, False, "Hà Nội", Decimal("100")))
        check("SecretStr ẩn", "toinoi:toinoi" in repr(d.pg_url), False)
        check("SecretStr lấy thật", d.pg_url.get_secret_value().endswith("localhost:54329/toinoi"), True)
    with clean_env(TOINOI_QUOTE_TTL_SECONDS="600", TOINOI_DEBUG="yes", TOINOI_DEFAULT_CITY="Đà Nẵng"):
        e = AppSettings()
        check("settings từ env", (e.quote_ttl_seconds, e.debug, e.default_city), (600, True, "Đà Nẵng"))
        check("tham số thắng env", AppSettings(quote_ttl_seconds=45).quote_ttl_seconds, 45)
    with tempfile.TemporaryDirectory() as tmp:
        dotenv = Path(tmp) / ".env"
        dotenv.write_text("TOINOI_QUOTE_TTL_SECONDS=120\nTOINOI_DEFAULT_CITY=TP.HCM\n", encoding="utf-8")
        with clean_env(TOINOI_QUOTE_TTL_SECONDS="600"):
            f = AppSettings(_env_file=dotenv)
            check("env thắng .env", f.quote_ttl_seconds, 600)
            check(".env thắng mặc định", f.default_city, "TP.HCM")
    with clean_env(TOINOI_QUOTE_TTL_SECONDS="5", TOINOI_DEBUG="maybe", TOINOI_DEFAULT_CITY="Huế"):
        try:
            AppSettings()
            lines: list[str] = []
        except ValidationError as exc:
            lines = settings_error_report(exc)
        check("settings sai: số dòng lỗi", len(lines), 3)
        check("settings sai: nêu tên biến", all(line.startswith(ENV_PREFIX) for line in lines), True)

    if failures:
        print(f"FAIL {total - len(failures)}/{total}")
        sys.exit(1)
    print(f"OK {total}/{total}")


def main() -> None:
    demo_dataclass_vs_model()
    demo_valid_profile()
    demo_errors()
    demo_json_and_strict()
    demo_validate_assignment()
    demo_errors_leak()
    demo_settings()


if __name__ == "__main__":
    if "--verify" in sys.argv[1:]:
        verify()
    else:
        main()
