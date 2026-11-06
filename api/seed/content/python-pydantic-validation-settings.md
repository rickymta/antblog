---
slug: python-pydantic-validation-settings
title: "Pydantic v2: validate hồ sơ tài xế và cấu hình ứng dụng"
summary: "Bài 15: dùng pydantic v2 kiểm hồ sơ tài xế ở biên hệ thống: BaseModel, field và model validator, lỗi có cấu trúc, lax và strict; BaseSettings đọc TOINOI_*. Khi nào dùng dataclass. Có output thật."
category: programming
level: beginner
tags: [python, toi-noi, pydantic, validation]
language: vi
---

Tài xế mới của [Tới Nơi](/articles/du-an-mau-toi-noi) nộp hồ sơ qua app di động: họ tên, số CCCD, số GPLX, điện thoại, thành phố. Một người gõ số điện thoại có dấu cách, người khác thiếu một chữ số CCCD, người thứ ba chọn «Huế», một thành phố chưa có trong hệ thống. Hồ sơ này đi tiếp tới các service Python làm OCR và chấm điểm rủi ro; dữ liệu sai chui sâu sẽ gây lỗi ở chỗ xa nơi nó sinh ra.

Đây là bài 15 của lộ trình **Python cơ bản qua Tới Nơi**. Bạn cần quen type hints (bài 12) và `dataclass` (bài 11). Bài này đưa vào **pydantic v2**, thư viện kiểm tra dữ liệu dựa trên type hints, rồi dùng nó để đọc cấu hình từ biến môi trường.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Trên một <code>dataclass</code> thường, trường khai báo <code>driver_id: str</code> có chặn được <code>driver_id=123</code> không? Còn trên pydantic <code>BaseModel</code> thì sao?</p>
<details><summary>Xem đáp án</summary><p><b>Dataclass không chặn.</b> Type hint chỉ là chú thích cho công cụ kiểm kiểu như mypy (bài 12); lúc chạy Python không đọc nó để từ chối giá trị. <b>BaseModel có chặn</b>: số nguyên 123 vào trường <code>str</code> bị báo lỗi <code>string_type</code>. Nhưng đừng khái quát thành «pydantic luôn khắt khe»: chuỗi <code>"2.5"</code> vào trường <code>Decimal</code> vẫn được chuyển thành <code>Decimal("2.5")</code>. Bài sẽ chạy cả hai và giải thích ranh giới đó.</p></details>
</div>

## Một cửa kiểm ở biên hệ thống

**Dữ liệu bên trong** do code của ta tạo ra và kiểm soát. **Dữ liệu từ ngoài vào** (JSON từ app, message Kafka, biến môi trường, file CSV) thì không: người dùng gõ sai, client cũ gửi định dạng cũ, kẻ xấu gửi cố ý. Lỗi nên bị chặn tại **biên**, nơi dữ liệu ngoài thành dữ liệu trong; bên trong ta mới yên tâm giả định hồ sơ hợp lệ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Payload từ bên ngoài đi qua BaseModel ở biên để được kiểm tra, ép kiểu và chuẩn hóa; đạt thì vào domain bên trong, sai thì dừng với ValidationError."><defs><marker id="py15-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py15-0-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="50" width="230" height="100" rx="10" class="f-warn-soft s-warn"/><text x="36" y="86" class="t-b">Payload ngoài</text><text x="36" y="114" class="t-small">dict, JSON, biến môi trường</text><path d="M250 100 L290 100" class="f-none s-phys" stroke-width="2" marker-end="url(#py15-0-arrow)"/><rect x="290" y="50" width="300" height="100" rx="10" class="f-pk-soft s-pk"/><text x="306" y="86" class="t-b">Biên: BaseModel</text><text x="306" y="114" class="t-small">kiểm tra, ép kiểu, chuẩn hóa</text><path d="M590 100 L630 100" class="f-none s-phys" stroke-width="2" marker-end="url(#py15-0-arrow)"/><rect x="630" y="50" width="230" height="100" rx="10" class="f-phys-soft s-phys"/><text x="646" y="86" class="t-b">Domain bên trong</text><text x="646" y="114" class="t-small">dataclass, giả định đã đúng</text><path d="M440 150 L440 185" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py15-0-warn)"/><rect x="290" y="185" width="300" height="46" rx="10" class="f-warn-soft s-warn"/><text x="306" y="213" class="t-small">ValidationError: dừng ngay tại biên</text></svg>
</div>
<figcaption>Hình 1. Kiểm tra một lần ở biên. Dữ liệu qua được cửa này mới được vào các hàm nghiệp vụ; lỗi dừng ở nơi có đủ ngữ cảnh để báo cho người gửi.</figcaption>
</figure>

Với `dataclass`, cửa này không tồn tại. Lab có một `DriverRecord` viết bằng dataclass và một `DriverProfile` viết bằng `BaseModel`, cho cùng một payload sai ba chỗ (`driver_id` là số, `full_name` là `None`, `phone` là `"abc"`):

```python
# models.py
@dataclass
class DriverRecord:
    driver_id: str
    full_name: str
    phone: str
```

```python
# main.py (trích): cùng payload rác vào dataclass rồi vào BaseModel
record = DriverRecord(driver_id=123, full_name=None, phone="abc")
print("dataclass:", record)
DriverProfile.model_validate(sample_payload(driver_id=123, full_name=None, phone="abc"))
```

Kết quả (trích):

```text
dataclass: DriverRecord(driver_id=123, full_name=None, phone='abc')
[
  {
    "field": "driver_id",
    "code": "string_type",
    "message": "Input should be a valid string"
  },
  {
    "field": "full_name",
    "code": "string_type",
    "message": "Input should be a valid string"
  },
  {
    "field": "phone",
    "code": "phone_format",
    "message": "số điện thoại phải gồm 10 chữ số bắt đầu bằng 0"
  }
]
```

Dataclass nhận cả ba giá trị sai mà không phàn nàn. `BaseModel` báo **cả ba lỗi cùng lúc** chứ không dừng ở lỗi đầu tiên, nên người gửi sửa một lượt xong. Lưu ý: pydantic v2 không tự biến số `123` thành chuỗi `"123"`; bảng chuyển đổi đầy đủ nằm trong tài liệu gốc.

## Khai báo ràng buộc cho DriverProfile

Một `BaseModel` khai báo trường bằng type hints giống dataclass, thêm `Field(...)` cho ràng buộc và các hàm validator cho quy tắc phức tạp hơn. Phần đầu của `DriverProfile` (trích từ `models.py`):

```python
class DriverProfile(BaseModel):
    model_config = ConfigDict(
        str_strip_whitespace=True,  # cắt khoảng trắng đầu/cuối mọi chuỗi trước khi kiểm
        extra="forbid",  # khóa lạ trong payload là lỗi, không bị bỏ qua lặng lẽ
        validate_assignment=True,  # gán lại thuộc tính cũng chạy lại validator (xem giới hạn trong bài)
        hide_input_in_errors=True,  # str(ValidationError) không in input_value: số giấy tờ không vào log
    )

    driver_id: str = Field(pattern=r"^DR-[0-9]{3}$")  # [0-9], không dùng \d: \d khớp cả chữ số Ả Rập
    # alias: payload cũ của app di động dùng "fullName"; payload mới dùng "full_name"
    full_name: str = Field(
        min_length=2, max_length=80, validation_alias=AliasChoices("full_name", "fullName")
    )
    # repr=False: số giấy tờ không xuất hiện trong repr()
    id_number: str = Field(repr=False)
    license_number: str = Field(repr=False)
    phone: str
    city: City
```

Mỗi dòng trong cấu hình có lý do riêng:

- `extra="forbid"` làm khóa lạ thành lỗi `extra_forbidden`. Mặc định pydantic bỏ qua khóa thừa, nên gõ nhầm `licence_number` chỉ báo thiếu trường chứ không báo «khóa này không tồn tại».
- `validate_assignment=True` kiểm lại khi gán `profile.phone = "abc"`; mặc định pydantic chỉ kiểm lúc dựng. Giới hạn của nó ở mục «Gán lại thuộc tính».
- `Field(repr=False)` giấu số CCCD và GPLX khỏi `repr()`, nhưng **chỉ** khỏi `repr()`: `model_dump()` vẫn xuất đủ. `hide_input_in_errors=True` ngăn `str(ValidationError)` in giá trị đầu vào. Cả hai chỉ giảm rò rỉ vô ý, không phải mã hóa.
- `[0-9]` thay cho `\d`: với `^DR-\d{3}$`, pydantic cho qua `driver_id="DR-٠٠١"` (chữ số Ả Rập-Ấn Độ), vì `\d` khớp cả chữ số Unicode. Lab thử ba trường hợp (CCCD, `driver_id`, điện thoại) và `[0-9]` từ chối cả ba.
- `City` là `Literal[...]` với sáu thành phố của Tới Nơi, nên «Huế» bị từ chối với mã `literal_error`.

Dữ liệu trong lab là dữ liệu giả: CCCD bắt đầu bằng `099` (mã tỉnh không tồn tại), GPLX bắt đầu bằng `99`. Model chỉ kiểm **hình dạng** (12 chữ số), không kiểm số có thật hay không, việc đó cần nguồn dữ liệu chính thống chứ không phải một validator.

Quy tắc phức tạp hơn regex dùng `@field_validator`, viết thành classmethod nhận giá trị đã qua kiểm kiểu:

```python
    @field_validator("id_number", "license_number")
    @classmethod
    def twelve_digits(cls, value: str, info: ValidationInfo) -> str:
        """Chỉ kiểm hình dạng (12 chữ số), không kiểm số có thật hay không."""
        if not re.fullmatch(r"[0-9]{12}", value):
            raise PydanticCustomError(
                "digits_12",
                "{field} phải gồm đúng 12 chữ số",
                {"field": info.field_name},
            )
        return value

    @field_validator("phone")
    @classmethod
    def normalize_phone(cls, value: str) -> str:
        """'0900 000 001' hay '0900.000.001' đều lưu thành '0900000001'."""
        digits = re.sub(r"[ .\-]", "", value)
        if not re.fullmatch(r"0[0-9]{9}", digits):
            raise PydanticCustomError(
                "phone_format", "số điện thoại phải gồm 10 chữ số bắt đầu bằng 0"
            )
        return digits
```

Validator không chỉ kiểm mà còn **chuẩn hóa**: giá trị hàm trả về mới là giá trị được lưu. `PydanticCustomError(mã, mẫu thông báo, ngữ cảnh)` báo lỗi với **mã ổn định**; `raise ValueError("...")` vẫn được bắt nhưng mã luôn là `value_error`, thông báo có tiền tố «Value error, » và ngữ cảnh chứa đối tượng exception, không chuyển được sang JSON.

Quy tắc liên quan nhiều trường đặt ở `@model_validator(mode="after")`, chạy khi từng trường đã hợp lệ:

```python
    @model_validator(mode="after")
    def documents_are_distinct(self) -> Self:
        """Quy tắc chéo trường của lab: hai giấy tờ không trùng số (bắt lỗi dán nhầm cột)."""
        if self.id_number == self.license_number:
            raise PydanticCustomError(
                "documents_same", "CCCD và GPLX không được cùng một số"
            )
        return self
```

Đây là quy tắc riêng của lab, không phải quy định của cơ quan nào: hai số trùng nhau thường là dấu hiệu dán nhầm cột. Field validator thấy một giá trị, model validator thấy cả hồ sơ. Vì kiểu `after` chỉ chạy khi mọi trường đã qua, hồ sơ vừa sai điện thoại vừa trùng số chỉ báo lỗi điện thoại trước.

### Xuất dữ liệu: model_dump

`model_dump()` đổi model thành `dict`, `model_dump_json()` thành chuỗi JSON. Với hồ sơ hợp lệ (đã chuẩn hóa tên và số điện thoại), lab in:

```text
model_dump đầy đủ: {'driver_id': 'DR-001', 'full_name': 'Nguyễn Văn A', 'id_number': '099000000001', 'license_number': '990000000001', 'phone': '0900000001', 'city': 'Hà Nội'}
model_dump(exclude): {'driver_id': 'DR-001', 'full_name': 'Nguyễn Văn A', 'phone': '0900000001', 'city': 'Hà Nội'}
json      : {"driver_id":"DR-001","full_name":"Nguyễn Văn A","phone":"0900000001","city":"Hà Nội"}
```

Dòng đầu cho thấy `repr=False` không ngăn `model_dump()`: số giấy tờ vẫn ở đó. Muốn đưa hồ sơ ra log hay API công khai, hãy loại tường minh bằng `exclude={...}` như dòng thứ hai.

### Gán lại thuộc tính

Ba cách gán lại một hồ sơ đã validate:

```text
lỗi ở trường  : [{'field': 'phone', 'code': 'phone_format', 'message': 'số điện thoại phải gồm 10 chữ số bắt đầu bằng 0'}]
phone vẫn là  : 0900000001
lỗi cấp model : [{'field': '', 'code': 'documents_same', 'message': 'CCCD và GPLX không được cùng một số'}]
nhưng license_number đã thành: 099000000001
frozen        : [{'field': 'phone', 'code': 'frozen_instance', 'message': 'Instance is frozen'}]
bản mới       : 0911111111 | bản cũ: 0900000001
bản mới sai   : [{'field': '', 'code': 'documents_same', 'message': 'CCCD và GPLX không được cùng một số'}] | bản cũ giữ nguyên: 990000000001
```

Gán `phone = "abc"` bị chặn và giá trị cũ còn nguyên. Nhưng gán `license_number = id_number` báo `documents_same` **mà thuộc tính vẫn đã bị đổi**: lỗi cấp model chỉ được phát hiện sau khi giá trị đã ghi. Vậy `validate_assignment` không giữ được bất biến liên trường. Hai cách an toàn hơn: `frozen=True` (lớp `FrozenDriverProfile` của lab) cấm gán và báo `frozen_instance`; hoặc dựng hồ sơ **mới** từ hồ sơ cũ cộng thay đổi qua `model_validate`, như hàm `updated`: bản mới được kiểm lại toàn bộ, bản cũ không đổi dù bản mới sai. Hướng này khớp với value object bất biến ở bài 13.

## Đọc lỗi có cấu trúc

Khi sai, pydantic ném `ValidationError`. Đừng in nó ra rồi bắt client đọc chuỗi: `exc.errors()` trả danh sách dict, mỗi lỗi có `type` (mã), `loc` (đường dẫn tới trường), `msg` và `input`. Hàm nhỏ trong lab đổi danh sách đó thành định dạng phẳng, ổn định cho client:

```python
def error_report(exc: ValidationError) -> list[dict[str, str]]:
    """Đổi ValidationError thành danh sách phẳng, ổn định cho client (field, code, message)."""
    return [
        {
            "field": ".".join(str(part) for part in err["loc"]),
            "code": err["type"],
            "message": err["msg"],
        }
        for err in exc.errors(include_url=False, include_context=False)
    ]
```

`include_url=False` và `include_context=False` bỏ đường link tài liệu và phần `ctx`. Lỗi cấp model như `documents_same` có `loc` rỗng, nên cột `field` là chuỗi rỗng. Thử với hồ sơ sai năm chỗ (`DR-1`, CCCD 5 chữ số, điện thoại `12`, thành phố `Huế`, khóa lạ `extra_field`):

```text
5 lỗi
[
  {
    "field": "driver_id",
    "code": "string_pattern_mismatch",
    "message": "String should match pattern '^DR-[0-9]{3}$'"
  },
  {
    "field": "id_number",
    "code": "digits_12",
    "message": "id_number phải gồm đúng 12 chữ số"
  },
  {
    "field": "phone",
    "code": "phone_format",
    "message": "số điện thoại phải gồm 10 chữ số bắt đầu bằng 0"
  },
  {
    "field": "city",
    "code": "literal_error",
    "message": "Input should be 'Hà Nội', 'TP.HCM', 'Đà Nẵng', 'Hải Phòng', 'Cần Thơ' or 'Nha Trang'"
  },
  {
    "field": "extra_field",
    "code": "extra_forbidden",
    "message": "Extra inputs are not permitted"
  }
]
```

Client nên dựa vào cột `code`; cột `message` chỉ để người đọc, và nếu muốn tiếng Việt hãy ánh xạ từ `code`. Mã `digits_12`, `phone_format` do ta đặt, các mã còn lại do pydantic định nghĩa. Đây là lỗi của **đầu vào**: ở service HTTP (bài 31), `error_report` thành thân phản hồi 422 hoặc 400; ở consumer Kafka, nó là lý do đưa message vào kho cách ly. Phía .NET xem [validation, Problem Details và OpenAPI](/articles/dotnet10-validation-problem-details-openapi).

Có một chỗ rò rỉ dễ bỏ sót: mặc định `str(ValidationError)` in cả `input_value`, tức chính số giấy tờ vừa nhập sai, và lỗi cấp model còn in cả payload. `hide_input_in_errors=True` xử lý điều đó. Với CCCD thiếu ba chữ số, lab in:

```text
1 validation error for DriverProfile
id_number
  id_number phải gồm đúng 12 chữ số [type=digits_12]
errors()[0]['input']: 099000000
```

Chuỗi lỗi không còn `input_value`, nhưng `errors()` vẫn giữ khóa `input`, nên `error_report` cố ý chỉ lấy `field`, `code`, `message`: đưa nguyên `errors()` ra log hay phản hồi thì số giấy tờ vẫn lộ.

## Ép kiểu lax, strict và JSON

Mặc định pydantic chạy ở chế độ **lax**: nó chấp nhận một số phép chuyển an toàn giữa các kiểu. JSON thường mang khoảng cách dưới dạng chuỗi hoặc số, nhưng bài 12 đã chỉ ra `float` là kiểu sai cho khoảng cách (bài 02 giải thích vì sao). `QuoteRequest` vì thế nhận chuỗi và số nguyên, còn **từ chối** số thực bằng một validator chạy trước bước ép kiểu:

```python
class QuoteBody(BaseModel):
    """Hai trường của yêu cầu báo giá, chưa có quy tắc nào khác."""

    distance_km: Decimal = Field(ge=0, le=100, decimal_places=3)
    service: ServiceKind = ServiceKind.RIDE


class QuoteRequest(QuoteBody):
    """Chuỗi "2.5" và số nguyên đều thành Decimal; số thực (float) bị từ chối."""

    @field_validator("distance_km", mode="before")
    @classmethod
    def reject_float(cls, value: Any) -> Any:
        """Chạy TRƯỚC khi pydantic ép kiểu. float đã mất độ chính xác (bài 02): đòi chuỗi hoặc int."""
        if isinstance(value, float):
            raise PydanticCustomError(
                "float_not_allowed",
                "khoảng cách không được là số thực; gửi chuỗi như \"2.5\" hoặc số nguyên",
            )
        return value
```

`mode="before"` nghĩa là validator nhận giá trị thô, trước khi ép về `Decimal`; không có nó, số `2.5` trong JSON được nhận lặng lẽ. Đổi lại, client phải gửi `"2.5"` thay vì `2.5`: một **quyết định về hợp đồng API** cần ghi vào tài liệu. Kết quả với `model_validate_json` (trích):

```text
{"distance_km": "2.5"} -> QuoteRequest(distance_km=Decimal('2.5'), service=<ServiceKind.RIDE: 'ride'>)
{"distance_km": 2, "service": "food"} -> QuoteRequest(distance_km=Decimal('2'), service=<ServiceKind.FOOD: 'food'>)
{"distance_km": 2.5} -> [{'field': 'distance_km', 'code': 'float_not_allowed', 'message': 'khoảng cách không được là số thực; gửi chuỗi như "2.5" hoặc số nguyên'}]
{"distance_km": "100.5"} -> [{'field': 'distance_km', 'code': 'less_than_equal', 'message': 'Input should be less than or equal to 100'}]
{"distance_km": "abc"} -> [{'field': 'distance_km', 'code': 'decimal_parsing', 'message': 'Input should be a valid decimal'}]
{"service": "taxi"} -> [{'field': 'distance_km', 'code': 'missing', 'message': 'Field required'}, {'field': 'service', 'code': 'enum', 'message': "Input should be 'ride', 'food' or 'parcel'"}]
```

`ServiceKind` là `StrEnum`: `"food"` thành `ServiceKind.FOOD`, `"taxi"` bị từ chối với mã `enum` mà không che lỗi thiếu `distance_km`. Xuất ngược ra, `model_dump()` giữ `Decimal` và thành viên enum, còn `model_dump(mode="json")` đổi chúng thành chuỗi mà `json.dumps` xử lý được:

```text
model_dump         : {'distance_km': Decimal('2.5'), 'service': <ServiceKind.FOOD: 'food'>}
model_dump(json)   : {'distance_km': '2.5', 'service': 'food'}
```

Khi cần chặt hơn, bật **strict** bằng `ConfigDict(strict=True)`. Hành vi đã chạy, với `StrictQuoteBody` (cùng hai trường, không có validator `before`):

```text
strict, python mode, chuỗi: [{'field': 'distance_km', 'code': 'is_instance_of', 'message': 'Input should be an instance of Decimal'}]
strict, python mode, Decimal: distance_km=Decimal('2.5') service=<ServiceKind.RIDE: 'ride'>
strict, JSON, chuỗi        : distance_km=Decimal('2.5') service=<ServiceKind.RIDE: 'ride'>
strict + validator before, JSON, chuỗi: [{'field': 'distance_km', 'code': 'is_instance_of', 'message': 'Input should be an instance of Decimal'}]
```

Điều cần nhớ: **strict không chặn đường JSON**. Với dữ liệu Python, chuỗi `"2.5"` bị từ chối và phải đưa đúng `Decimal`; nhưng `model_validate_json` vẫn đổi chuỗi `"2.5"` thành `Decimal`. Muốn từ chối chuỗi ở đường JSON, hãy tự viết validator `mode="before"`. Dòng cuối chỉ là quan sát ở pydantic 2.14.3 (strict cộng validator `before` thì JSON chuỗi cũng bị từ chối), không phải hợp đồng đã tài liệu hóa. Strict hợp với dữ liệu nội bộ có kiểu rõ; lax hợp với dữ liệu text từ ngoài.

## BaseSettings: cấu hình cũng là dữ liệu từ ngoài

Cấu hình (URL database, thời gian sống của báo giá, bật debug) cũng là dữ liệu từ ngoài: ai đó sửa biến môi trường trên máy chủ. Gói `pydantic-settings` cho `BaseSettings`, cùng cơ chế kiểm tra như `BaseModel` nhưng nguồn giá trị là môi trường:

```python
class AppSettings(BaseSettings):
    # env_prefix: trường pg_url đọc từ TOINOI_PG_URL, quote_ttl_seconds từ TOINOI_QUOTE_TTL_SECONDS.
    model_config = SettingsConfigDict(env_prefix=ENV_PREFIX, extra="ignore", hide_input_in_errors=True)

    # SecretStr: repr và log chỉ in '**********'; lấy giá trị thật bằng .get_secret_value()
    pg_url: SecretStr = SecretStr("postgresql://toinoi:toinoi@localhost:54329/toinoi")
    mongo_url: str = "mongodb://localhost:27019/toinoi"
    quote_ttl_seconds: int = Field(300, ge=30, le=3600)
    max_distance_km: Decimal = Field(Decimal("100"), gt=0, le=1000)
    default_city: City = "Hà Nội"
    debug: bool = False
```

Biến môi trường luôn là chuỗi; `BaseSettings` chuyển `"600"` thành `600` và `"yes"` thành `True` bằng cơ chế lax ở trên. Tên biến là tiền tố cộng tên trường (`TOINOI_QUOTE_TTL_SECONDS`), và mặc định không phân biệt hoa thường.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Bốn nguồn cấu hình xếp theo mức ưu tiên tăng dần: mặc định trong class, file .env, biến môi trường TOINOI, tham số khi dựng AppSettings. Giá trị sai ở nguồn nào cũng gây ValidationError khi khởi động."><defs><marker id="py15-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="180" height="90" rx="10" class="f-phys-soft s-phys"/><text x="34" y="76" class="t-b">Mặc định</text><text x="34" y="104" class="t-small">giá trị trong class</text><path d="M200 85 L240 85" class="f-none s-phys" stroke-width="2" marker-end="url(#py15-1-arrow)"/><rect x="240" y="40" width="180" height="90" rx="10" class="f-phys-soft s-phys"/><text x="254" y="76" class="t-b">File .env</text><text x="254" y="104" class="t-small">_env_file=...</text><path d="M420 85 L460 85" class="f-none s-phys" stroke-width="2" marker-end="url(#py15-1-arrow)"/><rect x="460" y="40" width="180" height="90" rx="10" class="f-pk-soft s-pk"/><text x="474" y="76" class="t-b">Biến môi trường</text><text x="474" y="104" class="t-small">TOINOI_*</text><path d="M640 85 L680 85" class="f-none s-phys" stroke-width="2" marker-end="url(#py15-1-arrow)"/><rect x="680" y="40" width="180" height="90" rx="10" class="f-pk-soft s-pk"/><text x="694" y="76" class="t-b">Tham số</text><text x="694" y="104" class="t-small">AppSettings(x=1)</text><text x="20" y="165" class="t-small">ưu tiên tăng dần: nguồn bên phải thắng nguồn bên trái</text><rect x="20" y="190" width="840" height="50" rx="10" class="f-warn-soft s-warn"/><text x="36" y="220" class="t-small">Giá trị sai ở bất kỳ nguồn nào: ValidationError ngay khi khởi động, không đợi tới lúc dùng</text></svg>
</div>
<figcaption>Hình 2. Các nguồn cấu hình của BaseSettings. Mọi nguồn đi qua cùng bộ kiểm tra kiểu và ràng buộc như BaseModel.</figcaption>
</figure>

Kết quả chạy demo (trích):

```text
mặc định        : pg_url=SecretStr('**********') mongo_url='mongodb://localhost:27019/toinoi' quote_ttl_seconds=300 max_distance_km=Decimal('100') default_city='Hà Nội' debug=False
từ env          : 600 True Đà Nẵng
tham số thắng env: 45
env thắng .env  : 600 | .env cho city: TP.HCM
repr SecretStr  : SecretStr('**********')
giá trị thật    : postgresql://toinoi:toinoi@db.noi-bo:5432/toinoi
```

Đọc theo thứ tự: không có gì thì dùng mặc định; biến môi trường ghi đè mặc định; tham số truyền thẳng khi dựng (`AppSettings(quote_ttl_seconds=45)`) thắng cả biến môi trường; còn `.env` đứng giữa mặc định và môi trường (Hình 2): `600` trong môi trường thắng `120` trong file, còn `default_city` chỉ có trong file nên lấy `TP.HCM`. Lab truyền `_env_file=...` tường minh thay vì đặt `env_file` cứng trong class, để không vô tình đọc một `.env` thật. Dòng không có tiền tố như `OTHER=1` trong `.env` gây lỗi `extra_forbidden` nếu không đặt `extra="ignore"`.

`SecretStr` giữ chuỗi nối có mật khẩu khỏi `repr` và log; muốn dùng phải gọi `.get_secret_value()` có chủ ý. Giá trị `toinoi:toinoi` chỉ là tài khoản hạ tầng cục bộ của bài học; production không để mật khẩu trong mã nguồn.

Lợi ích lớn nhất là **fail fast**: dựng `AppSettings()` ngay lúc khởi động, cấu hình sai làm service dừng ngay thay vì hỏng ở request thứ một nghìn. `settings_error_report` của lab ghép tiền tố vào tên trường để người vận hành biết sửa biến nào:

```text
cấu hình sai, dừng khi khởi động:
  TOINOI_QUOTE_TTL_SECONDS: Input should be greater than or equal to 30
  TOINOI_DEFAULT_CITY: Input should be 'Hà Nội', 'TP.HCM', 'Đà Nẵng', 'Hải Phòng', 'Cần Thơ' or 'Nha Trang'
  TOINOI_DEBUG: Input should be a valid boolean, unable to interpret input
```

Cả ba lỗi hiện cùng lúc; báo cáo cố ý không in giá trị nhận được vì biến cấu hình có thể là bí mật. Phía C# dùng Options, xem [bài middleware, DI, options và logging](/articles/dotnet10-middleware-di-options-logging).

## Dataclass hay pydantic

Hai công cụ đứng ở hai vị trí khác nhau của hệ thống:

| Tình huống | Chọn | Lý do |
| --- | --- | --- |
| JSON, message Kafka, form, biến môi trường, file CSV vào hệ thống | `BaseModel` / `BaseSettings` | Cần kiểm, ép kiểu, báo lỗi có cấu trúc ngay biên |
| Giá trị nghiệp vụ bên trong như `Money`, `Quote`, `Trip` | `dataclass` (bài 11, 13) | Dữ liệu đã được kiểm; ít chi phí, ít phụ thuộc |
| Ba hằng số chính sách như `FarePolicy` | `NamedTuple` | Giá trị nhỏ, bất biến, không cần kiểm lúc chạy |

Quy tắc ngón tay cái: **pydantic ở biên, dataclass ở trong**. Đừng đổi mọi class thành `BaseModel` chỉ vì tiện: dữ liệu nội bộ không cần kiểm lại, và pydantic kéo thêm một phụ thuộc vào lõi nghiệp vụ. Ngược lại, đừng tin type hint của dataclass ở chỗ dữ liệu từ ngoài đi vào.

## Chạy và kiểm chứng

Cần uv và Python 3.14. Lần đầu uv tải `pydantic` và `pydantic-settings` nên cần mạng. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/15-pydantic-validation-settings examples/toi-noi-python/15-pydantic-validation-settings/main.py
uv run --project examples/toi-noi-python/15-pydantic-validation-settings examples/toi-noi-python/15-pydantic-validation-settings/main.py --verify
```

Lệnh đầu in toàn bộ demo trong bài (các đoạn kết quả ở trên là trích từ đó). Lệnh thứ hai chạy 53 ca kiểm chứng và kết thúc bằng:

```text
OK 53/53
```

Bản đã chạy dùng Python 3.14.7, pydantic 2.14.3 và pydantic-settings 2.16.2 (đúng như `uv.lock`). `--verify` tự gỡ mọi biến `TOINOI_*` trong lúc chạy nên không phụ thuộc môi trường của bạn. Nếu một ca lệch, nó in `SAI: <tên ca>`, rồi `FAIL n/m` (n là số ca đạt, m là tổng) và thoát với mã 1.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Gán `profile.phone = "abc"` mà không lỗi | Model có `validate_assignment=True` chưa; mặc định pydantic chỉ kiểm lúc dựng |
| Gán báo `documents_same` nhưng thuộc tính vẫn đổi | `validate_assignment` không hoàn tác lỗi cấp model; dùng `frozen=True` hoặc dựng bản mới |
| `strict=True` mà `model_validate_json` vẫn nhận `"2.5"` | Strict không chặn đường JSON; thêm validator `mode="before"` nếu muốn từ chối |
| Số giấy tờ vẫn lộ trong log lỗi | Đã đặt `hide_input_in_errors=True` chưa; đừng log nguyên `errors()` vì khóa `input` vẫn còn |
| Biến `TOINOI_*` bị bỏ qua | Tên có đúng tiền tố cộng tên trường không; `_env_file` có đúng đường dẫn không |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Validator chỉ kiểm hình dạng dữ liệu giả: số CCCD 12 chữ số, điện thoại 10 chữ số, thành phố trong danh sách. Nó không biết số có thật, giấy tờ còn hạn hay người đó có đúng là chủ giấy tờ không; kết quả OCR và so khớp khuôn mặt ở các bài sau cũng chỉ là đầu vào cho người duyệt. Production còn cần kiểm tra trùng hồ sơ, giới hạn kích thước payload, kho bí mật thay vì biến môi trường thuần, và mã hóa dữ liệu cá nhân khi lưu. <code>SecretStr</code>, <code>repr=False</code> và <code>hide_input_in_errors</code> chỉ ngăn lộ vô ý trong log: <code>model_dump()</code> và khóa <code>input</code> của <code>errors()</code> vẫn trả giá trị thật.</p>
</div>

**Bài tập 1:** Trong `DriverProfile`, bỏ dòng `validate_assignment=True`. Chạy `profile.city = "Huế"`. Điều gì xảy ra, và vì sao điều này nguy hiểm hơn một lỗi `ValidationError`?

**Bài tập 2:** Đặt `TOINOI_QUOTE_TTL_SECONDS=5` và `TOINOI_DEBUG=maybe`, rồi dựng `AppSettings()`. Có mấy lỗi, mỗi lỗi nêu biến nào?

<details><summary>Xem lời giải</summary><p><b>Bài 1.</b> Phép gán thành công mà không báo gì: <code>profile.city</code> trở thành <code>"Huế"</code>, một giá trị mà <code>DriverProfile.model_validate</code> sẽ từ chối. Nguy hiểm vì ta có một đối tượng mang kiểu «đã validate» nhưng chứa dữ liệu sai, mà mọi mã phía sau tin vào kiểu đó. (Bật <code>validate_assignment=True</code> chặn được ca này; <code>frozen=True</code> cấm gán hoàn toàn và hợp hơn với value object ở bài 13.) <b>Bài 2.</b> Hai lỗi: <code>TOINOI_QUOTE_TTL_SECONDS</code> (giá trị <code>5</code> nhỏ hơn mức tối thiểu 30) và <code>TOINOI_DEBUG</code> (<code>maybe</code> không phải boolean hợp lệ). Hai lỗi được báo cùng lúc, theo thứ tự khai báo trường;</p></details>

Bài trước: [bài 14](/articles/python-abc-protocol-composition-dependency-injection). Bài tiếp: [bài 16](/articles/python-match-pattern-matching-none).

## Tài liệu gốc

- [Pydantic: Models](https://docs.pydantic.dev/latest/concepts/models/).
- [Pydantic: Validators](https://docs.pydantic.dev/latest/concepts/validators/).
- [Pydantic: Validation errors](https://docs.pydantic.dev/latest/errors/validation_errors/).
- [Pydantic: Conversion table](https://docs.pydantic.dev/latest/concepts/conversion_table/).
- [Pydantic: Strict mode](https://docs.pydantic.dev/latest/concepts/strict_mode/).
- [Pydantic Settings management](https://docs.pydantic.dev/latest/concepts/pydantic_settings/).
