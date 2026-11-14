---
slug: python-module-package-import
title: "Module, package và import: tổ chức mã tính cước thành gói"
summary: "Bài 07: import chạy file một lần và cache trong sys.modules, sys.path quyết định tìm module ở đâu, package, import tuyệt đối và tương đối, import vòng, src layout và uv với package = true."
category: python-beginner
level: beginner
tags: [python, toi-noi, uv, import]
language: vi
---

Sau sáu bài, hàm tính cước của [Tới Nơi](/articles/du-an-mau-toi-noi) vẫn nằm chung một file với phần in kết quả. Giờ có thêm hai nhu cầu: service báo giá cần gọi hàm này, và script kiểm thử cũng cần gọi chính hàm đó, không phải một bản sao. Sao chép file sang thư mục khác chỉ tạo ra hai quy tắc cước dễ lệch nhau. Việc cần làm là tách mã thành **module** và gom chúng vào một **package** để mọi nơi `import` cùng một bản.

Đây là bài 07 của lộ trình **Python cơ bản qua Tới Nơi**. Bạn cần biết hàm và cách chạy project bằng uv (bài 01 và 03). Lab dùng lại mô hình cước quen thuộc: 12.000 VND cho 2 km đầu, 4.500 VND cho mỗi km vượt quá, và thêm giới hạn trên 100 km.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Nếu hai file cùng viết <code>import pricing</code>, thân file <code>pricing.py</code> chạy mấy lần?</p>
<details><summary>Xem đáp án</summary><p><b>Một lần.</b> Lần đầu Python tìm file, tạo object module, ghi nó vào <code>sys.modules</code> rồi chạy thân file từ trên xuống. Lần sau chỉ lấy lại object đã lưu, không chạy lại.</p></details>
</div>

Đích đến của bài là gói `toinoi_fare` đặt dưới `src/`, còn `main.py` và `pyproject.toml` ở gốc lab. Hình 1 là bản đồ của cả bài: các mục sau giải thích vì sao mỗi file nằm đúng chỗ và vì sao dòng `import` nào viết được hay không.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 1. Cây thư mục src layout của lab 07: pyproject.toml, main.py và uv.lock ở gốc; src/toinoi_fare chứa __init__.py, __main__.py, models.py, pricing.py."><rect x="20" y="15" width="420" height="300" rx="10" class="f-bg s-line"/><text x="36" y="45" class="t-mono t-b">07-module-package-import/</text><text x="36" y="72" class="t-mono">├── pyproject.toml</text><text x="36" y="97" class="t-mono">├── uv.lock</text><text x="36" y="122" class="t-mono">├── main.py</text><text x="36" y="147" class="t-mono">└── src/</text><rect x="52" y="160" width="372" height="140" rx="8" class="f-pk-soft s-pk"/><text x="62" y="187" class="t-mono t-b">└── toinoi_fare/</text><text x="100" y="212" class="t-mono">├── __init__.py</text><text x="100" y="237" class="t-mono">├── __main__.py</text><text x="100" y="262" class="t-mono">├── models.py</text><text x="100" y="287" class="t-mono">└── pricing.py</text><rect x="480" y="30" width="380" height="80" rx="10" class="f-phys-soft s-phys"/><text x="494" y="58" class="t-b">Gốc lab</text><text x="494" y="82" class="t-small">main.py chạy ở đây; thư mục của nó là</text><text x="494" y="100" class="t-small">sys.path[0] khi chạy bằng đường dẫn</text><rect x="480" y="135" width="380" height="80" rx="10" class="f-warn-soft s-warn"/><text x="494" y="163" class="t-b">src/ không tự có trong sys.path</text><text x="494" y="187" class="t-small">cần cài gói vào venv (editable) để</text><text x="494" y="205" class="t-small">uv thêm src vào qua file .pth</text><rect x="480" y="240" width="380" height="75" rx="10" class="f-pk-soft s-pk"/><text x="494" y="268" class="t-b">Gói toinoi_fare</text><text x="494" y="292" class="t-small">import toinoi_fare ở bất kỳ đâu trong venv</text></svg>
</div>
<figcaption>Hình 1. Cây thư mục src layout. Gói nằm dưới <code>src/</code> để việc import chỉ thành công khi gói đã được cài, giống lúc chạy thật.</figcaption>
</figure>

## Module là một file, import chạy nó một lần

Một **module** là một file `.py`. Câu lệnh `import pricing` làm theo thứ tự: tra `sys.modules` xem tên đã có chưa; nếu chưa, tìm `pricing.py` trong các thư mục của `sys.path`; tạo object module và **ghi nó vào `sys.modules` ngay lúc này**; sau đó mới chạy thân file từ trên xuống trong namespace của module; cuối cùng gắn tên `pricing` vào nơi gọi. Nếu thân file ném lỗi, Python gỡ module khỏi `sys.modules` và lỗi tiếp tục truyền lên nơi import. Phần định nghĩa hàm, hằng số và cả lệnh `print` ở mức module đều chạy trong bước chạy thân file.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Trình tự import: tra sys.modules; nếu thiếu thì tìm trong sys.path, ghi module vào sys.modules rồi mới thực thi thân file."><defs><marker id="py07-2-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="105" width="170" height="80" rx="10" class="f-pk-soft s-pk"/><text x="34" y="135" class="t-b">import x</text><text x="34" y="160" class="t-small">câu lệnh của bạn</text><rect x="250" y="105" width="190" height="80" rx="10" class="f-phys-soft s-phys"/><text x="264" y="135" class="t-b">1. sys.modules?</text><text x="264" y="160" class="t-small">tên đã có chưa</text><rect x="250" y="215" width="190" height="60" rx="10" class="f-bg s-line"/><text x="264" y="240" class="t-small">có: trả object cũ</text><text x="264" y="260" class="t-small">không chạy lại file</text><rect x="510" y="20" width="170" height="70" rx="10" class="f-warn-soft s-warn"/><text x="524" y="50" class="t-b">2. sys.path</text><text x="524" y="73" class="t-small">tìm x.py / x/</text><rect x="510" y="120" width="170" height="70" rx="10" class="f-warn-soft s-warn"/><text x="524" y="150" class="t-b">3. Ghi cache</text><text x="524" y="173" class="t-small">module vào sys.modules</text><rect x="710" y="105" width="150" height="80" rx="10" class="f-phys-soft s-phys"/><text x="724" y="135" class="t-b">4. Thực thi</text><text x="724" y="160" class="t-small">chạy thân; lỗi thì gỡ</text><path d="M190 145 L250 145" class="f-none s-phys" stroke-width="2" marker-end="url(#py07-2-arrow)"/><path d="M345 105 L345 100 L510 55" class="f-none s-phys dash" stroke-width="2" marker-end="url(#py07-2-arrow)"/><path d="M595 90 L595 120" class="f-none s-phys" stroke-width="2" marker-end="url(#py07-2-arrow)"/><path d="M680 155 L710 145" class="f-none s-phys" stroke-width="2" marker-end="url(#py07-2-arrow)"/><path d="M345 185 L345 215" class="f-none s-phys dash" stroke-width="2" marker-end="url(#py07-2-arrow)"/><text x="360" y="75" class="t-small">chưa có</text></svg>
</div>
<figcaption>Hình 2. Trình tự import. Chỉ lần đầu mới đi qua các bước 2 đến 4; các lần sau dừng ở bước 1 và trả lại cùng một object. Module được ghi vào <code>sys.modules</code> trước khi thân file chạy, và bị gỡ nếu thân file ném lỗi.</figcaption>
</figure>

Việc ghi cache **trước** khi chạy thân file là chi tiết quan trọng, vì chi tiết này giải thích lỗi import vòng ở mục sau. Ta kiểm chứng bằng một module tạm do `main.py` tự tạo. Thân module in ra một dòng, trong đó có việc tên của nó đã nằm trong `sys.modules` chưa, rồi ta import hai lần:

```python
# main.py: tạo probe_mod.py trong thư mục tạm rồi import hai lần
(Path(tmp) / "probe_mod.py").write_text(
    "import sys\n"
    'print("probe_mod: thân module đang chạy; trong sys.modules:", __name__ in sys.modules)\n'
    "VALUE = 42\n",
    encoding="utf-8",
)
sys.path.insert(0, tmp)
importlib.invalidate_caches()  # file mới tạo: xóa cache của bộ tìm module
first = importlib.import_module("probe_mod")   # chạy thân module
second = importlib.import_module("probe_mod")  # lấy từ sys.modules
```

Kết quả khi chạy lab:

```text
thân probe_mod chạy 1 lần sau 2 lần import
hai lần import cùng một object: True
probe_mod đã nằm trong sys.modules khi thân module chạy: True
```

Hệ quả thực tế: biến ở mức module là trạng thái dùng chung cho cả tiến trình. Nếu bạn sửa file rồi `import` lại trong cùng một tiến trình đang chạy, Python không đọc lại file; ta phải khởi động lại chương trình (`importlib.reload` chỉ nên dùng khi thử nghiệm). Cũng vì thế, đặt lệnh nặng hoặc có tác dụng phụ (mở kết nối, gọi mạng) ở mức module là thói quen xấu: chỉ cần import là chúng chạy.

Mỗi module có biến `__name__`. Khi file được **import**, `__name__` là tên module, như `toinoi_fare.pricing`. Khi file được **chạy làm chương trình chính** (`python main.py` hoặc `python -m toinoi_fare`), `__name__` là chuỗi `"__main__"`. Cấu trúc quen thuộc `if __name__ == "__main__":` nghĩa là đoạn này chỉ chạy khi file là điểm vào, không chạy khi ai đó import nó. Lab in ba giá trị:

```text
__name__ của main.py : __main__
__name__ của gói     : toinoi_fare
__name__ của pricing : toinoi_fare.pricing
```

Có hai cách chạy mã và chúng không thay thế nhau. `python main.py` chạy một **file** làm script. `python -m toinoi_fare` chạy **module** `toinoi_fare.__main__` với tư cách thành viên của gói. Lỗi kinh điển là chạy thẳng một file nằm trong gói như script: Python không biết file thuộc gói nào, nên import tương đối mất nghĩa. Lab chạy thử và in dòng lỗi:

```text
chạy pricing.py như script: ImportError: attempted relative import with no known parent package
```

Quy tắc dễ nhớ: file trong gói thì không chạy trực tiếp bằng đường dẫn; hãy cho gói một file `__main__.py` làm điểm vào và chạy bằng `python -m`.

## `sys.path` và lỗi «No module named»

Danh sách thư mục Python tìm module nằm ở `sys.path`. Trong môi trường của lab, khi chạy `python -c`, danh sách gồm sáu phần tử: chuỗi rỗng (thư mục hiện tại), file zip thư viện chuẩn, thư mục thư viện chuẩn, `lib-dynload`, `site-packages` của virtualenv, và cuối cùng là thư mục `src` của lab (do uv thêm, xem mục src layout). Khi chạy `python main.py`, phần tử đầu là **thư mục chứa `main.py`**, không phải thư mục hiện tại của terminal; với `python -m`, phần tử đầu là thư mục hiện tại. Vì vậy `import` tìm theo vị trí của script, không theo nơi bạn đứng khi gõ lệnh. Việc mở file dữ liệu bằng `open("data.csv")` là chuyện khác hẳn: lệnh này đi theo thư mục hiện tại của tiến trình, chứ không theo `sys.path` (bài 09 sẽ dùng `pathlib` để định vị file một cách rõ ràng).

Khi `sys.path` không có thư mục chứa module cần tìm, Python báo `ModuleNotFoundError: No module named '...'`, lớp con của `ImportError`. Lab chủ động tạo lỗi này trước khi thêm thư mục tạm vào `sys.path`:

```text
import probe_mod khi chưa có trong sys.path: No module named 'probe_mod'
```

Ta cũng gặp lỗi này khi chạy `uv run --no-project python -c "import toinoi_fare"`: môi trường đó không có gói nào tên `toinoi_fare`. Khi gặp lỗi, đừng vội sửa `sys.path` trong code. Kiểm tra theo thứ tự: bạn đang dùng đúng Python/virtualenv chưa, gói đã được cài vào môi trường đó chưa, tên module có gõ đúng chưa. Chèn `sys.path.insert(...)` vào code ứng dụng chỉ chữa triệu chứng: mã chạy được trên máy bạn và hỏng ở nơi khác.

## Package, `__init__.py` và hai kiểu import

**Package** là thư mục chứa các module. Thư mục `toinoi_fare/` có `__init__.py` là package thông thường; file này chạy một lần khi ai đó `import toinoi_fare`, và các tên nó định nghĩa trở thành tên công khai của gói.

```python
# src/toinoi_fare/__init__.py
from .models import DEFAULT_POLICY, FarePolicy, Quote
from .pricing import calculate_fare, quote

__all__ = ["DEFAULT_POLICY", "FarePolicy", "Quote", "calculate_fare", "quote"]
```

```python
# src/toinoi_fare/pricing.py (rút gọn)
from decimal import ROUND_HALF_UP, Decimal

from .models import DEFAULT_POLICY, FarePolicy, Quote


def calculate_fare(distance_km, policy: FarePolicy = DEFAULT_POLICY) -> Decimal:
    distance = Decimal(distance_km)
    if distance < 0 or distance > policy.max_km:
        raise ValueError(f"khoảng cách ngoài miền 0..{policy.max_km} km: {distance}")
    extra = max(Decimal(0), distance - policy.included_km)
    raw = policy.base_fare + extra * policy.price_per_extra_km
    return raw.quantize(Decimal("1"), rounding=ROUND_HALF_UP)


def quote(distance_km, policy: FarePolicy = DEFAULT_POLICY) -> Quote:
    return Quote(distance_km=Decimal(distance_km), total_vnd=calculate_fare(distance_km, policy))
```

`models.py` giữ `FarePolicy` và `Quote` (hai `NamedTuple` như ở bài 05) cùng hằng `DEFAULT_POLICY`. Lõi tính cước là `calculate_fare`; `quote()` chỉ đóng gói kết quả, nên mọi nơi gọi cùng một quy tắc.

- **Import tuyệt đối** bắt đầu từ tên gói trên cùng: `from toinoi_fare.pricing import calculate_fare`. Kiểu này rõ ràng, đọc ở đâu cũng hiểu, và là lựa chọn mặc định theo PEP 8. `main.py` và `__main__.py` dùng kiểu này.
- **Import tương đối** dùng dấu chấm: `.models` là module `models` cùng gói, `..` là gói cha, `..x` là module `x` trong gói cha. Kiểu này chỉ dùng được bên trong một package, và giúp đổi tên gói mà không sửa từng dòng import nội bộ. PEP 8 coi import tương đối tường minh là chấp nhận được khi gói có bố cục phức tạp. `pricing.py` và `__init__.py` dùng kiểu này; lab cố ý trộn hai kiểu để bạn thấy cả hai. Trong dự án thật, hãy thống nhất một quy ước, thường là tuyệt đối ở mọi nơi.

Còn một lựa chọn nữa là viết `import x` hay `from x import y`. Với `import toinoi_fare.pricing`, bạn phải viết đầy đủ `toinoi_fare.pricing.calculate_fare(...)` nhưng luôn thấy hàm đến từ đâu. Với `from toinoi_fare import calculate_fare`, dòng lệnh ngắn hơn, nhưng bạn nhận một **tham chiếu** tới object tại thời điểm import; nếu module gốc gán lại tên đó sau này, tên trong module của bạn không đổi theo. Tránh `from x import *` vì cách này khiến không ai biết tên đến từ đâu. Lab xác nhận `toinoi_fare.quote` và `toinoi_fare.pricing.quote` là cùng một object, và `toinoi_fare.pricing` đã nằm trong `sys.modules` sau khi import gói, vì `__init__.py` đã import nó.

## Import vòng: khi hai module đợi nhau

Nếu `fare_pricing` import `fare_models` ở dòng đầu, còn `fare_models` lại import ngược `fare_pricing`, ta có **import vòng**. Python bắt đầu chạy `fare_pricing`; module này đã nằm trong `sys.modules` nhưng mới chạy tới dòng import, chưa định nghĩa gì. Nó chạy tiếp `fare_models`, và tại đó cần tên `quote` của `fare_pricing`, nhưng tên đó chưa tồn tại. Gói `cycle_demo/` trong lab tái hiện lỗi; ta chạy từ gốc lab và lab in dòng lỗi (đã cắt đường dẫn tuyệt đối ở cuối):

```text
import vòng: ImportError: cannot import name 'quote' from partially initialized module 'cycle_demo.fare_pricing' (most likely due to a circular import)
```

Hãy nhớ cụm **«partially initialized module»**: nó cho biết module đang được import dở, đúng dấu hiệu của import vòng. Cách chữa tốt nhất là sửa thiết kế: tách phần dùng chung thành **module thứ ba** mà cả hai cùng import một chiều (trong gói của lab, `models.py` không import `pricing`, chỉ `pricing` import `models`). Import đặt trong thân hàm cũng gỡ được vòng, nhưng chỉ nên dùng khi bạn hiểu vì sao.

## src layout và cấu hình để chạy được thật

Cây ở Hình 1 theo **src layout**: mã của gói nằm dưới `src/`. Nếu gói nằm ngay gốc project, `import toinoi_fare` thành công chỉ vì thư mục chứa script hoặc thư mục hiện tại tình cờ nằm trong `sys.path`, kể cả khi gói chưa hề được cài. Với `src/`, bạn buộc phải cài gói vào môi trường, nên lỗi đóng gói lộ ra sớm trên máy bạn thay vì trên server.

Ở các bài trước, `[tool.uv] package = false` nghĩa là project không được cài như một gói. Bài này đổi thành `package = true` và khai báo `[build-system]` để uv biết cách đóng gói:

```toml
# pyproject.toml
[project]
name = "toi-noi-py-07-module-package-import"
version = "0.1.0"
requires-python = ">=3.14"
dependencies = []

[build-system]
requires = ["uv_build>=0.12.11,<0.13.0"]
build-backend = "uv_build"

[tool.uv]
package = true

[tool.uv.build-backend]
module-name = "toinoi_fare"
```

Tên project có dấu gạch (`toi-noi-py-07-...`) khác tên package dùng để `import` (`toinoi_fare`), nên ta khai báo `module-name` tường minh; nếu để mặc định, uv_build suy ra tên thư mục từ tên project. Ràng buộc `uv_build>=0.12.11,<0.13.0` là do uv 0.12.11 sinh ra khi tạo project; phiên bản bạn ghi sẽ theo phiên bản uv bạn đang cài. Khi chạy `uv run`, uv dựng virtualenv, cài gói ở chế độ **editable** và để lại file `.pth` có một dòng trỏ vào `src/`. Từ lúc đó `sys.path` có thêm thư mục này, và sửa file trong `src/toinoi_fare` có hiệu lực ngay, không cần cài lại.

## Phụ thuộc, `uv.lock` và giới hạn của ví dụ

`import decimal` hay `import logging` lấy từ **thư viện chuẩn** đi kèm Python: không cần cài thêm. Thư viện bên ngoài nằm trên **PyPI** và phải khai báo thành phụ thuộc. Lệnh `uv add` ghi vào `[project] dependencies`, cập nhật `uv.lock` và cài vào virtualenv. Thử trên một bản sao ngoài lab với gói `tzdata`:

```bash
uv add tzdata
```

```toml
dependencies = [
    "tzdata>=2026.6",
]
```

Phiên bản được chọn phụ thuộc vào thời điểm bạn chạy lệnh. `pyproject.toml` ghi **ràng buộc** bạn chấp nhận (`>=2026.6`); `uv.lock` ghi **phiên bản cụ thể** của cả cây phụ thuộc. Commit cả hai để đồng đội và CI cài đúng như bạn. Lab 07 không có phụ thuộc bên ngoài nào, nên `uv.lock` chỉ có một mục, và ta không `uv add` thật trong lab vì lab không cần mạng khi chạy. Khi thêm phụ thuộc, hãy đọc kỹ tên gói: gói có tên gần giống gói nổi tiếng là một đường tấn công chuỗi cung ứng.

Gói của lab rất nhỏ. Gói chưa được kiểm tra kiểu tĩnh bằng mypy, chưa có test (bài 10), chưa khai báo entry point để cài ra lệnh `toinoi-fare` và chưa phát hành lên registry. Bài 20 sẽ nâng cấu hình đóng gói, thêm ruff, mypy và cấu trúc dự án sẵn sàng để đưa thành service. Mô hình cước ở đây cũng chỉ là bản học tập: không phụ phí, thuế hay giảm giá.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `ModuleNotFoundError: No module named 'toinoi_fare'` | Có chạy qua `uv run --project` không; project có `package = true` và `[build-system]` chưa |
| `attempted relative import with no known parent package` | Có đang chạy file trong gói bằng đường dẫn không; chuyển sang `python -m` |
| `cannot import name ... from partially initialized module` | Hai module có import vòng không; tách phần chung ra module thứ ba |
| Sửa file nhưng kết quả cũ | Tiến trình có đang chạy từ trước không; import chỉ chạy file một lần mỗi tiến trình |
| Import chạy trên máy này nhưng hỏng ở máy khác | Có phụ thuộc vào thư mục hiện tại hoặc `sys.path.insert` không; dùng src layout và cài gói |

## Chạy và kiểm chứng

```bash
uv run --project examples/toi-noi-python/07-module-package-import examples/toi-noi-python/07-module-package-import/main.py
uv run --project examples/toi-noi-python/07-module-package-import examples/toi-noi-python/07-module-package-import/main.py --verify
uv run --project examples/toi-noi-python/07-module-package-import python -m toinoi_fare 2.5
```

Lệnh đầu in bảng cước, kết quả thí nghiệm `sys.modules` và hai lỗi import. Lệnh thứ hai chạy 18 kiểm tra và kết thúc bằng `OK 18/18`; có kiểm tra nào sai thì in tên kiểm tra đó, dòng `FAIL n/18` và thoát với mã 1. Lệnh thứ ba in `2.5 km => 14250 VND`; bỏ đối số thì mặc định 5 km và in `5 km => 25500 VND`. Bốn dòng bảng cước do lab in ra:

```text
0 km => 12000 VND
2 km => 12000 VND
2.5 km => 14250 VND
5 km => 25500 VND
```

**Bài tập:** thêm module `src/toinoi_fare/rounding.py` có hàm `to_vnd(amount)` làm tròn `ROUND_HALF_UP` về số nguyên, rồi cho `calculate_fare` trong `pricing.py` dùng hàm này thay cho đoạn `quantize`. Dùng import kiểu nào trong `pricing.py`? Nếu `rounding.py` lại `from .pricing import calculate_fare`, điều gì xảy ra khi import gói?

<details><summary>Xem lời giải</summary><p>Trong <code>pricing.py</code> dùng import tương đối vì cùng gói: <code>from .rounding import to_vnd</code>. Hàm: <code>def to_vnd(amount: Decimal) -> Decimal: return amount.quantize(Decimal("1"), rounding=ROUND_HALF_UP)</code>. Nếu <code>rounding.py</code> import ngược <code>calculate_fare</code> từ <code>pricing</code>, ta có import vòng và gặp <code>ImportError: cannot import name ... from partially initialized module</code> ngay khi <code>import toinoi_fare</code>. Cách chữa: <code>rounding.py</code> không được biết về <code>pricing</code>; phụ thuộc chỉ đi một chiều, <code>pricing</code> tới <code>rounding</code>.</p></details>

Bài trước: [bài 06](/articles/python-comprehension-generator-iterator). Bài tiếp: [bài 08](/articles/python-exception-debug-logging).

## Tài liệu gốc

- [Python Tutorial: Modules](https://docs.python.org/3.14/tutorial/modules.html).
- [Python Language Reference: The import system](https://docs.python.org/3.14/reference/import.html).
- [PEP 328: Imports: Multi-Line and Absolute/Relative](https://peps.python.org/pep-0328/).
- [uv: Project layout](https://docs.astral.sh/uv/concepts/projects/layout/).
- [uv: Managing dependencies](https://docs.astral.sh/uv/concepts/projects/dependencies/).
