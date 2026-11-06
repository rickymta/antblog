"""Lab 67: service hồ sơ KYC chia bốn lớp. Chạy demo, hoặc `--verify` để kiểm các kết quả trong bài.

    uv run --project examples/toi-noi-python/67-clean-architecture-layers \
        examples/toi-noi-python/67-clean-architecture-layers/main.py [--verify]
"""

import sys
import warnings
from collections.abc import Callable
from datetime import UTC, datetime

from fastapi.testclient import TestClient

from kyc_service.api.app import create_app
from kyc_service.api.container import Container, build_container
from kyc_service.application.errors import ApplicationNotFound
from kyc_service.application.use_cases import SubmitDriverApplicationRequest
from kyc_service.application.views import ApplicationView
from kyc_service.domain.errors import DomainError, InvalidStatusTransition
from kyc_service.domain.kyc import DocumentImage, DocumentKind, KycApplication, KycStatus, RiskScore
from kyc_service.infrastructure.clocks import FixedClock
from kyc_service.infrastructure.fake_services import FakeFaceMatch, FakeOcr
from kyc_service.infrastructure.memory_repository import InMemoryKycRepository

# Demo cố ý dùng adapter giả nên tắt cảnh báo của build_container(); một ca --verify riêng kiểm
# rằng cảnh báo vẫn được phát khi không tắt.
warnings.filterwarnings("ignore", message=r"build_container\(\) dùng adapter giả")

CLOCK = FixedClock(datetime(2026, 11, 6, 3, 0, tzinfo=UTC))


def request_for(driver: str, *, consent: bool = True) -> SubmitDriverApplicationRequest:
    return SubmitDriverApplicationRequest(
        driver_id=driver,
        cccd_ref=f"synthetic/cccd/{driver}.png",
        gplx_ref=f"synthetic/gplx/{driver}.png",
        selfie_ref=f"synthetic/selfie/{driver}.png",
        face_consent=consent,
    )


def show(view: ApplicationView) -> None:
    ids = ",".join(o.id_number_masked for o in view.ocr if o.id_number_masked) or "-"
    face = f"{view.face_score:.2f}" if view.face_score is not None else "-"
    risk = f"{view.risk_score:.2f}" if view.risk_score is not None else "-"
    pending = f" pending={view.pending_check}" if view.pending_check else ""
    print(
        f"{view.application_id} {view.driver_id} {view.status:<12} id={ids:<14}"
        f" face={face:<4} risk={risk:<4}{pending}"
    )
    for reason in view.risk_reasons:
        print(f"    lý do rủi ro: {reason}")
    if view.note:
        print(f"    ghi chú: {view.note}")


def demo() -> None:
    print("== Ba hồ sơ, mọi service sẵn sàng")
    container = build_container(clock=CLOCK)
    for driver, consent in (("DR-001", True), ("DR-002", True), ("DR-003", False)):
        show(container.submit.execute(request_for(driver, consent=consent)))

    print("== Người duyệt RV-001 quyết định KYC-0001, rồi thử quyết định lần hai")
    show(
        container.review.execute(
            "KYC-0001", reviewer_id="RV-001", decision=KycStatus.APPROVED, reason="ảnh rõ, khớp"
        )
    )
    try:
        container.review.execute(
            "KYC-0001", reviewer_id="RV-001", decision=KycStatus.REJECTED, reason="đổi ý"
        )
    except InvalidStatusTransition as exc:
        print(f"    bị từ chối: {exc}")

    print("== kyc-ocr tắt: hồ sơ vẫn được nhận")
    down = build_container(ocr=FakeOcr(available=False), clock=CLOCK)
    show(down.submit.execute(request_for("DR-001")))


class ExplodingRisk:
    """Lỗi lập trình bất ngờ (không phải PortFailure) trong lúc chấm điểm."""

    def score(self, application: object) -> RiskScore:
        raise RuntimeError("bug trong fraud-scoring adapter")


def run_verify() -> int:
    cases: list[tuple[str, Callable[[], bool]]] = []

    def case(name: str) -> Callable[[Callable[[], bool]], Callable[[], bool]]:
        def register(fn: Callable[[], bool]) -> Callable[[], bool]:
            cases.append((name, fn))
            return fn

        return register

    def fresh(**kwargs: object) -> Container:
        return build_container(clock=CLOCK, **kwargs)  # type: ignore[arg-type]

    @case("ba hồ sơ đều dừng ở needs_review, không hồ sơ nào tự approved/rejected")
    def _() -> bool:
        c = fresh()
        views = [
            c.submit.execute(request_for(d, consent=k))
            for d, k in (("DR-001", True), ("DR-002", True), ("DR-003", False))
        ]
        return [v.status for v in views] == ["needs_review"] * 3

    @case("số giấy tờ ra ngoài dạng mặt nạ 099*******83")
    def _() -> bool:
        view = fresh().submit.execute(request_for("DR-001"))
        masked = {o.id_number_masked for o in view.ocr}
        return "099*******83" in masked and "099090000183" not in repr(view)

    @case("DR-002 có hai lý do rủi ro và điểm cao hơn DR-001")
    def _() -> bool:
        c = fresh()
        a, b = c.submit.execute(request_for("DR-001")), c.submit.execute(request_for("DR-002"))
        return len(a.risk_reasons) == 0 and len(b.risk_reasons) == 2 and b.risk_score > a.risk_score  # type: ignore[operator]

    @case("không đồng ý khuôn mặt: kyc-face và fraud-scoring không được gọi")
    def _() -> bool:
        face = FakeFaceMatch()
        c = fresh(face=face)
        view = c.submit.execute(request_for("DR-003", consent=False))
        return (
            face.calls == 0
            and view.face_score is None
            and view.risk_score is None
            and view.status == "needs_review"
        )

    @case("kyc-ocr tắt: hồ sơ vẫn được nhận ở submitted, pending_check=kyc-ocr")
    def _() -> bool:
        repo = InMemoryKycRepository()
        view = fresh(repository=repo, ocr=FakeOcr(available=False)).submit.execute(
            request_for("DR-001")
        )
        return view.status == "submitted" and view.pending_check == "kyc-ocr" and repo.count() == 1

    @case("kyc-face tắt: hồ sơ giữ kết quả OCR ở ocr_done, pending_check=kyc-face")
    def _() -> bool:
        view = fresh(face=FakeFaceMatch(available=False)).submit.execute(request_for("DR-001"))
        return view.status == "ocr_done" and view.pending_check == "kyc-face" and len(view.ocr) == 2

    @case("kyc-face trả điểm 1.7 (sai miền): adapter dịch thành lỗi, hồ sơ không nhận điểm đó")
    def _() -> bool:
        view = fresh(face=FakeFaceMatch(score_override=1.7)).submit.execute(request_for("DR-001"))
        return (
            view.status == "ocr_done"
            and view.face_score is None
            and view.pending_check == "kyc-face"
        )

    @case("người duyệt approved hồ sơ needs_review: ghi RV-001")
    def _() -> bool:
        c = fresh()
        c.submit.execute(request_for("DR-001"))
        view = c.review.execute(
            "KYC-0001", reviewer_id="RV-001", decision=KycStatus.APPROVED, reason="khớp"
        )
        return view.status == "approved" and view.reviewed_by == "RV-001"

    def raises(exc: type[Exception], fn: Callable[[], object]) -> bool:
        try:
            fn()
        except exc:
            return True
        return False

    @case("approved/rejected chỉ từ needs_review: hồ sơ submitted bị từ chối")
    def _() -> bool:
        c = fresh(ocr=FakeOcr(available=False))
        c.submit.execute(request_for("DR-001"))
        return raises(
            InvalidStatusTransition,
            lambda: c.review.execute(
                "KYC-0001", reviewer_id="RV-001", decision=KycStatus.APPROVED, reason="ép duyệt"
            ),
        )

    @case("quyết định lần hai sau approved bị từ chối")
    def _() -> bool:
        c = fresh()
        c.submit.execute(request_for("DR-001"))
        c.review.execute(
            "KYC-0001", reviewer_id="RV-001", decision=KycStatus.APPROVED, reason="khớp"
        )
        return raises(
            InvalidStatusTransition,
            lambda: c.review.execute(
                "KYC-0001", reviewer_id="RV-001", decision=KycStatus.REJECTED, reason="đổi ý"
            ),
        )

    @case("quyết định không phải approved/rejected bị từ chối")
    def _() -> bool:
        c = fresh()
        c.submit.execute(request_for("DR-001"))
        return raises(
            DomainError,
            lambda: c.review.execute(
                "KYC-0001", reviewer_id="RV-001", decision=KycStatus.NEEDS_REVIEW, reason="x"
            ),
        )

    @case("hồ sơ không tồn tại: ApplicationNotFound")
    def _() -> bool:
        return raises(ApplicationNotFound, lambda: fresh().get.execute("KYC-9999"))

    @case("hai CCCD thay vì CCCD + GPLX: domain từ chối")
    def _() -> bool:
        two_cccd = (DocumentImage(DocumentKind.CCCD, "a"), DocumentImage(DocumentKind.CCCD, "b"))
        return raises(
            DomainError,
            lambda: KycApplication.submit(
                application_id="KYC-0001",
                driver_id="DR-001",
                documents=two_cccd,
                selfie_ref="s",
                face_consent=True,
                now=CLOCK.now(),
            ),
        )

    @case("ảnh GPLX nộp vào ô CCCD: kyc-ocr báo sai loại, hồ sơ dừng ở submitted")
    def _() -> bool:
        lie = SubmitDriverApplicationRequest(
            "DR-001",
            "synthetic/gplx/DR-001.png",
            "synthetic/gplx/DR-001.png",
            "synthetic/selfie/DR-001.png",
            True,
        )
        view = fresh().submit.execute(lie)
        return view.status == "submitted" and view.pending_check == "kyc-ocr" and not view.ocr

    @case("kyc-face trả cosine âm -0.4: vẫn tới người duyệt, điểm rủi ro nằm trong 0..1")
    def _() -> bool:
        view = fresh(face=FakeFaceMatch(score_override=-0.4)).submit.execute(request_for("DR-001"))
        return view.face_score == -0.4 and view.risk_score is not None and 0 <= view.risk_score <= 1

    @case("build_container() không truyền adapter: cảnh báo dùng adapter giả")
    def _() -> bool:
        with warnings.catch_warnings(record=True) as caught:
            warnings.simplefilter("always")
            build_container(clock=CLOCK)
        return any("adapter giả cho: repository, ocr, face, risk" in str(w.message) for w in caught)

    @case("lỗi bất ngờ giữa chừng để lại hồ sơ submitted: giới hạn lab 67, bài 68 thêm transaction")
    def _() -> bool:
        repo = InMemoryKycRepository()
        c = fresh(repository=repo, risk=ExplodingRisk())  # type: ignore[arg-type]
        raised = raises(RuntimeError, lambda: c.submit.execute(request_for("DR-001")))
        left = repo.get("KYC-0001")
        return (
            raised
            and left is not None
            and left.status is KycStatus.SUBMITTED
            and not left.ocr_results
        )

    # --- lớp api: cùng use case, qua HTTP ---
    def api_client(**kwargs: object) -> TestClient:
        return TestClient(create_app(fresh(**kwargs)))

    body = {
        "driver_id": "DR-001",
        "cccd_ref": "synthetic/cccd/DR-001.png",
        "gplx_ref": "synthetic/gplx/DR-001.png",
        "selfie_ref": "synthetic/selfie/DR-001.png",
        "face_consent": True,
    }

    @case("api: POST /applications -> 201, Location, không lộ số giấy tờ đầy đủ")
    def _() -> bool:
        r = api_client().post("/applications", json=body)
        return (
            r.status_code == 201
            and r.headers["location"] == "/applications/KYC-0001"
            and "099090000183" not in r.text
            and "099*******83" in r.text
        )

    @case("api: kyc-ocr tắt -> 202 Accepted, pending_check=kyc-ocr")
    def _() -> bool:
        r = api_client(ocr=FakeOcr(available=False)).post("/applications", json=body)
        return r.status_code == 202 and r.json()["pending_check"] == "kyc-ocr"

    @case("api: client gửi kèm status=approved -> 422 extra_forbidden, không lưu")
    def _() -> bool:
        repo = InMemoryKycRepository()
        r = api_client(repository=repo).post("/applications", json={**body, "status": "approved"})
        errs = r.json()["errors"]
        return (
            r.status_code == 422
            and errs[0]["code"] == "extra_forbidden"
            and errs[0]["field"] == "body.status"
            and repo.count() == 0
        )

    @case('api: face_consent="false" dạng chuỗi -> 422 bool_type, không đoán thành false')
    def _() -> bool:
        repo = InMemoryKycRepository()
        r = api_client(repository=repo).post(
            "/applications", json={**body, "face_consent": "false"}
        )
        return (
            r.status_code == 422
            and r.json()["errors"][0]["code"] == "bool_type"
            and repo.count() == 0
        )

    @case("api: review hồ sơ submitted -> 409 invalid-status-transition")
    def _() -> bool:
        client = api_client(ocr=FakeOcr(available=False))
        client.post("/applications", json=body)
        r = client.post(
            "/applications/KYC-0001/review",
            json={"reviewer_id": "RV-001", "decision": "approved", "reason": "ép"},
        )
        return r.status_code == 409 and r.json()["type"].endswith("invalid-status-transition")

    @case("api: GET hồ sơ lạ -> 404 problem+json")
    def _() -> bool:
        r = api_client().get("/applications/KYC-9999")
        return r.status_code == 404 and r.headers["content-type"].startswith(
            "application/problem+json"
        )

    passed = 0
    for name, fn in cases:
        try:
            ok = fn()
        except Exception as exc:  # ca kiểm lỗi: in lý do thay vì dừng cả bộ
            print(f"SAI: {name} ({type(exc).__name__}: {exc})")
            continue
        if ok:
            passed += 1
        else:
            print(f"SAI: {name}")
    if passed == len(cases):
        print(f"OK {passed}/{len(cases)}")
        return 0
    print(f"FAIL {passed}/{len(cases)}")
    return 1


if __name__ == "__main__":
    if "--verify" in sys.argv[1:]:
        sys.exit(run_verify())
    demo()
