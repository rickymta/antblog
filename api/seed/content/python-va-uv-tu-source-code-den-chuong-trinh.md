---
slug: python-va-uv-tu-source-code-den-chuong-trinh
title: "Python 3.14 và uv: từ source code đến chương trình tính cước"
summary: "Bài 01: Python và CPython là gì, từ file .py tới bytecode và máy ảo; cài Python 3.14 bằng uv, đọc pyproject.toml, chạy chương trình tính cước 5 km đầu tiên của Tới Nơi. Có bài tập và lab chạy được."
category: python
level: beginner
tags: [python, toi-noi, uv]
language: vi
---

Một khách đặt xe trên [Tới Nơi](/articles/du-an-mau-toi-noi), nhập quãng đường 5 km và muốn biết giá trước khi xác nhận. Ta bắt đầu loạt Python bằng đúng việc nhỏ đó: tính một báo giá trong terminal. Sau này cùng quy tắc ấy sẽ nằm trong module, có test, rồi trở thành một phần của service nhận event và trả kết quả qua mạng.

Đây là bài 01 của lộ trình **Python cơ bản qua Tới Nơi**. Trong Tới Nơi, các backend nghiệp vụ chính (đặt chuyến, đơn hàng, điều phối) do C#/.NET đảm nhiệm. Python giữ những phần khác: **OCR giấy tờ định danh**, **so khớp khuôn mặt** của hồ sơ tài xế, **chấm điểm gian lận** và **phân tích, dự đoán**. Loạt này đi từ cú pháp đầu tiên tới các service Python hoàn chỉnh. Bạn chỉ cần mở được terminal và sửa file văn bản; không cần biết Python trước.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Python là ngôn ngữ thông dịch, nên khi chạy <code>python main.py</code> mỗi dòng được đọc và thực thi trực tiếp từ văn bản gốc, không có bước biên dịch nào. Đúng hay sai?</p>
<details><summary>Xem đáp án</summary><p><b>Sai.</b> Trước khi thực thi, CPython biên dịch toàn bộ source thành <b>bytecode</b>, rồi máy ảo của nó chạy bytecode đó. Điểm khác C# là bước này diễn ra tự động mỗi lần chạy, bạn không gọi một lệnh build riêng.</p></details>
</div>

## Python là gì, CPython là gì?

**Python** là ngôn ngữ: cú pháp và quy tắc, được mô tả trong tài liệu ngôn ngữ. **CPython** là bản hiện thực chuẩn viết bằng C, và là thứ `python` trong terminal thường trỏ tới. Có bản hiện thực khác (PyPy, và nhiều bản khác), nhưng cả loạt này dùng CPython 3.14.

Ngôn ngữ này nổi tiếng với cú pháp ngắn: khối lệnh dùng **thụt lề** thay cho dấu ngoặc nhọn, kiểu dữ liệu gắn với giá trị chứ không với biến (bài 02), và thư viện chuẩn rộng. Với Tới Nơi, lý do thực tế hơn: hệ sinh thái xử lý ảnh, học máy và phân tích dữ liệu ở Python là nơi OCR, face matching và fraud scoring dễ tìm công cụ nhất. Ta chọn Python cho các service đó vì thư viện và công cụ, không vì Python nhanh hơn C#. Nhiều thư viện nặng tính toán thực chất chạy mã native bên dưới, còn Python điều phối.

Người đã đọc loạt C# có thể đối chiếu như sau.

| Khía cạnh | C# trên .NET | Python (CPython) |
| --- | --- | --- |
| Cách đến CPU | Build ra IL, runtime dùng JIT dịch sang mã máy khi cần | Source dịch sang bytecode, máy ảo thông dịch từng lệnh bytecode |
| Bước build | Có (`dotnet build`), lỗi kiểu bắt lúc build | Không có lệnh build riêng; lỗi cú pháp bắt khi nạp file, lỗi kiểu thường chỉ lộ lúc chạy |
| Kiểu dữ liệu | Khai báo, kiểm tra lúc biên dịch | Gắn với đối tượng lúc chạy; type hint là chú thích tùy chọn |
| Môi trường | SDK + runtime cài trên máy | Interpreter + virtual environment cho từng project |
| Công cụ | `dotnet` (SDK) | `uv` (loạt này chọn), cùng `pip`, `venv` và nhiều công cụ khác |

Bảng này nói về cách chạy mặc định. Cả hai hệ sinh thái có nhánh khác (Native AOT bên .NET; JIT thử nghiệm và các bản hiện thực khác bên Python), ta không dùng ở đây.

## Từ file .py tới CPU

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 180" role="img" aria-label="File nguồn .py được CPython biên dịch thành bytecode, máy ảo thông dịch bytecode, và CPU thực thi lệnh máy của máy ảo."><rect x="20" y="45" width="186" height="100" rx="10" class="f-phys-soft s-phys"/><text x="32" y="82" class="t-b">Source .py</text><text x="32" y="113" class="t-small">Văn bản bạn viết</text><text x="210" y="100" class="t-b">→</text><rect x="230" y="45" width="186" height="100" rx="10" class="f-phys-soft s-phys"/><text x="242" y="82" class="t-b">Bytecode</text><text x="242" y="113" class="t-small">Biên dịch tự động</text><text x="420" y="100" class="t-b">→</text><rect x="440" y="45" width="186" height="100" rx="10" class="f-pk-soft s-pk"/><text x="452" y="82" class="t-b">Máy ảo CPython</text><text x="452" y="113" class="t-small">Thông dịch từng lệnh</text><text x="630" y="100" class="t-b">→</text><rect x="650" y="45" width="186" height="100" rx="10" class="f-phys-soft s-phys"/><text x="662" y="82" class="t-b">CPU</text><text x="662" y="113" class="t-small">Chạy mã của máy ảo</text></svg>
</div>
<figcaption>Hình 1. Luồng chạy mặc định của CPython. Bytecode không phải mã máy: CPU chạy chính chương trình interpreter, và interpreter đọc bytecode của ta.</figcaption>
</figure>

Khi gõ `python main.py`, CPython đọc file, phân tích cú pháp, rồi **biên dịch thành bytecode**: danh sách lệnh đơn giản cho máy ảo của Python. Mỗi lệnh bytecode lại được vòng lặp chính của interpreter thực thi. Vì vậy câu «Python là ngôn ngữ thông dịch» đúng một nửa: có thông dịch, nhưng trên bytecode chứ không trên văn bản gốc.

Bạn có thể nhìn bytecode bằng module `dis` trong thư viện chuẩn. Đây là kết quả thật khi biên dịch một dòng tính tổng (chạy bằng `uv run --project examples/toi-noi-python/01-python-va-uv python -c "..."` với `dis.dis(compile('total = base + extra * price', 'demo.py', 'exec'))`):

```text
  0           RESUME                   0

  1           LOAD_NAME                0 (base)
              LOAD_NAME                1 (extra)
              LOAD_NAME                2 (price)
              BINARY_OP                5 (*)
              BINARY_OP                0 (+)
              STORE_NAME               3 (total)
              LOAD_CONST               0 (None)
              RETURN_VALUE
```

Máy ảo dùng một ngăn xếp: nạp `base`, `extra`, `price`, nhân hai số trên cùng, cộng, rồi gán vào `total`. Tên lệnh bytecode là chi tiết của CPython và thay đổi giữa các phiên bản; ta đọc để hiểu cơ chế, không phải để học thuộc.

**`__pycache__`** liên quan thế nào? Khi một file được `import` từ file khác (bài 07), CPython có thể lưu bytecode đã biên dịch thành `.pyc` trong thư mục `__pycache__`, lần sau nạp nhanh hơn nếu source không đổi. Với tên như `main.cpython-314.pyc`, phần `cpython-314` cho biết bytecode thuộc phiên bản nào. Đây chỉ là bộ nhớ đệm: xóa đi thì được sinh lại; ta không sửa file `.pyc` và không commit thư mục này vào Git. File script chạy trực tiếp (`main.py`) thì thường không được lưu `.pyc`; bạn thấy `__pycache__` chủ yếu khi có module được import.

## Cài Python 3.14 bằng uv

**uv** là công cụ quản lý project và phiên bản Python. Nó cài interpreter, tạo virtual environment, ghi lại phiên bản thư viện trong `uv.lock` và chạy lệnh trong môi trường đó. Loạt này chọn uv để mọi bài có cùng cách chạy và không đụng vào Python hệ thống. Cài uv theo [hướng dẫn chính thức](https://docs.astral.sh/uv/getting-started/installation/), rồi kiểm tra:

```bash
uv --version
uv python install 3.14
uv python find 3.14
```

Lệnh `uv --version` cho một dòng dạng `uv 0.12.11 (…)`; số phiên bản trên máy bạn có thể khác. Lệnh thứ hai tải Python 3.14 nếu máy chưa có; nếu đã có, uv báo `Python 3.14 is already installed`. Lệnh thứ ba in đường dẫn interpreter mà uv tìm thấy, thường nằm trong thư mục dữ liệu của uv chứ không phải `/usr/bin`. Đó là điều ta muốn: Python của hệ điều hành vẫn nguyên vẹn.

**Virtual environment** (venv) là một thư mục `.venv` gắn với project, chứa interpreter được chọn và các gói của riêng project đó. Hai project dùng hai phiên bản thư viện khác nhau sẽ không dẫm lên nhau. Với uv bạn hiếm khi phải kích hoạt venv thủ công vì `uv run` làm việc đó.

## Tạo project và đọc pyproject.toml

```bash
uv init fare-demo --app --no-package --python 3.14
cd fare-demo
```

`--app` tạo project dạng ứng dụng; `--no-package` nói rằng ta chưa đóng gói nó thành thư viện cài được (bài 07 và bài 20 quay lại). Ta ghi rõ hai cờ vì mặc định của uv có thể đổi giữa các phiên bản: với uv 0.12.11 mà bài này dùng, chạy `uv init` trơn tạo ra layout có thư mục `src/` và cấu hình build, không có `main.py`. Lệnh trên tạo ra các file sau (cùng một repo Git mới nếu thư mục chưa nằm trong repo nào):

```text
.gitignore  .python-version  README.md  main.py  pyproject.toml
```

`.python-version` ghi `3.14`. `main.py` là chương trình mẫu in một dòng chào. `pyproject.toml` mô tả project. File do `uv init` sinh ra có thêm `readme`, `description = "Add your description here"` và không có mục `[tool.uv]`. Dưới đây là `pyproject.toml` tối thiểu mà ta tự viết cho lab của bài này, không phải đầu ra của lệnh trên:

```toml
# examples/toi-noi-python/01-python-va-uv/pyproject.toml
[project]
name = "toi-noi-py-01-python-va-uv"
version = "0.1.0"
description = "Bài 01: chương trình tính cước Tới Nơi đầu tiên bằng Python 3.14 và uv"
requires-python = ">=3.14"
dependencies = []

[tool.uv]
package = false
```

Mục `[project]` là phần chuẩn của hệ sinh thái Python: tên, phiên bản, `requires-python` (interpreter nào được phép chạy project) và `dependencies` (danh sách gói bên thứ ba, đang rỗng vì ta chỉ dùng thư viện chuẩn). Mục `[tool.uv]` là cấu hình riêng của uv; `package = false` nghĩa là project này chỉ để chạy, không build thành gói. Lần đầu `uv run` còn tạo thư mục `.venv` và file `uv.lock`; ta commit `uv.lock` để mọi người có cùng phiên bản gói, còn `.venv` thì không.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 190" role="img" aria-label="Bốn vai trò: Python và CPython chạy mã; uv quản lý interpreter và môi trường; thư viện chuẩn đi kèm Python; gói bên thứ ba cài thêm bằng uv add."><rect x="20" y="40" width="192" height="116" rx="10" class="f-phys-soft s-phys"/><text x="32" y="76" class="t-b">Python / CPython</text><text x="32" y="106" class="t-small">Ngôn ngữ và máy ảo</text><text x="32" y="130" class="t-small">Chạy mã của ta</text><rect x="236" y="40" width="192" height="116" rx="10" class="f-pk-soft s-pk"/><text x="248" y="76" class="t-b">uv</text><text x="248" y="106" class="t-small">Công cụ ngoài Python</text><text x="248" y="130" class="t-small">Cài bản, venv, lock</text><rect x="452" y="40" width="192" height="116" rx="10" class="f-phys-soft s-phys"/><text x="464" y="76" class="t-b">Thư viện chuẩn</text><text x="464" y="106" class="t-small">Đi kèm Python</text><text x="464" y="130" class="t-small">decimal, sys, json</text><rect x="668" y="40" width="192" height="116" rx="10" class="f-warn-soft s-warn"/><text x="680" y="76" class="t-b">Gói bên thứ ba</text><text x="680" y="106" class="t-small">Cài thêm bằng uv add</text><text x="680" y="130" class="t-small">Có phiên bản, có rủi ro</text></svg>
</div>
<figcaption>Hình 2. Bốn vai trò dễ bị gộp thành một. Bài này chỉ dùng ba ô đầu; ô cuối xuất hiện từ bài 07, khi project bắt đầu khai báo dependency.</figcaption>
</figure>

## Chương trình tính cước đầu tiên

Quy tắc minh họa: 12.000 VND bao gồm 2 km đầu, mỗi km vượt thêm 4.500 VND. Chưa có phụ phí, thuế, giảm giá hoặc làm tròn. Đây là giả định để học, không phải bảng giá thực tế của sản phẩm. Đây là phần chính của `main.py` trong lab:

```python
# examples/toi-noi-python/01-python-va-uv/main.py (rút gọn)
import sys
from decimal import Decimal

# Quy tắc cước: 12.000 VND gồm 2 km đầu, mỗi km vượt thêm 4.500 VND.
BASE_FARE = Decimal("12000")
INCLUDED_KM = Decimal("2")
PRICE_PER_EXTRA_KM = Decimal("4500")


def calculate_fare(distance_km: Decimal) -> Decimal:
    extra_km = max(Decimal("0"), distance_km - INCLUDED_KM)
    return BASE_FARE + extra_km * PRICE_PER_EXTRA_KM


def main() -> None:
    # Mặc định 5 km; có thể truyền khoảng cách khác làm đối số đầu tiên.
    distance_km = Decimal(sys.argv[1]) if len(sys.argv) > 1 else Decimal("5")
    total = calculate_fare(distance_km)
    print(f"Quãng đường: {distance_km} km")
    print(f"Cước minh họa: {total} VND")
```

Ta dùng `Decimal` ngay từ đầu vì đây là tiền và khoảng cách; cách nó hoạt động, vì sao `Decimal("5")` truyền bằng chuỗi, sẽ giải thích ở bài 02. Hiện tại chỉ cần biết: `Decimal` là kiểu số thập phân của thư viện chuẩn (`from decimal import Decimal`), và `max(a, b)` trả về số lớn hơn. Đi 1 km thì phần vượt bằng 0, không bị trừ ngược khỏi giá cơ bản. Hàm `def` và khối thụt lề sẽ được học kỹ ở bài 03.

Chạy từ gốc repository:

```bash
uv run --project examples/toi-noi-python/01-python-va-uv examples/toi-noi-python/01-python-va-uv/main.py
```

```text
Quãng đường: 5 km
Cước minh họa: 25500 VND
```

`uv run` kiểm tra interpreter và môi trường của project, tạo nếu thiếu, rồi chạy file bằng Python của project. Với 5 km, phần vượt là 3 km, nên tổng là `12000 + 3 × 4500 = 25500`. Dòng `if __name__ == "__main__":` ở cuối file (bài 07 giải thích) cho phép chạy `main()` khi chạy trực tiếp, và cho `--verify` kiểm tra lại các con số trong bài.

## Chạy và kiểm chứng: chạy được vẫn có thể sai

Truyền khoảng cách âm:

```bash
uv run --project examples/toi-noi-python/01-python-va-uv examples/toi-noi-python/01-python-va-uv/main.py -1
```

```text
Quãng đường: -1 km
Cước minh họa: 12000 VND
```

Chương trình chạy trơn tru và in ra một báo giá cho quãng đường không thể có. Interpreter không biết khoảng cách âm là vô nghĩa với nghiệp vụ; công thức chặn phần vượt âm bằng `max` không phải validation đầu vào. Truyền `abc` thì nhận traceback với `InvalidOperation`: chuỗi không phải số bị `Decimal` từ chối (`ConversionSyntax`), nhưng thông báo chưa thân thiện. Bài 02 và bài 08 sẽ xử lý các trường hợp này.

Giới hạn của ví dụ: khoảng cách đến từ dòng lệnh chứ không phải từ khách hàng thật; chưa từ chối số âm; chưa có tiền tệ, làm tròn, thuế hay phụ phí; chưa có test tự động. Đừng đặt đoạn code này sau một endpoint công khai khi chưa bổ sung các điều kiện đó.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `uv: command not found` | Cài uv chưa; mở terminal mới sau khi cài |
| Báo không tìm thấy Python 3.14 | Chạy `uv python install 3.14`; kiểm tra `requires-python` trong `pyproject.toml` |
| `python` ra phiên bản khác với bài | Đang gọi Python hệ thống; dùng `uv run` thay vì gọi `python` trực tiếp |
| `No such file or directory` khi chạy | Đang đứng sai thư mục; lệnh trong bài chạy từ gốc repository |
| Sửa file nhưng kết quả không đổi | Có chạy đúng file không; thử xóa `__pycache__` nếu có module được import |

### Tự kiểm tra bằng tay

Mỗi lần chạy với một khoảng cách khác, so với bảng. Dòng `Cước minh họa` phải khớp:

| Khoảng cách | Phần vượt | Giá mong đợi |
| --- | --- | --- |
| 0 km | 0 km | 12000 VND theo giả định hiện tại |
| 2 km | 0 km | 12000 VND |
| 2,5 km | 0,5 km | 14250.0 VND |
| 5 km | 3 km | 25500 VND |

Với 2,5 km chương trình in `14250.0` chứ không phải `14250`: `Decimal` nhớ số chữ số thập phân của số đã nhập (`2.5` có một chữ số lẻ) và giữ lại trong kết quả. Giá trị vẫn bằng 14.250; việc chốt cách làm tròn và hiển thị tiền là nội dung bài 02. Riêng 0 km vẫn tính giá cơ bản là lựa chọn của mô hình này, không phải quy luật bắt buộc.

Lab có sẵn chế độ kiểm lại các con số này:

```bash
uv run --project examples/toi-noi-python/01-python-va-uv examples/toi-noi-python/01-python-va-uv/main.py --verify
```

```text
OK 6/6
```

**Bài tập:** (1) Đổi giá mỗi km vượt thành 5.000 VND. Khách đi 4 km phải trả bao nhiêu? (2) Trong thư mục `fare-demo` vừa tạo bằng `uv init`, chạy `ls -a`, rồi chạy `uv run main.py`, rồi `ls -a` lần nữa: uv đã tạo thêm những gì?

<details><summary>Xem lời giải</summary><p>1. Phần vượt là 2 km, tổng bằng 12.000 + 2 × 5.000 = <b>22.000 VND</b>. Trong lab, đổi <code>PRICE_PER_EXTRA_KM</code> thành <code>Decimal("5000")</code> rồi chạy với đối số <code>4</code>; nhớ đặt lại 4.500 để <code>--verify</code> tiếp tục pass. 2. Sau lần <code>uv run</code> đầu, thư mục có thêm <code>.venv</code> và <code>uv.lock</code>. Ta commit <code>uv.lock</code> để mọi người có cùng phiên bản gói; <code>.venv</code> được bỏ qua trong <code>.gitignore</code>.</p></details>

## Ba cấp độ của loạt bài

Lộ trình chia làm ba cấp, mỗi cấp có đích rõ:

- **Cơ bản (bài 01 tới 20):** viết, kiểm thử và đóng gói một chương trình Python có kiểu, có mô hình nghiệp vụ; bài này là bài đầu.
- **Trung bình (bài 21 tới 40):** viết service HTTP, gRPC, Kafka bất đồng bộ, nối PostgreSQL, MongoDB, Redis, có integration test.
- **Nâng cao (bài 41 tới 79):** xây OCR, face matching, fraud scoring và dự đoán thành service theo kiến trúc đích, có đo đạc và vận hành.

Ảnh giấy tờ, khuôn mặt ở cấp nâng cao là dữ liệu tổng hợp sinh bằng code; kết quả OCR và điểm fraud là đầu vào cho quy trình nghiệp vụ, không tự động là quyết định xác minh hay kết luận gian lận. Với các backend chính, Tới Nơi hướng tới DDD, Clean Architecture, CQRS, Mediator, Repository và Unit of Work; Python giữ các service OCR, so khớp mặt, fraud và phân tích bên cạnh đó. Xem [hồ sơ Tới Nơi](/articles/du-an-mau-toi-noi) cho bức tranh đầy đủ.

Bài trước: không có. Bài tiếp: [bài 02](/articles/python-bien-kieu-du-lieu-decimal), đi sâu vào biến, kiểu dữ liệu, `float` so với `Decimal` và quy tắc làm tròn tiền.

Code bài này nằm trong repository tại `examples/toi-noi-python/01-python-va-uv`; khung biên tập của cả loạt nằm trong `examples/toi-noi-python/README.md`.

## Tài liệu gốc

- [Python Tutorial: dùng Python interpreter](https://docs.python.org/3.14/tutorial/interpreter.html).
- [Module dis: phân tích bytecode](https://docs.python.org/3.14/library/dis.html).
- [Module decimal](https://docs.python.org/3.14/library/decimal.html).
- [Tài liệu uv: dùng project](https://docs.astral.sh/uv/guides/projects/).
- [Tài liệu uv: cài Python](https://docs.astral.sh/uv/guides/install-python/).
