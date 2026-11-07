---
slug: python-file-json-csv-pathlib
title: "File, JSON, CSV và pathlib: đọc bộ dữ liệu mẫu Tới Nơi"
summary: "Bài 09: đọc CSV và JSON bằng pathlib với UTF-8, tính cước chuyến theo thành phố, ghi summary.json bằng file tạm rồi replace. Giải thích cwd, newline, Decimal trong JSON; lab chạy được."
category: python-beginner
level: beginner
tags: [python, toi-noi, pathlib, json, csv]
language: vi
---

Mỗi sáng, bộ phận vận hành của [Tới Nơi](/articles/du-an-mau-toi-noi) xuất danh sách chuyến hôm trước ra một file CSV, kèm danh mục sáu thành phố dạng JSON. Ta cần đọc hai file đó, tính cước các chuyến đã hoàn thành, gom theo thành phố rồi ghi một bản tóm tắt cho người khác đọc tiếp. File trong bài chỉ có 30 chuyến giả; với 2 triệu chuyến mỗi ngày, cách đọc phải khác (xem phần giới hạn ở cuối).

Đây là bài 09 của lộ trình **Python cơ bản qua Tới Nơi**. Bạn cần nhớ `Decimal` (bài 02), `dict` và `list` (bài 05). Cước vẫn là 12.000 VND gồm 2 km đầu, mỗi km vượt 4.500 VND.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Chương trình có dòng <code>open("data/trips.csv")</code> chạy tốt khi bạn bấm Run trong editor, nhưng báo <code>FileNotFoundError</code> khi chạy từ thư mục khác. File vẫn nằm đúng chỗ. Python sai ở đâu?</p>
<details><summary>Xem đáp án</summary><p>Python không sai. Đường dẫn tương đối được tính từ <b>thư mục làm việc hiện tại</b> (cwd) của tiến trình, không phải từ thư mục chứa file <code>.py</code>. Editor tình cờ đặt cwd đúng chỗ; terminal ở nơi khác thì không.</p></details>
</div>

## Đường dẫn: cwd khác với vị trí file nguồn

Mỗi tiến trình có một **cwd** (current working directory), thư mục nó đang «đứng». Chuỗi `"data/trips.csv"` không nói gì về nơi chương trình nằm; hệ điều hành ghép nó vào cwd rồi tìm. Chạy từ thư mục gốc repository thì cwd là thư mục gốc, và file không có ở đó:

```text
FileNotFoundError: [Errno 2] No such file or directory: 'data/trips.csv'
```

Thứ ổn định là **vị trí file nguồn**: biến `__file__` chứa đường dẫn tới chính file `.py` đang chạy (từ Python 3.9, với script chạy trực tiếp đó là đường dẫn tuyệt đối). Neo dữ liệu vào `__file__` thì chạy từ đâu cũng đúng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Chạy từ thư mục gốc repository: đường dẫn tương đối data/trips.csv ghép vào cwd nên không tìm thấy; đường dẫn neo theo __file__ trỏ đúng vào thư mục data của lab."><defs><marker id="py09-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py09-0-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="20" width="840" height="46" rx="10" class="f-bg s-line"/><text x="36" y="49" class="t-small">Terminal đang đứng ở: thư mục gốc repository (cwd)</text><rect x="20" y="100" width="380" height="70" rx="10" class="f-warn-soft s-warn"/><text x="36" y="128" class="t-b">open("data/trips.csv")</text><text x="36" y="154" class="t-small">tính từ cwd</text><path d="M400 135 L450 135" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py09-0-warn)"/><rect x="450" y="100" width="410" height="70" rx="10" class="f-warn-soft s-warn dash"/><text x="466" y="128" class="t-b">./data/trips.csv</text><text x="466" y="154" class="t-small">không có ở đây: FileNotFoundError</text><rect x="20" y="190" width="380" height="70" rx="10" class="f-phys-soft s-phys"/><text x="36" y="218" class="t-b">Path(__file__).parent</text><text x="36" y="244" class="t-small">tính từ vị trí file nguồn</text><path d="M400 225 L450 225" class="f-none s-phys" stroke-width="2" marker-end="url(#py09-0-arrow)"/><rect x="450" y="190" width="410" height="70" rx="10" class="f-phys-soft s-phys"/><text x="466" y="218" class="t-b">09-…/data/trips.csv</text><text x="466" y="244" class="t-small">luôn đúng, chạy từ đâu cũng được</text></svg>
</div>
<figcaption>Hình 1. Cùng một lệnh chạy, hai cách viết đường dẫn cho hai kết quả. Đường dẫn tương đối phụ thuộc nơi bạn đứng; đường dẫn neo theo file nguồn thì không.</figcaption>
</figure>

## pathlib: đường dẫn là đối tượng, không phải chuỗi

Module `pathlib` biểu diễn đường dẫn bằng đối tượng `Path`; toán tử `/` ghép các thành phần mà không cần nhớ dấu phân cách của từng hệ điều hành:

```python
from pathlib import Path

# Đường dẫn neo theo vị trí file nguồn, không theo thư mục đang đứng (cwd).
BASE_DIR = Path(__file__).parent
DATA_DIR = BASE_DIR / "data"
OUT_DIR = BASE_DIR / "out"
```

`Path(__file__).parent` là thư mục chứa `main.py`. Từ `DATA_DIR`, ta dùng được những thao tác hay gặp:

- `p.exists()` kiểm tra tồn tại; `p.is_file()` và `p.is_dir()` phân biệt file với thư mục.
- `p.name`, `p.stem`, `p.suffix`, `p.parent`: với `Path("data/trips.csv")` lần lượt là `trips.csv`, `trips`, `.csv`, `data`. `p.with_suffix(".json")` đổi phần đuôi.
- `p.read_text(encoding="utf-8")` đọc cả file thành một chuỗi, chỉ hợp với file nhỏ vì nạp hết vào bộ nhớ; `p.write_text(...)` là chiều ghi.
- `p.mkdir(parents=True, exist_ok=True)` tạo thư mục kèm thư mục cha và không lỗi nếu đã có. Thiếu `exist_ok=True`, chạy lần thứ hai sẽ lỗi.

Một cái bẫy: nếu vế phải của `/` là đường dẫn tuyệt đối, nó **thay** vế trái chứ không nối thêm: `Path("a") / "/etc"` cho `/etc`. Vì vậy đừng ghép đường dẫn với chuỗi người dùng gửi lên mà không kiểm tra.

## Mở file: with, encoding và newline

`Path.open()` (và hàm `open()` quen thuộc) trả về một đối tượng file. File mở là tài nguyên của hệ điều hành: không đóng thì dữ liệu ghi có thể còn kẹt trong bộ đệm, và số file mở mỗi tiến trình có hạn. Cách đúng là **context manager**, câu lệnh `with`:

```python
with path.open(encoding="utf-8") as f:
    data = json.load(f)
# ra khỏi khối with: file đã đóng, kể cả khi bên trong có exception
```

Khối `with` gọi `__enter__` khi vào và `__exit__` khi ra, kể cả khi exception văng ra giữa chừng; với file, `__exit__` đẩy bộ đệm xuống và đóng file.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Vòng đời file: with open mở file, khối with đọc hoặc ghi, ra khỏi khối thì file được đóng; nếu có exception giữa chừng thì file vẫn đóng rồi exception tiếp tục lan lên."><defs><marker id="py09-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py09-1-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="40" width="240" height="90" rx="10" class="f-phys-soft s-phys"/><text x="36" y="76" class="t-b">1. Mở</text><text x="36" y="104" class="t-small">with path.open(...) as f</text><path d="M260 85 L310 85" class="f-none s-phys" stroke-width="2" marker-end="url(#py09-1-arrow)"/><rect x="310" y="40" width="260" height="90" rx="10" class="f-pk-soft s-pk"/><text x="326" y="76" class="t-b">2. Đọc / ghi</text><text x="326" y="104" class="t-small">f.read(), writer.writerow()</text><path d="M570 85 L620 85" class="f-none s-phys" stroke-width="2" marker-end="url(#py09-1-arrow)"/><rect x="620" y="40" width="240" height="90" rx="10" class="f-phys-soft s-phys"/><text x="636" y="76" class="t-b">3. Đóng</text><text x="636" y="104" class="t-small">__exit__ đóng và flush</text><rect x="310" y="170" width="260" height="60" rx="10" class="f-warn-soft s-warn"/><text x="326" y="206" class="t-small">Exception giữa chừng</text><path d="M440 130 L440 170" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py09-1-warn)"/><path d="M570 200 C 620 200, 740 190, 740 135" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py09-1-warn)"/></svg>
</div>
<figcaption>Hình 2. Vòng đời của một file mở bằng with. Đường đứt đoạn là nhánh lỗi: file vẫn được đóng trước khi exception lan lên nơi gọi.</figcaption>
</figure>

### encoding="utf-8": tiếng Việt cần nó

File văn bản là dãy byte; **encoding** quy định byte nào ứng với ký tự nào. Nếu không truyền `encoding`, Python dùng encoding mặc định của máy (phụ thuộc locale, có thể khác nhau giữa Windows và Linux). Theo PEP 686, UTF-8 chỉ trở thành mặc định từ Python 3.15; với 3.14, và để mã chạy giống nhau ở mọi nơi, hãy luôn ghi `encoding="utf-8"` khi mở file văn bản. Chữ "Đà Nẵng" có 7 ký tự nhưng 11 byte UTF-8; đọc sai encoding là ký tự vỡ ngay.

### newline: ai chuẩn hóa xuống dòng?

Windows kết thúc dòng bằng `\r\n`, Linux và macOS bằng `\n`. Khi mở ở chế độ văn bản với tham số mặc định, Python **chuẩn hóa** mọi kiểu xuống dòng thành `\n` lúc đọc. Riêng với module `csv`, tài liệu yêu cầu mở file bằng `newline=""` để chính `csv` xử lý xuống dòng, kể cả khi một ô có chứa xuống dòng bên trong dấu ngoặc kép. Bạn thấy sự khác biệt bằng chế độ `--demo` của lab (trích năm trong số mười một dòng output):

```text
cwd-relative exists: False
source-relative exists: True
chars/bytes: 7 11
read_text: 'TN-0001\nTN-0002\n'
newline='': 'TN-0001\r\nTN-0002\r\n'
```

Hai dòng đầu là Hình 1 bằng số liệu thật (chạy từ thư mục gốc repository). Dòng ba: tiếng Việt, 7 ký tự, 11 byte. Hai dòng cuối là cùng một file chứa `\r\n`: `read_text` trả về `\n`, `newline=""` giữ nguyên.

## JSON: file cấu hình và dữ liệu có cấu trúc

JSON hợp với dữ liệu lồng nhau. Đây là `data/cities.json` (rút gọn từ sáu còn hai thành phố):

```json
{
  "cities": [
    {"code": "SG", "name": "TP. Hồ Chí Minh", "center": {"lat": 10.7769, "lng": 106.7009}},
    {"code": "HN", "name": "Hà Nội", "center": {"lat": 21.0285, "lng": 105.8542}}
  ]
}
```

`json.load(f)` và `json.dump(obj, f)` làm việc với **file**; `json.loads(s)` và `json.dumps(obj)` làm việc với **chuỗi** (`s` là *string*). Đọc file như sau:

```python
def read_cities(path: Path) -> list[dict]:
    """Đọc cities.json. parse_float=Decimal để tọa độ không đi qua float."""
    with path.open(encoding="utf-8") as f:
        return json.load(f, parse_float=Decimal)["cities"]
```

`parse_float` được gọi với chuỗi của mỗi số thực trong JSON; mặc định nó là `float`, và ta đã biết `0.1 + 0.2` ra `0.30000000000000004`. Cộng hai số đọc từ JSON theo hai cách (kết quả `--demo`):

```text
float  : 0.30000000000000004
Decimal: 0.3
```

Chiều ngược lại cũng có điều đáng nói. JSON không có kiểu `Decimal`, và `json` từ chối đoán:

```text
TypeError: Object of type Decimal is not JSON serializable
```

Có hai lối: đổi sang chuỗi trước khi ghi, hoặc truyền `default=`, hàm mà `json` gọi cho mọi đối tượng nó không biết. Lab **ghi tiền và khoảng cách thành chuỗi**, vì `"14250"` giữ nguyên từng chữ số và người đọc ở ngôn ngữ khác tự quyết định cách parse:

```python
def _json_default(value: object) -> str:
    """json không biết Decimal/datetime: ta chọn cách ghi tường minh."""
    if isinstance(value, Decimal):
        return str(value)  # chuỗi giữ nguyên chữ số, không qua float
    if isinstance(value, datetime):
        return value.isoformat()
    raise TypeError(f"Không serialize được {type(value).__name__}")
```

Dòng `raise TypeError` cuối quan trọng: kiểu ta chưa nghĩ tới phải nổ rõ ràng, không được lặng lẽ ghi ra rác. Tiền là chuỗi hay số JSON là một **hợp đồng** giữa người ghi và người đọc, không phải chuẩn của JSON.

Khi ghi, `indent=2` làm file dễ đọc bằng mắt, còn `ensure_ascii=False` giữ nguyên tiếng Việt thay vì thoát thành `\u...`:

```text
ensure_ascii=True : "\u0110\u00e0 N\u1eb5ng"
ensure_ascii=False: "Đà Nẵng"
```

Cả hai đều là JSON hợp lệ, parse ra cùng một chuỗi. Nhưng `ensure_ascii=False` đòi hỏi file mở với `encoding="utf-8"`.

## CSV: DictReader, DictWriter và newline=""

Đây là `data/trips.csv`, 30 chuyến giả với thời điểm UTC (`Z` ở cuối); ba dòng đầu:

```text
trip_id,city,service,distance_km,status,requested_at
TN-0001,SG,food,1.9,completed,2026-11-02T00:23:00Z
TN-0002,HN,parcel,2.0,completed,2026-11-02T01:15:00Z
```

`csv.DictReader` đọc dòng đầu làm tên cột rồi trả mỗi dòng sau thành một `dict`. Điều cần nhớ: **mọi giá trị đều là `str`**, `"1.9"` chưa phải số. Ta tự đổi kiểu và gắn số dòng vào thông báo lỗi để người sửa dữ liệu biết chỗ sai:

```python
def read_trips(path: Path) -> list[dict]:
    """Đọc trips.csv thành danh sách dict; đổi kiểu từng cột cần dùng."""
    trips = []
    # newline="" để module csv tự xử lý xuống dòng (kể cả trong ô có dấu ngoặc kép).
    with path.open(encoding="utf-8", newline="") as f:
        reader = csv.DictReader(f)
        for row in reader:
            try:
                row["distance_km"] = Decimal(row["distance_km"])
            except InvalidOperation as exc:
                # reader.line_num là số dòng trong file: lỗi dữ liệu chỉ rõ chỗ sai.
                raise ValueError(f"{path.name}, dòng {reader.line_num}: "
                                 f"distance_km {row['distance_km']!r} không phải số") from exc
            # ... requested_at: datetime.fromisoformat, tương tự ...
            trips.append(row)
    return trips
```

Ta đổi thẳng `str` sang `Decimal`, không qua `float`. `datetime.fromisoformat` đọc được hậu tố `Z` và trả về `datetime` aware ở UTC, đúng quy ước lưu thời gian của loạt bài. Chiều ghi dùng `csv.DictWriter` với `writerow(dict)`:

```python
with path.open("w", encoding="utf-8", newline="") as f:
    writer = csv.DictWriter(f, fieldnames=fields)
    writer.writeheader()
    writer.writerow({"trip_id": trip["trip_id"], "fare_vnd": fare, ...})
```

`DictWriter` tự thêm dấu ngoặc kép khi một ô chứa dấu phẩy. Đó cũng là lý do không nên tự `split(",")`: ô `"2,5"` sẽ bị chẻ làm đôi.

## Ghi file an toàn: file tạm rồi replace

`json.dump` ghi từng mẩu vào file. Nếu chương trình chết giữa chừng (hết đĩa, bị kill, exception khi serialize một phần tử), `summary.json` trên đĩa chỉ là nửa JSON, và ai đọc đúng lúc đó sẽ gặp file hỏng.

Cách chuẩn: ghi toàn bộ vào **file tạm cùng thư mục**, đẩy xuống đĩa, rồi đổi tên đè lên file đích bằng `replace`. Người đọc thấy file cũ nguyên vẹn hoặc file mới nguyên vẹn, không bao giờ thấy ở giữa.

```python
def write_json_atomic(path: Path, data: object) -> None:
    """Ghi file tạm cùng thư mục rồi replace: người đọc không thấy file ghi dở."""
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = None
    try:
        with tempfile.NamedTemporaryFile(
            "w", encoding="utf-8", newline="\n", dir=path.parent,
            suffix=".tmp", delete=False,
        ) as tmp:
            tmp_path = Path(tmp.name)
            json.dump(data, tmp, ensure_ascii=False, indent=2, default=_json_default)
            tmp.write("\n")
            tmp.flush()
            os.fsync(tmp.fileno())  # đẩy dữ liệu xuống đĩa trước khi đổi tên
        tmp_path.replace(path)  # đổi tên nguyên tử trên cùng filesystem
    except BaseException:
        if tmp_path is not None:
            tmp_path.unlink(missing_ok=True)
        raise
```

Vài chi tiết đáng đọc kỹ. Tên file tạm do `tempfile` sinh nên hai tiến trình không đè lên nhau. `dir=path.parent` là bắt buộc: đổi tên chỉ nguyên tử khi nguồn và đích cùng filesystem (POSIX; Windows có hành vi riêng, xem tài liệu `os.replace`). `flush()` rồi `os.fsync()` đẩy dữ liệu xuống đĩa trước khi đổi tên. Khối `except` xóa file tạm rồi `raise` lại: lỗi vẫn lan lên, chỉ là không để rác. `--verify` thử đúng kịch bản này: ghi một đối tượng không serialize được, rồi xác nhận file cũ còn nguyên và không sót `.tmp`.

## Chạy chương trình và đọc kết quả

`calculate_fare` cùng công thức với bài 03, bỏ tham số `service` vì mọi dịch vụ cùng giá (miền 0 đến 100 km, phần vượt `max(0, km − 2)`, `quantize` với `ROUND_HALF_UP`). `summarize` chỉ tính tiền chuyến `completed`, đếm `cancelled`, còn trạng thái dở dang như `in_transit` chưa tính. `run` nối đọc, tính, ghi. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/09-file-json-csv-pathlib examples/toi-noi-python/09-file-json-csv-pathlib/main.py
```

```text
Đã đọc 30 chuyến, ghi out/summary.json
SG  TP. Hồ Chí Minh   8 xong, 0 hủy,  42.8 km,   219300 VND
HN  Hà Nội            6 xong, 1 hủy,  31.8 km,   161100 VND
DN  Đà Nẵng           2 xong, 1 hủy,   7.7 km,    40650 VND
HP  Hải Phòng         3 xong, 1 hủy,  23.2 km,   122400 VND
CT  Cần Thơ           4 xong, 0 hủy,  11.0 km,    65100 VND
NT  Nha Trang         1 xong, 1 hủy,   6.7 km,    33150 VND
Tổng doanh thu chuyến hoàn thành: 641700 VND
```

Tổng có 24 chuyến hoàn thành, 4 chuyến hủy và 2 chuyến còn `in_transit`, đủ 30. Thư mục `out/` do `mkdir(parents=True, exist_ok=True)` tạo khi chạy và nằm trong `.gitignore` của lab. Trong `out/summary.json`, tiếng Việt còn nguyên (nhờ `ensure_ascii=False`), tiền là chuỗi (nhờ `default=`, ví dụ `"revenue_vnd": "641700"`), thời điểm là ISO 8601 có offset (`"first_requested_at": "2026-11-02T00:23:00+00:00"`). Chạy `--verify` để so với các số trên; lệnh này dùng thư mục tạm nên không ghi gì vào repo:

```bash
uv run --project examples/toi-noi-python/09-file-json-csv-pathlib examples/toi-noi-python/09-file-json-csv-pathlib/main.py --verify
```

```text
OK 15/15
```

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `FileNotFoundError` dù file có thật | Đường dẫn tương đối đang tính từ cwd; neo bằng `Path(__file__).parent` |
| Chữ tiếng Việt vỡ, hoặc `UnicodeDecodeError` | Mở file có `encoding="utf-8"` chưa; file có đúng là UTF-8 không |
| CSV ghi ra có dòng trống xen kẽ trên Windows | Mở file ghi bằng `newline=""` |
| `TypeError: Object of type Decimal is not JSON serializable` | Truyền `default=` hoặc đổi sang chuỗi trước khi `dump` |
| `summary.json` cụt hoặc hỏng sau khi chương trình lỗi | Ghi trực tiếp vào file đích; chuyển sang file tạm rồi `replace` |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Ta đọc toàn bộ CSV vào một <code>list</code>. Với 730 triệu dòng chuyến mỗi năm, hay 6.250 dòng vị trí mỗi giây, cách này hết bộ nhớ; bài 06 (generator) cho cách đọc từng dòng. Ví dụ chưa kiểm định schema đầy đủ (mã thành phố lạ chỉ nổ <code>KeyError</code>, file rỗng làm <code>min()</code> lỗi), chưa khóa khi nhiều tiến trình cùng ghi, và <code>replace</code> chưa bảo đảm bền khi mất điện nếu không fsync cả thư mục chứa. Đường dẫn trong bài do ta viết nên tin cậy được; đường dẫn từ người dùng phải được kiểm tra để không thoát ra ngoài thư mục cho phép. File tạm của <code>tempfile</code> có quyền 0600, nên file đích sau <code>replace</code> cũng vậy. Phía C#, <a href="/articles/csharp-integration-test-json-filesystem">bài 15</a> cũng ghi trực tiếp bằng <code>WriteAllTextAsync</code> và nêu cùng giới hạn.</p>
</div>

**Bài tập:** (1) Thêm vào cuối `trips.csv` dòng `TN-0031,SG,ride,"2,5",completed,2026-11-02T18:00:00Z` (số km dùng dấu phẩy thập phân, đặt trong ngoặc kép). Chương trình báo gì, ở dòng mấy của file? Nên sửa ở đâu? (2) Vì sao file tạm của `write_json_atomic` phải nằm cùng thư mục với file đích?

<details><summary>Xem lời giải</summary><p>(1) <code>DictReader</code> đọc <code>"2,5"</code> thành chuỗi <code>2,5</code> (nhờ ngoặc kép, ô không bị chẻ), nhưng <code>Decimal("2,5")</code> không hợp lệ, nên <code>read_trips</code> ném <code>ValueError: trips.csv, dòng 32: distance_km '2,5' không phải số</code>. Dòng 32 vì dòng 1 là tiêu đề và có 30 dòng dữ liệu trước nó. Cách sửa gốc rễ là chuẩn hóa tại nguồn xuất file (dấu chấm thập phân); nếu hợp đồng dữ liệu cho phép dấu phẩy thì thêm <code>.replace(",", ".")</code> một cách tường minh trong <code>read_trips</code>, đừng để Python đoán. (2) <code>replace</code> là đổi tên, và đổi tên chỉ làm được khi nguồn và đích cùng filesystem. Thư mục tạm của hệ thống thường nằm trên phân vùng khác; khi đó <code>os.replace</code> báo lỗi (trên POSIX là <code>OSError</code> kiểu EXDEV) thay vì ghi được, và chương trình sẽ hỏng đúng lúc cần lưu kết quả.</p></details>

Bài trước: [bài 08](/articles/python-exception-debug-logging). Bài tiếp: [bài 10](/articles/python-pytest-kiem-thu-chinh-sach-cuoc).

## Tài liệu gốc

- [pathlib: Object-oriented filesystem paths](https://docs.python.org/3.14/library/pathlib.html).
- [json: JSON encoder and decoder](https://docs.python.org/3.14/library/json.html).
- [csv: CSV File Reading and Writing](https://docs.python.org/3.14/library/csv.html).
- [Tutorial: Reading and Writing Files](https://docs.python.org/3.14/tutorial/inputoutput.html#reading-and-writing-files).
- [PEP 686: Make UTF-8 mode default](https://peps.python.org/pep-0686/).
