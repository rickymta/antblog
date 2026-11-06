"""Hình dạng lỗi: mọi ca lỗi LIỆT KÊ DƯỚI ĐÂY phải là application/problem+json với năm trường của RFC 9457.

Test này chỉ bảo vệ những ca đã liệt kê; một dạng lỗi mới (route mới, exception mới) chỉ được bảo vệ khi ta thêm ca vào bảng.
"""

import pytest

REQUIRED = {"type", "title", "status", "instance"}
BASE = "https://toinoi.example/problems/"

# (mô tả, method, url, kwargs của request, vai trò, mã kỳ vọng)
ERROR_CASES = [
    ("thiếu token", "POST", "/v1/quotes", {"json": {"distance_km": "2"}}, None, 401),
    ("thiếu quyền", "POST", "/v1/quotes", {"json": {"distance_km": "2"}}, ("DR-001", "driver"), 403),
    ("ngoài miền", "POST", "/v1/quotes", {"json": {"distance_km": "101"}}, ("CU-001", "customer"), 400),
    ("body sai", "POST", "/v1/quotes", {"json": {"distance_km": "x"}}, ("CU-001", "customer"), 422),
    ("không có chuyến", "GET", "/v1/trips/TN-9999", {}, ("DP-001", "dispatcher"), 404),
    ("sai trạng thái", "POST", "/v1/trips/TN-0003/cancel", {}, ("DP-001", "dispatcher"), 409),
    ("route không có", "GET", "/v1/khong-co", {}, None, 404),
    ("sai phương thức", "DELETE", "/v1/quotes", {}, ("CU-001", "customer"), 405),
]


@pytest.mark.parametrize(("method", "url", "kwargs", "role", "status"),
                         [case[1:] for case in ERROR_CASES], ids=[case[0] for case in ERROR_CASES])
async def test_every_listed_error_is_problem_json(client, auth, method, url, kwargs, role, status):
    headers = auth(*role) if role else {}

    response = await client.request(method, url, headers=headers, **kwargs)

    assert response.status_code == status
    assert response.headers["content-type"] == "application/problem+json"
    body = response.json()
    assert REQUIRED <= body.keys()
    assert body["status"] == status  # trường status trong body phải khớp mã HTTP
    assert body["instance"] == url
    assert body["type"] == "about:blank" or body["type"].startswith(BASE)
    assert body["request_id"] == response.headers["x-request-id"]  # bài 33: nối được với log


async def test_framework_errors_use_about_blank_with_the_standard_phrase(client, auth):
    not_found = await client.get("/v1/khong-co")
    wrong_method = await client.delete("/v1/quotes", headers=auth())

    assert (not_found.json()["type"], not_found.json()["title"]) == ("about:blank", "Not Found")
    assert (wrong_method.json()["type"], wrong_method.json()["title"]) == ("about:blank", "Method Not Allowed")
    assert wrong_method.headers["allow"] == "POST"  # giữ header Allow của framework


async def test_validation_problem_does_not_echo_the_submitted_value(client, auth):
    response = await client.post("/v1/quotes", json={"distance_km": "so-dien-thoai-0900000001"},
                                 headers=auth())

    assert response.status_code == 422
    assert "0900000001" not in response.text
