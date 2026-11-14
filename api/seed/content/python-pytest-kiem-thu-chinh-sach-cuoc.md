---
slug: python-pytest-kiem-thu-chinh-sach-cuoc
title: "Unit test với pytest: kiểm thử chính sách cước"
summary: "Bài 10: chuyển bảng --verify thành test pytest: Arrange–Act–Assert, parametrize giá trị biên, pytest.raises, fixture và conftest, chính sách cước thay thế. Có kết quả chạy thật và lab chạy được."
category: python-beginner
level: beginner
tags: [python, toi-noi, pytest, unit-test]
language: vi
---

Bộ phận sản phẩm của [Tới Nơi](/articles/du-an-mau-toi-noi) muốn thử đổi giá mỗi km vượt từ 4.500 lên 5.000 VND ở một thành phố. Bạn sửa một con số và thấy giá 5 km đổi theo. Nhưng ai bảo đảm 0 km, 2 km hay 100 km vẫn đúng, và chính sách cũ vẫn tính đúng? Từ bài 01, ta so kết quả với một bảng kỳ vọng; từ bài 03, bảng đó nằm trong chế độ `--verify` tự chạy. Bài này chuyển nó sang **pytest**, công cụ kiểm thử chuẩn của Python, và xem pytest mang lại thêm điều gì.

Đây là bài 10 của lộ trình **Python cơ bản qua Tới Nơi**, cũng là bài cuối của phần 01. Bạn cần quen hàm và tham số (bài 03) cùng cách tổ chức code thành module (bài 07).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bảng kiểm của bài 01 có bốn dòng: 0, 2, 2,5 và 5 km, cả bốn đều ra đúng giá. Nếu ai đó đổi <code>ROUND_HALF_UP</code> thành <code>ROUND_HALF_EVEN</code> trong hàm tính cước, bảng bốn dòng này có phát hiện ra không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Cả bốn test case cho kết quả nguyên (12.000, 14.250, 25.500), nên không test case nào rơi đúng vào nửa đồng để làm tròn. Chỉ một test case như 2,001 km (cước thô 12.004,5) mới lộ ra; bảng của bài 03 đã có test case đó. Bài sẽ chạy thử đúng thay đổi này và xem pytest báo gì.</p></details>
</div>

## pytest thêm gì so với --verify

Chế độ `--verify` ở bài 01 và bài 03 đã tự chạy, so kết quả với bảng kỳ vọng, in `OK n/n` và thoát mã 1 khi sai; bảng của bài 03 còn có 1,999, 2,001, 2,499, 100 km và các test case bị từ chối. pytest không trả lời thay ta câu hỏi «test case nào đáng kiểm»; nó thay phần ta tự viết bằng tay: biến đếm `passed`, `total`, dòng `print("SAI: ...")`, vòng lặp qua bảng. Cụ thể:

- **Tên và báo cáo từng test case.** `--verify` gộp thành một con số; pytest cho mỗi test case một tên như `[2.001km-food]`, pass hoặc fail độc lập.
- **Tự tìm test.** Thêm hàm `test_*` là pytest tự chạy hàm đó.
- **Hiện giá trị trung gian khi test sai**, không chỉ «nhận X, kỳ vọng Y».
- **Fixture** dựng dữ liệu dùng chung và dọn sau mỗi test.
- **Chọn test để chạy**: một test, một nhóm theo tên (`-k`), dừng ở lỗi đầu (`-x`).
- **Tích hợp CI**: mã thoát khác 0 khi có test case sai, báo cáo theo định dạng chung.

Bảng bốn dòng của bài 01 không bắt được lỗi làm tròn như câu hỏi đầu bài. Cách chọn test case theo phân vùng và giá trị biên nằm trong loạt Kiểm thử của blog, ví dụ [phân vùng tương đương](/articles/equivalence-partitioning) và [tổng quan component testing](/articles/component-testing-unit-testing-tong-quan).

## Test đầu tiên: Arrange, Act, Assert

Lab có `fare.py` chứa chính sách cước. Phần cốt lõi (trích):

```python
class FarePolicy(NamedTuple):
    """Ba tham số của chính sách; bất biến, nên dùng chung giữa các test được."""

    base_fare: Decimal = Decimal("12000")
    included_km: Decimal = Decimal("2")
    price_per_extra_km: Decimal = Decimal("4500")


STANDARD = FarePolicy()
```

```python
    extra_km = max(Decimal("0"), distance - policy.included_km)
    total = policy.base_fare + extra_km * policy.price_per_extra_km
    return total.quantize(Decimal("1"), rounding=ROUND_HALF_UP)
```

`FarePolicy` là `NamedTuple` (bài 05): ba con số gói thành một giá trị bất biến có tên. `calculate_fare(distance_km, policy=STANDARD, *, service="ride")` ném `ValueError` nếu ngoài miền 0 đến 100 km, `TypeError` nếu không phải `Decimal` hoặc `int`.

Test đầu tiên có ba bước, gọi là **Arrange, Act, Assert**: dựng dữ liệu, thực hiện đúng một hành động cần kiểm, rồi khẳng định kết quả.

```python
def test_two_km_is_just_base_fare(standard_policy):
    # Arrange: chính sách chuẩn đến từ fixture; dữ liệu đặt ngay tại chỗ
    distance = D("2")

    # Act
    fare = calculate_fare(distance, standard_policy)

    # Assert
    assert fare == D("12000")
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 200" role="img" aria-label="Arrange dựng chính sách và khoảng cách, Act gọi calculate_fare một lần, Assert so kết quả với giá kỳ vọng viết sẵn."><defs><marker id="py10-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="45" width="256" height="100" rx="10" class="f-phys-soft s-phys"/><text x="36" y="82" class="t-b">Arrange</text><text x="36" y="113" class="t-small">policy, distance = D("2")</text><path d="M276 95 L300 95" class="f-none s-phys" stroke-width="2" marker-end="url(#py10-0-arrow)"/><rect x="300" y="45" width="256" height="100" rx="10" class="f-pk-soft s-pk"/><text x="316" y="82" class="t-b">Act</text><text x="316" y="113" class="t-small">calculate_fare(distance, policy)</text><path d="M556 95 L580 95" class="f-none s-phys" stroke-width="2" marker-end="url(#py10-0-arrow)"/><rect x="580" y="45" width="280" height="100" rx="10" class="f-phys-soft s-phys"/><text x="596" y="82" class="t-b">Assert</text><text x="596" y="113" class="t-small">assert fare == D("12000")</text></svg>
</div>
<figcaption>Hình 1. Ba bước của một test. Giá kỳ vọng được viết sẵn từ đặc tả, không tính lại bằng chính công thức đang kiểm.</figcaption>
</figure>

Pytest tự tìm test theo tên: file `test_*.py`, hàm `test_*`, không cần kế thừa lớp nào. Câu kiểm tra là `assert` thuần của Python; khác với `unittest` có cả họ `assertEqual`, `assertIn`, pytest viết lại `assert` khi import module test để khi test sai, pytest in được từng giá trị trung gian (xem phần cuối).

Nguyên tắc ở bước Assert là **kỳ vọng độc lập**: nếu test tự tính `12000 + (km - 2) * 4500` rồi so với `calculate_fare`, công thức sai sẽ sai ở cả hai chỗ và test vẫn xanh. Con số kỳ vọng phải đến từ đặc tả, như 2,5 km là 14.250 (12.000 + 0,5 × 4.500 tính trên giấy).

## parametrize: một test, nhiều ca biên

Viết `test_zero_km`, `test_five_km`... cho từng khoảng cách là lặp một khung. `@pytest.mark.parametrize` giữ khung một lần và đưa dữ liệu vào bảng:

```python
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
```

Hai decorator xếp chồng nhân với nhau: 8 khoảng cách × 3 dịch vụ (`ride`, `food`, `parcel`) thành 24 test case, mỗi test case chạy như một test riêng, tên ghép từ `ids`. Danh sách test case do pytest liệt kê (`pytest --collect-only -q -k boundaries`, trích):

```text
tests/test_fare.py::test_fare_at_boundaries[0km-ride]
tests/test_fare.py::test_fare_at_boundaries[0km-food]
tests/test_fare.py::test_fare_at_boundaries[0km-parcel]
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Một hàm test cùng bảng tám khoảng cách và ba dịch vụ được pytest nhân thành hai mươi bốn ca, mỗi ca pass hoặc fail riêng."><defs><marker id="py10-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="70" width="260" height="120" rx="10" class="f-phys-soft s-phys"/><text x="36" y="106" class="t-b">1 hàm + 1 bảng</text><text x="36" y="134" class="t-small">8 khoảng cách</text><text x="36" y="154" class="t-small">x 3 dịch vụ</text><path d="M280 130 L340 50" class="f-none s-phys" stroke-width="2" marker-end="url(#py10-1-arrow)"/><path d="M280 130 L340 100" class="f-none s-phys" stroke-width="2" marker-end="url(#py10-1-arrow)"/><path d="M280 130 L340 160" class="f-none s-phys" stroke-width="2" marker-end="url(#py10-1-arrow)"/><path d="M280 130 L340 210" class="f-none s-phys" stroke-width="2" marker-end="url(#py10-1-arrow)"/><rect x="340" y="32" width="250" height="36" rx="8" class="f-pk-soft s-pk"/><text x="354" y="55" class="t-mono">[0km-ride]</text><rect x="340" y="82" width="250" height="36" rx="8" class="f-pk-soft s-pk"/><text x="354" y="105" class="t-mono">[2.001km-food]</text><rect x="340" y="142" width="250" height="36" rx="8" class="f-pk-soft s-pk"/><text x="354" y="165" class="t-mono">[100km-parcel]</text><text x="354" y="205" class="t-small">... tổng cộng 24 ca</text><path d="M590 130 L630 130" class="f-none s-phys" stroke-width="2" marker-end="url(#py10-1-arrow)"/><rect x="630" y="70" width="230" height="120" rx="10" class="f-phys-soft s-phys"/><text x="646" y="106" class="t-b">Báo cáo riêng</text><text x="646" y="134" class="t-small">mỗi ca pass hoặc fail</text><text x="646" y="154" class="t-small">độc lập với ca khác</text></svg>
</div>
<figcaption>Hình 2. parametrize nhân một test thành nhiều test case. Một test case đỏ không che khuất các test case còn lại, nên nhìn danh sách thất bại là biết biên nào sai.</figcaption>
</figure>

Hai test case 2,001 và 2,499 km rơi đúng vào nửa đồng, nên trả lời được câu hỏi đầu bài.

Ba loại dịch vụ cho cùng một giá vì mô hình học tập này chưa có phụ phí (như bài 03, `service` chỉ được kiểm là thuộc `SERVICES`). Test là **đặc tả có thể chạy**: ai đọc cũng thấy «hiện nay dịch vụ không làm đổi giá». Khi sản phẩm thêm phụ phí, test đỏ đúng chỗ cần thảo luận.

## pytest.raises: kiểm chứng điều phải bị từ chối

Hàm phải từ chối đầu vào vô nghĩa chứ không lặng lẽ trả về một mức giá. `pytest.raises` là context manager (khối `with` như bài 09): test chỉ qua nếu đúng loại exception xuất hiện trong khối.

```python
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
```

Các test case được chọn có lý do. Nhóm ngoài miền gồm hai test case sát biên của bài 03 (−0,001 và 100,001), cộng `NaN` và `Infinity` vì `Decimal` biểu diễn được chúng. Nhóm sai kiểu gồm `float` (mất độ chính xác của tiền), `str`, `None` và `True` (trong Python `bool` là một `int`, không loại ra thì `True` thành 1 km). `match="ngoài miền"` là biểu thức chính quy tìm trong thông báo lỗi, giúp phân biệt các `ValueError` khác nhau; đừng khớp nguyên văn một câu dài, vì đổi chữ là test gãy dù hành vi không đổi.

## Fixture, conftest và chính sách thay thế

Gần như test nào cũng cần một chính sách chuẩn. **Fixture** là hàm gắn `@pytest.fixture`; test khai báo tên nó trong danh sách tham số và pytest tự gọi, rồi truyền kết quả vào. Fixture dùng chung nhiều file đặt trong `conftest.py`, pytest nạp tự động, không cần `import`:

```python
@pytest.fixture(scope="session")
def standard_policy() -> FarePolicy:
    # FarePolicy là NamedTuple bất biến nên dùng chung cả phiên test là an toàn.
    return STANDARD


@pytest.fixture
def policy_5000() -> FarePolicy:
    """Chính sách thay thế: giá mỗi km vượt 5.000 thay vì 4.500."""
    return FarePolicy(price_per_extra_km=Decimal("5000"))
```

**Scope** quyết định fixture được dựng bao nhiêu lần. Mặc định `function` dựng lại cho mỗi test, an toàn nhất. `scope="session"` dựng một lần cho cả lần chạy; chỉ dùng khi giá trị bất biến như `FarePolicy` này, vì nếu fixture trả về một `list` mà test sửa, test sau sẽ nhận dữ liệu đã bị test trước làm bẩn. `--setup-show` cho thấy thứ tự dựng và dọn:

```text
SETUP    S standard_policy
        tests/test_fare.py::test_two_km_is_just_base_fare (fixtures used: standard_policy) .
        SETUP    F policy_5000
        tests/test_fare.py::test_replacement_policy_changes_the_price (fixtures used: policy_5000) .
        TEARDOWN F policy_5000
TEARDOWN S standard_policy
```

Chữ `S` là session, `F` là function. Fixture dùng `yield` thay `return` thì chạy được đoạn dọn dẹp sau test; pytest cũng có sẵn fixture như `tmp_path`, thư mục tạm riêng cho mỗi test. Nhờ `policy_5000`, câu chuyện đổi giá mở đầu bài trở thành một test:

```python
def test_replacement_policy_changes_the_price(policy_5000):
    # 4 km: 12.000 + 2 x 5.000 = 22.000 (bài tập bài 01 của loạt C#)
    assert calculate_fare(D("4"), policy_5000) == D("22000")
    # chính sách thay thế không đổi giá cơ bản trong 2 km đầu
    assert calculate_fare(D("2"), policy_5000) == D("12000")
```

Không phải sửa biến toàn cục để thử giá khác: chính sách là tham số, nên việc kiểm thử đơn giản. Thiết kế dễ test thường là thiết kế nhận phụ thuộc từ bên ngoài; bài 14 đi sâu hướng này.

## Chạy test, đọc output khi fail

Lab có `pyproject.toml` khai báo pytest trong nhóm `dev` và mục `[tool.pytest.ini_options]` với hai dòng. pytest nhận `pyproject.toml` của lab làm `configfile` và thư mục lab làm `rootdir` (`pytest -v` in cả hai). `pythonpath = ["."]` là cấu hình của **test runner**: pytest thêm thư mục này vào `sys.path` khi thu thập test, để `import fare` thấy `fare.py` đặt phẳng ở gốc lab. Cấu hình này không phải code ứng dụng; bài 07 đã nói vì sao ứng dụng không nên tự chỉnh `sys.path`. Dự án thật cài gói ở chế độ editable như bài 07; bài 20 sẽ kiểm thử trên cấu trúc `src`. Còn `testpaths = ["tests"]` chỉ có tác dụng khi chạy `pytest` không đối số trong thư mục lab; lệnh dưới đây đã truyền đường dẫn nên không dùng tới. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/10-pytest-chinh-sach-cuoc pytest examples/toi-noi-python/10-pytest-chinh-sach-cuoc -q
```

```text
....................................                                     [100%]
36 passed in 0.02s
```

36 test = 1 (2 km) + 24 (8 khoảng cách × 3 dịch vụ) + 1 (`int`) + 4 (ngoài miền) + 4 (sai kiểu) + 1 (dịch vụ lạ) + 1 (chính sách thay thế). Test xanh chỉ có nghĩa nếu nó **có thể đỏ**, nên ta thử phá có chủ ý: trong bản sao ở thư mục tạm, đổi `ROUND_HALF_UP` thành `ROUND_HALF_EVEN`. Dòng tiến trình (trích):

```text
..........FFF.......................                                     [100%]
```
Chi tiết của test case thất bại đầu tiên (trích):

```text
_____________________ test_fare_at_boundaries[2.001km-ride] _____________________
>       assert calculate_fare(distance, standard_policy, service=service) == expected
E       AssertionError: assert Decimal('12004') == Decimal('12005')
E        +  where Decimal('12004') = calculate_fare(Decimal('2.001'), FarePolicy(base_fare=Decimal('12000'), included_km=Decimal('2'), price_per_extra_km=Decimal('4500')), service='ride')
```

Dòng cuối:

```text
3 failed, 33 passed in 0.03s
```

Cách đọc: ba dấu `F` ở vị trí 11 đến 13 là ba test case 2,001 km; mũi tên `>` chỉ dòng assert sai; các dòng `E` là giá trị thật pytest tính ra nhờ viết lại `assert` khi import module test, kể cả đối số truyền vào hàm. Đáng để ý cả điều không đỏ: test case 2,499 km vẫn xanh, vì 14.245,5 làm tròn half-even cũng ra 14.246 (số chẵn gần nhất); chỉ test case 2,001 km phân biệt được hai chế độ. Thí nghiệm khác: đổi giá cơ bản 12.000 thành 11.000 cho `27 failed, 9 passed`; 9 test case còn xanh là các test case từ chối đầu vào, vì chúng không tính tiền.

Ba cờ giúp làm việc nhanh khi một test đỏ: `-k` chọn test theo biểu thức trên tên (`-k "boundaries and food"` chạy 8 test case), `-x` dừng ở thất bại đầu tiên (thí nghiệm trên cho `1 failed, 10 passed`), `-q` giảm bớt chữ in ra, còn `-v` liệt kê từng test.

Test đáng giá khi kiểm tra **hành vi quan sát được**: nhận khoảng cách, trả cước. Đừng kiểm tra chi tiết cài đặt («hàm có gọi `quantize` không», tên biến, thứ tự câu lệnh): test kiểu đó đỏ mỗi lần ta dọn code dù giá vẫn đúng, và người ta sẽ tắt chúng thay vì tin chúng.

## So với xUnit bên C#

[Bài C# về unit test và test double](/articles/csharp-unit-test-test-double) làm đúng việc này bằng xUnit; khái niệm giống nhau, chỉ khác cách viết:

| xUnit (C#) | pytest (Python) |
| --- | --- |
| `[Fact]` | hàm `test_*` thường |
| `[Theory]` + `[InlineData]` | `@pytest.mark.parametrize` (nhận cả `Decimal`, không cần parse chuỗi) |
| `Assert.Equal(expected, actual)` | `assert actual == expected` |
| `Assert.Throws<T>(...)` | `with pytest.raises(T):` |
| constructor, `IDisposable` | `@pytest.fixture`, `yield`, `scope` |
| `ProjectReference` tới thư viện | gói cài editable (bài 07); lab này dùng `pythonpath` của pytest cho gọn |

Bài C# có 9 test case; ở đây 36 là số test case pytest đếm sau khi mở `parametrize`. Hai bài chung cách nghĩ: kỳ vọng độc lập, tên test mô tả hành vi, coverage chỉ là tín hiệu bổ trợ ([condition coverage](/articles/condition-coverage)).

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `ModuleNotFoundError: No module named 'fare'` | `pythonpath = ["."]` trong `[tool.pytest.ini_options]` và pytest có nhận đúng `configfile` không |
| `no tests ran` | Tên file và hàm có bắt đầu bằng `test_` không |
| `fixture 'standard_policy' not found` | `conftest.py` phải nằm trong `tests/` hoặc thư mục cha của file test |
| Tên test case xấu như `[bad_distance0]` | Thêm `ids=[...]` đủ số dòng dữ liệu |
| Test qua khi chạy riêng, đỏ khi chạy cả bộ | Fixture `scope` rộng trả về giá trị có thể sửa; để `scope="function"` |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>36 test chỉ chứng minh các test case ta đã chọn, không chứng minh mọi khoảng cách đúng; coverage và mutation testing bổ sung điều đó nhưng ngoài phạm vi bài. Đây là unit test thuần trong bộ nhớ, chưa nói gì về việc báo giá đúng khi ghép với file, HTTP hay database. Các con số 12.000, 4.500 là mô hình học tập, không phải bảng giá thật. Lab khai báo <code>pytest&gt;=8</code>; <code>uv.lock</code> ghi phiên bản đã chạy, và lần đầu cần mạng để tải gói.</p>
</div>

**Bài tập:** (1) Trong `fare.py`, đổi `<= MAX_KM` thành `< MAX_KM`. Test nào đỏ, và vì sao test case 100,001 (đang bị từ chối) không phát hiện được? (2) Viết thêm một test dùng chính sách `FarePolicy(included_km=Decimal("3"))` cho khoảng cách 4 km. Kỳ vọng là bao nhiêu?

<details><summary>Xem lời giải</summary><p>(1) Ba test case <code>test_fare_at_boundaries[100km-…]</code> đỏ (<code>3 failed, 33 passed</code>): đúng 100 km bị từ chối với <code>ValueError</code>. Test case 100,001 bị từ chối ở cả hai phiên bản, và các khoảng cách nhỏ hơn 100 hợp lệ ở cả hai, nên chỉ test case <b>nằm đúng trên biên</b> mới phân biệt <code>&lt;</code> với <code>&lt;=</code>; đó là lý do người ta kiểm giá trị biên. (2) Phần vượt là 4 − 3 = 1 km, cước = 12.000 + 1 × 4.500 = <b>16.500</b>: <code>assert calculate_fare(D("4"), FarePolicy(included_km=D("3"))) == D("16500")</code>.</p></details>

Bài trước: [bài 09](/articles/python-file-json-csv-pathlib). Bài tiếp: bài 11, mở đầu phần 02 (mô hình hoá nghiệp vụ bằng Python).

## Tài liệu gốc

- [pytest: Get Started](https://docs.pytest.org/en/stable/getting-started.html).
- [pytest: How to parametrize fixtures and test functions](https://docs.pytest.org/en/stable/how-to/parametrize.html).
- [pytest: How to use fixtures](https://docs.pytest.org/en/stable/how-to/fixtures.html).
- [pytest: How to write and report assertions in tests](https://docs.pytest.org/en/stable/how-to/assert.html).
- [uv: Running scripts and commands in a project](https://docs.astral.sh/uv/concepts/projects/run/).
