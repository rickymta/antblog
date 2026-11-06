"""Hợp đồng POST /v1/quotes (như lab 32): giá đúng theo đặc tả, tiền là chuỗi, 201 kèm Location, lỗi đúng mã và dạng."""

import asyncio
from decimal import Decimal

import pytest
from fastapi.testclient import TestClient

from app.domain import DistanceOutOfRange, calculate_fare


@pytest.mark.parametrize(
    ("distance", "fare"),
    [("0", "12000"), ("2", "12000"), ("2.001", "12005"), ("2.5", "14250"), ("5", "25500"), ("100", "453000")],
)
async def test_quote_prices_follow_the_spec(client, auth, distance, fare):
    response = await client.post("/v1/quotes", json={"distance_km": distance, "service": "ride"},
                                 headers=auth())

    assert response.status_code == 201
    body = response.json()
    assert body["fare"] == fare  # chuỗi, không phải số: so bằng "14250", không phải 14250
    assert body["currency"] == "VND"
    assert body["distance_km"] == distance


async def test_created_quote_has_an_id_a_location_and_can_be_read_back(client, auth):
    created = await client.post("/v1/quotes", json={"distance_km": "2.5"}, headers=auth())
    quote_id = created.json()["quote_id"]

    assert quote_id == "QT-0001"
    assert created.headers["location"] == "/v1/quotes/QT-0001"
    read = await client.get(created.headers["location"], headers=auth())
    assert read.status_code == 200
    assert read.json() == created.json()


async def test_another_customer_cannot_read_the_quote(client, auth):
    await client.post("/v1/quotes", json={"distance_km": "2.5"}, headers=auth("CU-001"))

    mine = await client.get("/v1/quotes/QT-0001", headers=auth("CU-002"))
    missing = await client.get("/v1/quotes/QT-9999", headers=auth("CU-002"))

    assert mine.status_code == missing.status_code == 404
    assert mine.json()["detail"].replace("QT-0001", "X") == missing.json()["detail"].replace("QT-9999", "X")


async def test_quote_expires_five_minutes_after_the_injected_clock(client, auth):
    response = await client.post("/v1/quotes", json={"distance_km": "2.5"}, headers=auth())

    assert response.json()["expires_at"] == "2026-11-06T03:05:00Z"


@pytest.mark.parametrize("distance", ["-0.001", "100.001"])
async def test_distance_outside_the_domain_is_a_400_problem(client, auth, distance):
    response = await client.post("/v1/quotes", json={"distance_km": distance}, headers=auth())

    assert response.status_code == 400
    assert response.headers["content-type"] == "application/problem+json"
    body = response.json()
    assert body["type"] == "https://toinoi.example/problems/distance-out-of-range"
    assert body["title"] == "Khoảng cách ngoài miền cho phép"
    assert (body["min_km"], body["max_km"]) == ("0", "100")


@pytest.mark.parametrize(
    ("payload", "field", "code"),
    [
        ({"distance_km": "abc"}, "body.distance_km", "decimal_parsing"),
        ({"distance_km": "NaN"}, "body.distance_km", "finite_number"),
        ({"distance_km": "2.5001"}, "body.distance_km", "decimal_max_places"),
        ({"service": "ride"}, "body.distance_km", "missing"),
        ({"distance_km": "2", "service": "bike"}, "body.service", "enum"),
        ({"distance_km": "2", "promo": "X"}, "body.promo", "extra_forbidden"),
    ],
    ids=["not-a-number", "nan", "four-decimals", "missing-distance", "unknown-service", "extra-field"],
)
async def test_malformed_body_is_a_422_problem_with_field_and_code(client, auth, payload, field, code):
    response = await client.post("/v1/quotes", json=payload, headers=auth())

    assert response.status_code == 422
    assert response.headers["content-type"] == "application/problem+json"
    first = response.json()["errors"][0]
    assert (first["field"], first["code"]) == (field, code)
    assert set(first) == {"field", "code", "message"}  # không có `input`


async def test_broken_json_is_also_a_422_problem(client, auth):
    response = await client.post("/v1/quotes", content=b'{"distance_km": ',
                                 headers={**auth(), "content-type": "application/json"})

    assert response.status_code == 422
    first = response.json()["errors"][0]
    assert (first["field"], first["code"]) == ("body", "json_invalid")


def test_unknown_service_is_a_plain_value_error_in_the_domain():
    with pytest.raises(ValueError, match="bike") as caught:
        calculate_fare(Decimal("2"), service="bike")
    assert not isinstance(caught.value, DistanceOutOfRange)  # lỗi dịch vụ khác lỗi khoảng cách


async def test_twenty_concurrent_quotes_all_succeed_with_distinct_ids(client, auth):
    headers = auth()
    calls = [client.post("/v1/quotes", json={"distance_km": str(km)}, headers=headers) for km in range(20)]

    responses = await asyncio.gather(*calls)

    assert [r.status_code for r in responses] == [201] * 20
    assert len({r.json()["quote_id"] for r in responses}) == 20
    assert {r.json()["distance_km"]: r.json()["fare"] for r in responses}["3"] == "16500"  # 12.000 + 1 x 4.500


def test_same_contract_with_the_sync_testclient(app, auth):
    # TestClient là cầu nối đồng bộ (chạy app trong thread riêng). `with` mới chạy lifespan.
    with TestClient(app) as client:
        response = client.post("/v1/quotes", json={"distance_km": "5"}, headers=auth())

    assert response.status_code == 201
    assert response.json()["fare"] == "25500"
