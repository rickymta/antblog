---
slug: python-exception-debug-logging
title: "Exception, debug và logging: lần theo một báo giá sai"
summary: "Bài 08: lần theo báo giá 2,5 km bị tính 25.500 VND, đọc traceback từ dưới lên, try/except/else/finally, exception tuỳ biến và chaining, pdb, vì sao assert không phải validation và logging cơ bản."
category: python-beginner
level: beginner
tags: [python, toi-noi, debug, logging]
language: vi
---

Bộ phận hỗ trợ của [Tới Nơi](/articles/du-an-mau-toi-noi) nhận một phản ánh: khách đi 2,5 km nhưng app báo **25.500 VND**, đúng giá của chuyến 5 km. Log không có dòng lỗi nào, service không crash, không có traceback. Chương trình chạy thành công và đưa ra con số sai. Đây là loại sự cố khó nhất, vì không có gì tự báo cho ta biết hướng cần tìm.

Đây là bài 08 của lộ trình **Python cơ bản qua Tới Nơi**. Bài này dùng lại mô hình cước đã quen (12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt) và thêm một lớp nhận dữ liệu từ thiết bị theo đơn vị **mét**.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Chương trình in 25.500 VND cho chuyến 2,5 km mà không có traceback nào. Có phải chương trình không có lỗi?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Chương trình vẫn có lỗi logic. Exception chỉ xuất hiện khi code nhận ra một điều kiện bị vi phạm; phép tính sai mà vẫn ra một con số hợp lệ thì Python không có cách nào tự phát hiện.</p></details>
</div>

## Báo giá sai mà không có lỗi nào

Lab của bài có hàm `quote_from_metres()` nhận chuỗi mét từ thiết bị, đổi sang km rồi tính cước. Để người đọc tự lần theo, lab có thêm cờ `--buggy` bật một lỗi cố ý. Chạy hai bản với cùng đầu vào 2500 mét:

```bash
uv run --project examples/toi-noi-python/08-exception-debug-logging examples/toi-noi-python/08-exception-debug-logging/main.py 2500 --log-level WARNING
uv run --project examples/toi-noi-python/08-exception-debug-logging examples/toi-noi-python/08-exception-debug-logging/main.py 2500 --buggy --log-level WARNING
```

```text
2500 m => 14250 VND
2500 m => 25500 VND
```

Bản đúng ra 14.250 như bảng cước. Bản `--buggy` ra 25.500. Hãy nhìn con số trước khi mở debugger: 25.500 là đúng giá 5 km, nên khoảng cách có thể đã bị nhân đôi trước khi tới công thức cước. Đó mới là **giả thuyết**; debug là kiểm tra nó bằng trạng thái thực tế của chương trình.

Cách kiểm tra nhanh nhất là cho chương trình dừng ngay trước đoạn nghi ngờ. Hàm `breakpoint()` có sẵn trong Python gọi debugger `pdb`. Lab đặt nó trong `quote_from_metres()` và bật bằng cờ `--pdb`:

```python
# fare.py (rút gọn)
def quote_from_metres(raw_metres: str, *, buggy: bool = False, pause: bool = False) -> Decimal:
    if pause:
        breakpoint()  # dừng tại đây: n (next), s (step), p biểu_thức, c (continue), q (quit)
    metres = parse_device_metres(raw_metres)
    km = to_km(metres, buggy=buggy)
    total = calculate_fare(km)
    return total
```

Các lệnh `pdb` bạn cần ngay từ đầu:

| Lệnh | Việc |
| --- | --- |
| `n` (next) | Chạy hết dòng hiện tại, dừng ở dòng kế tiếp trong cùng hàm |
| `s` (step) | Đi vào hàm được gọi ở dòng hiện tại |
| `p biểu_thức` | In giá trị của biểu thức tại thời điểm dừng |
| `c` (continue) | Chạy tiếp cho tới breakpoint sau hoặc hết chương trình |
| `q` (quit) | Thoát debugger và dừng chương trình |

Một phiên thật, nhập lần lượt `n`, `n`, `p metres`, `s`, `n`, `n`, `p divisor`, `p metres / divisor`, `c` (đường dẫn đã rút gọn):

```text
> .../fare.py(57)quote_from_metres()
-> breakpoint()  # dừng tại đây: n (next), s (step), p biểu_thức, c (continue), q (quit)
(Pdb) > .../fare.py(58)quote_from_metres()
-> metres = parse_device_metres(raw_metres)
(Pdb) > .../fare.py(59)quote_from_metres()
-> km = to_km(metres, buggy=buggy)
(Pdb) Decimal('2500')
(Pdb) --Call--
> .../fare.py(37)to_km()
-> def to_km(metres: Decimal, *, buggy: bool = False) -> Decimal:
(Pdb) > .../fare.py(39)to_km()
-> divisor = Decimal("500") if buggy else METRES_PER_KM
(Pdb) > .../fare.py(40)to_km()
-> return metres / divisor
(Pdb) Decimal('500')
(Pdb) Decimal('5')
(Pdb) 2500 m => 25500 VND
```

Từ Python 3.13, `pdb` dừng ngay tại dòng gọi `breakpoint()` (dòng `-> breakpoint()` đầu tiên ở trên); bản cũ hơn dừng ở dòng kế tiếp. `metres` đúng là 2500. Nhưng `divisor` là 500, nên 2500 mét thành **5 km** thay vì 2,5 km. Lỗi nằm ở hằng số đổi đơn vị, đúng chỗ một bước nhỏ trước công thức cước; đọc code công thức tính cước một trăm lần cũng không thấy. Sửa bằng cách dùng `METRES_PER_KM = 1000` ở mọi nơi, và thêm một kiểm tra hồi quy cố định: 2500 mét phải ra 14.250. `--verify` của lab giữ cả hai chiều: bản đúng ra 14.250, bản `--buggy` tái hiện đúng 25.500.

## Đọc traceback từ dưới lên

Khi chương trình ném exception mà không ai bắt, Python in **traceback**. Quy tắc đọc: bắt đầu từ **dòng cuối**. Dòng cuối cho loại exception và thông điệp; ngay phía trên là dòng code đã `raise`; càng đi lên là các hàm đã gọi tới đó, cho tới lời gọi đầu tiên ở trên cùng. Chạy lab với cờ `--unhandled` để không có `except` nào chặn:

```bash
uv run --project examples/toi-noi-python/08-exception-debug-logging examples/toi-noi-python/08-exception-debug-logging/main.py -100 --unhandled
```

```text
Traceback (most recent call last):
  File ".../08-exception-debug-logging/main.py", line 131, in <module>
    main()
    ~~~~^^
  File ".../08-exception-debug-logging/main.py", line 127, in main
    demo(args)
    ~~~~^^^^^^
  File ".../08-exception-debug-logging/main.py", line 37, in demo
    run_once(raw, buggy=args.buggy, unhandled=args.unhandled, use_pdb=args.pdb)
    ~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File ".../08-exception-debug-logging/main.py", line 28, in run_once
    total = fare.quote_from_metres(raw, buggy=buggy, pause=use_pdb)  # lỗi nào cũng bay thẳng ra, in traceback
  File ".../08-exception-debug-logging/fare.py", line 60, in quote_from_metres
    total = calculate_fare(km)
  File ".../08-exception-debug-logging/fare.py", line 46, in calculate_fare
    raise DistanceOutOfRangeError(f"khoảng cách ngoài miền 0..{MAX_KM} km: {distance_km}")
fare.DistanceOutOfRangeError: khoảng cách ngoài miền 0..100 km: -0.1
```

Các dấu `~` và `^` dưới mỗi dòng là Python chỉ ra biểu thức đang chạy khi lỗi xảy ra; frame `<module>` là lời gọi đầu tiên của script. Đọc từ dưới lên: exception là `DistanceOutOfRangeError`, khoảng cách -0,1 km; nó bị `raise` ở `calculate_fare`, được gọi từ `quote_from_metres`, được gọi từ `run_once`. Ba frame đầu (`<module>`, `main`, `demo`) chỉ là khung của script; frame đáng đọc nhất là ba frame cuối. Traceback chỉ cho biết **nơi từ chối**; câu hỏi tiếp theo là ai đưa -0,1 km vào hàm.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Traceback đọc từ dưới lên: dòng cuối là loại exception và thông điệp, phía trên là nơi raise, rồi lần lượt các hàm gọi, tới lời gọi đầu tiên."><defs><marker id="py08-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="15" width="560" height="52" rx="8" class="f-bg s-line"/><text x="34" y="47" class="t-mono">main.py  run_once  →  quote_from_metres(...)</text><rect x="20" y="77" width="560" height="52" rx="8" class="f-bg s-line"/><text x="34" y="109" class="t-mono">fare.py  quote_from_metres  →  calculate_fare(km)</text><rect x="20" y="139" width="560" height="52" rx="8" class="f-warn-soft s-warn"/><text x="34" y="171" class="t-mono">fare.py  calculate_fare  →  raise ...</text><rect x="20" y="201" width="560" height="52" rx="8" class="f-pk-soft s-pk"/><text x="34" y="233" class="t-mono t-b">fare.DistanceOutOfRangeError: ... -0.1</text><path d="M620 227 L620 40" class="f-none s-warn" stroke-width="2" marker-end="url(#py08-1-arrow)"/><text x="640" y="48" class="t-b">1. Đọc dòng cuối trước</text><text x="640" y="72" class="t-small">loại lỗi và thông điệp</text><text x="640" y="110" class="t-b">2. Dòng raise ngay trên đó</text><text x="640" y="134" class="t-small">nơi từ chối, không phải nguyên nhân</text><text x="640" y="172" class="t-b">3. Đi lên theo chuỗi gọi</text><text x="640" y="196" class="t-small">ai truyền dữ liệu vào hàm</text><text x="640" y="234" class="t-small">ưu tiên frame code của bạn</text></svg>
</div>
<figcaption>Hình 1. Traceback đọc từ dưới lên. Mũi tên chỉ hướng đọc; frame trên cùng là lời gọi đầu tiên của chương trình.</figcaption>
</figure>

## try, except, else, finally có trách nhiệm khác nhau

Hàm `handle_request()` của lab dùng đủ bốn khối:

```python
# fare.py
def handle_request(raw_metres: str, *, trip_id: str = "TN-0001", buggy: bool = False, pause: bool = False) -> str:
    logger.info("nhận yêu cầu báo giá trip=%s", trip_id)
    try:
        total = quote_from_metres(raw_metres, buggy=buggy, pause=pause)
    except DistanceOutOfRangeError as exc:
        logger.warning("từ chối trip=%s: %s", trip_id, exc)
        return "REJECT"
    except DeviceDataError:
        logger.exception("dữ liệu thiết bị hỏng trip=%s", trip_id)
        return "DEVICE_ERROR"
    else:
        logger.info("báo giá thành công trip=%s total=%s", trip_id, total)
        return f"{total} VND"
    finally:
        logger.debug("kết thúc xử lý trip=%s", trip_id)
```

- `try` chỉ bao đoạn có thể ném lỗi mà ta định xử lý; càng rộng, càng dễ bắt nhầm lỗi của dòng khác.
- `except` chọn **đúng loại** exception tầng này biết cách đáp ứng. Python duyệt các `except` từ trên xuống và dùng khối đầu tiên khớp, nên lớp con phải đứng trước lớp cha.
- `else` chạy khi `try` không có exception. Việc ghi log thành công đặt ở đây, thay vì trong `try`, để lỗi của chính dòng log không bị `except` phía trên nuốt nhầm.
- `finally` chạy dù `try` thành công, dù có lỗi bị bắt, dù có `return` giữa chừng. Nó dành cho dọn dẹp (đóng file, nhả khoá). Nó không đảm bảo chạy khi tiến trình bị buộc dừng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Luồng try, except, else, finally. Không có lỗi thì chạy else rồi finally. Có lỗi khớp một nhánh except thì chạy nhánh đó rồi finally."><defs><marker id="py08-2-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="110" width="170" height="80" rx="10" class="f-phys-soft s-phys"/><text x="34" y="140" class="t-b">try</text><text x="34" y="165" class="t-small">quote_from_metres</text><rect x="320" y="20" width="230" height="80" rx="10" class="f-warn-soft s-warn"/><text x="334" y="50" class="t-b">except đúng loại</text><text x="334" y="75" class="t-small">có lỗi, khớp một nhánh</text><rect x="320" y="200" width="230" height="80" rx="10" class="f-phys-soft s-phys"/><text x="334" y="230" class="t-b">else</text><text x="334" y="255" class="t-small">không có lỗi</text><rect x="680" y="110" width="180" height="80" rx="10" class="f-pk-soft s-pk"/><text x="694" y="140" class="t-b">finally</text><text x="694" y="165" class="t-small">luôn chạy</text><path d="M190 135 L320 70" class="f-none s-warn" stroke-width="2" marker-end="url(#py08-2-arrow)"/><path d="M190 165 L320 230" class="f-none s-phys" stroke-width="2" marker-end="url(#py08-2-arrow)"/><path d="M550 60 L680 130" class="f-none s-warn" stroke-width="2" marker-end="url(#py08-2-arrow)"/><path d="M550 240 L680 170" class="f-none s-phys" stroke-width="2" marker-end="url(#py08-2-arrow)"/><text x="190" y="88" class="t-small">exception</text><text x="190" y="215" class="t-small">thành công</text></svg>
</div>
<figcaption>Hình 2. Luồng try, except, else, finally. Mọi nhánh đều kết thúc ở finally; nếu exception không khớp except nào, nó vẫn chạy finally rồi tiếp tục bay lên tầng gọi.</figcaption>
</figure>

Hai điều cần tránh. Một: `except:` trần hoặc `except Exception:` rồi tiếp tục như không có gì. Cả hai bắt lỗi lập trình (`NameError`, `TypeError`) lẫn lỗi dữ liệu và gộp vào cùng một nhánh; `except:` trần còn bắt cả `KeyboardInterrupt` và `SystemExit`, nghĩa là chặn luôn Ctrl+C và `sys.exit()`, còn `except Exception` thì không. Hai: `except Exception: return 0` biến lỗi thành báo giá miễn phí, nhập nhằng ba trạng thái (miễn phí thật, đầu vào sai, hệ thống hỏng). Hàm tính cước nhận khoảng cách âm là vi phạm hợp đồng, nên ném lỗi ở đó là đúng.

## Exception tuỳ biến, `raise from` và chaining

Lab định nghĩa một cây exception nhỏ cho nghiệp vụ cước:

```python
class FareError(ValueError):
    """Gốc của mọi lỗi nghiệp vụ tính cước."""

class DistanceOutOfRangeError(FareError): ...
class DeviceDataError(FareError): ...
```

Kế thừa `ValueError` giúp code cũ đang bắt `ValueError` vẫn bắt được lỗi mới, trong khi tầng gọi có thể chọn mức chính xác: `except DistanceOutOfRangeError` cho một ca cụ thể, hay `except FareError` cho mọi lỗi nghiệp vụ. Thông điệp lỗi ghi giá trị nhìn thấy, không chứa dữ liệu nhạy cảm.

Khi bắt một lỗi cấp thấp rồi ném lỗi có ý nghĩa nghiệp vụ hơn, dùng `raise ... from exc` để **giữ nguyên nhân**:

```python
def parse_device_metres(raw: str) -> Decimal:
    try:
        return Decimal(raw)
    except InvalidOperation as exc:
        raise DeviceDataError(f"khoảng cách không đọc được: {raw!r}") from exc
```

`from exc` lưu lỗi gốc vào thuộc tính `__cause__` của exception mới. Khi in traceback, Python nối hai phần bằng câu «The above exception was the direct cause of the following exception», nên người đọc thấy cả `InvalidOperation` gốc lẫn `DeviceDataError`. Nếu `raise` exception khác trong `except` mà không có `from`, Python vẫn gắn lỗi cũ vào `__context__` và in «During handling of the above exception, another exception occurred»: lỗi thứ hai xảy ra *trong lúc xử lý* lỗi thứ nhất, không nhất thiết do nó gây ra. Còn `raise` trần trong `except` ném lại đúng exception đang xử lý, giữ nguyên traceback. Chạy lab với đầu vào hỏng để xem log kèm traceback có chaining (đường dẫn rút gọn; dòng cuối là kết quả `print` ở stdout):

```bash
uv run --project examples/toi-noi-python/08-exception-debug-logging examples/toi-noi-python/08-exception-debug-logging/main.py abc
```

```text
2026-11-06 13:31:22,583 INFO    fare: nhận yêu cầu báo giá trip=TN-0001
2026-11-06 13:31:22,583 ERROR   fare: dữ liệu thiết bị hỏng trip=TN-0001
Traceback (most recent call last):
  File ".../08-exception-debug-logging/fare.py", line 32, in parse_device_metres
    return Decimal(raw)
decimal.InvalidOperation: [<class 'decimal.ConversionSyntax'>]

The above exception was the direct cause of the following exception:

Traceback (most recent call last):
  File ".../08-exception-debug-logging/fare.py", line 71, in handle_request
    total = quote_from_metres(raw_metres, buggy=buggy, pause=pause)
  File ".../08-exception-debug-logging/fare.py", line 58, in quote_from_metres
    metres = parse_device_metres(raw_metres)
  File ".../08-exception-debug-logging/fare.py", line 34, in parse_device_metres
    raise DeviceDataError(f"khoảng cách không đọc được: {raw!r}") from exc
fare.DeviceDataError: khoảng cách không đọc được: 'abc'
abc m => DEVICE_ERROR
```

## `assert` không dùng cho validation

`assert điều_kiện, "thông điệp"` ném `AssertionError` khi điều kiện sai. Nó phục vụ khẳng định điều lập trình viên tin là luôn đúng (bất biến nội bộ). Điểm mấu chốt: khi chạy Python với cờ `-O`, mọi `assert` bị **bỏ qua**. Hàm `unsafe_guard` trong lab cố ý dùng `assert` để kiểm tra đầu vào và chứng minh điều đó:

```bash
uv run --directory examples/toi-noi-python/08-exception-debug-logging python -O -c "from fare import unsafe_guard; print(unsafe_guard(-5))"
uv run --directory examples/toi-noi-python/08-exception-debug-logging python -c "from fare import unsafe_guard; print(unsafe_guard(-5))"
```

```text
-5
```

Lệnh thứ hai ra traceback (trích dòng cuối) và không in `-5`:

```text
AssertionError: khoảng cách phải >= 0
```

Cùng một đầu vào, với `-O` hàm chấp nhận -5 km và trả về như hợp lệ. Vì thế dữ liệu đến từ bên ngoài (người dùng, thiết bị, file) phải được kiểm bằng `if ... raise` như `calculate_fare()` làm. Trong `calculate_fare()` vẫn có một dòng `assert total >= BASE_FARE`: đó là kiểm tra điều ta tin là đúng, và nếu nó bị bỏ qua thì hành vi đúng của hàm không đổi. `--verify` xác nhận cả hai hành vi bằng hai tiến trình con.

## Logging cho người vận hành

`print` đủ cho bài tập, nhưng service chạy lúc nửa đêm cần **log**: có thời điểm, mức độ, nơi phát sinh, và có thể bật tắt không cần sửa code. Quy tắc chuẩn của thư viện `logging`: mỗi module lấy logger riêng bằng `logger = logging.getLogger(__name__)`; chỉ **điểm vào của chương trình** cấu hình `basicConfig`, các module thư viện không tự cấu hình. Trong lab, `main.py` làm việc đó. Lưu ý `basicConfig` không làm gì nếu root logger đã có handler (ví dụ một thư viện đã cấu hình trước); khi đó thêm `force=True` để cấu hình lại:

```python
# main.py
logging.basicConfig(
    level=args.log_level.upper(),
    format="%(asctime)s %(levelname)-7s %(name)s: %(message)s",
)
```

Chạy với mức mặc định `INFO` và mức `DEBUG` cho cùng đầu vào 2500 (timestamp thay đổi mỗi lần chạy; dòng cuối mỗi khối là kết quả `print`, mức `INFO` bỏ khỏi khối thứ hai):

```text
2026-11-06 13:31:28,342 INFO    fare: nhận yêu cầu báo giá trip=TN-0001
2026-11-06 13:31:28,342 INFO    fare: báo giá thành công trip=TN-0001 total=14250
2500 m => 14250 VND
```

```text
2026-11-06 13:31:28,030 INFO    fare: nhận yêu cầu báo giá trip=TN-0001
2026-11-06 13:31:28,030 DEBUG   fare: quote_from_metres metres=2500 km=2.5 total=14250
2026-11-06 13:31:28,030 INFO    fare: báo giá thành công trip=TN-0001 total=14250
2026-11-06 13:31:28,030 DEBUG   fare: kết thúc xử lý trip=TN-0001
2500 m => 14250 VND
```

Cột thứ ba là tên logger, nhờ `getLogger(__name__)` nên biết dòng log đến từ module nào. Với mức `INFO`, các dòng `DEBUG` ẩn đi. Nếu sự cố đầu bài xảy ra ở môi trường thật, log `INFO` chỉ ghi `báo giá thành công total=25500`: một dòng ghi nhận **thành công**, không có dấu hiệu lỗi. Bật `DEBUG` tạm thời thì dòng `quote_from_metres metres=2500 km=5 ...` làm lệch đơn vị lộ ra ngay.

Một số quy ước:

- Dùng `logger.info("... %s", giá_trị)` thay vì f-string: chỉ định dạng khi dòng log thực sự được ghi.
- Chọn mức: `DEBUG` để chẩn đoán, `INFO` cho sự kiện bình thường, `WARNING` cho điều bất thường nhưng đã xử lý (từ chối đầu vào sai), `ERROR` khi một thao tác thất bại.
- Trong khối `except`, `logger.exception("...")` ghi mức `ERROR` kèm traceback hiện hành. Ngoài `except`, nó vẫn ghi mức `ERROR` nhưng không có traceback thật: log chỉ có dòng `NoneType: None` gây nhầm lẫn, nên chỉ gọi nó trong `except`.
- **Không đưa dữ liệu nhạy cảm vào log**: số điện thoại, giấy tờ, token, số thẻ. Log đi qua nhiều hệ thống và nhiều người đọc. Ghi mã tương quan như `TN-0001` hay `CU-001`; nếu bắt buộc ghi định danh, che bớt như hàm `mask_phone("0912345678")` của lab trả về `09******78`.

Phân biệt hai vai trò: **exception là tín hiệu điều khiển luồng**, buộc tầng gọi quyết định (từ chối, thử lại, dừng); **log là bản ghi cho người vận hành**, không đổi hành vi chương trình. Bắt exception rồi chỉ `logger.error` và chạy tiếp thường là che lỗi bằng cách khác. Bắt ở ranh giới, ghi log rồi trả mã kết quả có nghĩa như `REJECT` là hợp lý, vì tầng này đã quyết định.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Giá sai nhưng không có exception | Đơn vị, hằng số đổi đơn vị, thứ tự phép tính; so với bảng kiểm 0, 2, 2,5, 5 km |
| Traceback chỉ thấy chỗ `raise` | Đi ngược chuỗi gọi: ai truyền giá trị này; `p` giá trị bằng `pdb` |
| Log không hiện dòng `DEBUG` | Đã gọi `basicConfig` với `level=DEBUG` chưa; logger khác có đặt mức riêng không |
| Log không có traceback | Có dùng `logger.exception` trong `except` hay chỉ `logger.error` |
| Validation biến mất khi triển khai | Có dùng `assert` để kiểm đầu vào không; chạy bản production có thể thêm `-O` |

## Chạy, kiểm chứng và giới hạn của ví dụ

```bash
L=examples/toi-noi-python/08-exception-debug-logging
uv run --project $L $L/main.py
uv run --project $L $L/main.py --verify
```

Lệnh đầu chạy ba ca: 2500 mét (14250 VND), -100 mét (REJECT) và `abc` (DEVICE_ERROR, log có traceback chaining). Lệnh thứ hai chạy 21 kiểm tra: bảng cước, bản `--buggy` tái hiện 25.500, từng loại exception đúng lớp, `__cause__`, số dòng log theo mức, hành vi `assert` với `-O`. Kết quả cuối là `OK 21/21`; có ca sai thì in tên ca, dòng `FAIL n/21` và thoát với mã 1.

Lab dùng chuỗi mét tự sinh và một lỗi có chủ đích; lỗi thật không tự giới thiệu bằng cờ. Log ghi ra stderr bằng `basicConfig`; service thật dùng log có cấu trúc (JSON), gom về hệ thống tập trung và gắn mã tương quan theo từng yêu cầu. Không để `breakpoint()` trong code triển khai, vì nó chặn tiến trình chờ nhập lệnh. `--verify` là bộ kiểm tra tự viết cho lab, chưa phải kiểm thử có khung: bài 10 giới thiệu `pytest` (kể cả `pytest.raises` cho các ca ném lỗi) và áp dụng nó cho chính sách cước.

**Bài tập:** thiết bị gửi `"2,5"` (dấu phẩy, kiểu địa phương) thay vì `"2500"`. Hàm `quote_from_metres("2,5")` hiện ném lỗi gì, ở dòng nào, và `handle_request` trả gì? Có nên đoán `"2,5"` là 2,5 km không?

<details><summary>Xem lời giải</summary><p><code>Decimal("2,5")</code> ném <code>InvalidOperation</code> trong <code>parse_device_metres</code>, hàm này bọc thành <code>DeviceDataError</code> bằng <code>raise ... from exc</code>, và <code>handle_request</code> ghi <code>logger.exception</code> rồi trả <code>DEVICE_ERROR</code>. Không nên đoán: hợp đồng với thiết bị là chuỗi số mét, tự diễn giải dấu phẩy là thêm quy tắc ngầm và có thể sai đơn vị lần nữa. Nếu cần hỗ trợ định dạng này, hãy thêm hàm phân tích riêng có kiểm thử.</p></details>

Bài trước: [bài 07](/articles/python-module-package-import). Bài tiếp: [bài 09](/articles/python-file-json-csv-pathlib).

## Tài liệu gốc

- [Python Tutorial: Errors and Exceptions](https://docs.python.org/3.14/tutorial/errors.html).
- [pdb: The Python Debugger](https://docs.python.org/3.14/library/pdb.html).
- [logging: Logging facility for Python](https://docs.python.org/3.14/library/logging.html).
- [Logging HOWTO](https://docs.python.org/3.14/howto/logging.html).
- [Python Language Reference: Simple statements (assert, raise)](https://docs.python.org/3.14/reference/simple_stmts.html).
