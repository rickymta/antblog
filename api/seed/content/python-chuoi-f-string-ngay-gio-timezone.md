---
slug: python-chuoi-f-string-ngay-gio-timezone
title: "Chuỗi, f-string, ngày giờ và múi giờ: hiển thị đúng, lưu đúng"
summary: "Bài 04: str, bytes và UTF-8 với «Tới Nơi», f-string và format_vnd, datetime naive và aware, zoneinfo Asia/Ho_Chi_Minh, ISO 8601, timedelta; lưu UTC, hiển thị giờ địa phương, tính đúng ngày chuyến."
category: python-beginner
level: beginner
tags: [python, toi-noi, datetime, unicode]
language: vi
---

Email của [Tới Nơi](/articles/du-an-mau-toi-noi) xác nhận một chuyến đón khách lúc 06:30 sáng, nhưng báo cáo cuối ngày của chị vận hành lại xếp chuyến đó vào ngày hôm trước. Cùng lúc, một tin nhắn từ dịch vụ cũ hiển thị tên ứng dụng thành `Tá»i NÆ¡i`. Hai lỗi nhìn khác nhau nhưng cùng một gốc: ta chưa phân biệt rõ **dữ liệu** với **cách hiển thị** nó, ở cả văn bản lẫn thời gian.

Đây là bài 04 của lộ trình **Python cơ bản qua Tới Nơi**. Bài dùng `Decimal` từ [bài 02](/articles/python-bien-kieu-du-lieu-decimal) cho tiền và chỉ dùng thư viện chuẩn, không cài thêm gói nào.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p><code>"Tới Nơi"</code> có 7 ký tự. Khi mã hoá UTF-8, nó có bao nhiêu byte, và vì sao con số khác 7?</p>
<details><summary>Xem đáp án</summary><p>10 byte. UTF-8 dùng 1 đến 4 byte cho mỗi ký tự: chữ ASCII như <code>T</code>, <code>N</code>, <code>i</code> mất 1 byte, còn <code>ớ</code> mất 3 byte và <code>ơ</code> mất 2 byte. <code>len(str)</code> đếm ký tự (code point), <code>len(bytes)</code> đếm byte.</p></details>
</div>

## str và bytes: văn bản khác với chuỗi byte

Kiểu `str` của Python 3 là dãy ký tự **Unicode** và là kiểu **bất biến**: mọi phương thức như `upper()`, `strip()`, `casefold()` đều trả chuỗi mới và không sửa chuỗi cũ. Gán vào một vị trí sẽ lỗi:

```text
TypeError: 'str' object does not support item assignment
```

Bất biến không có nghĩa biến không gán lại được. `s = "A"; s = "B"` vẫn hợp lệ: biến trỏ sang một giá trị chuỗi khác. Để so sánh không phân biệt hoa thường, dùng `casefold()` thay vì `lower()`: nó xử lý đúng cả những chữ mà `lower()` bỏ sót, và `"Tới Nơi".casefold()` cho `tới nơi`.

Khác với `str`, `bytes` là dãy byte thô. Điểm dễ nhầm: lấy một phần tử của `bytes` cho ra **số nguyên** từ 0 đến 255, không phải chuỗi một ký tự. Khi ghi file, gửi qua mạng hay lưu database, chữ phải đi qua một **encoding** để thành byte, và khi đọc lại phải dùng đúng encoding đó:

```python
APP_NAME = "Tới Nơi"

def text_vs_bytes(text: str) -> tuple[int, int, bytes]:
    """Trả về (số ký tự, số byte UTF-8, bytes). str đếm ký tự, bytes đếm byte."""
    raw = text.encode("utf-8")
    return len(text), len(raw), raw
```

Kết quả chạy thật trong lab (trích):

```text
'Tới Nơi': 7 ký tự, 10 byte UTF-8
bytes: b'T\xe1\xbb\x9bi N\xc6\xa1i'; raw[0] = 84 (int, không phải str)
decode UTF-8: Tới Nơi
decode sai (latin-1): 'Tá»\x9bi NÆ¡i'
decode ascii strict: UnicodeDecodeError: ordinal not in range(128)
decode ascii replace: 'T���i N��i'
```

Dòng `decode sai (latin-1)` là lỗi hiển thị trong mở bài: byte UTF-8 bị đọc như Latin-1 nên mỗi byte thành một ký tự riêng (hiện tượng **mojibake**). Ký tự `\x9b` là một ký tự điều khiển vô hình khi in, nên trên màn hình ta chỉ thấy `Tá»i NÆ¡i`. Dữ liệu byte không hỏng; chỉ cần decode đúng là có lại chữ.

Tham số `errors=` quyết định điều gì xảy ra khi byte không hợp lệ: mặc định `"strict"` ném `UnicodeDecodeError` (tốt cho dữ liệu cần chính xác, vì lỗi hiện ra sớm), còn `"replace"` thay chỗ hỏng bằng ký tự thay thế `�` để chương trình chạy tiếp (chỉ hợp với log hay hiển thị, vì thông tin đã mất). Với file, hãy luôn nêu `encoding="utf-8"` ở cả đọc lẫn ghi thay vì tin mặc định của máy, như hàm `file_roundtrip` của lab (`write_text(..., encoding="utf-8")` rồi `read_text(..., encoding="utf-8")`, in lại đúng `Tới Nơi`).

Tiếng Việt còn một chi tiết nữa: chữ `ớ` có thể là **một** code point dựng sẵn (NFC) hoặc `ơ` cộng dấu sắc rời (NFD). Hai chuỗi trông giống hệt nhưng so sánh `==` ra `False`. Chuẩn hoá trước khi so sánh hoặc làm khoá tìm kiếm:

```python
import unicodedata

def normalized_forms(text: str) -> tuple[bool, str]:
    nfd = unicodedata.normalize("NFD", text)
    nfc = unicodedata.normalize("NFC", nfd)
    return text == nfd, nfc
```

Với `"Tới Nơi"`, lab in `NFC == NFD? False; NFC: Tới Nơi`. Chuỗi nhập từ bàn phím, từ macOS hay từ file Word có thể mang dạng khác nhau; quy ước của ta là chuẩn hoá về NFC ở cửa vào.

## f-string, format spec và số tiền VND

f-string (`f"..."`) chèn biểu thức vào `{}` và nhận thêm **format spec** sau dấu `:`. Cú pháp tổng quát của spec là `[fill][align][sign][width][grouping][.precision][type]`; không cần đủ mọi phần.

| Thành phần | Ý nghĩa | Ví dụ → kết quả |
| --- | --- | --- |
| fill và align | `<` trái, `>` phải, `^` giữa, kèm ký tự lấp | `{APP_NAME:*^11}` → `**Tới Nơi**` |
| sign | `+` luôn hiện dấu | `{n:+,}` → `+14,250` |
| width | độ rộng tối thiểu | `{n:>10,}` → `    14,250` |
| grouping | `,` hoặc `_` phân nhóm nghìn | `{n:,}` → `14,250`; `{n:_}` → `14_250` |
| precision và type | `.2f` hai chữ số thập phân | `{price:.2f}` → `2.50` |

Ba cú pháp phụ hay gặp: `{x!r}` chèn `repr(x)` thay vì `str(x)` (hữu ích khi gỡ lỗi, như `'Tới Nơi'` có cả dấu nháy); `{n=}` in cả tên biểu thức lẫn giá trị (`n=14250`); và muốn in dấu ngoặc nhọn thật thì viết đôi: `f"{{n}}"` cho ra `{n}`. Độ rộng cũng có thể lồng vào spec, như `{label:<{width}}`.

Còn dấu phân cách tiền Việt. Spec `,` luôn dùng dấu phẩy kiểu Mỹ, trong khi người Việt đọc `14.250`. Có hai lối đi: module `locale` (kết quả phụ thuộc cấu hình máy và locale đã cài, nên không đáng tin trong container) hoặc tự viết một hàm nhỏ có kết quả xác định. Lab chọn cách thứ hai.

Có một cái bẫy trước khi viết hàm: spec `.0f` **không** chỉ «bỏ phần thập phân» mà làm tròn, và với `Decimal` nó làm tròn theo context hiện hành (mặc định HALF_EVEN, làm tròn về số chẵn gần nhất). Lab in ra:

```text
{Decimal('12004.5'):,.0f} -> 12,004
format_vnd(12004.5) -> 12.005
```

Cước thô 12.004,5 theo quy ước của loạt bài phải thành 12.005 (HALF_UP), nên `format_vnd` phải `quantize` bằng `ROUND_HALF_UP` **trước**, rồi mới dùng spec để nhóm nghìn:

```python
def format_vnd(amount: Decimal) -> str:
    """14250 -> '14.250'. Làm tròn HALF_UP trước, rồi đổi dấu phẩy thành dấu chấm."""
    rounded = amount.quantize(Decimal("1"), rounding=ROUND_HALF_UP)
    return f"{rounded:,.0f}".replace(",", ".")

def receipt_line(label: str, amount: Decimal, width: int = 32) -> str:
    """Căn lề bằng format spec: nhãn căn trái, số tiền căn phải."""
    money = format_vnd(amount) + " VND"
    return f"{label:<{width - 12}}{money:>12}"
```

```text
format_vnd -> 14.250
Cước 2,5 km           14.250 VND
Cước 5 km             25.500 VND
```

`format_vnd` chỉ để **hiển thị**: số tiền đi qua API vẫn là số hoặc chuỗi thập phân có quy ước rõ, không phải chuỗi `"14.250"`.

Ba phương thức chuỗi thường dùng khi xử lý dữ liệu nhập: `strip()` bỏ khoảng trắng hai đầu, `split(";")` tách thành list, `", ".join(...)` ghép lại. Mỗi bước tạo ra chuỗi hoặc list mới:

```python
def parse_trip_line(line: str) -> tuple[str, str, Decimal]:
    """'  TN-0001 ; ride ; 5.0  ' -> ('TN-0001', 'ride', Decimal('5.0'))."""
    parts = [p.strip() for p in line.strip().split(";")]
    if len(parts) != 3:
        raise ValueError(f"cần 3 trường, nhận {len(parts)}: {line!r}")
    trip_id, service, km = parts
    return trip_id, service, Decimal(km)
```

Dòng `[p.strip() for p in ...]` là list comprehension, sẽ được giải thích ở bài 06; hiện tại đọc nó như «bỏ khoảng trắng của từng phần». Đừng tự cắt chuỗi theo chỉ số cố định cho dữ liệu có cấu trúc: dữ liệu nhập luôn có khoảng trắng thừa hay thiếu một trường, và khi đó hàm phải từ chối thay vì đoán.

## datetime: naive, aware và múi giờ

`datetime` có hai dạng. **Naive** không mang thông tin múi giờ: `datetime(2026, 11, 6, 8, 30)` chỉ nói «8:30» mà không nói ở đâu. **Aware** mang `tzinfo`, nên xác định một thời điểm duy nhất trên trục thời gian. Python xử lý hai dạng này khác nhau tuỳ phép toán, và đó là chỗ dễ sai:

- So sánh thứ tự (`<`, `>`) hoặc trừ giữa naive và aware ném `TypeError`.
- Nhưng `==` giữa naive và aware **không** ném lỗi: nó trả `False` một cách âm thầm, vì theo tài liệu, naive không bao giờ bằng aware.

```text
naive < aware lỗi? True
naive == aware là: False
```

Dòng đầu là `TypeError: can't compare offset-naive and offset-aware datetimes`. Dòng sau nguy hiểm hơn: một câu `if started == expected` có thể sai mãi mà không có thông báo nào. Cách phòng là không để naive lọt vào trong hệ thống: kiểm một lần ở biên bằng một hàm dùng chung.

```python
def require_aware(value: datetime) -> datetime:
    """Một chỗ kiểm duy nhất: datetime phải aware (có tzinfo và utcoffset)."""
    if value.tzinfo is None or value.utcoffset() is None:
        raise ValueError("cần datetime aware; datetime naive không biết mình ở múi giờ nào")
    return value
```

Quy ước của loạt bài: mọi thời điểm trong hệ thống là **aware**, **lưu bằng UTC**, và chỉ đổi sang giờ địa phương khi hiển thị. Lấy «bây giờ» đúng cách là `datetime.now(timezone.utc)` (hoặc `datetime.now(UTC)`: `UTC` và `timezone.utc` là cùng một đối tượng). Hàm cũ `datetime.utcnow()` đã bị đánh dấu **deprecated** từ Python 3.12 vì trả về datetime *naive*, trông như UTC nhưng thiếu `tzinfo`, nên dễ bị trộn với giờ địa phương. Lab xác nhận cả hai điều:

```text
utcnow() deprecated và naive? True
```

Để kết quả tái lập, lab dùng thời điểm cố định `2026-11-06 01:30 UTC` thay vì giờ chạy. Múi giờ Việt Nam lấy từ `zoneinfo.ZoneInfo("Asia/Ho_Chi_Minh")`; ta dùng tên IANA thay vì cộng tay 7 giờ, vì tên này mang theo quy tắc lịch sử của vùng:

```python
from datetime import UTC, datetime, timedelta
from zoneinfo import ZoneInfo

VN_TZ = ZoneInfo("Asia/Ho_Chi_Minh")

def to_local(instant: datetime) -> datetime:
    """Đổi một thời điểm aware sang giờ Việt Nam. Từ chối datetime naive."""
    return require_aware(instant).astimezone(VN_TZ)
```

Ta từ chối naive một cách chủ động vì `astimezone()` trên datetime naive sẽ *giả định* đó là giờ của máy đang chạy: máy ở Hà Nội và máy trong container UTC cho hai kết quả khác nhau.

### replace và astimezone: gắn nhãn khác đổi giờ

Hai phương thức này hay bị dùng lẫn. `astimezone(tz)` **đổi** cùng một thời điểm sang cách viết ở múi giờ khác (giờ trên đồng hồ thay đổi, thời điểm không đổi). `replace(tzinfo=tz)` chỉ **gắn nhãn** múi giờ lên con số giờ đã có, không đổi con số (thời điểm đổi theo). Dùng `replace` khi ta *biết chắc* một datetime naive đang ở múi giờ nào, chẳng hạn khách gõ 08:30 theo giờ Việt Nam:

```python
def attach_vs_convert() -> tuple[str, str]:
    typed = datetime(2026, 11, 6, 8, 30)  # naive: người dùng gõ giờ địa phương
    right = typed.replace(tzinfo=VN_TZ).astimezone(UTC)
    wrong = typed.replace(tzinfo=UTC).astimezone(VN_TZ)
    return right.isoformat(), wrong.isoformat()
```

```text
08:30 gắn VN_TZ rồi đổi UTC: 2026-11-06T01:30:00+00:00
08:30 gắn UTC rồi đổi VN   : 2026-11-06T15:30:00+07:00
```

Cách đầu đúng: 08:30 giờ Việt Nam là 01:30 UTC. Cách sau gắn nhãn UTC lên giờ gốc không phải UTC, nên cộng thêm 7 giờ lần nữa và ra 15:30, lệch 7 giờ so với ý khách. Quy tắc nhớ: gắn nhãn đúng với **nguồn** của con số, rồi mới đổi sang múi giờ **đích**.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Một thời điểm có hai cách hiển thị là UTC 01:30 và giờ Việt Nam 08:30, và một cách lưu là chuỗi ISO 8601 theo quy ước UTC.">
<defs>
<marker id="py04-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<rect x="320" y="15" width="240" height="70" rx="12" class="f-pk-soft s-pk"/>
<text x="336" y="43" class="t-b">Một thời điểm (instant)</text>
<text x="336" y="64" class="t-small">không phụ thuộc cách hiển thị</text>
<rect x="30" y="150" width="260" height="75" rx="12" class="f-phys-soft s-phys"/>
<text x="46" y="178" class="t-b">Hiển thị UTC</text>
<text x="46" y="198" class="t-mono t-small">01:30 +00:00</text>
<text x="46" y="214" class="t-small">log, API, báo cáo kỹ thuật</text>
<rect x="590" y="150" width="260" height="75" rx="12" class="f-phys-soft s-phys"/>
<text x="606" y="178" class="t-b">Hiển thị +07:00</text>
<text x="606" y="198" class="t-mono t-small">08:30 +07:00</text>
<text x="606" y="214" class="t-small">ứng dụng, email cho khách</text>
<rect x="320" y="150" width="240" height="75" rx="12" class="f-warn-soft s-warn"/>
<text x="336" y="178" class="t-b">Một cách lưu</text>
<text x="336" y="198" class="t-mono t-small">2026-11-06T01:30:00+00:00</text>
<text x="336" y="214" class="t-small">quy ước lưu UTC (ISO 8601)</text>
<line x1="380" y1="85" x2="240" y2="148" class="s-pk f-none" stroke-width="1.6" marker-end="url(#py04-0-arrow)"/>
<line x1="440" y1="85" x2="440" y2="148" class="s-pk f-none" stroke-width="1.6" marker-end="url(#py04-0-arrow)"/>
<line x1="500" y1="85" x2="640" y2="148" class="s-pk f-none" stroke-width="1.6" marker-end="url(#py04-0-arrow)"/>
</svg>
</div>
<figcaption>Hình 1. Một thời điểm, hai cách hiển thị, một cách lưu. Lưu UTC ở giữa; mỗi người xem đổi sang múi giờ của mình khi cần.</figcaption>
</figure>

## Lưu bằng ISO 8601, hiển thị theo giờ địa phương

`isoformat()` cho chuỗi ISO 8601 như `2026-11-06T08:30:00+07:00`, còn `datetime.fromisoformat()` đọc ngược lại. Bản thân ISO 8601 không bắt buộc UTC; **quy ước của ta** là lưu về UTC (offset `+00:00`) và chỉ chấp nhận chuỗi có offset khi đọc, để không bao giờ tạo ra datetime naive từ dữ liệu ngoài:

```python
def to_storage(instant: datetime) -> str:
    """Chuỗi ISO 8601 theo quy ước lưu UTC: luôn kết thúc bằng '+00:00'."""
    return require_aware(instant).astimezone(UTC).isoformat()

def from_storage(text: str) -> datetime:
    """Đọc lại chuỗi ISO 8601 có offset. Kết quả là datetime aware."""
    return require_aware(datetime.fromisoformat(text))
```

```text
UTC   : 2026-11-06T01:30:00+00:00
+07:00: 2026-11-06T08:30:00+07:00
cùng thời điểm? True
lưu    : 2026-11-06T01:30:00+00:00
đọc lại: 2026-11-06T01:30:00+00:00
```

Hai giá trị `01:30 +00:00` và `08:30 +07:00` khác nhau về cách viết nhưng **bằng nhau** (`==` là `True`) vì là cùng một thời điểm. Ta lưu UTC để so sánh, sắp xếp và nhóm dữ liệu không phụ thuộc người nhìn; ta không lưu `"06/11/2026 08:30"` bỏ mất offset.

Nửa còn lại của «hiển thị đúng» là định dạng cho người đọc. `isoformat()` dành cho máy: ổn định, sắp xếp được. Với khách, ta dùng spec của datetime ngay trong f-string (`strftime` cho kết quả tương đương) và `strptime` để đọc giờ khách gõ:

```python
def format_local(instant: datetime) -> str:
    local = to_local(instant)
    return f"{local:%d/%m/%Y %H:%M}"

def parse_local_input(text: str) -> datetime:
    naive = datetime.strptime(text, "%d/%m/%Y %H:%M")
    return naive.replace(tzinfo=VN_TZ)   # ta biết chắc khách gõ giờ Việt Nam
```

```text
hiển thị: 06/11/2026 08:30
đọc giờ khách gõ: 2026-11-06T08:30:00+07:00
```

`strptime` luôn trả datetime *naive* nên ta gắn nhãn bằng `replace` ngay trong hàm đọc, đúng như mục trên. Trừ hai datetime aware cho `timedelta`, một thời lượng. Với chuyến TN-0001 trả khách sau 22 phút 15 giây:

```python
def duration_text(start: datetime, end: datetime) -> str:
    delta = require_aware(end) - require_aware(start)
    if delta < timedelta(0):
        raise ValueError("thời điểm kết thúc trước thời điểm bắt đầu")
    minutes, seconds = divmod(int(delta.total_seconds()), 60)
    return f"{minutes} phút {seconds} giây"
```

```text
thời lượng: 22 phút 15 giây
```

## Chuyến này thuộc ngày nào?

Quay về mở bài. Chị vận hành cần «số chuyến mỗi ngày», và ngày của một chuyến phụ thuộc múi giờ ta dùng để cắt. Vì Việt Nam là UTC+7, nửa đêm giờ Việt Nam (00:00) rơi vào 17:00 UTC của hôm trước. Mọi chuyến từ 17:00 đến 24:00 UTC có **ngày UTC cũ nhưng ngày Việt Nam mới**.

```python
def local_date(instant: datetime) -> date:
    """Ngày theo giờ Việt Nam: đổi múi giờ TRƯỚC khi lấy .date()."""
    return to_local(instant).date()
```

Lab dựng ba chuyến mẫu sát ranh giới và in đủ bốn cột:

```text
mã       UTC          giờ VN       ngày UTC    ngày VN
TN-0001  05/11 23:30  06/11 06:30  2026-11-05  2026-11-06
TN-0002  05/11 17:00  06/11 00:00  2026-11-05  2026-11-06
TN-0003  05/11 16:59  05/11 23:59  2026-11-05  2026-11-05
```

TN-0001 chính là chuyến 06:30 sáng ở mở bài: nếu báo cáo gọi `.date()` trên giá trị UTC, nó thành ngày 5/11. TN-0003 chỉ sớm hơn TN-0002 một phút nhưng nằm ở ngày khác theo giờ Việt Nam. Báo cáo theo ngày cần chọn rõ ngày nào (ngày địa phương của thành phố hay ngày UTC) và đổi múi giờ **trước** khi lấy `.date()`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Trục thời gian UTC từ 12:00 ngày 5 đến 12:00 ngày 6 tháng 11. Nửa đêm giờ Việt Nam rơi vào 17:00 UTC; vùng từ 17:00 đến 24:00 UTC có ngày UTC là 5 nhưng ngày Việt Nam là 6.">
<rect x="40" y="45" width="400" height="44" rx="8" class="f-phys-soft s-phys"/>
<text x="52" y="72" class="t-b">Ngày UTC: 5/11</text>
<rect x="440" y="45" width="400" height="44" rx="8" class="f-phys-soft s-phys"/>
<text x="452" y="72" class="t-b">Ngày UTC: 6/11</text>
<rect x="40" y="150" width="167" height="44" rx="8" class="f-pk-soft s-pk"/>
<text x="52" y="177" class="t-b">VN 5/11</text>
<rect x="207" y="150" width="633" height="44" rx="8" class="f-pk-soft s-pk"/>
<text x="219" y="177" class="t-b">Ngày Việt Nam: 6/11</text>
<rect x="207" y="30" width="233" height="180" rx="10" class="f-none s-warn dash" stroke-width="1.6"/>
<text x="219" y="108" class="t-small t-warn">17:00–24:00 UTC: vùng «sai ngày»</text>
<text x="219" y="124" class="t-small t-warn">ngày UTC vẫn là 5/11</text>
<text x="219" y="140" class="t-small t-warn">ngày Việt Nam đã là 6/11</text>
<text x="40" y="228" class="t-small">12:00</text>
<text x="207" y="228" class="t-small" text-anchor="middle">17:00 = 00:00 VN</text>
<text x="440" y="228" class="t-small" text-anchor="middle">00:00 UTC</text>
<text x="840" y="228" class="t-small" text-anchor="end">12:00</text>
</svg>
</div>
<figcaption>Hình 2. Ranh giới ngày theo giờ Việt Nam lệch 7 giờ so với UTC. Chuyến trong vùng nét đứt thuộc ngày UTC cũ nhưng đã sang ngày Việt Nam mới.</figcaption>
</figure>

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Mở rộng: template string (t-string) trong Python 3.14</span>
<p>Python 3.14 thêm template string literal (PEP 750), viết như f-string nhưng tiền tố <code>t</code>. Khác với f-string trả ngay một <code>str</code>, t-string trả một đối tượng <code>Template</code> tách phần văn bản tĩnh và phần nội suy, để hàm xử lý do bạn viết quyết định cách ghép (ví dụ escape). Bản thân t-string không tự escape gì. Lab có ví dụ nhỏ ngay dưới đây; không cần nó để làm bài tập.</p>
</div>

```python
from string.templatelib import Interpolation, Template

def render_html(template: Template) -> str:
    """Escape HTML cho phần nội suy, giữ nguyên phần văn bản tĩnh."""
    out: list[str] = []
    for item in template:
        if isinstance(item, Interpolation):
            out.append(html.escape(str(item.value)))
        else:
            out.append(item)
    return "".join(out)

customer = "<b>An</b>"
print("t-string:", render_html(t"Khách: {customer}"))
```

```text
t-string: Khách: &lt;b&gt;An&lt;/b&gt;
```

## Chạy lab và kiểm chứng

Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/04-chuoi-ngay-gio-timezone examples/toi-noi-python/04-chuoi-ngay-gio-timezone/main.py
uv run --project examples/toi-noi-python/04-chuoi-ngay-gio-timezone examples/toi-noi-python/04-chuoi-ngay-gio-timezone/main.py --verify
```

Lệnh thứ hai chạy `verify()` với 39 ca: độ dài và index của `bytes`, decode đúng và sai, `errors="strict"` và `"replace"`, `casefold`, file round-trip, NFC/NFD; `.0f` làm tròn HALF_EVEN so với `format_vnd`, bảng format spec; `parse_trip_line`, `join`; UTC sang +07:00, round-trip ISO, `strftime`/`strptime`, thời lượng, `naive < aware` lỗi và `naive == aware` là `False`, `utcnow()`, `replace` khác `astimezone`, bảng ba chuyến, t-string; và ba ca bị từ chối bằng `ValueError`. Hai đáp án của bài tập dưới đây cũng nằm trong số đó. Nếu có ca sai, lab in tên ca rồi `FAIL n/m` và thoát với mã 1. Khi đúng hết:

```text
OK 39/39
```

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Chữ tiếng Việt thành `Tá»i NÆ¡i` | Nơi ghi và nơi đọc có cùng `encoding="utf-8"` không |
| Hai chuỗi trông giống nhưng `==` là `False` | Có khác dạng NFC và NFD không; chuẩn hoá về NFC |
| `TypeError` khi so sánh hai datetime | Một vế naive, một vế aware; đưa cả hai về aware |
| `==` luôn `False` dù cùng giờ | Có đang so naive với aware không (không có lỗi nào báo) |
| Giờ lệch 7 tiếng giữa máy này và máy kia | Có `astimezone()` trên datetime naive, hoặc `replace(tzinfo=UTC)` lên giờ địa phương không |
| Chuyến nằm sai ngày trong báo cáo | Lấy `.date()` trước hay sau khi đổi sang múi giờ địa phương |
| Số tiền `12.004` thay vì `12.005` | Có dùng `.0f` mà quên `quantize` HALF_UP trước không |

## Giới hạn của ví dụ

Lab dùng thời điểm cố định, dữ liệu giả, và mới chỉ xét một múi giờ không có giờ mùa hè (Asia/Ho_Chi_Minh). `zoneinfo` đọc cơ sở dữ liệu múi giờ của hệ điều hành; trên Windows hoặc image container tối giản có thể cần gói `tzdata`. Lab không cài thêm gì, nên chạy trên macOS và Linux có sẵn tzdata. Lịch hẹn trong tương lai (đặt trước chuyến) cần thêm tên múi giờ riêng vì quy tắc múi giờ có thể đổi; `format_vnd` chưa xử lý số âm hay tiền tệ khác VND. Ta cũng đọc «bây giờ» trực tiếp bằng `datetime.now`; muốn kiểm thử logic hết hạn báo giá thì cần một đồng hồ thay thế được truyền vào qua constructor, và bài 14 (dependency injection) sẽ làm việc đó.

**Bài tập 1:** đổi thời điểm gốc thành `2026-11-06T23:30:00+00:00`. Giờ Việt Nam và ngày địa phương là gì? **Bài tập 2:** `format_vnd(Decimal("1234567.5"))` trả gì?

<details><summary>Xem lời giải</summary><p>(1) Cộng 7 giờ: <b>2026-11-07T06:30:00+07:00</b>. Ngày địa phương là 7/11 trong khi ngày UTC vẫn là 6/11. (2) Giá trị được làm tròn HALF_UP thành 1234568 rồi nhóm nghìn: <b>1.234.568</b>. Cả hai đáp án đều có trong <code>--verify</code>.</p></details>

Bài trước: [bài 03](/articles/python-dieu-kien-vong-lap-ham). Bài tiếp: [bài 05](/articles/python-list-dict-set-tuple).

## Tài liệu gốc

- [Unicode HOWTO](https://docs.python.org/3.14/howto/unicode.html).
- [Format Specification Mini-Language](https://docs.python.org/3.14/library/string.html#format-specification-mini-language).
- [datetime: Basic date and time types](https://docs.python.org/3.14/library/datetime.html).
- [zoneinfo: IANA time zone support](https://docs.python.org/3.14/library/zoneinfo.html).
- [PEP 750: Template Strings](https://peps.python.org/pep-0750/).
