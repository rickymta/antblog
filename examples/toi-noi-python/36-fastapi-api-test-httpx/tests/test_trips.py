"""Chuyến: quyền xem, hủy theo máy trạng thái, và kho bị thay bằng bản giả."""

import httpx
import pytest

from fastapi.testclient import TestClient

from app.domain import Trip
from app.main import create_app, get_repo
from app.repo import RepositoryUnavailable
from tests.conftest import make_settings


async def test_customer_sees_own_trip(client, auth):
    response = await client.get("/v1/trips/TN-0001", headers=auth("CU-001", "customer"))

    assert response.status_code == 200
    assert response.json() == {"trip_id": "TN-0001", "customer_id": "CU-001", "driver_id": "DR-001",
                               "service": "ride", "distance_km": "5", "status": "accepted"}


async def test_other_customers_trip_looks_like_it_does_not_exist(client, auth):
    other = await client.get("/v1/trips/TN-0002", headers=auth("CU-001", "customer"))
    missing = await client.get("/v1/trips/TN-9999", headers=auth("CU-001", "customer"))

    assert other.status_code == missing.status_code == 404  # không lộ sự tồn tại của chuyến người khác
    assert other.json()["type"].endswith("/trip-not-found")
    assert other.json()["detail"].replace("TN-0002", "X") == missing.json()["detail"].replace("TN-9999", "X")


async def test_driver_sees_only_the_assigned_trip(client, auth):
    mine = await client.get("/v1/trips/TN-0001", headers=auth("DR-001", "driver"))
    other = await client.get("/v1/trips/TN-0003", headers=auth("DR-001", "driver"))

    assert (mine.status_code, other.status_code) == (200, 404)


async def test_dispatcher_sees_every_trip(client, auth):
    response = await client.get("/v1/trips/TN-0002", headers=auth("DP-001", "dispatcher"))

    assert response.status_code == 200


async def test_dispatcher_cancels_an_accepted_trip_once(client, auth):
    headers = auth("DP-001", "dispatcher")

    first = await client.post("/v1/trips/TN-0001/cancel", headers=headers)
    second = await client.post("/v1/trips/TN-0001/cancel", headers=headers)

    assert (first.status_code, first.json()["status"]) == (200, "cancelled")
    assert second.status_code == 409
    assert second.json()["type"].endswith("/invalid-transition")


async def test_trip_in_transit_cannot_be_cancelled(client, auth):
    response = await client.post("/v1/trips/TN-0003/cancel", headers=auth("DP-001", "dispatcher"))

    assert response.status_code == 409
    assert "in_transit" in response.json()["detail"]


async def test_customer_cannot_cancel(client, auth):
    response = await client.post("/v1/trips/TN-0001/cancel", headers=auth("CU-001", "customer"))

    assert response.status_code == 403


class DownRepo:
    """Kho giả luôn lỗi: ca này không thể dựng bằng kho thật mà không phá hạ tầng."""

    async def get(self, trip_id: str) -> Trip | None:
        raise RepositoryUnavailable

    async def save(self, trip: Trip) -> None:
        raise RepositoryUnavailable


async def test_repository_outage_becomes_503_with_retry_after(app, client, auth):
    app.dependency_overrides[get_repo] = lambda: DownRepo()

    response = await client.get("/v1/trips/TN-0001", headers=auth("DP-001", "dispatcher"))

    assert response.status_code == 503
    assert response.headers["retry-after"] == "5"
    assert response.json()["type"].endswith("/repository-unavailable")


class BrokenRepo(DownRepo):
    async def get(self, trip_id: str) -> Trip | None:
        raise RuntimeError("mật khẩu db = hunter2")  # bug bất ngờ, chứa chi tiết nhạy cảm


async def test_unexpected_error_becomes_a_generic_500_without_leaking(app, auth):
    app.dependency_overrides[get_repo] = lambda: BrokenRepo()
    # Mặc định httpx ném lại ngoại lệ của app vào test; raise_app_exceptions=False cho thấy response 500.
    transport = httpx.ASGITransport(app=app, raise_app_exceptions=False)
    async with app.router.lifespan_context(app), httpx.AsyncClient(transport=transport, base_url="http://t") as c:
        response = await c.get("/v1/trips/TN-0001", headers=auth("DP-001", "dispatcher"))

    assert response.status_code == 500
    assert "hunter2" not in response.text
    assert response.json()["type"].endswith("/internal-error")


async def test_default_transport_re_raises_what_the_app_raised(app, auth):
    app.dependency_overrides[get_repo] = lambda: BrokenRepo()
    transport = httpx.ASGITransport(app=app)  # raise_app_exceptions=True (mặc định)
    async with app.router.lifespan_context(app), httpx.AsyncClient(transport=transport, base_url="http://t") as c:
        with pytest.raises(RuntimeError, match="hunter2"):
            await c.get("/v1/trips/TN-0001", headers=auth("DP-001", "dispatcher"))


async def test_without_lifespan_the_app_state_is_missing(auth):
    # Tài liệu hóa một cái bẫy: client không mở lifespan thì app.state.repo chưa được dựng.
    app = create_app(make_settings())
    async with httpx.AsyncClient(transport=httpx.ASGITransport(app=app), base_url="http://t") as c:
        with pytest.raises(AttributeError, match="repo"):
            await c.get("/v1/trips/TN-0001", headers=auth("DP-001", "dispatcher"))


def test_testclient_without_with_has_the_same_trap(auth):
    client = TestClient(create_app(make_settings()))  # không `with`: lifespan không chạy

    with pytest.raises(AttributeError, match="repo"):
        client.get("/v1/trips/TN-0001", headers=auth("DP-001", "dispatcher"))


def test_testclient_with_with_runs_the_lifespan(auth):
    with TestClient(create_app(make_settings())) as client:  # `with`: lifespan chạy, app.state.repo có mặt
        response = client.get("/v1/trips/TN-0001", headers=auth("DP-001", "dispatcher"))

    assert response.status_code == 200
