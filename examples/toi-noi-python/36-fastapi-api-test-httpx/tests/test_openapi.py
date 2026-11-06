"""Hợp đồng OpenAPI: snapshot so khớp file, và các bất biến mà snapshot một mình không nói rõ."""

import difflib
import json
from pathlib import Path

import jsonschema
import pytest

SNAPSHOT = Path(__file__).parent / "snapshots" / "openapi.json"


def dump(schema: dict) -> str:
    return json.dumps(schema, indent=2, ensure_ascii=False, sort_keys=True) + "\n"


async def test_openapi_matches_the_committed_snapshot(client, update_snapshots):
    actual = dump((await client.get("/openapi.json")).json())

    if update_snapshots:
        SNAPSHOT.write_text(actual, encoding="utf-8")
    expected = SNAPSHOT.read_text(encoding="utf-8")

    if actual != expected:
        diff = "".join(difflib.unified_diff(expected.splitlines(True), actual.splitlines(True),
                                            "snapshot", "hiện tại", n=1))
        # pytest.fail thay vì assert: tránh pytest in thêm bản so sánh hai chuỗi dài làm chìm phần diff.
        pytest.fail(f"hợp đồng OpenAPI đã đổi; chạy lại với --update-snapshots nếu cố ý:\n{diff}", pytrace=False)


@pytest.fixture
async def spec(client) -> dict:
    return (await client.get("/openapi.json")).json()


def operations(spec: dict):
    for path, item in spec["paths"].items():
        for method, operation in item.items():
            yield path, method.upper(), operation


def test_operation_ids_are_unique_and_stable(spec):
    ids = [op["operationId"] for _, _, op in operations(spec)]

    assert sorted(ids) == ["cancelTrip", "createQuote", "getQuote", "getTrip"]


def test_security_scopes_show_up_in_the_spec(spec):
    by_id = {op["operationId"]: op for _, _, op in operations(spec)}

    assert by_id["createQuote"]["security"] == [{"OAuth2PasswordBearer": ["customer"]}]
    assert by_id["cancelTrip"]["security"] == [{"OAuth2PasswordBearer": ["dispatcher"]}]
    assert by_id["getTrip"]["security"] == [{"OAuth2PasswordBearer": []}]  # chỉ đòi token, không đòi scope


def test_every_operation_documents_401_and_scoped_ones_also_403(spec):
    for path, method, operation in operations(spec):
        assert "401" in operation["responses"], f"{method} {path}"
        scopes = operation["security"][0]["OAuth2PasswordBearer"]
        assert ("403" in operation["responses"]) == bool(scopes), f"{method} {path}"


def test_error_responses_are_declared_as_problem_json(spec):
    for path, method, operation in operations(spec):
        for code, response in operation["responses"].items():
            if code[0] in "45":
                assert list(response["content"]) == ["application/problem+json"], f"{method} {path} {code}"


async def test_real_responses_conform_to_their_documented_schemas(client, auth, spec):
    def validate(body: dict, schema_name: str) -> None:
        # Gắn components vào gốc để $ref "#/components/schemas/..." tự phân giải.
        root = {"$ref": f"#/components/schemas/{schema_name}", "components": spec["components"]}
        jsonschema.validate(body, root)

    quote = await client.post("/v1/quotes", json={"distance_km": "2.5"}, headers=auth())
    trip = await client.get("/v1/trips/TN-0001", headers=auth())
    bad = await client.post("/v1/quotes", json={"distance_km": "x"}, headers=auth())
    nope = await client.get("/v1/trips/TN-9999", headers=auth("DP-001", "dispatcher"))

    validate(quote.json(), "QuoteResponse")
    validate(trip.json(), "TripOut")
    validate(bad.json(), "ProblemDetails")
    validate(nope.json(), "ProblemDetails")
