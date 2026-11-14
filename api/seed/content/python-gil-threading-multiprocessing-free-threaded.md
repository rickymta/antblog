---
slug: python-gil-threading-multiprocessing-free-threaded
title: "GIL, threading, multiprocessing và free-threaded 3.14: chọn mô hình cho OCR"
summary: "Bài 22: phân biệt chờ I/O với tính toán CPU bằng số đo thật, so ThreadPoolExecutor, ProcessPoolExecutor và InterpreterPoolExecutor, giải thích GIL và thử bản free-threaded 3.14t cho tiền xử lý ảnh OCR."
category: python-intermediate
level: intermediate
tags: [python, toi-noi, concurrency, gil, multiprocessing]
language: vi
---

Khi [Tới Nơi](/articles/du-an-mau-toi-noi) mở một đợt tuyển tài xế mới, hồ sơ KYC đổ về `kyc-ocr` thành từng lô: mỗi hồ sơ có ảnh căn cước và ảnh giấy phép lái xe. Trước khi chữ được đọc, mỗi ảnh phải qua tiền xử lý: lấy từ object storage, làm mượt, nhị phân hoá, nén lại. Đội Python gói việc đó vào `ThreadPoolExecutor(10)` trên máy 10 lõi và chờ thời gian giảm mười lần; nó không giảm. Họ đổi sang `ProcessPoolExecutor` thì nhanh hơn rõ rệt, nhưng với lô chỉ vài ảnh thì phí khởi động pool ăn hết phần lời. Bài này giải thích vì sao, bằng số đo chạy thật trên cùng một lô ảnh giả.

Đây là bài 22 của lộ trình **Python trung bình qua Tới Nơi**. Bài 21 cho ta `asyncio` để chờ nhiều I/O mà không cần thread; bài này trả lời câu hỏi còn lại: khi việc nặng là **tính toán**, Python chạy song song bằng cách nào, và GIL nằm ở đâu.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Máy có 10 lõi. Bạn nhị phân hoá 20 ảnh bằng một vòng <code>for</code> Python thuần, mỗi ảnh khoảng 0,2 giây, rồi chuyển sang <code>ThreadPoolExecutor(10)</code> với đúng hàm đó. Cả lô sẽ nhanh hơn tuần tự khoảng mấy lần?</p>
<details><summary>Xem đáp án</summary><p><b>Khoảng 1 lần, tức không nhanh hơn.</b> Ở bản Python có GIL (bản mặc định), tại mỗi thời điểm chỉ một thread chạy bytecode Python, nên 10 thread tranh nhau một lõi hiệu dụng. Lab đo được 3,889 giây tuần tự và 4,206 giây với 10 thread (chậm hơn một chút). Với process (pool sẵn), mỗi process có GIL riêng và cùng lô đó chạy trong 0,618 giây.</p></details>
</div>

## Hai loại việc: chờ và tính

Mọi quyết định đồng thời bắt đầu bằng một câu hỏi: trong lúc việc này «đang chạy», CPU của ta làm gì? Có hai trường hợp.

- **I/O-bound**: phần lớn thời gian là chờ thứ bên ngoài, như object storage, mạng, ổ đĩa hay một tiến trình con. CPU gần như rảnh.
- **CPU-bound**: phần lớn thời gian là tính toán. Muốn nhanh hơn thì phải có thêm lõi làm việc cùng lúc.

Lab `22-gil-threading-multiprocessing` dựng ba việc đại diện cho `kyc-ocr`, mọi thứ chỉ dùng thư viện chuẩn. Ảnh là «ảnh xám giả» 2.000.000 điểm (cỡ một ảnh thẻ quét), sinh từ `random.Random(page_id).randbytes`, nên lặp lại được và không có ảnh thật nào.

```python
# workload.py (trích)
def fetch_page(page_id: int, delay: float = IO_DELAY) -> int:
    """Việc I/O-bound: chờ «storage» trả ảnh. Gần như không tốn CPU."""
    time.sleep(delay)
    return page_id


def binarize_page(page_id: int, pixels: int = PAGE_PIXELS) -> int:
    page = make_page(page_id, pixels)
    dark = 0
    for i in range(1, pixels - 1):
        if (page[i - 1] + page[i] + page[i + 1]) // 3 < 128:
            dark += 1
    return dark


def compress_page(page_id: int, pixels: int = PAGE_PIXELS) -> int:
    page = make_page(page_id, pixels).translate(_LOW_NIBBLE)
    return len(zlib.compress(page, 9))
```

| Việc | Hàm | Thời gian khi chạy tuần tự (một ảnh) | Bản chất |
| --- | --- | --- | --- |
| Chờ storage | `fetch_page` | 0,1 s (cố định) | I/O-bound |
| Nhị phân hoá bằng vòng lặp Python | `binarize_page` | khoảng 0,2 s | CPU-bound, bytecode Python |
| Nén zlib mức 9 | `compress_page` | khoảng 0,07 s | CPU-bound, nhưng chạy trong mã C |

`time.sleep` chỉ thay cho I/O thật: nó nhả GIL như một lần chờ socket. Hàng thứ ba có mặt để cho thấy rằng «CPU-bound» chưa đủ để kết luận; điều quyết định là **đoạn tính toán nằm trong bytecode Python hay trong mã C**.

## GIL: một khoá trong interpreter

GIL (Global Interpreter Lock) là một mutex trong CPython. Ở bản build thường, một thread phải giữ nó mới được chạy bytecode Python, nên dù process có 10 thread và máy có 10 lõi, tại mỗi thời điểm chỉ một thread đang thực thi bytecode trong một interpreter. Hệ quả cần nhớ:

- **GIL được nhả khi thread chờ I/O** (`time.sleep`, socket, đọc file) và khi mã C của một thư viện chủ động nhả nó. `zlib` nhả GIL trong lúc nén; thư viện số như NumPy hay OpenCV thường làm vậy ở phần nặng, nhưng đó là quyết định của từng thư viện, nên đáng đo trước khi tin.
- **Thread CPU-bound thuần Python thay phiên nhau**, không chạy song song. `sys.getswitchinterval()` in `0.005`: thread đang giữ GIL được yêu cầu nhả sau khoảng 5 mili giây, để thread khác có lượt.
- **GIL không thay cho khoá của ta.** Nó bảo vệ trạng thái nội bộ của interpreter (như bộ đếm tham chiếu), không bảo vệ logic nghiệp vụ. Hai thread vẫn có thể cùng đọc «chuyến còn trống» rồi cùng ghi; đó là chủ đề bài 23, và là điều [bài Concurrency và thread safety của loạt C#](/articles/csharp-concurrency-thread-safety) mô tả bằng `lock`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Với GIL, ba thread T1, T2, T3 trong một process luân phiên chạy trên một dải thời gian; ba process P1, P2, P3 mỗi cái một dải riêng chạy song song."><defs><marker id="py22-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="34" class="t-b">Một process, GIL bật: một thread chạy bytecode mỗi lúc</text><rect x="20" y="46" width="840" height="48" rx="8" class="f-bg s-line"/><rect x="30" y="52" width="160" height="36" rx="6" class="f-pk-soft s-pk"/><text x="94" y="75" class="t-b">T1</text><rect x="190" y="52" width="160" height="36" rx="6" class="f-phys-soft s-phys"/><text x="254" y="75" class="t-b">T2</text><rect x="350" y="52" width="160" height="36" rx="6" class="f-warn-soft s-warn"/><text x="414" y="75" class="t-b">T3</text><rect x="510" y="52" width="160" height="36" rx="6" class="f-pk-soft s-pk"/><text x="574" y="75" class="t-b">T1</text><rect x="670" y="52" width="160" height="36" rx="6" class="f-phys-soft s-phys"/><text x="734" y="75" class="t-b">T2</text><text x="20" y="118" class="t-small">Luồng khác chờ lượt giữ GIL, hoặc đang ngủ trong I/O (khi ngủ thì đã nhả GIL).</text><text x="20" y="160" class="t-b">Ba process: mỗi process một interpreter, một GIL riêng</text><rect x="20" y="172" width="840" height="30" rx="6" class="f-phys-soft s-phys"/><text x="34" y="192" class="t-small">P1 binarize ảnh 1, 4, 7 ...</text><rect x="20" y="210" width="840" height="30" rx="6" class="f-phys-soft s-phys"/><text x="34" y="230" class="t-small">P2 binarize ảnh 2, 5, 8 ...</text><rect x="20" y="248" width="840" height="30" rx="6" class="f-phys-soft s-phys"/><text x="34" y="268" class="t-small">P3 binarize ảnh 3, 6, 9 ...</text><path d="M20 138 L860 138" class="f-none s-muted dash" stroke-width="1"/></svg>
</div>
<figcaption>Hình 1. Thread cùng một process thay phiên nhau giữ GIL; process tách riêng có GIL riêng nên chạy song song trên nhiều lõi.</figcaption>
</figure>

## Ba pool, một giao diện

`concurrent.futures` cho cả ba mô hình cùng một giao diện, nên đổi mô hình là đổi một tên lớp. Hàm `run_pool` trong `main.py` tạo pool, chạy cả lô bằng `map` (giữ thứ tự đầu vào), rồi đóng pool; vì thế thời gian đo gồm cả khởi động:

```python
# main.py (trích)
def run_pool(make, fn, ids):
    with make() as executor:
        return list(executor.map(fn, ids))  # map giữ thứ tự đầu vào
```

- **`ThreadPoolExecutor`**: các thread chia sẻ bộ nhớ, tạo rẻ, đối số và kết quả không cần serialize. Hợp với I/O và mã C nhả GIL.
- **`ProcessPoolExecutor`**: mỗi worker là một process Python riêng, có GIL riêng nên chạy song song thật. Cái giá: hàm, đối số và kết quả đều phải qua `pickle`; khởi động mỗi worker tốn thời gian; bộ nhớ không dùng chung.
- **`InterpreterPoolExecutor`** (mới trong 3.14, PEP 734): nhiều interpreter trong **cùng một process**, mỗi cái có GIL riêng, nên chạy song song mà không tạo process mới. Giống process, hàm, đối số và giá trị trả về cũng được `pickle` khi qua ranh giới interpreter (theo tài liệu 3.14). Tài liệu nêu các hạn chế hiện tại: khởi động interpreter chưa được tối ưu, chia sẻ đối tượng còn ít, và nhiều extension module trên PyPI chưa tương thích (module thư viện chuẩn thì có). Lab chỉ dùng thư viện chuẩn nên chạy được.

Với process có ba điều bắt buộc. **Thứ nhất**, hàm phải import được theo tên, nên `binarize_page` nằm trong `workload.py` chứ không trong file chạy chính. **Thứ hai**, chương trình chính phải có `if __name__ == "__main__":`. Với `spawn` (mặc định ở macOS và Windows) và cả `forkserver`, process con khởi động interpreter mới rồi import lại module chính; tài liệu `multiprocessing` nói đúng hai start method này sẽ lỗi nếu module chính không an toàn khi import. Thiếu guard, process con lại cố tạo pool và Python dừng với `RuntimeError`. Ta thử bằng một file ngắn gọi `ProcessPoolExecutor` ở mức module (trích):

```text
RuntimeError:
        An attempt has been made to start a new process before the
        current process has finished its bootstrapping phase.
...
concurrent.futures.process.BrokenProcessPool: A process in the process pool was terminated abruptly while the future was running or pending.
```

Từ Python 3.14, trên Unix không phải macOS, mặc định đổi từ `fork` sang `forkserver`; macOS và Windows vẫn là `spawn`. Đây là chỗ code Linux hay hỏng khi nâng lên 3.14: với `fork`, script thiếu guard hoặc dựa vào biến global đã nạp sẵn vẫn chạy; với `forkserver` thì không. **Thứ ba**, mọi thứ đi qua pipe phải pickle được. `executor.submit(lambda page_id: page_id, 1)` không chạy được với process; lỗi nổi lên ở `future.result()` chứ không ở dòng `submit`. Ngoại lệ trong worker thì ngược lại: nó được ném lại ở process chính với đúng loại và thông điệp, (`--verify` kiểm cả hai).

Pool process có thể «nguội» (vừa tạo, cần spawn) hoặc «sẵn» (đã khởi động). Lab đo cả hai, vì service thật nên giữ pool sống.

## Chạy và kiểm chứng

Lab không cần mạng hay hạ tầng:

```bash
uv run --project examples/toi-noi-python/22-gil-threading-multiprocessing examples/toi-noi-python/22-gil-threading-multiprocessing/main.py
```

Máy đo: MacBook Pro M1 Max, 10 lõi, 64 GB RAM, macOS 26.6.2, Python 3.14.7 (build thường). Máy có 8 lõi hiệu năng (P) và 2 lõi tiết kiệm điện (E), nên 10 process CPU-bound không bao giờ đạt gấp mười một lõi P kể cả khi máy rảnh. Máy cũng **không rảnh**: có phiên khác chạy cùng lúc, load average lúc bắt đầu là 26,60 (build thường) và 41,89 (3.14t) trên 10 lõi, tức cao hơn số lõi, và tải đổi giữa các hàng đo (từ 26,6 đến 41,9 ở build thường). Lab in load trước và sau mỗi hàng, và ta chỉ khẳng định thứ bậc lớn (vài lần), không chênh nhỏ. Output đầy đủ:

```text
Máy: macOS-26.6.2-arm64-arm-64bit-Mach-O, os.cpu_count()=10, process_cpu_count()=10
Python 3.14.7 (CPython), Py_GIL_DISABLED=0, GIL bật: True
start method: spawn, switch interval: 0.005 s
lõi theo loại: 8 lõi P + 2 lõi E
load average 1 phút lúc bắt đầu: 26.60 (số lõi: 10); mỗi hàng đo in thêm load trước và sau, vì tải máy đổi giữa các hàng

I/O: chờ storage: 20 ảnh, trung vị 3 lần
  tuần tự                                2.144 s   x 1.00   (load  26.6 ->  33.5)
  thread x10                             0.217 s   x 9.90   (load  33.5 ->  33.5)
  process x10 (gồm khởi động)            0.493 s   x 4.35   (load  33.5 ->  33.5)
  process x10 (pool sẵn)                 0.213 s   x10.07   (load  33.5 ->  33.5)
  interpreter x10 (pool sẵn)             0.217 s   x 9.89   (load  33.5 ->  33.5)

CPU: vòng lặp Python (binarize): 20 ảnh, trung vị 3 lần
  tuần tự                                3.889 s   x 1.00   (load  33.5 ->  33.4)
  thread x10                             4.206 s   x 0.92   (load  33.4 ->  35.3)
  process x10 (gồm khởi động)            0.902 s   x 4.31   (load  35.3 ->  37.2)
  process x10 (pool sẵn)                 0.618 s   x 6.30   (load  37.2 ->  37.2)
  interpreter x10 (pool sẵn)             0.649 s   x 6.00   (load  37.2 ->  39.7)

CPU trong C: zlib.compress: 20 ảnh, trung vị 3 lần
  tuần tự                                1.308 s   x 1.00   (load  39.7 ->  39.4)
  thread x10                             0.240 s   x 5.45   (load  39.4 ->  39.4)
  process x10 (gồm khởi động)            0.428 s   x 3.05   (load  39.4 ->  39.4)
  process x10 (pool sẵn)                 0.279 s   x 4.69   (load  39.4 ->  39.4)
  interpreter x10 (pool sẵn)             0.224 s   x 5.85   (load  39.4 ->  41.9)
```

Cách đọc:

- **I/O**: 20 ảnh chờ 0,1 s chia cho 10 thread là 2 lượt, lý tưởng 0,2 s; đo được 0,217 s. Thread đủ, process chỉ thêm phí khởi động (0,493 s).
- **Vòng lặp Python**: thread ra 0,92 lần (không nhanh hơn), tức GIL làm mười thread thành một. Process và interpreter ra khoảng 6 lần ở pool sẵn (0,618 s và 0,649 s); gồm khởi động pool là 0,902 s, tức 4,31 lần. Cả hai chưa tới 10 vì chỉ có 8 lõi P và máy đang bận. Chênh 0,902 − 0,618 ≈ 0,28 s là phí khởi động pool nguội, cùng bậc với một ảnh (khoảng 0,2 s), nên lô chỉ vài ảnh không có lời; phí này dao động mạnh khi máy bận (hàng I/O của 3.14t: 1,486 s nguội so với 0,229 s sẵn).
- **zlib**: thread nhanh gấp 5,45 lần dù đây là việc CPU, vì mã C nhả GIL. Thread (0,240 s), interpreter pool sẵn (0,224 s) và process pool sẵn (0,279 s) nằm cùng cỡ vài lần, chênh nhau trong khoảng mà máy bận không cho phép xếp hạng. Chỉ pool process nguội (0,428 s, 3,05 lần) rõ ràng chậm hơn, vì phải spawn trước khi nén.

Cùng lệnh với `--verify` so kết quả của mọi mô hình với bản tuần tự (cả `binarize_page(1) == 1_006_619`), rồi kiểm hai hành vi lỗi ở trên và trạng thái GIL. Nó không kiểm tốc độ nào, vì tỉ lệ đổi theo tải máy:

```bash
uv run --project examples/toi-noi-python/22-gil-threading-multiprocessing examples/toi-noi-python/22-gil-threading-multiprocessing/main.py --verify
```

```text
OK 10/10
```

## Free-threaded 3.14t

PEP 703 cho phép build CPython không GIL. Từ 3.14, theo PEP 779, build này được hỗ trợ chính thức, nhưng là bản **riêng** (`python3.14t`), không phải bản mặc định, và chưa có quyết định biến nó thành mặc định. Với Tới Nơi ta vẫn coi là thử nghiệm: ta phải tự kiểm thread safety (bài 23), và mỗi thư viện C phải có bản dựng cho build này. Ngày 06/11/2026, `uv python install 3.14t` chạy được; lab chỉ dùng thư viện chuẩn:

```bash
uv run --no-project --python 3.14t examples/toi-noi-python/22-gil-threading-multiprocessing/main.py
```

```text
Máy: macOS-26.6.2-arm64-arm-64bit-Mach-O, os.cpu_count()=10, process_cpu_count()=10
Python 3.14.7 (CPython), Py_GIL_DISABLED=1, GIL bật: False
start method: spawn, switch interval: 0.005 s
lõi theo loại: 8 lõi P + 2 lõi E
load average 1 phút lúc bắt đầu: 41.89 (số lõi: 10); mỗi hàng đo in thêm load trước và sau, vì tải máy đổi giữa các hàng

I/O: chờ storage: 20 ảnh, trung vị 3 lần
  tuần tự                                2.126 s   x 1.00   (load  41.9 ->  42.8)
  thread x10                             0.259 s   x 8.22   (load  42.8 ->  42.8)
  process x10 (gồm khởi động)            1.486 s   x 1.43   (load  42.8 ->  50.8)
  process x10 (pool sẵn)                 0.229 s   x 9.29   (load  50.8 ->  50.8)
  interpreter x10 (pool sẵn)             0.222 s   x 9.60   (load  50.8 ->  58.2)

CPU: vòng lặp Python (binarize): 20 ảnh, trung vị 3 lần
  tuần tự                                3.799 s   x 1.00   (load  58.2 ->  54.2)
  thread x10                             0.877 s   x 4.33   (load  54.2 ->  57.8)
  process x10 (gồm khởi động)            1.295 s   x 2.93   (load  57.8 ->  57.8)
  process x10 (pool sẵn)                 0.660 s   x 5.75   (load  57.8 ->  57.6)
  interpreter x10 (pool sẵn)             0.636 s   x 5.97   (load  57.6 ->  58.2)

CPU trong C: zlib.compress: 20 ảnh, trung vị 3 lần
  tuần tự                                1.242 s   x 1.00   (load  58.2 ->  58.2)
  thread x10                             0.201 s   x 6.19   (load  58.2 ->  58.1)
  process x10 (gồm khởi động)            0.719 s   x 1.73   (load  58.1 ->  58.1)
  process x10 (pool sẵn)                 0.210 s   x 5.91   (load  58.1 ->  58.1)
  interpreter x10 (pool sẵn)             0.200 s   x 6.20   (load  58.1 ->  55.8)
```

Điều đổi là hàng `thread` của việc Python thuần: từ 0,92 lên 4,33 lần mà không cần pickle hay spawn. Lần đo này pool process sẵn (5,75) và interpreter sẵn (5,97) nhỉnh hơn thread (4,33), nhưng máy bận (load 42 đến 58) nên ta không xếp hạng ba mô hình đó, cũng không rút ra «bản free-threaded nhanh hơn bao nhiêu». Hai dòng `tuần tự` (3,799 s trên 3.14t và 3,889 s trên build thường) chênh nhau trong nhiễu, nên không kết luận được về chi phí chạy đơn luồng. Tài liệu có hai con số cho chi phí đơn luồng: What's New 3.14 nói khoảng 5–10% tuỳ nền tảng và trình biên dịch, còn trang free-threading howto ghi trung bình trên pyperformance từ khoảng 1% trên macOS aarch64 đến 8% trên x86-64 Linux.

Về thư viện: cùng ngày, `numpy` 2.5.4 cài được trên 3.14t và sau `import numpy`, `sys._is_gil_enabled()` vẫn trả `False`. Với `opencv-python-headless`, `uv` không tìm thấy bản dựng sẵn cho 3.14t và bắt đầu biên dịch từ mã nguồn; ta dừng lệnh đó. Nghĩa là phần OCR dùng OpenCV (từ bài 30) chưa có đường chạy thẳng trên build này.

## Chọn mô hình cho kyc-ocr

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Câu hỏi nút cổ chai của lô ảnh dẫn tới ba nhánh: chờ I/O dùng thread hoặc asyncio, mã C nhả GIL dùng thread rồi đo lại, vòng lặp Python thuần dùng process hoặc thử free-threaded."><defs><marker id="py22-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="105" width="210" height="90" rx="10" class="f-pk-soft s-pk"/><text x="36" y="138" class="t-b">Nút cổ chai</text><text x="36" y="162" class="t-small">của lô ảnh là gì?</text><path d="M230 150 L310 55" class="f-none s-phys" stroke-width="2" marker-end="url(#py22-1-arrow)"/><path d="M230 150 L310 150" class="f-none s-phys" stroke-width="2" marker-end="url(#py22-1-arrow)"/><path d="M230 150 L310 245" class="f-none s-phys" stroke-width="2" marker-end="url(#py22-1-arrow)"/><rect x="310" y="25" width="280" height="60" rx="10" class="f-phys-soft s-phys"/><text x="326" y="60" class="t-small">Chờ I/O: storage, mạng, tiến trình con</text><rect x="310" y="120" width="280" height="60" rx="10" class="f-phys-soft s-phys"/><text x="326" y="155" class="t-small">Mã C nhả GIL: zlib, thư viện số</text><rect x="310" y="215" width="280" height="60" rx="10" class="f-warn-soft s-warn"/><text x="326" y="250" class="t-small">Vòng lặp Python thuần</text><path d="M590 55 L640 55" class="f-none s-phys" stroke-width="2" marker-end="url(#py22-1-arrow)"/><path d="M590 150 L640 150" class="f-none s-phys" stroke-width="2" marker-end="url(#py22-1-arrow)"/><path d="M590 245 L640 245" class="f-none s-phys" stroke-width="2" marker-end="url(#py22-1-arrow)"/><rect x="640" y="25" width="220" height="60" rx="10" class="f-pk-soft s-pk"/><text x="654" y="60" class="t-small">thread hoặc asyncio (bài 21)</text><rect x="640" y="120" width="220" height="60" rx="10" class="f-pk-soft s-pk"/><text x="654" y="155" class="t-small">thread, rồi đo lại</text><rect x="640" y="215" width="220" height="60" rx="10" class="f-pk-soft s-pk"/><text x="654" y="243" class="t-small">process; hoặc thread trên 3.14t</text><text x="654" y="261" class="t-small">nếu mọi thư viện hỗ trợ</text></svg>
</div>
<figcaption>Hình 2. Chọn mô hình theo nút cổ chai thật của lô ảnh, không theo thói quen. Mọi nhánh đều kết thúc bằng một lần đo.</figcaption>
</figure>

Với `kyc-ocr`, quy tắc rút ra từ số đo là:

1. **Đo trước, rồi chọn, và đo khi máy rảnh.** Tốc độ phụ thuộc máy, loại lõi và tải lúc đó; chỉ tin chênh lệch cỡ lớn, như thread với I/O so với thread với vòng lặp Python thuần.
2. **Nếu phần nặng đã nằm trong mã C nhả GIL**, như nén zlib ở đây, thread là đủ và rẻ nhất. Bài 30 thay vòng lặp của `binarize_page` bằng phép toán mảng của NumPy và OpenCV rồi đo thread trên một lô thẻ; khi đó câu hỏi «cần process không» đổi hẳn, và ta đo thay vì giả định.
3. **Nếu phần nặng là vòng lặp Python thuần**, dùng process. Giữ pool sống trong service để trả phí spawn một lần, và truyền cho worker một khoá nhỏ (`page_id`, đường dẫn storage) thay vì cả ảnh qua pickle.
4. **Chờ I/O thì asyncio hoặc thread**; process chỉ thêm chi phí.

Một lưu ý nghiệp vụ: kết quả tiền xử lý và OCR là đầu vào cho người duyệt hồ sơ, không tự động là quyết định xác minh.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| 10 thread CPU-bound không nhanh hơn tuần tự | Bản Python có GIL không (`sys._is_gil_enabled()`); việc là bytecode Python hay mã C |
| `BrokenProcessPool` và `RuntimeError ... bootstrapping phase` | Có `if __name__ == "__main__":` không (cần với cả `spawn` và `forkserver`, mặc định Linux từ 3.14); hàm worker nằm ở module import được không |
| `PicklingError` ở `future.result()` | Hàm là lambda hay hàm lồng nhau; đối số và kết quả có pickle được không |
| Process chậm hơn tuần tự ở lô nhỏ | Phí spawn: giữ pool sống, gom lô lớn hơn |
| Chạy tốt trên Linux, lỗi trên macOS (hoặc ngược lại) | Start method: `mp.get_start_method()`; từ 3.14 Linux dùng `forkserver`, macOS `spawn` |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Ảnh là byte ngẫu nhiên, vòng lặp nhị phân hoá là bài toán chủ ý đơn giản, và việc I/O là <code>time.sleep</code>, không phải object storage thật. Số đo đến từ một máy đang bận (load average từ 26,6 đến 41,9 ở build thường, từ 41,9 đến 58,2 ở 3.14t, trên 10 lõi), mỗi cấu hình chỉ lấy trung vị 3 lần. Chưa có hàng đợi, giới hạn bộ nhớ, timeout, hay xử lý một worker chết giữa chừng; phần backpressure là bài 23. <code>os.process_cpu_count()</code> theo CPU affinity (ví dụ cpuset của container), nên có thể nhỏ hơn <code>os.cpu_count()</code>; nó không phản ánh quota CPU kiểu <code>--cpus</code> hay giới hạn của Kubernetes, nên ở đó ta tự đặt số worker.</p>
</div>

**Bài tập 1:** Lô 20 ảnh, mỗi ảnh tốn 0,2 giây CPU thuần Python. Dùng `ProcessPoolExecutor(4)` trên máy rảnh. Thời gian lý tưởng là bao nhiêu, và hai nguyên nhân nào làm đo thực tế dài hơn?

**Bài tập 2:** Vì sao `executor.submit(lambda ...)` chạy tốt với `ThreadPoolExecutor` nhưng lỗi với `ProcessPoolExecutor`? Lỗi hiện ra ở dòng nào?

<details><summary>Xem lời giải</summary><p>(1) 20 ảnh chia cho 4 worker là 5 lượt, mỗi lượt 0,2 giây, nên lý tưởng là <b>1,0 giây</b>. Thực tế dài hơn vì phải spawn 4 process (và import module) trước khi chạy, và vì đối số (<code>page_id</code>) cùng kết quả đi qua pickle; thêm nữa, nếu máy đang chạy việc khác thì 4 worker không có đủ 4 lõi trọn vẹn. (2) Thread cùng process nên hàm lambda được truyền như một tham chiếu đối tượng. Process phải gửi hàm sang process khác qua pickle, mà pickle lưu hàm theo tên import được; lambda không có tên như vậy. Lỗi (<code>PicklingError</code>) hiện ra khi gọi <code>future.result()</code>, không phải ở dòng <code>submit</code>, như <code>--verify</code> của lab kiểm.</p></details>

Bài trước: [bài 21](/articles/python-asyncio-coroutine-task-timeout-cancellation). Bài tiếp: [bài 23](/articles/python-concurrency-lock-queue-semaphore).

## Tài liệu gốc

- [concurrent.futures: ThreadPoolExecutor, ProcessPoolExecutor, InterpreterPoolExecutor](https://docs.python.org/3.14/library/concurrent.futures.html).
- [multiprocessing: contexts and start methods](https://docs.python.org/3.14/library/multiprocessing.html).
- [Glossary: global interpreter lock](https://docs.python.org/3.14/glossary.html).
- [Python support for free threading](https://docs.python.org/3.14/howto/free-threading-python.html).
- [PEP 703: Making the Global Interpreter Lock Optional in CPython](https://peps.python.org/pep-0703/).
- [PEP 779: Criteria for supported status for free-threaded Python](https://peps.python.org/pep-0779/).
