---
slug: python-dong-goi-ruff-pre-commit-cau-truc-du-an
title: "Đóng gói, ruff, mypy và cấu trúc dự án: sẵn sàng cho service"
summary: "Bài 20: gom P01 và P02 thành gói toinoi_pricing có src layout, entry point, ruff, mypy strict, pytest và pre-commit. Chạy từng công cụ, xem nó bắt gì và không bắt gì, rồi build wheel thật."
category: python-beginner
level: beginner
tags: [python, toi-noi, ruff, mypy, packaging]
language: vi
---

Sau mười chín bài, mã tính cước của [Tới Nơi](/articles/du-an-mau-toi-noi) nằm rải ở nhiều thư mục lab. Mỗi lab chạy được, nhưng chưa thứ nào là một thứ mà đồng nghiệp có thể cài vào môi trường của họ, gõ một lệnh để biết «có hỏng gì không», rồi mở pull request mà không bị nhắc ba lần vì import thừa. Sắp tới nhóm bắt đầu viết service đầu tiên bằng Python, và nếu móng không chắc thì mọi bài sau sẽ dựng trên cát. Bài này gom lại những gì đã học thành một gói hoàn chỉnh, rồi thêm lớp công cụ giữ nó sạch.

Đây là bài 20 của lộ trình **Python cơ bản qua Tới Nơi**, bài tổng hợp của phần 01 và phần 02. Bạn cần quen `src` layout và import (bài 07), pytest (bài 10) và type hints (bài 12). Lab là gói `toinoi_pricing`, dùng lại đúng mô hình cước đã quen: 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, không phụ phí.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Nếu <code>ruff check</code>, <code>ruff format --check</code> và <code>mypy --strict</code> đều xanh thì có thể bỏ pytest được không? Thử nghĩ: ai đó đổi <code>ROUND_HALF_UP</code> thành <code>ROUND_HALF_EVEN</code> trong hàm tính cước, ba công cụ kia có báo gì không?</p>
<details><summary>Xem đáp án</summary><p><b>Không bỏ được, và không công cụ tĩnh nào báo.</b> Cả hai chế độ làm tròn đều là hằng số hợp lệ của <code>decimal</code>, dòng code đúng kiểu và đúng định dạng. Bài chạy đúng thay đổi này trên bản sao: ba công cụ tĩnh vẫn xanh, còn pytest đỏ <code>3 failed, 57 passed</code>. Mỗi công cụ trả lời một câu hỏi khác nhau, và hành vi nghiệp vụ chỉ có test mới trả lời.</p></details>
</div>

## Từ các bài trước tới một gói

Gói `toinoi_pricing` không thêm luật nghiệp vụ nào: giá, miền 0 đến 100 km, tám trạng thái chuyến đều là của các bài trước. Nó chép lại `Money` và `Quote` cùng toàn bộ phần kiểm tra đầu vào từ lab 13 (kiểu `Decimal`, số hữu hạn, không âm, VND không có đồng lẻ, thời điểm phải aware) và sắp xếp mọi thứ theo một cấu trúc mà service có thể bám vào. Bảng dưới liệt kê các bài góp trực tiếp vào lab này, không phải mọi bài từ 01 đến 19.

| Bài | Kiến thức | Chỗ nó xuất hiện trong lab |
| --- | --- | --- |
| 02 | `Decimal`, `ROUND_HALF_UP` | `calculate_fare` làm tròn cuối về 1 VND |
| 03 | Hàm, tham số keyword-only | `calculate_fare(distance_km, policy, *, service)` |
| 04 | `datetime` aware, UTC | `Quote.expires_at` phải aware |
| 07 | Package, `src` layout | `src/toinoi_pricing/`, `[build-system]` |
| 08 | Exception | `ValueError` có thông báo; CLI đổi thành mã thoát 2 |
| 10 | pytest, `parametrize`; `FarePolicy` | `tests/`, bảng ca biên 8 × 3; `FarePolicy` (`NamedTuple`) giữ nguyên từ bài 10 |
| 11 | Máy trạng thái chuyến bằng `dict` | `ALLOWED_TRANSITIONS` |
| 12 | Type hints, mypy | `distance_km: Decimal`, `mypy --strict` |
| 13 | `frozen` dataclass, `StrEnum`, `Money`, `Quote` | `Money`, `Quote`, `TripStatus` (khóa của `ALLOWED_TRANSITIONS` là `StrEnum`); `make_quote(now=...)` nhận thời điểm từ ngoài vào |

Bài 14 đi tiếp một bước: thay tham số `now` bằng một `Clock` được tiêm vào.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 410" role="img" aria-label="Cây thư mục của lab 20: pyproject.toml, uv.lock, README.md, .pre-commit-config.yaml ở gốc; src/toinoi_pricing có __init__.py, models.py, pricing.py, cli.py, __main__.py, py.typed; thư mục tests và demo-errors. Ba ô chú thích: pyproject.toml là nơi cấu hình duy nhất, gói chia ba tầng, kiểm chứng gồm tests và demo-errors."><rect x="20" y="15" width="420" height="390" rx="10" class="f-bg s-line"/><text x="36" y="45" class="t-mono t-b">20-dong-goi-ruff-cau-truc-du-an/</text><text x="36" y="69" class="t-mono">├── pyproject.toml</text><text x="36" y="93" class="t-mono">├── uv.lock</text><text x="36" y="117" class="t-mono">├── README.md</text><text x="36" y="141" class="t-mono">├── .pre-commit-config.yaml</text><text x="36" y="165" class="t-mono">├── src/toinoi_pricing/</text><text x="36" y="189" class="t-mono">│   ├── __init__.py</text><text x="36" y="213" class="t-mono">│   ├── models.py</text><text x="36" y="237" class="t-mono">│   ├── pricing.py</text><text x="36" y="261" class="t-mono">│   ├── cli.py</text><text x="36" y="285" class="t-mono">│   ├── __main__.py</text><text x="36" y="309" class="t-mono">│   └── py.typed</text><text x="36" y="333" class="t-mono">├── tests/</text><text x="36" y="357" class="t-mono">├── demo-errors/</text><text x="36" y="381" class="t-mono">└── verify.sh</text><rect x="480" y="30" width="380" height="100" rx="10" class="f-phys-soft s-phys"/><text x="496" y="60" class="t-b">pyproject.toml: một nơi cho mọi công cụ</text><text x="496" y="86" class="t-small">metadata, entry point, nhóm dev, ruff, mypy, pytest</text><text x="496" y="106" class="t-small">uv.lock ghim đúng phiên bản đã chạy</text><rect x="480" y="150" width="380" height="100" rx="10" class="f-pk-soft s-pk"/><text x="496" y="180" class="t-b">Gói chia ba tầng</text><text x="496" y="206" class="t-small">models.py: kiểu dữ liệu (Money, Quote, TripStatus)</text><text x="496" y="226" class="t-small">pricing.py: quy tắc tính cước; cli.py: vỏ mỏng</text><rect x="480" y="270" width="380" height="100" rx="10" class="f-warn-soft s-warn"/><text x="496" y="300" class="t-b">Kiểm chứng</text><text x="496" y="326" class="t-small">tests/ chạy bằng pytest</text><text x="496" y="346" class="t-small">demo-errors/ cố ý sai để thấy công cụ bắt gì</text></svg>
</div>
<figcaption>Hình 1. Cây thư mục của lab. Mã nằm dưới <code>src/</code>; <code>tests/</code> và <code>demo-errors/</code> nằm ngoài gói nên không vào wheel (README.md vào sdist vì <code>[project]</code> khai <code>readme</code>).</figcaption>
</figure>

Việc tách ba tầng có chủ đích. `models.py` chứa kiểu dữ liệu cùng phần tự kiểm tra của chúng, `pricing.py` chứa quy tắc tính cước, `cli.py` là vỏ mỏng: đọc đối số, gọi quy tắc, đổi lỗi thành mã thoát. Khi sau này có thêm vỏ HTTP, nó sẽ ngồi chỗ của `cli.py` và gọi cùng `pricing.py`, không phải sao chép luật tính tiền. Quy tắc nằm một chỗ để test một chỗ.

## pyproject.toml đầy đủ

`pyproject.toml` là nơi duy nhất mô tả dự án cho mọi công cụ. Phần định nghĩa gói (trích):

```toml
# pyproject.toml (trích)
[project]
name = "toi-noi-py-20-dong-goi-ruff-cau-truc-du-an"
version = "0.1.0"
readme = "README.md"
requires-python = ">=3.14"
dependencies = []

# Lệnh `toinoi-fare` được tạo khi cài gói: trỏ tới hàm main trong module cli.
[project.scripts]
toinoi-fare = "toinoi_pricing.cli:main"

[dependency-groups]
dev = [
    "mypy>=2.4",
    "pytest>=9",
    "ruff>=0.17",
]

[build-system]
requires = ["uv_build>=0.12.11,<0.13.0"]
build-backend = "uv_build"

[tool.uv]
package = true

# Tên project (toi-noi-py-20-...) khác tên package import nên khai báo tường minh.
[tool.uv.build-backend]
module-name = "toinoi_pricing"
```

Mỗi mục có một việc. `[project]` là metadata chuẩn (PEP 621): tên, phiên bản, phiên bản Python tối thiểu và danh sách phụ thuộc lúc chạy. Gói này chỉ dùng thư viện chuẩn nên `dependencies` rỗng. Khai `readme = "README.md"` thì file đó **phải tồn tại**; thiếu nó, `uv run` dừng ngay ở bước build (trích, rút gọn đường dẫn):

```text
× Failed to build `toi-noi-py-20-dong-goi-ruff-cau-truc-du-an @ file:///…/20-dong-goi-ruff-cau-truc-du-an`
╰─▶ failed to open file `…/20-dong-goi-ruff-cau-truc-du-an/README.md`: No such file or directory (os error 2)
```

`[project.scripts]` là entry point: khi cài gói, công cụ cài đặt tạo lệnh `toinoi-fare` gọi hàm `main` trong `toinoi_pricing.cli`. Ta không tự viết file shell hay dòng `#!` nào. `[dependency-groups]` (PEP 735) chứa công cụ **chỉ dành cho người phát triển**: ruff, mypy, pytest. Chúng không đi vào wheel; `METADATA` của wheel không có dòng `Requires-Dist` nào, còn `uv run` mặc định cài nhóm `dev`. Cuối cùng `[build-system]` cho biết dùng `uv_build` để dựng gói và `module-name` nối tên project (có gạch) với tên package import (`toinoi_pricing`), như bài 07 đã giải thích.

`uv.lock` ghim phiên bản chính xác của toàn bộ cây phụ thuộc: ở lần chạy này mypy 2.4.1, pytest 9.2.0, ruff 0.17.4 trên Python 3.14.7. Hãy commit nó. `uv lock --check` kiểm lock còn khớp `pyproject.toml` không, và `uv sync --locked` thất bại thay vì âm thầm cập nhật lock khi chúng lệch nhau, đúng điều ta muốn ở CI:

```text
$ uv lock --check
Resolved 14 packages in 6ms
$ uv sync --locked
Resolved 14 packages in 7ms
Checked 13 packages in 0.43ms
```

## src layout, entry point và wheel thật

Đóng gói chỉ có ý nghĩa nếu ta nhìn được thứ ra khỏi máy. `uv build` dựng cả sdist và wheel; ta xem bên trong wheel (trích):

```bash
L=examples/toi-noi-python/20-dong-goi-ruff-cau-truc-du-an
uv build --project $L --out-dir $L/out
```

```text
Successfully built …/out/toi_noi_py_20_dong_goi_ruff_cau_truc_du_an-0.1.0.tar.gz
Successfully built …/out/toi_noi_py_20_dong_goi_ruff_cau_truc_du_an-0.1.0-py3-none-any.whl
```

```text
toinoi_pricing/__init__.py
toinoi_pricing/__main__.py
toinoi_pricing/cli.py
toinoi_pricing/models.py
toinoi_pricing/pricing.py
toinoi_pricing/py.typed
…dist-info/entry_points.txt
…dist-info/METADATA
```

(danh sách rút gọn từ `unzip -l`, tổng cộng 12 mục kể cả thư mục và `RECORD`, `WHEEL`.) Wheel chỉ chứa gói `toinoi_pricing`; `tests/` và `demo-errors/` nằm ngoài `src/` nên không vào. `entry_points.txt` trong wheel ghi đúng dòng ta khai:

```text
[console_scripts]
toinoi-fare = toinoi_pricing.cli:main
```

Để chắc chắn wheel dùng được ngoài thư mục dự án, cài nó vào một môi trường cô lập rồi chạy lệnh :

```bash
uv run --no-project --isolated --with $L/out/*.whl toinoi-fare 2.5
```

```text
2.5 km => 14250 VND
```

Tệp `py.typed` là một file rỗng, theo PEP 561: nó nói với mypy rằng «gói này có kiểu, hãy tin các chú thích của nó». Thiếu nó thì người dùng gói không được kiểm kiểu. Ta thử trên hai bản wheel, có và không có `py.typed`, bằng `demo-errors/consumer.py`: một file «khách» gọi `calculate_fare` rồi gán kết quả cho biến kiểu `str` (README của lab ghi từng lệnh tái lập). Với `py.typed`, mypy thấy lỗi thật trong code của khách:

```text
demo-errors/consumer.py:7: error: Incompatible types in assignment (expression has type "Decimal", variable has type "str")  [assignment]
```

Bỏ `py.typed`, mypy từ chối phân tích cả gói và không kiểm gì:

```text
demo-errors/consumer.py:5: error: Skipping analyzing "toinoi_pricing": module is installed, but missing library stubs or py.typed marker  [import-untyped]
```

Còn `__main__.py` cho phép `python -m toinoi_pricing 5 --service parcel`, cùng một hàm `main` với entry point, nên hai cách chạy không thể lệch nhau.

## ruff: format và lint là hai việc

**ruff** là một công cụ, hai lệnh. `ruff format` (chạy trước) chỉ đổi **hình thức** (khoảng trắng, dấu phẩy, ngắt dòng) mà không đổi nghĩa của chương trình. `ruff check` đọc nghĩa của mã để tìm lỗi và mùi code. Cấu hình trong `pyproject.toml`:

```toml
[tool.ruff]
line-length = 100
target-version = "py314"
src = ["src", "tests"]
# demo-errors/ chứa file cố ý sai; chỉ chạy khi ta chỉ đích danh từng file.
extend-exclude = ["demo-errors"]

[tool.ruff.lint]
# E/W: pycodestyle, F: pyflakes, I: sắp xếp import, UP: cú pháp Python hiện đại,
# B: bẫy phổ biến (flake8-bugbear), SIM: đơn giản hóa, RUF: quy tắc riêng của ruff,
# PT: pytest, DTZ: datetime thiếu múi giờ, S: bảo mật (bandit).
select = ["E", "W", "F", "I", "UP", "B", "SIM", "RUF", "PT", "DTZ", "S"]

[tool.ruff.lint.per-file-ignores]
# Test dùng assert là đúng chỗ; S101 chỉ đáng cảnh báo trong mã chạy thật.
"tests/*" = ["S101"]
```

Ta không chọn «mọi quy tắc». Mỗi nhóm chữ cái là một họ quy tắc, và bật hết thì lần đầu chạy sẽ ra hàng trăm cảnh báo mà không ai đọc. Bắt đầu từ vài họ có lợi rõ, rồi thêm khi cả nhóm đồng ý. Hai họ ở đây gắn với quy ước của Tới Nơi: `DTZ` cảnh báo `datetime` không có múi giờ (đúng điều bài 04 dặn), và `S` cảnh báo mẫu code nguy hiểm. Ta tắt `S101` (dùng `assert`) riêng cho `tests/`, vì assert trong test là việc đúng.

Để xem ruff bắt gì, lab có file `demo-errors/bad_lint.py` cố ý sai (không thuộc gói, bị loại khỏi lệnh kiểm chung):

```python
# demo-errors/bad_lint.py
"""File cố ý sai để xem ruff check bắt gì. Bị loại khỏi lệnh kiểm chung (extend-exclude)."""

import sys
from decimal import Decimal
import os


def add_stop(stop, stops=[]):
    stops.append(stop)
    return stops


def fare(km: Decimal):
    if km == None:
        return 0
    return km * 4500
```

```bash
uv run --project examples/toi-noi-python/20-dong-goi-ruff-cau-truc-du-an ruff check examples/toi-noi-python/20-dong-goi-ruff-cau-truc-du-an/demo-errors/bad_lint.py --output-format concise
```

```text
…/demo-errors/bad_lint.py:3:1: I001 [*] Import block is un-sorted or un-formatted
…/demo-errors/bad_lint.py:3:8: F401 [*] `sys` imported but unused
…/demo-errors/bad_lint.py:5:8: F401 [*] `os` imported but unused
…/demo-errors/bad_lint.py:8:26: B006 Do not use mutable data structures for argument defaults
…/demo-errors/bad_lint.py:14:14: E711 Comparison to `None` should be `cond is None`
Found 5 errors.
[*] 3 fixable with `--fix` (2 unsafe fixes available with `--fix --unsafe-fixes`).
```

Năm lỗi, mỗi lỗi một mã. `B006` là cái bẫy kinh điển: giá trị mặc định `[]` được tạo **một lần** lúc định nghĩa hàm và dùng chung cho mọi lần gọi, nên danh sách `stops` tích dần giữa các lần gọi. Dấu `[*]` nghĩa là ruff sửa được tự động bằng `--fix`. Còn «unsafe fix» là bản sửa có thể đổi hành vi: thay `== None` bằng `is None` đổi kết quả nếu lớp có `__eq__` tùy biến, nên ruff không tự làm khi chưa được phép. Với những bản sửa an toàn, `ruff check --fix` dọn được import thừa và thứ tự import; phần còn lại cần người quyết định.

`ruff format` thì không báo «lỗi», nó chỉ cho thấy khác biệt. File `demo-errors/bad_format.py` viết cẩu thả, `ruff format --diff` in thứ nó sẽ đổi:

```text
-def total(a:Decimal,b:Decimal)->Decimal:
-    return   a+b
+
+def total(a: Decimal, b: Decimal) -> Decimal:
+    return a + b
```

(trích, bỏ phần tiêu đề). Con số `9 files already formatted` ở cuối bài là 8 file `.py` và `README.md`: ruff 0.17 định dạng cả đoạn mã Python trong Markdown. Trong CI, ta dùng `ruff format --check`: không sửa file, chỉ thoát mã khác 0 khi có file cần định dạng lại.

## mypy strict: kiểu là hợp đồng kiểm được

Bài 10 chặn đầu vào sai kiểu bằng `isinstance` lúc chạy và kiểm bằng ca `pytest.raises(TypeError)`. Ở gói này `calculate_fare` chỉ khai `distance_km: Decimal`, không còn nhánh kiểm kiểu nào: việc ngăn truyền `float` được giao cho mypy, **trước khi chạy**. Cấu hình ngắn:

```toml
[tool.mypy]
strict = true
python_version = "3.14"
mypy_path = "$MYPY_CONFIG_FILE_DIR/src"
exclude = ["demo-errors/"]
```

`strict = true` bật một bộ cờ gồm cấm hàm thiếu annotation, cấm generic thiếu tham số kiểu, cảnh báo `# type: ignore` thừa, và kiểm so sánh vô nghĩa (`strict_equality`). File `demo-errors/bad_types.py` có ba lỗi kiểu điển hình:

```python
# demo-errors/bad_types.py
from decimal import Decimal

from toinoi_pricing import calculate_fare


def quote_text() -> str:
    fare = calculate_fare(2.5)
    return fare


def first_digit(text: str | None) -> str:
    return text[0]
```

```text
…/demo-errors/bad_types.py:7: error: Argument 1 to "calculate_fare" has incompatible type "float"; expected "Decimal"  [arg-type]
…/demo-errors/bad_types.py:8: error: Incompatible return value type (got "Decimal", expected "str")  [return-value]
…/demo-errors/bad_types.py:12: error: Value of type "str | None" is not indexable  [index]
Found 3 errors in 1 file (checked 1 source file)
```

Lỗi đầu là chính cái bẫy `float` của bài 02. Lỗi cuối là kiểu «quên xử lý `None`» (bài 16 đi sâu vào). Có ba điều đáng biết về cách dùng. Thứ nhất, mypy cũng bắt lỗi trong test: khi viết `test_models.py`, dòng `assert TripStatus.PICKED_UP == "picked_up"` bị báo `Non-overlapping equality check`, vì mypy coi hai literal đó không bao giờ bằng nhau dù lúc chạy `StrEnum` so sánh bằng chuỗi. Ta đổi sang `TripStatus.PICKED_UP.value == "picked_up"`. Thứ hai, với `Money(2.5)` trong test, mypy báo `arg-type` nên dòng test phải ghi `# type: ignore[arg-type]`; ngược lại, khi ta ghi nhầm ignore cho `Money(D("1")) + 1` (vì `__add__` nhận `object`, không có lỗi), mypy phản ứng bằng `Unused "type: ignore" comment  [unused-ignore]`. Thứ ba, khi buộc phải bỏ qua một lỗi, hãy ghi mã cụ thể (`# type: ignore[misc]` ở test gán vào dataclass frozen) chứ không để `# type: ignore` trần; cờ `warn_unused_ignores` của strict sẽ báo ngay khi dòng đó không còn cần.

`Money` vẫn giữ `isinstance(self.amount, Decimal)` chép từ lab 13: giá trị đi vào từ JSON hay database không qua mypy, nên ranh giới dữ liệu ngoài vẫn cần kiểm lúc chạy. Hàm `calculate_fare` thì chỉ khai kiểu, vì mọi lời gọi trong dự án đều qua mypy. mypy chỉ kiểm điều ta đã annotate và chỉ kiểm tại ranh giới giữa các kiểu. `Decimal` đúng kiểu vẫn có thể là `Decimal("NaN")`, nên `calculate_fare` vẫn kiểm `is_finite()` và miền 0 đến 100 lúc chạy, và test vẫn có ca `NaN`, `Infinity`. Kiểu và test bổ sung cho nhau, không thay nhau.

## pre-commit: chạy công cụ trước khi commit

Chạy bốn lệnh bằng tay thì sớm muộn có người quên một lệnh. **pre-commit** là công cụ chạy một danh sách hook mỗi lần `git commit`, và từ chối commit nếu hook đỏ. Cấu hình `.pre-commit-config.yaml` của lab:

```yaml
repos:
  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.17.4 # khớp phiên bản ruff trong uv.lock
    hooks:
      - id: ruff-format
      - id: ruff-check
        args: [--fix, --exit-non-zero-on-fix]
  - repo: local
    hooks:
      - id: mypy
        name: mypy --strict
        entry: uv run mypy src
        language: system
        types: [python]
        pass_filenames: false # mypy cần nhìn cả gói, không chỉ file vừa sửa
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v6.0.0
    hooks:
      - id: end-of-file-fixer
      - id: trailing-whitespace
      - id: check-toml
      - id: check-yaml
```

Ba chi tiết cần đọc kỹ. `rev` ghim phiên bản của hook ruff và nên khớp với ruff trong `uv.lock`; hai phiên bản lệch nhau là nguồn của kiểu «máy tôi xanh, CI đỏ». `--exit-non-zero-on-fix` làm hook báo đỏ khi nó **vừa sửa** file, để bạn xem lại thay đổi và `git add` lại thay vì commit những dòng chưa đọc. Hook mypy là hook `local`, chạy `uv run mypy src` bằng chính môi trường của dự án, và `pass_filenames: false` vì mypy cần nhìn cả gói để biết kiểu giữa các module chứ không phải riêng file vừa sửa.

Lab **không** chạy `pre-commit install`, vì lệnh đó ghi file vào `.git/hooks` của repository. README của lab hướng dẫn thử trong một repo tạm. Lần chạy thật trên bản sao ở thư mục tạm (đã `git init` và `git add -A`):

```text
ruff format..............................................................Passed
ruff check...............................................................Passed
mypy --strict............................................................Passed
fix end of files.........................................................Passed
trim trailing whitespace.................................................Passed
check toml...............................................................Passed
check yaml...............................................................Passed
```

Rồi thêm một dòng `import os` thừa vào `cli.py` và chạy lại cho riêng file đó (trích, bỏ dòng `ruff format`):

```text
ruff check...............................................................Failed
- hook id: ruff-check
- exit code: 1
- files were modified by this hook

Found 1 error (1 fixed, 0 remaining).

mypy --strict............................................................Passed
fix end of files.........................................................Passed
trim trailing whitespace.................................................Passed
```

Hook sửa file và báo đỏ một lần; chạy lại thì các hook đều xanh. Điều quan trọng là phải hiểu giới hạn của hook: nó chạy trên máy bạn, và `git commit --no-verify` bỏ qua nó. Vì vậy CI vẫn phải chạy **đúng những lệnh đó** (kèm pytest và build). Hook là đường tắt tiện cho người, không phải cổng chặn duy nhất.

## Chạy và kiểm chứng

Các lệnh ở trên xếp theo thứ tự rẻ trước, đắt sau: định dạng, lint, kiểu, test, rồi build. Hình 2 vẽ cả chuỗi. Dừng ngay ở bước đầu tiên đỏ là cách tiết kiệm thời gian nhất.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 215" role="img" aria-label="Năm bước kiểm chứng nối nhau: ruff format, ruff check, mypy strict, pytest, uv build. Ba bước đầu nằm trong khung nét đứt của pre-commit. Mỗi bước ghi loại lỗi nó bắt: hình thức, bẫy phổ biến, hợp đồng kiểu, hành vi, gói cài được."><defs><marker id="py20-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="16" y="22" class="t-small">pre-commit: ba bước đầu, lúc commit</text><text x="540" y="22" class="t-small">CI chạy đủ cả năm bước</text><rect x="8" y="34" width="514" height="140" rx="10" class="f-none s-line dash"/><rect x="15" y="48" width="150" height="112" rx="8" class="f-phys-soft s-phys"/><text x="27" y="76" class="t-b">ruff format</text><text x="27" y="102" class="t-small">hình thức: khoảng</text><text x="27" y="122" class="t-small">trắng, dấu phẩy</text><rect x="190" y="48" width="150" height="112" rx="8" class="f-phys-soft s-phys"/><text x="202" y="76" class="t-b">ruff check</text><text x="202" y="102" class="t-small">bẫy phổ biến:</text><text x="202" y="122" class="t-small">import thừa, ==None</text><rect x="365" y="48" width="150" height="112" rx="8" class="f-pk-soft s-pk"/><text x="377" y="76" class="t-b">mypy --strict</text><text x="377" y="102" class="t-small">hợp đồng kiểu:</text><text x="377" y="122" class="t-small">float thay Decimal</text><rect x="540" y="48" width="150" height="112" rx="8" class="f-pk-soft s-pk"/><text x="552" y="76" class="t-b">pytest</text><text x="552" y="102" class="t-small">hành vi: ca biên,</text><text x="552" y="122" class="t-small">làm tròn, lỗi</text><rect x="715" y="48" width="150" height="112" rx="8" class="f-warn-soft s-warn"/><text x="727" y="76" class="t-b">uv build</text><text x="727" y="102" class="t-small">gói cài được:</text><text x="727" y="122" class="t-small">entry point, py.typed</text><path d="M165 104 L190 104" class="f-none s-phys" stroke-width="2" marker-end="url(#py20-1-arrow)"/><path d="M340 104 L365 104" class="f-none s-phys" stroke-width="2" marker-end="url(#py20-1-arrow)"/><path d="M515 104 L540 104" class="f-none s-phys" stroke-width="2" marker-end="url(#py20-1-arrow)"/><path d="M690 104 L715 104" class="f-none s-phys" stroke-width="2" marker-end="url(#py20-1-arrow)"/><text x="16" y="198" class="t-small">Từ rẻ tới đắt; mỗi bước bắt một loại lỗi mà các bước khác không bắt.</text></svg>
</div>
<figcaption>Hình 2. Năm lớp kiểm chứng. Pre-commit giữ ba lớp rẻ ở máy từng người; CI chạy đủ cả năm để không phụ thuộc vào việc ai đó có chạy hook hay không.</figcaption>
</figure>

`verify.sh` của lab gói cả năm lớp (bước build dựng wheel, kiểm `py.typed` và entry point, rồi cài vào môi trường cô lập) và thêm một bước đảo ngược: nó đòi các công cụ **phải bắt được** ba file trong `demo-errors/` với đúng số lỗi (5 lỗi ruff, 1 file cần định dạng lại, 3 lỗi mypy). Một bộ kiểm chứng chỉ chứng minh xanh nếu ta cũng chứng minh được nó biết đỏ.

Từ thư mục gốc repository (lần đầu cần mạng để uv tải ruff, mypy, pytest và build backend):

```bash
L=examples/toi-noi-python/20-dong-goi-ruff-cau-truc-du-an
uv run --project $L ruff format --check $L
uv run --project $L ruff check $L
uv run --project $L mypy --config-file $L/pyproject.toml $L/src $L/tests
uv run --project $L pytest $L -q
uv run --project $L toinoi-fare 2.5
sh $L/verify.sh
```

mypy cần `--config-file` vì nó đọc cấu hình từ thư mục hiện tại: chạy từ gốc repository mà thiếu cờ này, `mypy` trên `bad_lint.py` báo `Success: no issues found` vì không đọc `strict` của lab (chạy trong thư mục lab thì báo hai lỗi `no-untyped-def`). Ruff và pytest thì tự tìm `pyproject.toml` gần file nhất. `verify.sh` kết thúc bằng `OK 7/7`; khi một bước đỏ, nó in `FAIL n/7` (n là số bước đạt) và thoát mã khác 0. Output thật (trích, bỏ phần khung của file định dạng):

```text
== 1. ruff format --check
9 files already formatted
== 2. ruff check
All checks passed!
== 3. mypy --strict
Success: no issues found in 8 source files
== 4. pytest
............................................................             [100%]
60 passed in 0.18s
== 5. toinoi-fare 2.5
2.5 km => 14250 VND
== 6. uv build + cài wheel vào môi trường cô lập
wheel: toi_noi_py_20_dong_goi_ruff_cau_truc_du_an-0.1.0-py3-none-any.whl
Installed 1 package in 2ms
2.5 km => 14250 VND
== 7. demo-errors: công cụ phải bắt được lỗi
Found 5 errors.
1 file would be reformatted
Found 3 errors in 1 file (checked 1 source file)
OK 7/7
```

Con số 60 test gồm 33 test của `test_pricing.py` (24 ca biên là 8 khoảng cách × 3 dịch vụ, 4 ca ngoài miền, dịch vụ lạ, chính sách thay thế và báo giá), 22 test của `models` (`Money`, `Quote` và bảng chuyển trạng thái, trong đó phần lớn là các ca từ chối đầu vào chép từ lab 13) và 5 test của CLI. Lệnh `toinoi-fare 100.001` in `lỗi: khoảng cách ngoài miền 0..100 km: 100.001` ra stderr và thoát với mã 2; lỗi đi qua `stderr`, nên `stdout` chỉ chứa kết quả. Lab dùng Python 3.14.7, ruff 0.17.4, mypy 2.4.1, pytest 9.2.0, `uv_build` kèm uv 0.12.11.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `uv run` dừng ở bước build với «failed to open file README.md» | `readme` trong `[project]` trỏ tới file có tồn tại không |
| `ModuleNotFoundError: toinoi_pricing` khi chạy test | Có chạy qua `uv run --project` không; `package = true` và `module-name` đã khai chưa |
| mypy báo «missing library stubs or py.typed marker» ở nơi dùng gói | Gói có file `py.typed` và nó có nằm trong wheel không |
| Hook xanh ở máy bạn, CI đỏ | `rev` của ruff-pre-commit có khớp ruff trong `uv.lock`; CI có chạy đúng cùng lệnh không |
| `mypy` báo «Success» với mã rõ ràng thiếu kiểu | Chạy từ thư mục khác với `pyproject.toml` nên không đọc `strict`; thêm `--config-file` |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Gói chưa được phát hành lên registry (PyPI hay registry nội bộ), chưa có chiến lược đánh số phiên bản, changelog hay build trong CI; repository này cũng không cài pre-commit hook. Bộ quy tắc ruff chỉ là điểm xuất phát nhỏ, mỗi nhóm nên chọn lại theo thực tế. 60 test chứng minh các ca đã chọn chứ không chứng minh mọi khoảng cách đúng, và <code>mypy --strict</code> chỉ bảo đảm hợp đồng kiểu, không bảo đảm nghiệp vụ. Mô hình cước vẫn là bản học tập: không phụ phí, thuế hay giảm giá, và các con số 12.000, 4.500 không phải bảng giá thật.</p>
</div>

**Bài tập 1:** Thêm họ `T20` (cấm lệnh `print`) vào `select` của ruff. Lệnh nào đỏ, ở đâu, và bạn xử lý thế nào cho `cli.py`?

**Bài tập 2:** Vì sao wheel không chứa `tests/` và `demo-errors/` dù hai thư mục nằm cạnh `src/`? Muốn đưa test vào sdist thì đổi gì?

<details><summary>Xem lời giải</summary><p><b>Bài 1.</b> <code>ruff check</code> báo ba lỗi <code>T201 `print` found</code> tại <code>src/toinoi_pricing/cli.py</code> (dòng 30, 35 và 37 ở bản lab), còn <code>ruff format --check</code>, mypy, pytest vẫn xanh. Với một công cụ dòng lệnh, in ra <code>stdout</code> và <code>stderr</code> là đúng việc, nên cách xử lý hợp lý là cho phép riêng file này trong <code>[tool.ruff.lint.per-file-ignores]</code> (<code>"src/toinoi_pricing/cli.py" = ["T201"]</code>), thay vì tắt quy tắc cho cả dự án; mã thư viện như <code>pricing.py</code> vẫn bị cấm <code>print</code>. <b>Bài 2.</b> <code>uv_build</code> chỉ đóng gói module khai trong <code>module-name</code> nằm dưới <code>src/</code>; những gì ngoài đó, kể cả <code>tests/</code>, không vào wheel. Danh sách file của wheel ở trên cho thấy đúng như vậy. Sdist của lab chỉ có <code>PKG-INFO</code>, <code>README.md</code>, <code>pyproject.toml</code>, <code>pyproject.toml.orig</code> và <code>src/</code>. Muốn đưa test vào sdist, thêm <code>source-include = ["tests/**"]</code> dưới <code>[tool.uv.build-backend]</code>. Thử trên bản sao: sdist có thêm bốn mục (thư mục <code>tests</code> và ba file test), wheel vẫn không có mục nào của <code>tests</code>.</p></details>

Bài trước: [bài 19](/articles/python-design-pattern-strategy-state-adapter). Bài tiếp: [bài 21](/articles/python-asyncio-coroutine-task-timeout-cancellation), mở đầu phần 03 (lộ trình **Python trung bình qua Tới Nơi**).

## Tài liệu gốc

- [uv: Building and publishing a package](https://docs.astral.sh/uv/guides/package/).
- [uv: Build backend](https://docs.astral.sh/uv/concepts/build-backend/).
- [Ruff: Configuration](https://docs.astral.sh/ruff/configuration/) và [Rules](https://docs.astral.sh/ruff/rules/).
- [mypy: The mypy command line](https://mypy.readthedocs.io/en/stable/command_line.html).
- [PEP 561: Distributing and packaging type information](https://peps.python.org/pep-0561/).
- [pre-commit](https://pre-commit.com/).
