# Master Curriculum — Python qua dự án Tới Nơi

Bản quản lý loạt bài **Python** của Tech Knowledge Hub (antblog.xyz). Python là ngôn ngữ của các service **OCR giấy tờ định danh**, **so khớp khuôn mặt (face matching)**, **chấm điểm gian lận (fraud scoring)** và **phân tích / dự đoán** trong kiến trúc microservices của Tới Nơi (xem `toi-noi-overall-architecture.md` và bài `du-an-mau-toi-noi`). Loạt bài đi từ cú pháp cơ bản tới service chạy được trong kiến trúc đó.

Gồm **3 cấp độ**, **10 phần**, **79 bài**. Mỗi phần tối đa 10 bài. Mỗi cấp độ là một lộ trình học (learning path) riêng trên trang đọc.

| Cấp độ | Phần | Số bài | Lộ trình (slug) | Kết quả mong muốn |
|---|---|---:|---|---|
| Cơ bản | P01–P02 | 20 | `python-co-ban-qua-toi-noi` | Viết, kiểm thử và đóng gói một chương trình Python có kiểu, có mô hình nghiệp vụ |
| Trung bình | P03–P04 | 20 | `python-trung-binh-qua-toi-noi` | Viết service HTTP/gRPC/Kafka bất đồng bộ nối PostgreSQL, MongoDB, Redis, có integration test |
| Nâng cao | P05–P10 | 39 | `python-nang-cao-qua-toi-noi` | Xây OCR, face matching, fraud scoring và dự đoán thành service theo kiến trúc đích, có đo đạc và vận hành |

## 1. Cách dùng

1. **Người viết**: chọn bài theo bảng §3, đọc dòng mô tả và «Cần chứng minh» làm đề cương, viết theo template §2.2 và khung biên tập `examples/toi-noi-python/README.md`. Code minh họa đặt tại `examples/toi-noi-python/<NN-slug-ngan>/` và phải chạy được bằng `uv run`.
2. **Người đọc**: đi theo cột «Đọc trước». Mỗi phần nối tiếp phần trước; trong một phần các bài đọc theo số thứ tự.
3. **Theo dõi tiến độ**: cột «Trạng thái» (`Kế hoạch` → `Đang viết` → `Đã xuất bản`). Khi một bài xuất bản: thêm `.md` vào `api/seed/content`, thêm slug vào `api/seed/learning-paths.json`, thêm ảnh bìa vào `apps/web/src/lib/article-covers.json` rồi chạy `npm run covers:generate -w @tkh/web`.

## 2. Quy ước

### 2.1 Chung

| Mục | Quy ước |
|---|---|
| Mã bài | `PY-01` … `PY-79`, tăng dần theo phần. |
| Mã phần | `P01` … `P10`. |
| Slug | Chữ thường ASCII, gạch nối, bắt đầu bằng `python-`, ≤ 60 ký tự, duy nhất. |
| Category | `programming` (giống loạt C#). Tags luôn có `python`, `toi-noi`, thêm tag chủ đề (`fastapi`, `asyncio`, `ocr`, `fraud`…). |
| Level | `beginner` (P01–P02) · `intermediate` (P03–P04) · `advanced` (P05–P10). Có thể lệch một bậc khi bài thực sự khó/dễ hơn phần. |
| Phiên bản | **Python 3.14** (bản ổn định hiện tại), quản lý bằng **uv** (`uv python install 3.14`, `uv run`). Không dùng tính năng còn ở bản rc. Free-threaded build chỉ nhắc ở bài 22 với phạm vi thử nghiệm. |
| Thư viện chính | pytest · pydantic v2 · FastAPI · SQLAlchemy 2 + psycopg 3 · pymongo/motor · redis-py · aiokafka · grpcio · NumPy · OpenCV · Pillow · scikit-learn · LightGBM · polars · OpenTelemetry. Phiên bản pin trong `pyproject.toml` của từng lab. |
| Tiền tệ, thời gian | VND dùng `Decimal`, thời điểm lưu UTC (`datetime` aware), dữ liệu test giả. |
| Dữ liệu nhạy cảm | Ảnh giấy tờ, khuôn mặt trong lab là **dữ liệu tổng hợp** (tự vẽ/sinh), không dùng ảnh người thật. Kết quả OCR, điểm fraud là đầu vào cho quy trình nghiệp vụ, không tự động là quyết định. |
| Thời lượng đọc | 12–25 phút; bài tổng hợp cuối phần dài hơn. |

### 2.2 Template một bài (giống loạt C#)

1. Frontmatter: `slug`, `title`, `summary` (bắt đầu bằng «Bài NN: …»), `category: programming`, `level`, `tags`, `language: vi`.
2. Mở bài bằng một tình huống thật của Tới Nơi (1–2 đoạn), câu «Đây là bài NN của lộ trình …».
3. Callout «Thử trước khi đọc» (`tkh-callout tkh-callout--example`) với một câu hỏi và `<details>` đáp án.
4. 4–7 mục `##` giải thích cơ chế; ít nhất **một hình SVG nội tuyến** theo bộ lớp `packages/ui/src/article.css` (`f-phys-soft`, `s-phys`, `t-b`, `t-small`, `f-pk-soft`, `f-warn-soft`…), có `role="img"`, `aria-label`, `figcaption` «Hình N. …».
5. Code chạy được (`python`, `bash`, `toml`), bảng «Lỗi thường gặp / Kiểm tra trước» khi phù hợp.
6. Callout «Giới hạn của ví dụ» hoặc đoạn nói rõ demo chưa có gì so với production.
7. **Bài tập** có `<details>` lời giải.
8. «Bài trước / Bài tiếp» (chỉ link bài đã viết) và mục «Tài liệu gốc» (docs.python.org, PEP, tài liệu thư viện chính thức).

### 2.3 Bối cảnh nghiệp vụ dùng xuyên suốt

- **Tính cước** (P01–P02): cùng mô hình với loạt C# (giá cơ bản 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt) để người đọc hai loạt đối chiếu.
- **Hồ sơ tài xế (KYC)**: tài xế nộp ảnh CCCD/GPLX và ảnh chân dung → OCR trích trường → so khớp khuôn mặt với ảnh giấy tờ → chấm điểm rủi ro → nhân viên duyệt. Service `kyc-ocr`, `kyc-face` thuộc bounded context Driver/Partner Profile (§4.1 kiến trúc).
- **Gian lận**: khách/tài xế tạo chuyến ảo để nhận khuyến mãi, GPS giả, nhiều tài khoản một thiết bị, cấu kết tài xế–khách. Service `fraud-scoring` nhận event `BookingCreated`, `TripCompleted`, `PaymentCaptured` từ Kafka và trả điểm qua gRPC cho Dispatch/Promotion.
- **Phân tích, dự đoán**: ETA, nhu cầu theo khu vực, mô phỏng hệ số giờ cao điểm; chạy batch từ dữ liệu chuyến (730 triệu dòng/năm theo hồ sơ Tới Nơi).

## 3. Danh sách bài

### Cấp độ cơ bản · Lộ trình `python-co-ban-qua-toi-noi`

#### P01 · Nền tảng Python (10 bài)

| Mã | Slug | Tiêu đề | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|
| PY-01 | `python-va-uv-tu-source-code-den-chuong-trinh` | Python 3.14 và uv: từ source code đến chương trình tính cước | Interpreter, bytecode, venv, `uv init/run`, cấu trúc `pyproject.toml`; chương trình tính cước đầu tiên | — | Đã viết |
| PY-02 | `python-bien-kieu-du-lieu-decimal` | Biến, kiểu dữ liệu và Decimal: biểu diễn tiền và khoảng cách | int/float/Decimal/bool/str, dynamic typing, `0.1+0.2`, `Decimal` + `ROUND_HALF_UP`, chuyển đổi kiểu | PY-01 | Đã viết |
| PY-03 | `python-dieu-kien-vong-lap-ham` | Điều kiện, vòng lặp và hàm: tách quy tắc tính cước | if/elif, for/while, `range`, `enumerate`, hàm, tham số mặc định, keyword-only, docstring, `--verify` | PY-02 | Đã viết |
| PY-04 | `python-chuoi-f-string-ngay-gio-timezone` | Chuỗi, f-string, ngày giờ và múi giờ: hiển thị đúng, lưu đúng | str/bytes, f-string định dạng, `datetime` aware/naive, `zoneinfo` Asia/Ho_Chi_Minh, ISO 8601, UTC | PY-02 | Đã viết |
| PY-05 | `python-list-dict-set-tuple` | List, dict, set, tuple: quản lý chuyến và tài xế trong bộ nhớ | Mutability, hashing, dict là ordered, `collections` (Counter, defaultdict, deque), độ phức tạp cơ bản | PY-03 | Đã viết |
| PY-06 | `python-comprehension-generator-iterator` | Comprehension, iterator và generator: dòng vị trí tài xế không cần nạp hết | Iterator protocol, `yield`, lazy evaluation, `itertools`, bộ nhớ khi đọc 6.250 dòng/giây | PY-05 | Đã viết |
| PY-07 | `python-module-package-import` | Module, package và import: tổ chức mã tính cước thành gói | `import` hoạt động thế nào, `__init__.py`, `__name__`, package tuyệt đối/tương đối, `src` layout, `uv add` | PY-03 | Đã viết |
| PY-08 | `python-exception-debug-logging` | Exception, debug và logging: lần theo một báo giá sai | try/except/else/finally, exception tùy biến, traceback, `pdb`/`breakpoint()`, `logging` cấu hình chuẩn | PY-03 | Đã viết |
| PY-09 | `python-file-json-csv-pathlib` | File, JSON, CSV và pathlib: đọc bộ dữ liệu mẫu Tới Nơi | `pathlib`, context manager mở file, encoding UTF-8, `json`/`csv`, Decimal trong JSON, dữ liệu mẫu chuyến | PY-05 | Đã viết |
| PY-10 | `python-pytest-kiem-thu-chinh-sach-cuoc` | Unit test với pytest: kiểm thử chính sách cước | Cấu trúc test, assert, `parametrize`, fixture, `pytest.raises`, giá trị biên, chạy `uv run pytest` | PY-03, PY-07 | Đã viết |

#### P02 · Mô hình hóa nghiệp vụ bằng Python (10 bài)

| Mã | Slug | Tiêu đề | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|
| PY-11 | `python-class-dataclass-property-encapsulation` | Class, dataclass, property và encapsulation: bảo vệ trạng thái chuyến | `__init__`, `self`, `@dataclass`, `@property`, `_private`, `__slots__`, máy trạng thái chuyến 8 trạng thái | PY-10 | Đã viết |
| PY-12 | `python-type-hints-mypy-protocol-generics` | Type hints, mypy/pyright, Protocol và generics: hợp đồng kiểu cho mã nghiệp vụ | PEP 484/695, `list[T]`, `TypedDict`, `Protocol`, `TypeVar` cú pháp mới, annotation lười (PEP 649) trong 3.14 | PY-11 | Đã viết |
| PY-13 | `python-frozen-dataclass-enum-immutability` | Frozen dataclass, Enum, equality và immutability: thiết kế Money, Quote | `frozen=True`, `__eq__`/`__hash__`, `Enum`/`StrEnum`, `NamedTuple`, value object, quote có hạn | PY-11 | Đã viết |
| PY-14 | `python-abc-protocol-composition-dependency-injection` | ABC, Protocol, composition và dependency injection: thay chính sách cước | Duck typing vs ABC, `Protocol` structural, inject qua constructor, không cần framework DI | PY-12 | Đã viết |
| PY-15 | `python-pydantic-validation-settings` | Pydantic v2: validate hồ sơ tài xế và cấu hình ứng dụng | `BaseModel`, validator, `model_validate`, `BaseSettings`, lỗi có cấu trúc, khác dataclass ở đâu | PY-12 | Đã viết |
| PY-16 | `python-match-pattern-matching-none` | Pattern matching và xử lý None: chuyển trạng thái chuyến rõ ràng | `match/case` với class/mapping/guard, `Optional`, `None` không phải giá trị thiếu mặc định | PY-13 | Đã viết |
| PY-17 | `python-decorator-context-manager` | Decorator và context manager: retry, đo thời gian, transaction | Closure, `functools.wraps`, decorator có tham số, `contextlib`, `with` cho tài nguyên | PY-14 | Đã viết |
| PY-18 | `python-functools-itertools-pipeline` | functools, itertools và pipeline hàm: xử lý lô chuyến | `partial`, `reduce`, `lru_cache`, `groupby`, `chain`, pipeline generator, so với LINQ | PY-06, PY-17 | Đã viết |
| PY-19 | `python-design-pattern-strategy-state-adapter` | Design pattern kiểu Python: Strategy, State, Adapter cho Tới Nơi | Pattern GoF khi ngôn ngữ có first-class function; khi không cần class; liên kết loạt Design Pattern | PY-14, PY-16 | Đã viết |
| PY-20 | `python-dong-goi-ruff-pre-commit-cau-truc-du-an` | Đóng gói, ruff, mypy và cấu trúc dự án: sẵn sàng cho service | `pyproject.toml` đầy đủ, `uv lock`, ruff format/lint, mypy strict, `src/` layout, entry point, bài tổng hợp P01–P02 | PY-07, PY-10 | Đã viết |

### Cấp độ trung bình · Lộ trình `python-trung-binh-qua-toi-noi`

#### P03 · Bất đồng bộ, dữ liệu và lưu trữ (10 bài)

| Mã | Slug | Tiêu đề | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|
| PY-21 | `python-asyncio-coroutine-task-timeout-cancellation` | asyncio: coroutine, Task, timeout và hủy: gọi dịch vụ báo giá | Event loop, `await`, `gather`, `TaskGroup`, `asyncio.timeout`, cancellation, không chặn loop | PY-20 | Đã viết |
| PY-22 | `python-gil-threading-multiprocessing-free-threaded` | GIL, threading, multiprocessing và free-threaded 3.14: chọn mô hình cho OCR | I/O-bound vs CPU-bound, `ThreadPoolExecutor`, `ProcessPoolExecutor`, đo thật, free-threaded là thử nghiệm | PY-21 | Đã viết |
| PY-23 | `python-concurrency-lock-queue-semaphore` | Lock, Queue, Semaphore: tránh nhận trùng chuyến | Race condition tái hiện, `asyncio.Lock`, `Semaphore` giới hạn, `Queue` producer/consumer, backpressure | PY-22 | Đã viết |
| PY-24 | `python-sqlalchemy-psycopg-postgresql-mapping-migration` | SQLAlchemy 2 và psycopg 3 với PostgreSQL: mapping, migration Alembic | ORM 2.0 style, `Mapped[]`, session, Alembic, đọc SQL sinh ra, async engine | PY-20 | Đã viết |
| PY-25 | `python-transaction-optimistic-concurrency-idempotency` | Transaction, optimistic concurrency và idempotency: đặt chuyến an toàn | `begin()`, isolation, cột version, unique idempotency key, SQLSTATE 23505, retry | PY-24 | Đã viết |
| PY-26 | `python-mongodb-pymongo-motor-document-ocr` | MongoDB với pymongo/motor: lưu kết quả OCR dạng document | Document model, index, schema versioning, `insert_one`/`find`, khi nào chọn document thay bảng | PY-24 | Đã viết |
| PY-27 | `python-redis-cache-rate-limit-geo` | Redis: cache, rate limit và GEO cho vị trí tài xế | redis-py async, TTL, cache-aside, token bucket, `GEOADD/GEOSEARCH`, pipeline, không dùng Redis làm nguồn sự thật | PY-21 | Đã viết |
| PY-28 | `python-integration-test-testcontainers-fixture` | Integration test với PostgreSQL/MongoDB thật: testcontainers và fixture | Fixture scope, testcontainers, schema riêng mỗi ca, không InMemory thay PostgreSQL, CI | PY-24, PY-26 | Đã viết |
| PY-29 | `python-polars-pandas-parquet-phan-tich-chuyen` | polars, pandas và Parquet: phân tích 2 triệu chuyến mỗi ngày | DataFrame, lazy query, group by, Parquet vs CSV, bộ nhớ, khi nào không nên pandas | PY-09 | Đã viết |
| PY-30 | `python-numpy-pillow-opencv-tien-xu-ly-anh` | NumPy, Pillow và OpenCV: ảnh là mảng, tiền xử lý ảnh giấy tờ | ndarray, dtype, broadcasting, đọc/ghi ảnh, resize, grayscale, threshold, vectorize thay vòng lặp | PY-29 | Đã viết |

#### P04 · Dịch vụ HTTP, gRPC và thông điệp (10 bài)

| Mã | Slug | Tiêu đề | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|
| PY-31 | `python-fastapi-http-endpoint-bao-gia` | HTTP và FastAPI: xuất bản endpoint báo giá | ASGI, uvicorn, route, request/response model, status code, OpenAPI tự sinh | PY-21, PY-15 | Đã viết |
| PY-32 | `python-fastapi-validation-problem-details-openapi` | Validation, lỗi có cấu trúc (Problem Details) và OpenAPI: hợp đồng API | Lỗi 422 chuẩn, `application/problem+json`, exception handler, version API, tài liệu | PY-31 | Đã viết |
| PY-33 | `python-fastapi-dependency-middleware-settings-logging` | Dependency, middleware, cấu hình và structured logging | `Depends`, lifespan, middleware, `BaseSettings`, structlog/JSON log, correlation id | PY-31 | Đã viết |
| PY-34 | `python-fastapi-jwt-oauth2-authorization` | Authentication và authorization: khách, tài xế, điều phối | OAuth2 password/bearer, JWT ký/verify, scope, role, không tự viết crypto, token tối thiểu | PY-33 | Đã viết |
| PY-35 | `python-fastapi-upload-background-task-streaming` | Upload ảnh giấy tờ, background task và streaming | `UploadFile`, giới hạn kích thước/loại, lưu object storage giả, `BackgroundTasks` vs worker, SSE | PY-33 | Đã viết |
| PY-36 | `python-fastapi-api-test-httpx` | API test với httpx AsyncClient: kiểm chứng hợp đồng | `TestClient`/`AsyncClient`, override dependency, test auth, test lỗi, snapshot OpenAPI | PY-35, PY-28 | Đã viết |
| PY-37 | `python-grpc-fraud-scoring-service` | gRPC với grpcio: service chấm điểm gian lận cho Dispatch | Protobuf, codegen, server/client async, deadline, status code, interceptor, khi dùng gRPC thay REST | PY-33 | Đã viết |
| PY-38 | `python-kafka-aiokafka-producer-consumer-schema` | Kafka với aiokafka: producer, consumer và schema sự kiện | Topic/partition/consumer group, serialization, schema version, at-least-once, offset commit | PY-23 | Đã viết |
| PY-39 | `python-outbox-inbox-idempotent-consumer` | Outbox, Inbox và idempotent consumer trong Python | Dual-write, outbox trong transaction SQLAlchemy, dispatcher, inbox unique key, đối chiếu bài 26 loạt C# | PY-25, PY-38 | Đã viết |
| PY-40 | `python-worker-arq-scheduling-retry-dead-letter` | Worker nền với arq/taskiq: lịch, retry, dead-letter | Job queue trên Redis, retry backoff, DLQ, graceful shutdown, idempotent job, bài tổng hợp P03–P04 | PY-27, PY-39 | Đã viết |

### Cấp độ nâng cao · Lộ trình `python-nang-cao-qua-toi-noi`

#### P05 · OCR giấy tờ định danh (7 bài)

| Mã | Slug | Tiêu đề | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|
| PY-41 | `python-kyc-tai-xe-bai-toan-du-lieu-metric` | Bài toán KYC tài xế: luồng nghiệp vụ, dữ liệu tổng hợp và metric | Luồng nộp hồ sơ → OCR → face → duyệt; sinh ảnh giấy tờ tổng hợp; CER/WER, precision theo trường; ranh giới service | PY-40 | Kế hoạch |
| PY-42 | `python-opencv-phat-hien-canh-perspective-deskew` | Phát hiện cạnh, biến đổi phối cảnh và deskew ảnh giấy tờ | Canny, contour, `warpPerspective`, xoay, nhị phân hóa thích nghi, đo ảnh hưởng tới OCR | PY-30, PY-41 | Kế hoạch |
| PY-43 | `python-tesseract-easyocr-paddleocr-so-sanh` | Tesseract, EasyOCR và PaddleOCR: chạy và so sánh trên cùng bộ ảnh | Cài, gọi, ngôn ngữ vi, bounding box, confidence, thời gian/CPU, bảng so sánh có số đo | PY-42 | Kế hoạch |
| PY-44 | `python-hau-xu-ly-ocr-regex-parse-truong-checksum` | Hậu xử lý OCR: regex, tách trường CCCD/GPLX, checksum và fuzzy matching | Chuẩn hóa Unicode, regex có tên, `rapidfuzz`, kiểm tra định dạng số, ngày, confidence per field | PY-43 | Kế hoạch |
| PY-45 | `python-danh-gia-ocr-cer-wer-threshold` | Đánh giá OCR: CER/WER, bộ test vàng và ngưỡng chấp nhận | Tập ground truth, `jiwer`, phân tích lỗi theo loại, ngưỡng đưa người duyệt, regression test cho mô hình | PY-44 | Kế hoạch |
| PY-46 | `python-ocr-service-fastapi-worker-timeout` | Đóng gói OCR thành service: FastAPI nhận ảnh, worker xử lý, timeout | Tách API/worker, giới hạn ảnh, timeout, trạng thái job, idempotency theo hồ sơ, tải thử | PY-45, PY-40 | Kế hoạch |
| PY-47 | `python-luu-ket-qua-ocr-mongodb-pii-retention` | Lưu kết quả OCR: document MongoDB, mặt nạ PII, versioning và retention | Schema document, mask số giấy tờ, TTL index, phiên bản mô hình trong document, audit | PY-46, PY-26 | Kế hoạch |

#### P06 · So khớp khuôn mặt (6 bài)

| Mã | Slug | Tiêu đề | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|
| PY-48 | `python-face-detection-landmarks-mediapipe-opencv` | Face detection và landmarks: tìm khuôn mặt trong ảnh giấy tờ và selfie | OpenCV DNN / MediaPipe, bounding box, căn chỉnh theo landmark, chất lượng ảnh | PY-47 | Kế hoạch |
| PY-49 | `python-face-embedding-cosine-threshold-far-frr` | Embedding và so khớp: cosine, ngưỡng, FAR/FRR và ROC | Mô hình embedding (InsightFace/ONNX), cosine similarity, chọn ngưỡng từ ROC, chi phí sai | PY-48 | Kế hoạch |
| PY-50 | `python-liveness-anti-spoofing-chat-luong-anh` | Liveness và chống giả mạo cơ bản: ảnh in, màn hình, mờ, tối | Blur/brightness metric, heuristic đơn giản, giới hạn rõ của bài, khi cần nhà cung cấp chuyên | PY-48 | Kế hoạch |
| PY-51 | `python-vector-search-pgvector-faiss-trung-tai-xe` | Vector search với pgvector và FAISS: phát hiện một người nhiều tài khoản | Lưu embedding, HNSW/IVF, top-k, tích hợp PostgreSQL, giới hạn độ chính xác | PY-49, PY-24 | Kế hoạch |
| PY-52 | `python-face-match-grpc-service-batching-model-loading` | Service face match qua gRPC: nạp mô hình, batching, CPU/GPU | Nạp mô hình một lần, warm-up, batch inference, deadline, health, ONNX Runtime | PY-49, PY-37 | Kế hoạch |
| PY-53 | `python-face-matching-danh-gia-bias-privacy-consent` | Đánh giá, thiên lệch, quyền riêng tư và sự đồng ý | Đánh giá theo nhóm, lưu embedding thay ảnh, retention, đồng ý, quyết định cuối do người | PY-52 | Kế hoạch |

#### P07 · Phát hiện gian lận (8 bài)

| Mã | Slug | Tiêu đề | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|
| PY-54 | `python-fraud-toi-noi-bai-toan-nhan-du-lieu-metric` | Gian lận trong Tới Nơi: loại hình, nhãn, dữ liệu và metric | Chuyến ảo, GPS giả, lạm dụng khuyến mãi, cấu kết; nhãn trễ; precision/recall/PR-AUC; bộ dữ liệu tổng hợp | PY-40 | Kế hoạch |
| PY-55 | `python-feature-engineering-tu-event-kafka` | Feature engineering: từ event Kafka tới đặc trưng velocity, thiết bị, GPS | Cửa sổ thời gian, đếm theo khóa, khoảng cách haversine, tốc độ phi lý, leakage | PY-54, PY-38 | Kế hoạch |
| PY-56 | `python-rule-engine-scoring-strategy-explainability` | Rule engine và điểm rủi ro: Strategy, cấu hình và giải thích được | Rule có trọng số, cấu hình YAML, lý do kèm điểm, kiểm thử rule, khi rule đủ | PY-55, PY-19 | Kế hoạch |
| PY-57 | `python-scikit-learn-lightgbm-imbalanced-cross-validation` | scikit-learn và LightGBM: huấn luyện trên dữ liệu mất cân bằng | Pipeline, train/valid theo thời gian, class weight, threshold tuning, calibration, lưu mô hình | PY-56 | Kế hoạch |
| PY-58 | `python-networkx-graph-cau-ket-tai-xe-khach` | Đồ thị với networkx: tìm cấu kết tài xế–khách | Đồ thị hai phía, thành phần liên thông, độ trung tâm, thiết bị dùng chung, quy mô | PY-55 | Kế hoạch |
| PY-59 | `python-real-time-scoring-feature-store-redis-latency` | Chấm điểm thời gian thực: feature store trên Redis, ngân sách độ trễ, fallback | Online/offline feature, p99, timeout trả điểm mặc định, không chặn Dispatch | PY-57, PY-27 | Kế hoạch |
| PY-60 | `python-model-registry-ab-drift-retraining` | Model registry, A/B, drift và huấn luyện lại | Phiên bản mô hình (MLflow), shadow mode, PSI/drift, lịch retrain, rollback | PY-59 | Kế hoạch |
| PY-61 | `python-shap-explainability-audit-human-review` | SHAP, audit và quy trình người duyệt | Giải thích từng dự đoán, log quyết định, hàng đợi review, tỉ lệ sai cho phép | PY-60 | Kế hoạch |

#### P08 · Phân tích và dự đoán (5 bài)

| Mã | Slug | Tiêu đề | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|
| PY-62 | `python-eta-prediction-regression` | Dự đoán ETA: hồi quy từ dữ liệu chuyến | Feature khoảng cách/giờ/khu vực, baseline, MAE, so với routing provider | PY-57 | Kế hoạch |
| PY-63 | `python-demand-forecasting-time-series-khu-vuc` | Dự báo nhu cầu theo khu vực: time series | Mùa vụ ngày/tuần, lag feature, backtest, 6 thành phố × khung giờ | PY-62 | Kế hoạch |
| PY-64 | `python-surge-pricing-simulation` | Mô phỏng hệ số giờ cao điểm: cung–cầu và cước | Mô hình mô phỏng, tham số, kiểm tra tác động lên cước, không tự động áp giá | PY-63 | Kế hoạch |
| PY-65 | `python-batch-pipeline-duckdb-parquet-orchestration` | Batch pipeline với DuckDB, Parquet và orchestrator (Prefect/Dagster) | Phân vùng theo ngày, incremental, idempotent run, lịch, 730 triệu dòng/năm | PY-29 | Kế hoạch |
| PY-66 | `python-notebook-den-production-code` | Từ notebook tới mã production | Tách hàm, test, cấu hình, không hard-code đường dẫn, review notebook | PY-65 | Kế hoạch |

#### P09 · Kiến trúc service Python theo kiến trúc đích (5 bài)

| Mã | Slug | Tiêu đề | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|
| PY-67 | `python-clean-architecture-domain-application-infrastructure` | Clean Architecture trong Python: domain, application, infrastructure, api | Package theo lớp, chiều phụ thuộc, import-linter, Protocol làm port | PY-40 | Kế hoạch |
| PY-68 | `python-cqrs-command-query-handler-mediator` | CQRS và mediator: command/query handler, pipeline không cần thư viện nặng | Dispatch handler theo kiểu, pipeline (log, validate, transaction), so với MediatR | PY-67 | Kế hoạch |
| PY-69 | `python-repository-unit-of-work-sqlalchemy` | Repository và Unit of Work với SQLAlchemy | Repository theo aggregate, UoW bao session/transaction, test với PostgreSQL thật | PY-68, PY-25 | Kế hoạch |
| PY-70 | `python-use-case-ho-so-tai-xe-end-to-end` | Use case nộp hồ sơ tài xế end-to-end: OCR → face → điểm rủi ro → quyết định | Ghép P05–P07 qua use case, domain event, trạng thái hồ sơ, test toàn luồng | PY-69, PY-47, PY-53, PY-61 | Kế hoạch |
| PY-71 | `python-domain-event-outbox-integration-event-version` | Domain event, outbox và version hợp đồng sự kiện | Domain vs integration event, outbox trong UoW, schema version, tương thích với consumer C# | PY-70, PY-39 | Kế hoạch |

#### P10 · Hiệu năng, vận hành và tổng hợp (8 bài)

| Mã | Slug | Tiêu đề | Cần chứng minh | Đọc trước | Trạng thái |
|---|---|---|---|---|---|
| PY-72 | `python-profiling-cprofile-py-spy-tracemalloc` | Profiling: cProfile, py-spy, tracemalloc: tìm chỗ chậm của OCR | Đo trước khi tối ưu, flame graph, bộ nhớ, so sánh có số | PY-46 | Kế hoạch |
| PY-73 | `python-toi-uu-numpy-vectorize-multiprocessing-rust-extension` | Tối ưu: vectorize NumPy, multiprocessing và extension Rust (maturin) | Khi Python đủ, khi cần C/Rust, đo gain, chi phí bảo trì, liên kết loạt Rust | PY-72, PY-22 | Kế hoạch |
| PY-74 | `python-dockerfile-uv-multi-stage-model-file` | Container: Dockerfile đa stage với uv, image nhỏ, file mô hình | Base image, lock, non-root, cache layer, mô hình ngoài image, health | PY-20 | Kế hoạch |
| PY-75 | `python-ci-cd-github-actions-pytest-ruff-mypy` | CI/CD: GitHub Actions chạy pytest, ruff, mypy và scan phụ thuộc | Matrix, cache uv, service container PostgreSQL, artifact, pip-audit | PY-74, PY-28 | Kế hoạch |
| PY-76 | `python-trien-khai-aws-azure-secrets-health` | Triển khai AWS và Azure: cấu hình, secrets, health check, scale | ECS/Container Apps, biến môi trường, secrets manager, readiness, autoscale theo queue | PY-75 | Kế hoạch |
| PY-77 | `python-opentelemetry-trace-xuyen-csharp-grpc` | OpenTelemetry: lần theo một yêu cầu từ C# sang Python qua gRPC | Auto-instrument FastAPI/gRPC/SQLAlchemy, propagation W3C, metric, log tương quan | PY-76, PY-37 | Kế hoạch |
| PY-78 | `python-config-feature-flag-graceful-shutdown-sigterm` | Cấu hình, feature flag và graceful shutdown | 12-factor, flag bật mô hình mới, SIGTERM, drain queue, timeout tắt | PY-77 | Kế hoạch |
| PY-79 | `python-tong-hop-kyc-fraud-pipeline-tu-code-toi-van-hanh` | Bài tổng hợp: pipeline KYC và fraud từ code tới vận hành | Repo lab hoàn chỉnh, hợp đồng với service C#, kiểm chứng end-to-end, SLO, những gì lab chưa có | PY-71, PY-78 | Kế hoạch |

## 4. Chuỗi phụ thuộc chính

- Cước và mô hình nghiệp vụ: PY-01 → 03 → 10 → 11 → 13 → 14 → 19 → 67 → 68 → 69 → 70.
- Bất đồng bộ và thông điệp: PY-21 → 23 → 38 → 39 → 40 → 71.
- Ảnh và OCR: PY-30 → 42 → 43 → 44 → 45 → 46 → 47 → 70.
- Khuôn mặt: PY-48 → 49 → 52 → 53 → 70.
- Gian lận: PY-54 → 55 → 56 → 57 → 59 → 60 → 61 → 70.
- Vận hành: PY-74 → 75 → 76 → 77 → 78 → 79.

Đồ thị «Đọc trước» không có chu trình; mọi bài P05–P10 có thể đọc sau khi hoàn thành P04.

## 5. Điểm cần chủ blog quyết

1. **Thư viện OCR**: Tesseract có sẵn qua Homebrew/apt, dễ chạy trong CI; EasyOCR/PaddleOCR cần tải mô hình lớn. Bài 43 so sánh, các bài sau mặc định Tesseract để lab chạy được trong CI, có ghi rõ hạn chế với tiếng Việt.
2. **Mô hình face embedding**: dùng mô hình ONNX công khai (InsightFace buffalo_l) tải lúc chạy, không commit file mô hình vào repo. Nếu muốn lab offline hoàn toàn, cần quyết định nơi lưu mô hình.
3. **Dữ liệu tổng hợp**: ảnh giấy tờ và khuôn mặt trong lab đều sinh bằng code (Pillow vẽ mẫu, khuôn mặt từ bộ dữ liệu có giấy phép mở). Không dùng ảnh người thật.
4. **Orchestrator (bài 65)**: chọn Prefect hay Dagster; đề xuất Prefect vì chạy local nhẹ hơn.
