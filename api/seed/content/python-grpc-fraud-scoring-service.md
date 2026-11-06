---
slug: python-grpc-fraud-scoring-service
title: "gRPC với grpcio: service chấm điểm gian lận cho Dispatch"
summary: "Bài 37: dựng service fraud-scoring bằng grpcio: hợp đồng protobuf, sinh mã, server grpc.aio, client có deadline, status code, interceptor, và khi nào chọn gRPC thay REST. Lab chạy thật, --verify 41 ca."
category: python
level: intermediate
tags: [python, toi-noi, grpc, protobuf, fraud-scoring]
language: vi
---

Mỗi lần khách bấm đặt chuyến, Dispatch của [Tới Nơi](/articles/du-an-mau-toi-noi) muốn biết một điều trước khi ghép tài xế: chuyến này có đáng để người duyệt xem lại không. Câu trả lời đến từ service fraud-scoring viết bằng Python, vì đặc trưng và mô hình nằm ở đó. Ở giờ cao điểm (khoảng 70 chuyến mỗi giây) lời gọi này nằm ngay trên đường đặt chuyến, nên nó cần ba thứ mà một endpoint JSON ghép vội không cho sẵn: hợp đồng kiểu chặt dùng chung giữa C# và Python, giới hạn chờ rõ ràng, và mã lỗi phân biệt «đầu vào sai» với «service đang tắc».

Đây là bài 37 của lộ trình **Python trung bình qua Tới Nơi**. Bài 31 đến 33 dựng service HTTP bằng FastAPI; bài này làm đường gọi nội bộ bằng gRPC rồi so hai cách. Bạn cần quen `async`/`await` và hủy task ([bài 21](/articles/python-asyncio-coroutine-task-timeout-cancellation)), cùng `Semaphore` ([bài 23](/articles/python-concurrency-lock-queue-semaphore)). Điểm trả về là đầu vào cho người duyệt, không phải quyết định hay kết luận gian lận: Dispatch không từ chối khách chỉ vì một con số.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Dispatch gọi <code>Score</code> với giới hạn chờ 200 ms. Phía server, bước tra cứu đặc trưng mất 1 giây. Sau 200 ms client nhận lỗi. Server có tiếp tục chạy hết 1 giây rồi vứt kết quả không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Hết hạn thì handler bị hủy: nó nhận <code>asyncio.CancelledError</code> ngay tại chỗ đang <code>await</code> và không bao giờ chạy tới dòng chấm điểm. Điều đáng để ý: client nhận <code>DEADLINE_EXCEEDED</code>, còn handler chỉ thấy <code>CancelledError</code>, không thấy mã đó. Bài chạy đúng ca này và đếm.</p></details>
</div>

## gRPC khác gì một endpoint REST

gRPC là framework gọi hàm từ xa chạy trên HTTP/2, dùng Protocol Buffers (protobuf) làm định dạng thông điệp. Hai ý quan trọng hơn mọi chi tiết khác. Thứ nhất, **hợp đồng đi trước**: file `.proto` mô tả service và kiểu dữ liệu, công cụ sinh mã tạo stub cho client và khung servicer cho server ở nhiều ngôn ngữ. Dispatch (C#) và fraud-scoring (Python) sinh mã từ cùng một file, nên đổi hợp đồng là phải sinh lại mã, và trình kiểm tra kiểu (mypy, pyright) thấy chỗ gọi sai kiểu trước khi chạy. Thứ hai, **giới hạn chờ là khái niệm gốc**: deadline đi theo lời gọi sang server.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Một file .proto sinh stub cho Dispatch viết bằng C# và khung servicer cho fraud-scoring viết bằng Python; Dispatch gửi ScoreRequest qua HTTP/2 và nhận ScoreResponse."><defs><marker id="py37-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="320" y="16" width="240" height="64" rx="10" class="f-warn-soft s-warn"/><text x="336" y="42" class="t-b">fraud_scoring.proto</text><text x="336" y="64" class="t-small">service, message, số hiệu field</text><rect x="20" y="150" width="250" height="110" rx="10" class="f-pk-soft s-pk"/><text x="36" y="182" class="t-b">Dispatch (C#)</text><text x="36" y="208" class="t-small">stub sinh từ .proto</text><text x="36" y="228" class="t-small">gọi Score(request, deadline)</text><rect x="610" y="150" width="250" height="110" rx="10" class="f-phys-soft s-phys"/><text x="626" y="182" class="t-b">fraud-scoring (Python)</text><text x="626" y="208" class="t-small">servicer sinh từ .proto</text><text x="626" y="228" class="t-small">grpc.aio, chấm điểm theo luật</text><path d="M360 80 L200 150" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py37-0-arrow)"/><path d="M520 80 L680 150" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py37-0-arrow)"/><text x="196" y="108" class="t-small">sinh mã C#</text><text x="610" y="108" class="t-small">sinh mã Python</text><path d="M270 188 L610 188" class="f-none s-phys" stroke-width="2" marker-end="url(#py37-0-arrow)"/><path d="M610 232 L270 232" class="f-none s-phys" stroke-width="2" marker-end="url(#py37-0-arrow)"/><text x="350" y="178" class="t-small">ScoreRequest (HTTP/2, protobuf)</text><text x="372" y="256" class="t-small">ScoreResponse hoặc status</text></svg>
</div>
<figcaption>Hình 1. Một hợp đồng, hai ngôn ngữ. Mã ở hai đầu được sinh từ cùng một file nên không lệch nhau về kiểu.</figcaption>
</figure>

Đặt cạnh REST ở bài 31 ta được bảng sau; không bên nào «tốt hơn», chỉ khác chỗ dùng:

| | REST + JSON (bài 31) | gRPC + protobuf (bài này) |
| --- | --- | --- |
| Hợp đồng | OpenAPI sinh từ code hoặc viết tay | `.proto` viết trước, sinh mã hai đầu |
| Dây dẫn | JSON dạng chữ, tên field lặp mỗi bản ghi | nhị phân, field gọi bằng số hiệu |
| Giới hạn chờ | tự đặt timeout từng client | deadline mặc định không có, đặt tường minh và truyền sang server |
| Công cụ | trình duyệt, `curl`, cache HTTP | cần client gRPC; trình duyệt không gọi trực tiếp |

Dispatch gọi fraud-scoring từ bên trong hệ thống, nhiều lời gọi, hai ngôn ngữ, cần deadline: hợp với gRPC. API công khai cho ứng dụng khách, nơi người ta muốn `curl` và cache: giữ REST.

## Hợp đồng protobuf và việc sinh mã

Đây là toàn bộ hợp đồng, `proto/fraud_scoring.proto` (bỏ phần chú thích):

```proto
syntax = "proto3";
package toinoi.fraud.v1;

service FraudScoring {
  rpc Score(ScoreRequest) returns (ScoreResponse);
}

message ScoreRequest {
  string trip_id = 1;
  string customer_id = 2;
  optional string driver_id = 3;
  string device_id = 4;
  string distance_km = 5;
}

message ScoreResponse {
  double score = 1;
  repeated string reasons = 2;
  string model_version = 3;
}
```

Vài quyết định nhỏ đều có lý do. **Số hiệu field** (`= 1`, `= 2`) mới là thứ nằm trên dây, tên chỉ phục vụ mã sinh ra; đã phát hành thì không đổi số, không tái dùng số của field đã bỏ (đánh dấu `reserved`). `distance_km` là chuỗi thập phân giống `Money` ở các bài trước, vì `double` không biểu diễn đúng 2,5 hay 0,1; riêng `score` dùng `double` vì đó là điểm xấp xỉ, không phải tiền. `driver_id` là `optional`: lúc đặt chuyến có thể chưa có tài xế, và proto3 chỉ phân biệt «chưa gán» với «chuỗi rỗng» khi field có `optional`.

Sinh mã bằng `grpcio-tools`; từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/37-grpc-fraud-scoring python -m grpc_tools.protoc \
  -I examples/toi-noi-python/37-grpc-fraud-scoring/proto \
  --python_out=examples/toi-noi-python/37-grpc-fraud-scoring \
  --pyi_out=examples/toi-noi-python/37-grpc-fraud-scoring \
  --grpc_python_out=examples/toi-noi-python/37-grpc-fraud-scoring \
  examples/toi-noi-python/37-grpc-fraud-scoring/proto/fraud_scoring.proto
```

Ra ba file: `fraud_scoring_pb2.py` (message), `fraud_scoring_pb2.pyi` (kiểu cho trình kiểm tra) và `fraud_scoring_pb2_grpc.py` (stub và khung servicer). Ta commit chúng, và `--verify` sinh lại vào thư mục tạm rồi so từng byte để biết file đã commit còn khớp `.proto`. Một chỗ vướng quen thuộc: file `_grpc` import tuyệt đối `import fraud_scoring_pb2`, nên lab để file sinh cạnh `main.py`; dự án lớn thường đặt `.proto` theo đường dẫn gói (`toinoi/fraud/v1/`) để mã sinh import đúng gói. Mã sinh còn ghi phiên bản công cụ đã dùng (1.85.1), nên `pyproject.toml` pin `grpcio>=1.85.1` và `protobuf>=7.35.1`; thiếu thì import báo lỗi ngay thay vì chạy sai.

Dữ liệu trên dây nhỏ và có quy tắc tương thích. Trích output của lab:

```text
ScoreRequest: 40 byte protobuf, 108 byte JSON gọn
hex: 0a 07 54 4e 2d 30 30 30 31 12 06 43 55 2d ...  (0a = field 1, kiểu chuỗi; 07 = dài 7 byte)
driver_id chưa gán: HasField=False; gán chuỗi rỗng: HasField=True
thêm field số 15 lạ vào cuối: vẫn đọc được, trip_id=TN-0001, 42 byte
```

Mỗi field là một cặp (số hiệu và kiểu, độ dài) rồi nội dung: `0a 07` là «field 1, dài 7 byte», theo sau là `TN-0001`. JSON gọn của cùng dữ liệu là 108 byte vì lặp tên field mỗi bản ghi; ở khoảng 70 chuyến mỗi giây (hơn 250.000 mỗi giờ) khoảng chênh này có nghĩa, nhưng đừng kỳ vọng thần kỳ: nội dung chuỗi chiếm phần lớn cả hai. Dòng cuối là điểm mạnh về tiến hóa: bên gửi phiên bản mới thêm field số 15, bên nhận cũ vẫn đọc được phần nó biết, và còn giữ field lạ khi ghi lại.

## Server grpc.aio

Bộ luật chấm điểm của lab là luật viết tay, đủ để minh họa hợp đồng, không phải mô hình thật. Phần tính nằm trong một hàm thuần để test riêng (trích `service.py`):

```python
def score_trip(request, distance, store):
    score, reasons = 0.0, []
    others = store.device_customers.get(request.device_id, set()) - {request.customer_id}
    if len(others) >= 3:  # thiết bị này đã dùng cho từ 3 khách khác trở lên
        score += W_DEVICE_SHARED
        reasons.append("device_shared")
    if request.HasField("driver_id"):  # proto3 optional: phân biệt "chưa có" với chuỗi rỗng
        if store.pair_trips_24h.get((request.driver_id, request.customer_id), 0) >= 3:
            score += W_PAIR_REPEAT
            reasons.append("pair_repeat")
    if distance >= 50:
        score += W_LONG_TRIP
        reasons.append("long_trip")
    return round(min(score, 1.0), 2), reasons
```

`reasons` là mã ổn định cho máy và cho người duyệt đọc, không phải câu văn. `FeatureStore` là dict giả; ở hệ thống thật, đặc trưng đến từ Redis hoặc PostgreSQL như bài 24 và bài 27. Servicer là một lớp kế thừa khung sinh ra, và `async def` là đủ để chạy trên `grpc.aio`:

```python
class FraudScoringService(pb_grpc.FraudScoringServicer):
    async def Score(self, request: pb.ScoreRequest, context: grpc.aio.ServicerContext):
        if not request.trip_id or not request.customer_id:
            await context.abort(grpc.StatusCode.INVALID_ARGUMENT, "thiếu trip_id hoặc customer_id")
        try:
            distance = parse_distance(request.distance_km)
        except ValueError as exc:
            await context.abort(grpc.StatusCode.INVALID_ARGUMENT, str(exc))
        try:
            await asyncio.sleep(self._store.lookup_delay)   # điểm chờ I/O của tra cứu đặc trưng
            score, reasons = score_trip(request, distance, self._store)
            self._store.completed += 1
        except asyncio.CancelledError:
            self._store.cancelled += 1
            raise
        return pb.ScoreResponse(score=score, reasons=reasons, model_version=MODEL_VERSION)
```

Kiểm đầu vào ở server dù client đã có kiểu: `distance_km` là chuỗi, và client nào cũng có thể gửi `"abc"`. `context.abort` ném một exception đặc biệt kết thúc lời gọi với mã và thông báo; lỗi nghiệp vụ của người gọi là `INVALID_ARGUMENT`, và thông báo giữ đúng chữ «khoảng cách ngoài miền 0..100 km» của loạt bài. Hai bộ đếm `completed` và `cancelled` chỉ để lab chứng minh điều ở câu hỏi đầu bài. Dựng server cũng gọn:

```python
async def start_server(store, address="127.0.0.1:0"):
    log = LoggingServerInterceptor()
    server = grpc.aio.server(interceptors=[log])
    pb_grpc.add_FraudScoringServicer_to_server(FraudScoringService(store), server)
    port = server.add_insecure_port(address)  # không TLS: chỉ cho lab cục bộ
    await server.start()
    return server, port, log
```

Cổng `0` để hệ điều hành chọn cổng trống, nên `--verify` chạy được cả khi máy đang dùng bất kỳ cổng nào. Kết quả năm ca gọi thường, trích output:

```text
TN-0001 bình thường: score=0.0 reasons=[] model=rules-2026.11.1
TN-0002 thiết bị dùng chung: score=0.45 reasons=['device_shared'] model=rules-2026.11.1
TN-0004 cả ba luật: score=0.95 reasons=['device_shared', 'pair_repeat', 'long_trip'] model=rules-2026.11.1
TN-0005 cùng cặp nhưng chưa có tài xế: score=0.0 reasons=[] model=rules-2026.11.1
```

`model_version` đi cùng mỗi điểm để sau này biết điểm do bản luật nào cho; ca TN-0005 cho thấy `optional` có tác dụng thật: cặp tài xế-khách lặp chỉ được xét khi `driver_id` có mặt.

## Client, deadline và status code

Client bọc stub trong một lớp nhỏ để ép một thói quen: lời gọi nào cũng có deadline (trích `client.py`):

```python
async def score(self, trip_id, customer_id, device_id, distance_km, *, driver_id=None, timeout=None):
    request = pb.ScoreRequest(trip_id=trip_id, customer_id=customer_id,
                              device_id=device_id, distance_km=distance_km)
    if driver_id is not None:  # không gán thì field vắng mặt trên dây, khác với chuỗi rỗng
        request.driver_id = driver_id
    async with self._slots:
        return await self._stub.Score(request, timeout=timeout or self._default_timeout)
```

Tài liệu gRPC nói rõ: mặc định gRPC **không** đặt deadline, nghĩa là client có thể chờ gần như vô hạn, nên hãy luôn đặt một con số thực tế. `timeout=` của Python là khoảng thời gian; thư viện tính mốc hết hạn từ lúc gọi và gửi thời gian còn lại sang server trong header `grpc-timeout`. Hết mốc, client bỏ lời gọi và nhận `DEADLINE_EXCEEDED`. Lab đặt tra cứu chậm 1 giây rồi gọi với deadline 200 ms:

```text
client: DEADLINE_EXCEEDED sau 209 ms (deadline 200 ms, server cần 1000 ms)
server: completed +0, cancelled +1; interceptor ghi CANCELLED
deadline 3 giây: score=0.0, interceptor ghi OK sau 1003 ms
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Trục thời gian từ 0 đến 1000 mili giây. Client chờ 200 mili giây rồi nhận DEADLINE_EXCEEDED. Server cũng đếm 200 mili giây từ header grpc-timeout; hết hạn thì handler nhận CancelledError nên 800 mili giây còn lại không bao giờ chạy."><text x="20" y="62" class="t-b">Client</text><rect x="140" y="40" width="140" height="36" rx="6" class="f-pk-soft s-pk"/><text x="150" y="63" class="t-small">chờ 200 ms</text><text x="20" y="152" class="t-b">Server</text><rect x="140" y="130" width="140" height="36" rx="6" class="f-phys-soft s-phys"/><text x="150" y="153" class="t-small">tra cứu đặc trưng</text><rect x="280" y="130" width="560" height="36" rx="6" class="f-none s-dead dash"/><text x="292" y="153" class="t-small t-dead">800 ms còn lại: không bao giờ chạy</text><path d="M280 20 L280 190" class="f-none s-warn" stroke-width="2"/><text x="292" y="28" class="t-small t-warn">200 ms: client hết hạn, nhận DEADLINE_EXCEEDED</text><text x="292" y="104" class="t-small t-warn">hai đầu cùng đếm 200 ms (server nhận grpc-timeout)</text><text x="292" y="188" class="t-small t-warn">handler nhận CancelledError; nhãn CANCELLED do interceptor của lab gán</text><path d="M140 224 L840 224" class="f-none s-muted"/><text x="132" y="248" class="t-mono">0</text><text x="266" y="248" class="t-mono">200</text><text x="470" y="248" class="t-mono">500</text><text x="826" y="248" class="t-mono">1000</text></svg>
</div>
<figcaption>Hình 2. Deadline chạy ở hai đầu. Client ngừng chờ ở 200 ms; handler ở server cũng bị hủy quanh mốc đó nên không tốn tiếp 800 ms cho một kết quả không ai nhận.</figcaption>
</figure>

Ba điều rút ra. Một: cả hai đầu cùng đếm giờ. Client giữ đồng hồ riêng; server nhận `grpc-timeout` và tự hủy handler khi hết hạn (tài liệu Deadlines, mục «Deadlines on the Server»), rồi tự gửi trạng thái `DEADLINE_EXCEEDED`. Handler không thấy mã đó, chỉ thấy `CancelledError`; nhãn `CANCELLED` trong nhật ký là quy ước của interceptor lab (nó gán khi bắt `CancelledError`), không phải mã nhận từ dây. Hai: việc hủy chỉ dừng được code có điểm `await` (bài 21); nếu handler bận tính CPU hoặc gọi thư viện đồng bộ, nó chạy hết rồi mới biết. Ba: phương thức `context.time_remaining()` cho server biết còn bao nhiêu thời gian, để bỏ qua việc đắt tiền khi ngân sách đã quá ngắn.

Mã trạng thái cho Dispatch biết nên làm gì tiếp, nên lab kiểm cả bốn nhóm thường gặp:

| Mã | Nguyên nhân | Gọi lại y nguyên? |
| --- | --- | --- |
| `INVALID_ARGUMENT` | đầu vào sai (km ngoài miền, thiếu `trip_id`) | Không, vẫn sai |
| `DEADLINE_EXCEEDED` | hết giờ chờ, server chậm hoặc quá tải | Có thể, nếu còn ngân sách và lời gọi idempotent |
| `UNAVAILABLE` | không nối được (không ai nghe ở cổng) | Có thể, với lùi dần |
| `CANCELLED` | một bên chủ động hủy; server cũng trả khi hàng chờ quá tải | Tùy lý do hủy; với quá tải thì chỉ sau khi giảm tải |

`Score` không có tác dụng phụ nên gọi lại an toàn. gRPC có chính sách retry khai báo trong service config; lab không dùng, để ta thấy trọn từng mã. Còn một quan sát đo được, và nó nằm ở server chứ không phải client. Lõi gRPC của server giữ một hàng chờ cho các lời gọi chưa được giao cho handler: vượt ngưỡng mềm `grpc.server.max_pending_requests` (mặc định 1000) thì từ chối theo xác suất, chạm ngưỡng cứng `grpc.server.max_pending_requests_hard_limit` (mặc định 3000) thì từ chối hết. Lời gọi bị từ chối kết thúc `CANCELLED` ngay tại lõi gRPC, trước handler nên interceptor cũng không thấy. Bắn cùng lúc lên một channel, chạy trên máy này (grpcio 1.85.1): 1.000 lời gọi không mất cái nào, 1.500 mất 61, 2.000 mất 218, 3.000 mất 738. Đây là cơ chế bảo vệ server; mã nguồn gRPC ghi chú không nên đổi hai ngưỡng này nếu chưa hiểu hết hệ quả vận hành.

`Semaphore` 200 ở client (`self._slots` ở trên) cứu được trong lab vì chỉ có một client, tổng lời gọi chờ luôn dưới 1.000. Nó không phải giải pháp chung: mười client như thế, mỗi client giới hạn 200, cùng bắn 4.000 lời gọi vào một server thì máy này vẫn mất 1.175 lời gọi, vì tổng cộng lại vượt ngưỡng của server. Nhiều bản Dispatch cộng lại sẽ dính đúng như vậy. Hướng xử lý nằm ở tầng hệ thống: giữ tổng lời gọi dưới sức server và coi `CANCELLED` do quá tải như một dạng từ chối cần lùi dần khi thử lại. Thí nghiệm không nằm trong `--verify` vì kết quả đổi theo máy.

## Interceptor: nhật ký và id yêu cầu

Interceptor là móc nối vào mọi lời gọi mà không sửa từng handler, giống middleware ở bài 33. Server interceptor bọc handler để ghi lại method, mã trạng thái, thời gian và `x-request-id` (trích `interceptors.py`):

```python
class LoggingServerInterceptor(grpc.aio.ServerInterceptor):
    async def intercept_service(self, continuation, handler_call_details):
        handler = await continuation(handler_call_details)
        if handler is None or handler.unary_unary is None:
            return handler  # chỉ bọc unary-unary; kiểu khác để nguyên
        inner = handler.unary_unary
        metadata = dict(handler_call_details.invocation_metadata or ())
        request_id = metadata.get("x-request-id", "-")

        async def logged(request, context):
            start = time.perf_counter()
            code = grpc.StatusCode.OK
            try:
                return await inner(request, context)
            except grpc.aio.AbortError:
                code = context.code()
                raise
            except asyncio.CancelledError:
                code = grpc.StatusCode.CANCELLED  # nhãn quy ước của lab: handler bị hủy (client bỏ hoặc hết hạn)
                raise
            finally:
                self.records.append(CallRecord(handler_call_details.method, code,
                                               (time.perf_counter() - start) * 1000, request_id))
        return grpc.unary_unary_rpc_method_handler(logged, ...)
```

Phần `...` giữ nguyên `request_deserializer` và `response_serializer` của handler gốc. Phía client, một interceptor khác gắn `x-request-id` mới vào metadata trước mỗi lời gọi. Metadata của gRPC đóng vai header HTTP: bài 33 dùng `X-Request-Id` để lần theo một yêu cầu qua log, ở đây làm cùng việc cho gRPC, và bài 77 sẽ thay nó bằng trace của OpenTelemetry. Output cho thấy hai đầu khớp:

```text
client gắn x-request-id=7fb63a4a2929; server ghi /toinoi.fraud.v1.FraudScoring/Score OK 0.05 ms request_id=7fb63a4a2929
tổng 13 lời gọi server đã ghi: {'CANCELLED': 1, 'INVALID_ARGUMENT': 5, 'OK': 7}
```

Tổng 13 gồm 7 lời gọi thành công, 5 lời gọi bị từ chối vì đầu vào và 1 lời gọi bị hủy do deadline: mọi kết cục đều để lại dấu vết, kể cả cái server không bao giờ trả lời.

## Chạy và kiểm chứng

Cần uv và Python 3.14; không cần Docker hay mạng khi chạy (lần resolve đầu tải `grpcio`, `protobuf`, `grpcio-tools`). Phiên bản đã dùng: Python 3.14.7, grpcio 1.85.1, protobuf 7.36.2. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/37-grpc-fraud-scoring examples/toi-noi-python/37-grpc-fraud-scoring/main.py --verify
```

```text
OK 41/41
```

41 ca gồm: 9 kiểm về dữ liệu trên dây và file sinh; 15 kiểm trên 5 ca điểm; 6 ca lỗi; 5 kiểm về deadline; 3 kiểm về interceptor; 3 kiểm về lời gọi đồng thời. Phần đo thời gian của lần chạy thường (máy Apple M1 Max, macOS, loopback; lab tự in tải máy, ở lần chạy này load average 1 phút là 64.1 do nhiều tác vụ khác đang chạy, nên chỉ đọc theo bậc độ lớn):

```text
== 6. Đồng thời ==
tải máy lúc đo (load average 1 phút): 64.1
tuần tự 500 lời gọi: p50=0.67 ms, p99=6.13 ms
đồng thời 2000 lời gọi: 1741 ms (1149 lời gọi/giây)
100 lời gọi, mỗi lời chờ I/O 100 ms: tổng 139 ms (tuần tự sẽ là 10000 ms)
```

Dòng cuối là lợi ích của `grpc.aio`: 100 lời gọi, mỗi lời chờ I/O 100 ms, xong sau chưa tới 150 ms vì chờ chồng nhau trên một event loop. `--verify` chỉ kiểm bất biến này (dưới 5 giây), không kiểm con số tuyệt đối. Thí nghiệm bắn 2.000 lời gọi cùng lúc, ba cấu hình (cùng lần chạy, cùng tải máy):

```bash
uv run --project examples/toi-noi-python/37-grpc-fraud-scoring examples/toi-noi-python/37-grpc-fraud-scoring/main.py --burst 2000
```

```text
2000 lời gọi, server mặc định, client không giới hạn: client thấy {'OK': 1796, 'CANCELLED': 204}; handler nhận 1796
2000 lời gọi, server mặc định, Semaphore 200 ở client: client thấy {'OK': 2000}; handler nhận 2000
2000 lời gọi, server nâng ngưỡng hàng chờ lên 20000/40000, client không giới hạn: client thấy {'OK': 2000}; handler nhận 2000
```

Số `CANCELLED` đổi mỗi lần chạy (204 ở lần này; 218 và 237 ở các lần trước). Số handler nhận cộng với số bị từ chối luôn là 2.000: các lời gọi bị từ chối không bao giờ tới handler. Dòng thứ ba cho thấy nâng ngưỡng hàng chờ của server làm hết lỗi, đúng cơ chế đã nêu; đó là thí nghiệm để hiểu, không phải khuyến nghị cấu hình. Để chắc `--verify` đỏ khi nên đỏ, ta thử phá: trong bản sao ở thư mục tạm, thêm `string coupon = 6;` vào `.proto` mà không sinh lại mã:

```text
SAI: file sinh khớp .proto: nhận [False, False, True], kỳ vọng [True, True, True]
FAIL 40/41
```

Hai file `_pb2` lệch, file `_pb2_grpc` không đổi vì service không đổi; `FAIL 40/41` nghĩa là 40 ca đạt trên 41. Để tự gọi bằng client khác, chạy `main.py --serve --port 50051`.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `UNAVAILABLE` ngay lập tức | server đã chạy chưa, đúng host và cổng, kênh không TLS có khớp server không TLS |
| `DEADLINE_EXCEEDED` cho mọi lời gọi | deadline có nhỏ hơn thời gian thật; handler có chặn event loop bằng code đồng bộ (bài 21) |
| `ModuleNotFoundError: fraud_scoring_pb2` | file sinh phải nằm trong `sys.path`; import tuyệt đối của mã sinh |
| `RuntimeError` nói cần `grpcio>=1.85.1` | phiên bản `grpcio` cũ hơn công cụ đã sinh mã; pin cả hai cùng nhau |
| `CANCELLED` hàng loạt khi tải lớn, handler không thấy | tổng lời gọi chờ trên server vượt 1.000 (`grpc.server.max_pending_requests`); giảm tải từ phía gọi, đừng chỉ nâng ngưỡng |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Kênh không mã hóa (<code>insecure</code>): production dùng TLS hoặc mTLS và xác thực qua metadata (bài 34 nói về token). Bộ luật và <code>FeatureStore</code> là đồ chơi để minh họa hợp đồng; mô hình thật, đặc trưng online và việc đo độ chính xác nằm ở các bài sau. gRPC giữ kết nối HTTP/2 lâu dài, nên bộ cân bằng tải tầng 4 có thể dồn mọi lời gọi của một client vào một pod; cần cân bằng phía client hoặc proxy hiểu HTTP/2. Lab thiếu health check, reflection và retry khai báo (bài 52 sẽ thêm health). Thời gian đo trên một máy đang bận, chỉ để thấy bậc độ lớn.</p>
</div>

**Bài tập 1:** Thêm `string service = 6;` (ride, food hoặc parcel) vào `ScoreRequest`. Cần làm gì với mã sinh? Một client cũ chưa biết field 6 gọi server mới thì `request.service` bằng gì, và server nên xử lý ra sao?

**Bài tập 2:** Dispatch gọi `Score` với deadline 300 ms và nhận `DEADLINE_EXCEEDED` hoặc `UNAVAILABLE`. Đặt chuyến nên tiếp tục hay dừng?

<details><summary>Xem lời giải</summary><p>(1) Phải sinh lại ba file (lệnh ở mục «Hợp đồng protobuf»), nếu không <code>--verify</code> đỏ <code>FAIL 40/41</code> như thí nghiệm trên. Với client cũ, field 6 vắng mặt trên dây nên <code>request.service</code> là chuỗi rỗng, giá trị mặc định của proto3. Server mới phải coi rỗng là «chưa biết» (dùng mặc định hoặc bỏ qua luật theo dịch vụ), không trả <code>INVALID_ARGUMENT</code>, vì thêm field là thay đổi tương thích và client cũ không có lỗi gì. Không được dùng lại số 5 hay đổi kiểu field đang có. (2) <b>Tiếp tục.</b> Điểm gian lận là đầu vào cho người duyệt, không phải điều kiện để cho khách đặt xe. Dispatch ghi chuyến là «chưa chấm điểm» để một hàng đợi chấm lại hoặc cho người duyệt xem, và có thể thử lại một lần với <code>UNAVAILABLE</code> nếu còn ngân sách thời gian. Chặn đặt chuyến mỗi khi service điểm chậm là biến lỗi của một dịch vụ phụ thành sự cố của việc đặt xe.</p></details>

Bài trước: [bài 36](/articles/python-fastapi-api-test-httpx). Bài tiếp: [bài 38](/articles/python-kafka-aiokafka-producer-consumer-schema).

## Tài liệu gốc

- [gRPC: Python basics tutorial](https://grpc.io/docs/languages/python/basics/).
- [gRPC: Deadlines](https://grpc.io/docs/guides/deadlines/).
- [gRPC: Status codes and their use in gRPC](https://grpc.io/docs/guides/status-codes/).
- [gRPC: Interceptors](https://grpc.io/docs/guides/interceptors/).
- [gRPC Python: grpc.aio API](https://grpc.github.io/grpc/python/grpc_asyncio.html).
- [Protocol Buffers: Language Guide (proto 3)](https://protobuf.dev/programming-guides/proto3/).
