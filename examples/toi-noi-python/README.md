# Python qua dự án Tới Nơi

Khung biên tập cho loạt bài Python của Tech Knowledge Hub. Danh sách 79 bài, phụ thuộc và trạng thái nằm trong [`docs/python-master-curriculum.md`](../../docs/python-master-curriculum.md). Mỗi bài có câu hỏi mở đầu, hình SVG giải thích cơ chế, ví dụ chạy được, lỗi thường gặp, bài tập có đáp án và nguồn chính thức. Tách rõ code minh họa với giải pháp production. Link sang bài khác theo đúng slug trong curriculum; loạt bài chỉ seed khi các bài được link đã có mặt.

Đối tượng cấp cơ bản: người mới biết dùng terminal, chưa cần biết Python. Người đã đọc loạt C# sẽ gặp lại cùng mô hình cước, cùng chuyến xe để đối chiếu hai ngôn ngữ.

## Vai trò của Python trong Tới Nơi

C#/.NET giữ các backend nghiệp vụ chính (đặt chuyến, đơn hàng, điều phối). Python giữ **OCR giấy tờ định danh**, **so khớp khuôn mặt**, **chấm điểm gian lận** và **phân tích/dự đoán**. Các service Python nhận event từ Kafka, trả kết quả qua gRPC/REST, sở hữu dữ liệu riêng (PostgreSQL, MongoDB cho document OCR, Redis cho feature online). Kết quả OCR và điểm fraud là đầu vào cho quy trình nghiệp vụ, không tự động là quyết định xác minh hay kết luận gian lận. Xem bài `du-an-mau-toi-noi`, mục «Đa ngôn ngữ theo trách nhiệm service».

## Quy ước xuyên suốt

- **Python 3.14** quản lý bằng **uv**: `uv python install 3.14`, mỗi bài là một project độc lập có `pyproject.toml` (`requires-python = ">=3.14"`), chạy bằng `uv run`. Không cài gì vào Python hệ thống.
- Không dùng tính năng còn ở bản rc. Free-threaded build chỉ nhắc như thử nghiệm ở bài 22.
- Tiền VND dùng `Decimal` với `ROUND_HALF_UP`; thời điểm aware và lưu UTC; dữ liệu test giả. Mô hình cước học tập giống loạt C#: giá cơ bản 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt; không phụ phí, thuế, giảm giá.
- Mã chuyến `TN-0001`, tài xế `DR-001`, khách `CU-001`, thành phố theo hồ sơ Tới Nơi (6 thành phố). 8 trạng thái chuyến: requested, searching, accepted, arriving, picked_up, in_transit, completed, cancelled.
- Code trong bài là code trong thư mục lab tương ứng, có chú thích tiếng Việt, ghi tên file. Mỗi project cấp cơ bản có chế độ `--verify` (hoặc `pytest` từ bài 10) kiểm các kết quả kỳ vọng in trong bài.
- Ảnh giấy tờ, khuôn mặt (từ phần 05) là dữ liệu tổng hợp sinh bằng code. Không commit ảnh người thật, không commit file mô hình lớn, không có credential thật.
- Hình minh họa: SVG nội tuyến dùng bộ lớp `packages/ui/src/article.css` (`f-phys-soft s-phys t-b t-small f-pk-soft s-pk f-warn-soft s-warn f-bg s-line dash`), `viewBox` rộng 880, `role="img"`, `aria-label`, caption «Hình N. …». Ảnh bìa khai báo trong `apps/web/src/lib/article-covers.json`, sinh bằng `npm run covers:generate -w @tkh/web`.

## Cấu trúc một bài

1. Frontmatter: `slug`, `title`, `summary` («Bài NN: …», ≤ 234 ký tự), `category: python`, `level`, `tags` (`python`, `toi-noi`, thêm chủ đề), `language: vi`.
2. Mở bài bằng một tình huống của Tới Nơi, câu «Đây là bài NN của lộ trình **Python cơ bản qua Tới Nơi**» (hoặc trung bình / nâng cao) với link `[Tới Nơi](/articles/du-an-mau-toi-noi)`.
3. Callout «Thử trước khi đọc» với `<details>` đáp án.
4. 4–7 mục `##`, ít nhất một hình SVG, code chạy được, bảng «Hiện tượng / Kiểm tra trước» khi phù hợp.
5. Đoạn hoặc callout nói rõ giới hạn của ví dụ so với production.
6. **Bài tập** có `<details>` lời giải.
7. Dòng «Bài trước: … Bài tiếp: …» chỉ link bài đã viết; mục «Tài liệu gốc» trỏ docs.python.org, PEP hoặc tài liệu chính thức của thư viện.

## P01 · Nền tảng Python

01. Python 3.14 và uv: từ source code đến chương trình tính cước — **đã viết**
02. Biến, kiểu dữ liệu và Decimal: biểu diễn tiền và khoảng cách — **đã viết**
03. Điều kiện, vòng lặp và hàm: tách quy tắc tính cước — **đã viết**
04. Chuỗi, f-string, ngày giờ và múi giờ: hiển thị đúng, lưu đúng — **đã viết**
05. List, dict, set, tuple: quản lý chuyến và tài xế trong bộ nhớ — **đã viết**
06. Comprehension, iterator và generator: dòng vị trí tài xế không cần nạp hết — **đã viết**
07. Module, package và import: tổ chức mã tính cước thành gói — **đã viết**
08. Exception, debug và logging: lần theo một báo giá sai — **đã viết**
09. File, JSON, CSV và pathlib: đọc bộ dữ liệu mẫu Tới Nơi — **đã viết**
10. Unit test với pytest: kiểm thử chính sách cước — **đã viết**

## P02 · Mô hình hóa nghiệp vụ

11. Class, dataclass, property và encapsulation: bảo vệ trạng thái chuyến — **đã viết**
12. Type hints, mypy/pyright, Protocol và generics: hợp đồng kiểu cho mã nghiệp vụ — **đã viết**
13. Frozen dataclass, Enum, equality và immutability: thiết kế Money, Quote — **đã viết**
14. ABC, Protocol, composition và dependency injection: thay chính sách cước — **đã viết**
15. Pydantic v2: validate hồ sơ tài xế và cấu hình ứng dụng — **đã viết**
16. Pattern matching và xử lý None: chuyển trạng thái chuyến rõ ràng — **đã viết**
17. Decorator và context manager: retry, đo thời gian, transaction — **đã viết**
18. functools, itertools và pipeline hàm: xử lý lô chuyến — **đã viết**
19. Design pattern kiểu Python: Strategy, State, Adapter cho Tới Nơi — **đã viết**
20. Đóng gói, ruff, mypy và cấu trúc dự án: sẵn sàng cho service — **đã viết**

## P03 · Bất đồng bộ, dữ liệu và lưu trữ

21. asyncio: coroutine, Task, timeout và hủy: gọi dịch vụ báo giá — **đã viết**
22. GIL, threading, multiprocessing và free-threaded 3.14: chọn mô hình cho OCR — **đã viết**
23. Lock, Queue, Semaphore: tránh nhận trùng chuyến — **đã viết**
24. SQLAlchemy 2 và psycopg 3 với PostgreSQL: mapping, migration Alembic — **đã viết**
25. Transaction, optimistic concurrency và idempotency: đặt chuyến an toàn — **đã viết**
26. MongoDB với pymongo/motor: lưu kết quả OCR dạng document — **đã viết**
27. Redis: cache, rate limit và GEO cho vị trí tài xế — **đã viết**
28. Integration test với PostgreSQL/MongoDB thật: testcontainers và fixture — **đã viết**
29. polars, pandas và Parquet: phân tích 2 triệu chuyến mỗi ngày — **đã viết**
30. NumPy, Pillow và OpenCV: ảnh là mảng, tiền xử lý ảnh giấy tờ — **đã viết**

## P04 · Dịch vụ HTTP, gRPC và thông điệp

31. HTTP và FastAPI: xuất bản endpoint báo giá — **đã viết**
32. Validation, lỗi có cấu trúc (Problem Details) và OpenAPI: hợp đồng API — **đã viết**
33. Dependency, middleware, cấu hình và structured logging — **đã viết**
34. Authentication và authorization: khách, tài xế, điều phối — **đã viết**
35. Upload ảnh giấy tờ, background task và streaming — **đã viết**
36. API test với httpx AsyncClient: kiểm chứng hợp đồng — **đã viết**
37. gRPC với grpcio: service chấm điểm gian lận cho Dispatch — **đã viết**
38. Kafka với aiokafka: producer, consumer và schema sự kiện — **đã viết**
39. Outbox, Inbox và idempotent consumer trong Python — **đã viết**
40. Worker nền với arq/taskiq: lịch, retry, dead-letter — **đã viết**

## P05 · OCR giấy tờ định danh

41. Bài toán KYC tài xế: luồng nghiệp vụ, dữ liệu tổng hợp và metric — **đã viết**
42. Phát hiện cạnh, biến đổi phối cảnh và deskew ảnh giấy tờ — **đã viết**
43. Tesseract, EasyOCR và PaddleOCR: chạy và so sánh trên cùng bộ ảnh — **đã viết**
44. Hậu xử lý OCR: regex, tách trường CCCD/GPLX, checksum và fuzzy matching — **đã viết**
45. Đánh giá OCR: CER/WER, bộ test vàng và ngưỡng chấp nhận — **đã viết**
46. Đóng gói OCR thành service: FastAPI nhận ảnh, worker xử lý, timeout — **đã viết**
47. Lưu kết quả OCR: document MongoDB, mặt nạ PII, versioning và retention — **đã viết**

## P06 · So khớp khuôn mặt

48. Face detection và landmarks: tìm khuôn mặt trong ảnh giấy tờ và selfie — **đã viết**
49. Embedding và so khớp: cosine, ngưỡng, FAR/FRR và ROC — **đã viết**
50. Liveness và chống giả mạo cơ bản: ảnh in, màn hình, mờ, tối — **đã viết**
51. Vector search với pgvector và FAISS: phát hiện một người nhiều tài khoản — **đã viết**
52. Service face match qua gRPC: nạp mô hình, batching, CPU/GPU — **đã viết**
53. Đánh giá, thiên lệch, quyền riêng tư và sự đồng ý — **đã viết**

Các phần P07–P10 (bài 54–79) xem master curriculum.

## Chạy lab phần 01

Cần uv và Python 3.14 (`uv python install 3.14`). Mỗi thư mục `NN-…` có README ghi lệnh chạy, lệnh `--verify` và kết quả kỳ vọng; lab không cần mạng khi chạy (bài 10 cần mạng ở lần resolve pytest đầu tiên).

```bash
uv run --project examples/toi-noi-python/01-python-va-uv examples/toi-noi-python/01-python-va-uv/main.py
uv run --project examples/toi-noi-python/01-python-va-uv examples/toi-noi-python/01-python-va-uv/main.py --verify
sh examples/toi-noi-python/verify-01-10.sh
```

`verify-01-10.sh` chạy `--verify` của 9 lab console và pytest của bài 10. Mọi `--verify` in `OK n/n` khi đạt, `FAIL n/m` và exit 1 khi lệch, với n là số ca đạt; các con số in trong bài là output của chính lab.

## Chạy lab phần 02

Lần đầu cần mạng để uv tải mypy, pytest, pydantic, ruff. Lab 20 có `verify.sh` riêng gồm ruff, mypy, pytest, `uv build` và thử công cụ bắt lỗi.

```bash
sh examples/toi-noi-python/verify-11-20.sh
```

## Chạy lab phần 03

Bài 24–27 cần hạ tầng chung trong `infra/`, bài 28 cần Docker. Khởi động trước bằng `docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait`. Số đo thời gian trong bài in kèm load average; `--verify` chỉ kiểm bất biến.

```bash
sh examples/toi-noi-python/verify-21-30.sh
```

## Chạy lab phần 04

Bài 31–37 chạy trong tiến trình, không cần Docker. Bài 38–40 cần hạ tầng chung trong `infra/`. Bài 34 không có giá trị mặc định cho môi trường và khóa ký, nên chạy server tay cần đặt `TOINOI_ENVIRONMENT` và `TOINOI_JWT_SECRET`; xem README của lab.

```bash
sh examples/toi-noi-python/verify-31-40.sh
```

## Chạy lab phần 05

Cần Tesseract có gói `vie` (`brew install tesseract tesseract-lang`) và hạ tầng chung cho bài 46, 47. Lab 41, 42 chạy vài phút. PaddleOCR ở bài 43 cần Python 3.13 qua project con `paddle/`; `--verify` chỉ cần Tesseract. Ảnh giấy tờ đều là dữ liệu tổng hợp.

```bash
sh examples/toi-noi-python/verify-41-47.sh
```

## Chạy lab phần 06

Bài 51 cần hạ tầng chung trong `infra/`. Lần chạy đầu cần mạng để tải bộ Olivetti (AT&T Laboratories Cambridge, qua scikit-learn) và các mô hình YuNet, SFace, MediaPipe vào cache; repo không chứa ảnh người hay file mô hình. InsightFace chỉ là tùy chọn `--with-arcface` của lab 49 và có giới hạn giấy phép, xem bài 49.

```bash
sh examples/toi-noi-python/verify-48-53.sh
```
