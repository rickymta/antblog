"""Test chính sách cước. Mỗi test là một câu trong đặc tả có thể chạy."""

from decimal import Decimal

import pytest

from fare import calculate_fare

D = Decimal
SERVICES = ["ride", "food", "parcel"]


def test_two_km_is_just_base_fare(standard_policy):
    # Arrange: chính sách chuẩn đến từ fixture; dữ liệu đặt ngay tại chỗ
    distance = D("2")

    # Act
    fare = calculate_fare(distance, standard_policy)

    # Assert
    assert fare == D("12000")


# Bảng ca dùng lại của bài 03. 2,001 km: phần vượt 0,001 x 4.500 = 4,5 nên cước thô
# 12.004,5, ROUND_HALF_UP làm tròn lên 12.005. 2,499 km: thô 14.245,5 -> 14.246.
@pytest.mark.parametrize("service", SERVICES)
@pytest.mark.parametrize(
    ("distance", "expected"),
    [
        (D("0"), D("12000")),
        (D("1.999"), D("12000")),
        (D("2"), D("12000")),
        (D("2.001"), D("12005")),
        (D("2.499"), D("14246")),
        (D("2.5"), D("14250")),
        (D("5"), D("25500")),
        (D("100"), D("453000")),
    ],
    ids=["0km", "1.999km", "2km", "2.001km", "2.499km", "2.5km", "5km", "100km"],
)
def test_fare_at_boundaries(standard_policy, service, distance, expected):
    assert calculate_fare(distance, standard_policy, service=service) == expected


def test_int_distance_is_accepted(standard_policy):
    assert calculate_fare(5, standard_policy) == D("25500")


@pytest.mark.parametrize(
    "bad_distance",
    [D("-0.001"), D("100.001"), D("NaN"), D("Infinity")],
    ids=["-0.001", "100.001", "nan", "inf"],
)
def test_out_of_range_distance_is_rejected(standard_policy, bad_distance):
    with pytest.raises(ValueError, match="ngoài miền"):
        calculate_fare(bad_distance, standard_policy)


@pytest.mark.parametrize(
    "wrong_type",
    [2.5, "5", None, True],
    ids=["float", "str", "none", "bool"],
)
def test_wrong_type_is_rejected(standard_policy, wrong_type):
    with pytest.raises(TypeError):
        calculate_fare(wrong_type, standard_policy)


def test_unknown_service_is_rejected(standard_policy):
    with pytest.raises(ValueError, match="service"):
        calculate_fare(D("5"), standard_policy, service="taxi")


def test_replacement_policy_changes_the_price(policy_5000):
    # 4 km: 12.000 + 2 x 5.000 = 22.000 (bài tập bài 01 của loạt C#)
    assert calculate_fare(D("4"), policy_5000) == D("22000")
    # chính sách thay thế không đổi giá cơ bản trong 2 km đầu
    assert calculate_fare(D("2"), policy_5000) == D("12000")
