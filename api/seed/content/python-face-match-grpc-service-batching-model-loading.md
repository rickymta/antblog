---
slug: python-face-match-grpc-service-batching-model-loading
title: "Service face match qua gRPC: nạp mô hình, batching, CPU/GPU"
summary: "Bài 52: service kyc-face qua gRPC: nạp mô hình SFace một lần, warm-up, health, micro-batching bằng asyncio.Queue, deadline hai đầu, hàng chờ có giới hạn, tắt có thứ tự, providers của ONNX Runtime. Số đo thật kèm tải máy."
category: programming
level: advanced
tags: [python, toi-noi, grpc, onnx-runtime, face-matching]
language: vi
---

Mỗi hồ sơ tài xế mới của [Tới Nơi](/articles/du-an-mau-toi-noi) có một bước so ảnh selfie với ảnh chân dung trên giấy tờ. Service kyc-face viết bằng Python làm bước này, và khác service chấm điểm ở bài 37: nó giữ một mô hình 38,7 MB trong bộ nhớ và mỗi lời gọi tốn CPU thật chứ không chờ I/O. Điều đó đổi ba câu hỏi: khi nào service sẵn sàng, có nên gom yêu cầu nhỏ lại không, và khi client bỏ đi thì việc đã chạy trên thread ra sao.

Đây là bài 52 của lộ trình **Python nâng cao qua Tới Nơi**. Bài kế thừa deadline và status code từ [bài 37](/articles/python-grpc-fraud-scoring-service) và embedding cùng cosine từ [bài 49](/articles/python-face-embedding-cosine-threshold-far-frr); bên gọi là dịch vụ hồ sơ tài xế trong luồng KYC của [bài 41](/articles/python-kyc-tai-xe-bai-toan-du-lieu-metric). Điểm so khớp là đầu vào cho người duyệt, không phải quyết định; [bài 53](/articles/python-face-matching-danh-gia-bias-privacy-consent) nói tiếp về việc đó.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Dịch vụ hồ sơ tài xế gọi <code>Embed</code> với deadline 150 ms, còn lần suy luận trên thread mất 400 ms. Sau 150 ms client nhận <code>DEADLINE_EXCEEDED</code>. Ở bài 37 handler bị hủy tại chỗ <code>await</code> và việc dừng ngay. Lần này, thread suy luận có dừng không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> gRPC hủy handler ở chỗ nó đang <code>await</code>, nhưng hàm đang chạy trên thread không bị ngắt: nó chạy hết 400 ms rồi kết quả bị vứt. Lab đếm đúng ca này: suy luận +1, giao được +0, phí +1. Vì vậy phải chặn việc phí <em>trước</em> khi nó bắt đầu: từ chối khi ngân sách còn quá ít, và bỏ qua yêu cầu đã hủy khi lập batch.</p></details>
</div>

## Service có mô hình khác service chấm điểm ở đâu

Hợp đồng gồm hai lời gọi (trích `proto/face_match.proto`, bỏ chú thích):

```proto
service FaceMatch {
  rpc Embed(EmbedRequest) returns (EmbedResponse);
  rpc Compare(CompareRequest) returns (CompareResponse);
}
message EmbedRequest { bytes image = 1; }
message EmbedResponse { repeated float embedding = 1; string model_version = 2; }
message CompareRequest { bytes image_a = 1; bytes image_b = 2; }
message CompareResponse { double score = 1; string model_version = 2; }
```

`Embed` trả vector 128 số đã chuẩn hóa L2, `Compare` trả cosine của hai ảnh, và `model_version` đi cùng mọi kết quả vì hai vector chỉ so được khi cùng mô hình và tiền xử lý. Ảnh gửi lên là PNG hoặc JPEG của **một crop mặt đã căn chỉnh 112×112** như bài 48 làm; service không dò mặt hay căn chỉnh. Cỡ khác (16 đến 512 điểm ảnh mỗi cạnh, tối đa 1 MiB) được co giãn về 112 nhưng không căn chỉnh lại; `preprocess` kiểm định dạng và kích thước trước khi giải mã điểm ảnh.

Mô hình là SFace (OpenCV Zoo, bản 2021dec, ghi Apache 2.0) chạy bằng ONNX Runtime, cùng mô hình và tiền xử lý với bài 49: lab đưa một crop qua `cv2.FaceRecognizerSF` và qua session của lab rồi in cosine. Cùng cách căn chỉnh nên thang điểm so được với bài 49 (ngưỡng 0,567 ở FAR 0,1%); đổi căn chỉnh hay tiền xử lý thì không. Giấy phép và lý do không dùng mô hình InsightFace nằm ở bài 49. Dữ liệu là Olivetti faces, do AT&T Laboratories Cambridge thu thập, xem tài liệu scikit-learn (tải 4,26 MB, cache 1,4 MB): 400 ảnh xám 64×64 của 40 người, YuNet căn chỉnh thành crop 112×112; repository không chứa ảnh người nào.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Một lời gọi đi qua client có deadline, handler grpc.aio chạy trên event loop, hàng chờ asyncio.Queue, worker gom batch, rồi một thread riêng chạy phiên ONNX Runtime. Ba chỗ lời gọi rời đi sớm: handler trả INVALID_ARGUMENT hoặc DEADLINE_EXCEEDED, hàng chờ đầy trả RESOURCE_EXHAUSTED, worker bỏ qua yêu cầu đã hủy hoặc hết hạn."><defs><marker id="py52-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="110" width="130" height="90" rx="10" class="f-pk-soft s-pk"/><text x="36" y="144" class="t-b">Client</text><text x="36" y="168" class="t-small">có deadline</text><rect x="190" y="110" width="150" height="90" rx="10" class="f-pk-soft s-pk"/><text x="206" y="138" class="t-b">grpc.aio</text><text x="206" y="160" class="t-small">giải mã, kiểm ảnh</text><text x="206" y="178" class="t-small">xét ngân sách</text><rect x="370" y="110" width="140" height="90" rx="10" class="f-warn-soft s-warn"/><text x="386" y="144" class="t-b">asyncio.Queue</text><text x="386" y="168" class="t-small">có giới hạn</text><rect x="550" y="110" width="140" height="90" rx="10" class="f-phys-soft s-phys"/><text x="566" y="138" class="t-b">Worker</text><text x="566" y="160" class="t-small">gom tối đa 8 ảnh</text><text x="566" y="178" class="t-small">chờ tối đa 5 ms</text><rect x="730" y="110" width="130" height="90" rx="10" class="f-phys-soft s-phys"/><text x="746" y="138" class="t-b">Thread riêng</text><text x="746" y="160" class="t-small">ONNX Runtime</text><text x="746" y="178" class="t-small">một session</text><path d="M150 155 L190 155" class="f-none s-phys" stroke-width="2" marker-end="url(#py52-0-arrow)"/><path d="M340 155 L370 155" class="f-none s-phys" stroke-width="2" marker-end="url(#py52-0-arrow)"/><path d="M510 155 L550 155" class="f-none s-phys" stroke-width="2" marker-end="url(#py52-0-arrow)"/><path d="M690 155 L730 155" class="f-none s-phys" stroke-width="2" marker-end="url(#py52-0-arrow)"/><path d="M265 110 L265 70" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py52-0-arrow)"/><text x="190" y="36" class="t-small t-warn">INVALID_ARGUMENT</text><text x="190" y="56" class="t-small t-warn">DEADLINE_EXCEEDED</text><path d="M440 200 L440 240" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py52-0-arrow)"/><text x="372" y="262" class="t-small t-warn">RESOURCE_EXHAUSTED</text><path d="M620 200 L620 240" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py52-0-arrow)"/><text x="552" y="262" class="t-small t-warn">bỏ qua: hủy, hết hạn</text><path d="M795 200 L795 240" class="f-none s-dead dash" stroke-width="2" marker-end="url(#py52-0-arrow)"/><text x="700" y="262" class="t-small t-dead">không dừng giữa chừng</text></svg>
</div>
<figcaption>Hình 1. Ba chỗ một lời gọi rời đi sớm trước khi tốn suy luận, và chỗ cuối cùng nó không rời đi được nữa: khi thread đã chạy.</figcaption>
</figure>

## Nạp một lần, warm-up và health

Nạp mô hình ở mỗi lời gọi là lỗi phổ biến nhất của loại service này, nên mô hình sống cùng vòng đời của server. gRPC của Python không có `lifespan` như FastAPI ([bài 31](/articles/python-fastapi-http-endpoint-bao-gia)), nhưng `asynccontextmanager` cho cùng hình dạng. Trích `service.py` (rút gọn):

```python
@asynccontextmanager
async def serving(config, *, address="127.0.0.1:0", embedder_factory=None, hold_ready=None):
    health_servicer = health.aio.HealthServicer()
    for name in ("", SERVICE_NAME):   # mặc định "" là SERVING: chặn lại, chưa có mô hình
        await health_servicer.set(name, health_pb2.HealthCheckResponse.NOT_SERVING)
    server = grpc.aio.server(options=(("grpc.max_receive_message_length", MAX_MESSAGE_BYTES),
                                      *config.server_options))
    pb_grpc.add_FaceMatchServicer_to_server(FaceMatchService(state), server)
    health_pb2_grpc.add_HealthServicer_to_server(health_servicer, server)
    port = server.add_insecure_port(address)
    await server.start()                      # mở cổng TRƯỚC: health check đã trả lời được
    try:
        embedder = await asyncio.to_thread(factory)                    # nạp mô hình
        warm_up_ms = await asyncio.to_thread(embedder.warm_up, config.warm_sizes, config.pad_to)
        await asyncio.to_thread(warm_decoder)
        batcher = MicroBatcher(run_batch, ...); batcher.start(); state.batcher = batcher
        for name in ("", SERVICE_NAME):
            await health_servicer.set(name, health_pb2.HealthCheckResponse.SERVING)
        yield Running(server, port, batcher, health_servicer, embedder, load_ms, warm_up_ms)
    finally:
        await health_servicer.enter_graceful_shutdown()   # 1. NOT_SERVING
        await server.stop(grace=config.shutdown_grace)    # 2. đợi lời gọi đang chạy, worker vẫn sống
        await batcher.stop()                              # 3. chỉ sau đó mới dừng worker
```

Server mở cổng trước khi nạp xong để bộ điều phối hỏi được `grpc.health.v1` và nhận `NOT_SERVING` thay vì lỗi kết nối: «còn sống» và «sẵn sàng» là hai trạng thái, như `/health/live` và `/health/ready` trong bài [triển khai AWS và Azure của loạt .NET](/articles/dotnet10-aws-azure-deployment). Nạp và warm-up chạy trên thread để event loop vẫn trả lời health check. `HealthServicer` mặc định báo `SERVING` cho tên rỗng ngay khi tạo, nên phải đặt lại trước `start`; lab kiểm cả hai tên.

Warm-up tồn tại vì lần chạy đầu của một mô hình thường chậm hơn các lần sau. Nó chạy thử đúng hai cỡ batch, 1 và `max_batch` (hoặc cỡ đệm nếu có `pad_to`), không phải mọi cỡ ở giữa; provider biên dịch lại theo hình dạng thì cần `pad_to` (mục providers). `warm_decoder` làm tương tự cho Pillow. SFace gốc khai báo đầu vào `[1, 3, 112, 112]`, nên `patch_dynamic_batch` đặt chiều batch của đầu vào và đầu ra thành biến, không đụng trọng số. Output của lab (trích):

```text
400 ảnh Olivetti; YuNet lấy được mặt ở 400 ảnh, không lấy được ở 0 ảnh (bị loại, không lùi âm thầm sang resize)
provider có trong bản onnxruntime này: ['CoreMLExecutionProvider', 'AzureExecutionProvider', 'CPUExecutionProvider']
file gốc: batch 1 chạy được; batch 8 bị từ chối: ... Got: 8 Expected: 1 Please fix either the inputs/outputs or the model.
tạo session: 31 ms, provider đang dùng: ['CPUExecutionProvider']
ba lần gọi đầu (batch 1): 38.6, 35.5, 14.1 ms; trung vị 30 lần sau: 52.8 ms
8 ảnh chạy lẻ so với một batch 8: lệch lớn nhất 1.34e-07; batch 3 đệm tới 8 so với chạy lẻ: 1.19e-07
cùng một crop qua cv2.FaceRecognizerSF và qua session của lab: cosine 1.000000
ảnh hỏng đầu tiên của tiến trình mới: 291.7 ms, ảnh hỏng thứ hai: 0.10 ms; nếu gọi warm_decoder() trước: 0.00 ms
resolve_providers(CUDA, CoreML, CPU) trên máy chỉ có CPU: ['CPUExecutionProvider']
```

```text
server grpc.aio nghe 127.0.0.1:60135; nạp mô hình 57 ms; warm-up batch 1: 38.4 ms, batch 8: 104.7 ms
health toinoi.face.v1.FaceMatch: SERVING; health tổng: SERVING
Embed: 128 số float32, độ dài 1.000000, ảnh PNG 15399 byte
Compare 40 cặp cùng người: trung bình 0.854 (nhỏ nhất 0.557); 40 cặp khác người: trung bình 0.277 (lớn nhất 0.463)
Compare(a, a)=1.000000; Compare(a, b)=0.400863; Compare(b, a)=0.400863; tích vô hướng hai Embed=0.400863
```

Ba lần gọi đầu (38,6; 35,5; 14,1 ms) và trung vị 30 lần sau (52,8 ms) không theo thứ tự «đầu chậm hơn sau»: máy dùng chung quá tải, nhiễu lấn át, nên chúng không chứng minh lợi ích warm-up. Thứ đo rõ là ảnh hỏng, trong tiến trình con mới: ảnh hỏng đầu tiên 291,7 ms, ảnh thứ hai 0,10 ms, và 0,00 ms nếu đã gọi `warm_decoder()` trước. Warm-up của service in cỡ 1 và cỡ 8 (38,4 và 104,7 ms). Vòng đời được kiểm bằng cách giữ server ở trạng thái đang nạp (`hold_ready`, chỉ cho test):

```text
đang nạp mô hình: health=NOT_SERVING, health tổng=NOT_SERVING, Embed -> UNAVAILABLE (mô hình đang nạp)
sau warm-up: health=SERVING, Embed trả 128 số
sau khi tắt: health -> UNAVAILABLE
```

**Thứ tự tắt.** Bản đầu của lab dừng batcher trước server: lời gọi đang chạy bị hủy, client nhận `UNAVAILABLE` dù việc sắp xong. Thứ tự đúng là ba bước ở khối `finally`: `NOT_SERVING`, để gRPC đợi lời gọi đang chạy (tối đa `shutdown_grace`) khi worker còn sống, rồi mới dừng worker; Future còn treo khi worker bị hủy được giải bằng `UNAVAILABLE`. Ca kiểm bắt đầu tắt khi một lời gọi 400 ms đang chạy:

```text
lời gọi đang chạy lúc bắt đầu tắt: OK, 128 số; mô hình giả chạy 1 lần
lời gọi mới sau khi tắt: UNAVAILABLE
```

## Micro-batching bằng asyncio.Queue

Gom các yêu cầu đến gần nhau thành một lần suy luận có thể tăng thông lượng nếu mô hình chạy batch lớn hiệu quả hơn batch 1; lab đo xem máy này có vậy không. Mỗi yêu cầu đặt vào `asyncio.Queue` kèm một `Future`; một worker lấy yêu cầu đầu, chờ tối đa `max_wait_ms` cho đủ `max_batch` rồi chạy. Trích `batcher.py`:

```python
async def _collect(self) -> list[_Item]:
    loop = asyncio.get_running_loop()
    batch = [await self._queue.get()]            # yêu cầu đầu: chờ bao lâu cũng được
    cutoff = loop.time() + self.max_wait         # các yêu cầu sau: chỉ tới hết cửa sổ
    while len(batch) < self.max_batch:
        try:
            async with asyncio.timeout_at(cutoff):
                batch.append(await self._queue.get())
        except TimeoutError:
            break
    return batch
```

Phần còn lại của `_loop` lọc batch, chạy `embed_batch` trên thread rồi giao kết quả cho từng `Future`. Điều phải đúng: **gom batch không đổi kết quả của từng ảnh**. Lab chạy 8 ảnh lẻ rồi chạy chung một batch và so từng số: lệch lớn nhất cỡ 1e-7, sai số số học của float32 chứ không phải bằng 0 (dòng «8 ảnh chạy lẻ...» ở output trên, cả khi đệm batch 3 tới 8). `submit_many` đặt tất cả hoặc không cái nào, vì một `Compare` cần hai chỗ; không có `await` giữa lúc kiểm chỗ và lúc đặt, nên không coroutine nào chen vào được.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Trục thời gian của micro-batching. Ba yêu cầu r1, r2, r3 đến liên tiếp; worker mở cửa sổ chờ tối đa 5 mili giây từ r1 rồi chạy một lần suy luận cho cả ba. Yêu cầu r4 đến sau, mở cửa sổ mới, không có ai đến cùng nên chạy batch một ảnh sau khi chờ hết cửa sổ."><text x="20" y="62" class="t-b">Yêu cầu</text><text x="20" y="124" class="t-b">Worker</text><rect x="160" y="44" width="14" height="24" rx="3" class="f-pk-soft s-pk"/><text x="158" y="36" class="t-small t-mono">r1</text><rect x="200" y="44" width="14" height="24" rx="3" class="f-pk-soft s-pk"/><text x="198" y="36" class="t-small t-mono">r2</text><rect x="240" y="44" width="14" height="24" rx="3" class="f-pk-soft s-pk"/><text x="238" y="36" class="t-small t-mono">r3</text><rect x="520" y="44" width="14" height="24" rx="3" class="f-pk-soft s-pk"/><text x="518" y="36" class="t-small t-mono">r4</text><rect x="160" y="100" width="170" height="40" rx="6" class="f-warn-soft s-warn"/><text x="172" y="125" class="t-small">chờ tối đa 5 ms</text><rect x="330" y="100" width="190" height="40" rx="6" class="f-phys-soft s-phys"/><text x="342" y="125" class="t-small">suy luận batch 3</text><rect x="520" y="100" width="170" height="40" rx="6" class="f-warn-soft s-warn"/><text x="532" y="125" class="t-small">chờ tối đa 5 ms</text><rect x="690" y="100" width="150" height="40" rx="6" class="f-phys-soft s-phys"/><text x="702" y="125" class="t-small">suy luận batch 1</text><path d="M140 170 L840 170" class="f-none s-muted"/><text x="20" y="192" class="t-small">Lợi: r1, r2, r3 chia nhau một lần chạy. Giá: r1 có thể phải chờ thêm tới hết cửa sổ.</text><text x="20" y="214" class="t-small">Tải thấp: cửa sổ không có ai vào, batch chỉ còn một ảnh và chỉ còn lại cái giá chờ.</text></svg>
</div>
<figcaption>Hình 2. Cửa sổ chờ đổi độ trễ lấy thông lượng. Nó sinh lợi khi nhiều yêu cầu đến cùng lúc và chỉ gây thiệt khi yêu cầu đến lẻ tẻ.</figcaption>
</figure>

Lab đếm batch thật khi 32 lời gọi `Embed` đến cùng lúc (cửa sổ chờ nới lên 50 ms để kiểm bền khi máy bận):

```text
32 lời gọi Embed đồng thời -> 4 lần suy luận; cỡ batch (cỡ: số lần) {8: 4}
```

Gom có lợi bao nhiêu thì phải đo. `--bench` (ngoài `--verify`) đo chi phí mỗi ảnh theo cỡ batch trên session, rồi p50/p95 qua gRPC với server ở tiến trình riêng (C là số worker gọi song song; cửa sổ chờ lấy từ `--max-wait-ms`, mặc định 5 ms):

```text
máy: 10 lõi logic; tải máy lúc đo (load average 1 phút): 214.9; onnxruntime 1.31.1
-- chi phí mỗi ảnh theo cỡ batch (trực tiếp, không qua gRPC; trung vị 15 lần) --
batch  1:   49.2 ms cả batch, 49.24 ms mỗi ảnh
batch  2:  117.8 ms cả batch, 58.91 ms mỗi ảnh
batch  4:  106.6 ms cả batch, 26.64 ms mỗi ảnh
batch  8:  196.6 ms cả batch, 24.57 ms mỗi ảnh
batch 16:  276.7 ms cả batch, 17.30 ms mỗi ảnh
batch 32: 2335.4 ms cả batch, 72.98 ms mỗi ảnh
-- p50/p95 qua gRPC, server ở tiến trình riêng; 3 vòng mỗi ô, in trung vị (nhỏ nhất-lớn nhất giữa các vòng) --
C=1   CPU, không batching (max_batch=1)      p50  143.4 ms (62.0-164.7)  p95  399.5 ms (130.4-544.2)       6 lời gọi/giây
C=1   CPU, batching (max_batch=8, chờ 5 ms)  p50   23.4 ms (22.3-37.0)  p95   55.8 ms (51.1-100.2)      37 lời gọi/giây
C=16  CPU, không batching (max_batch=1)      p50  147.7 ms (134.1-308.7)  p95  425.9 ms (203.1-937.4)      82 lời gọi/giây
C=16  CPU, batching (max_batch=8, chờ 5 ms)  p50  327.9 ms (225.7-334.0)  p95  556.6 ms (497.0-1379.3)      46 lời gọi/giây
C=64  CPU, không batching (max_batch=1)      p50 2410.6 ms (2127.9-10478.2)  p95 4841.3 ms (4201.9-17660.7)      25 lời gọi/giây
C=64  CPU, batching (max_batch=8, chờ 5 ms)  p50 1910.9 ms (987.4-2512.0)  p95 5121.2 ms (5053.5-8034.3)      25 lời gọi/giây
tải máy lúc đo (load average 1 phút): 162.6 (cuối lần đo)
```

Tải máy là 214,9 lúc đầu và 162,6 lúc hết phần p50/p95, trên 10 lõi: số CPU chỉ đọc theo bậc độ lớn, và bảng không cho kết luận nào về batching. Ở C=1 cửa sổ chỉ thêm tối đa 5 ms mỗi lời gọi, nhỏ hơn nhiều so với nhiễu: ô không batching đo p50 143,4 ms, ô có batching 23,4 ms, chênh lệch đó đến từ tải máy. Ở C=16 hai cấu hình đo 82 và 46 lời gọi mỗi giây, ở C=64 cùng 25; khoảng giữa các vòng rộng và chồng nhau. Chi phí mỗi ảnh (49,2 ms ở batch 1, 24,6 ở batch 8, 17,3 ở batch 16) gợi ý batch vừa phải rẻ hơn nhưng không đơn điệu (batch 2 đo 58,9, batch 32 đo 73,0). Muốn kết luận, đo lại trên máy rảnh và phần cứng đích.

## Deadline hai đầu, hàng chờ đầy và max_pending_requests

Bài 37 cho thấy deadline chạy ở hai đầu: client giữ đồng hồ riêng, server hủy handler khi hết hạn. Với thread suy luận có bốn trường hợp, dựng bằng một mô hình giả mất 400 ms mỗi lần (`FakeEmbedder`, không phải mô hình thật):

```text
(a) B hết hạn khi xếp hàng: client nhận DEADLINE_EXCEEDED sau 155 ms; suy luận 1 ảnh, bỏ qua 1 (hủy 1, hết hạn 0); mô hình giả chạy 1 lần
(b) C hết hạn giữa lúc suy luận: client nhận DEADLINE_EXCEEDED; suy luận +1, giao được +0, phí +1
(c) deadline 3 ms: client nhận DEADLINE_EXCEEDED; mô hình giả nhận thêm 0 ảnh
(d) yêu cầu có hạn đã qua: DeadlineExpired (hết hạn trong hàng chờ); skipped_expired=1, suy luận 0 ảnh
```

Ca (a): B xếp hàng sau A và hết hạn khi chưa tới lượt. gRPC hủy handler, `gather` chuyển lệnh hủy xuống `Future`, và khi lập batch kế tiếp worker thấy `Future` đã hủy mà bỏ qua: mô hình chỉ chạy một lần, cho A. Ca (b) là câu hỏi đầu bài: yêu cầu đã vào thread thì không dừng được, kết quả bị vứt. Ca (c): ngân sách còn dưới `min_budget_ms` (5 ms) thì handler từ chối ngay, không xếp hàng. Ca (d) thử riêng batcher: mốc hết hạn đã qua đúng lúc lập batch bị loại với `DeadlineExpired`, không tốn suy luận. Mốc tuyệt đối đó đổi từ `context.time_remaining()` và đi cùng yêu cầu.

Hàng chờ là chỗ thứ hai cần giới hạn: `queue_size` mặc định 64; đầy thì `submit_many` ném `QueueFull` và handler trả `RESOURCE_EXHAUSTED` ngay, trong vài chục mili giây thay vì 400 ms. `Compare` cần hai chỗ nên thấy chỉ còn 1 chỗ vẫn bị từ chối, không đặt nửa chừng:

```text
12 lời gọi đồng thời, hàng chờ 4, batch 2: {'OK': 5, 'RESOURCE_EXHAUSTED': 7}; lời bị từ chối trả lời trong 47 ms (suy luận mất 400 ms)
Compare khi hàng chờ còn 1 chỗ: RESOURCE_EXHAUSTED; rejected_full=2
```

Hàng chờ ứng dụng không phải hàng chờ duy nhất. Lõi gRPC phía server cũng giữ hàng cho các lời gọi chưa giao cho handler, với hai cơ chế làm lời gọi bị `CANCELLED` mà handler và `Stats` không thấy. **Ngưỡng số lượng:** vượt ngưỡng mềm `grpc.server.max_pending_requests` (mặc định 1.000) thì từ chối theo xác suất, chạm ngưỡng cứng `..._hard_limit` (3.000) thì từ chối hết ([tài liệu channel arguments](https://grpc.github.io/grpc/core/group__grpc__arg__keys.html)); hàng chờ ứng dụng không chạm nó, vì handler chỉ đặt việc vào `Queue`, còn cú dồn 3.000 lời gọi đến nhanh hơn event loop nhận chúng ra khỏi lõi. **Giới hạn tuổi:** `grpc.server_max_unrequested_time_in_server` (tên có trong grpcio 1.85.1; theo mã nguồn `server.cc` mặc định 30 giây) hủy lời gọi nằm trong hàng của lõi quá lâu. Dấu khác nhau: ngưỡng số lượng hủy gần như ngay (t ≈ 0), giới hạn tuổi hủy ở đúng mốc tuổi. `--burst` bắn 3.000 lời gọi `Embed` cùng lúc vào server ở tiến trình riêng, in mốc t của từng `CANCELLED`, ở bốn cấu hình (cuối cùng: suy luận chặn event loop và giới hạn tuổi 5 giây):

```text
máy: 10 lõi logic; tải máy lúc đo (load average 1 phút): 271.9
3000 lời gọi cùng lúc, hàng chờ 64 (mặc định của lab), ngưỡng gRPC mặc định: client thấy {'CANCELLED': 719, 'OK': 376, 'RESOURCE_EXHAUSTED': 1905} trong 9.4 giây
    CANCELLED đến lúc t = 0.2 s (sớm nhất), 0.3 s (giữa), 0.3 s (muộn nhất) kể từ lúc bắn
3000 lời gọi cùng lúc, hàng chờ 5000, ngưỡng gRPC mặc định: client thấy {'CANCELLED': 695, 'DEADLINE_EXCEEDED': 1162, 'OK': 1143} trong 60.1 giây
    CANCELLED đến lúc t = 0.4 s (sớm nhất), 0.4 s (giữa), 0.4 s (muộn nhất) kể từ lúc bắn
3000 lời gọi cùng lúc, hàng chờ 5000, ngưỡng lõi 20000/40000: client thấy {'DEADLINE_EXCEEDED': 1694, 'OK': 1306} trong 60.1 giây
3000 lời gọi cùng lúc, hàng chờ 5000, ngưỡng lõi 20000/40000, suy luận chặn event loop, tuổi tối đa trong hàng lõi 5 s: client thấy {'CANCELLED': 2950, 'OK': 50} trong 5.6 giây
    CANCELLED đến lúc t = 5.2 s (sớm nhất), 5.3 s (giữa), 5.3 s (muộn nhất) kể từ lúc bắn
tải máy lúc đo (load average 1 phút): 194.9 (cuối lần đo)
```

Cấu hình đầu: 719 lời gọi `CANCELLED` ở t = 0,2 đến 0,3 giây, 1.905 lời gọi bị hàng chờ của ta từ chối, 376 được phục vụ: t gần 0 là dấu của ngưỡng số lượng. Cấu hình hai (hàng chờ 5.000): hết `RESOURCE_EXHAUSTED` nhưng vẫn 695 `CANCELLED` ở t = 0,4 giây, nên hàng chờ ứng dụng không chặn được ngưỡng lõi; 1.162 lời gọi hết hạn 60 giây sau khi chiếm chỗ. Cấu hình ba (ngưỡng lõi 20.000 và 40.000): hết `CANCELLED`, nhưng 1.694 lời gọi hết hạn: nâng ngưỡng chỉ đổi chỗ chờ. Cấu hình bốn: 2.950 `CANCELLED` ở t = 5,2 đến 5,3 giây, đúng mốc giới hạn tuổi đã đặt, chỉ 50 lời gọi được phục vụ. Mốc t trùng giới hạn tuổi là dấu hiệu, không phải bằng chứng từ bên trong lõi; `details` chỉ ghi «CANCELLED» ở cả hai cơ chế. Số đổi mỗi lần chạy, ở tải 200 đến gần 300.

Phía gọi nên coi `CANCELLED` không do mình hủy như một dạng quá tải và lùi dần, và giữ tổng số lời gọi đang bay (cộng mọi client) dưới sức server. Nâng `max_pending_requests` chỉ để hiểu cơ chế: nó đẩy lời gọi từ chỗ bị từ chối sang chỗ chờ lâu rồi hết hạn.

Cuối cùng, cái giá của suy luận chạy ngay trên event loop. Phần cuối của `--bench` đo một health check trong lúc 64 lời gọi `Embed` đang chạy, khi suy luận ở thread riêng và khi chạy trong event loop (`--inline`):

```text
thread riêng                 950 lần hỏi: p50 3.7 ms, p95 41.3 ms, lớn nhất 135.9 ms
chặn event loop (--inline)   247 lần hỏi: p50 243.0 ms, p95 1145.4 ms, lớn nhất 1534.2 ms
tải máy lúc đo (load average 1 phút): 267.8 (cuối lần đo)
```

Ở thread riêng, health check trả lời với p50 3,7 ms, p95 41,3 ms (950 lần hỏi). Chạy trong event loop, p50 là 243,0 ms, p95 1.145,4 ms, lớn nhất 1.534,2 ms (247 lần hỏi): mỗi lần suy luận chặn cả event loop nên health check và mọi lời gọi khác xếp hàng sau nó. Chênh lệch cỡ 65 lần ở p50 đủ lớn để nhiễu tải không che được, và probe có timeout ngắn hơn 1,5 giây sẽ báo hỏng một service vẫn khỏe.

| Mã | Khi nào ở service này | Gọi lại y nguyên? |
| --- | --- | --- |
| `INVALID_ARGUMENT` | ảnh rỗng, hỏng, sai định dạng, ngoài 16..512 điểm ảnh, khai báo quá nhiều điểm ảnh, quá 1 MiB | Không |
| `DEADLINE_EXCEEDED` | hết hạn khi xếp hàng, ngân sách quá ngắn, hoặc client tự hết hạn | Có thể, nếu còn ngân sách |
| `RESOURCE_EXHAUSTED` | hàng chờ suy luận đầy; tin nhắn vượt giới hạn nhận của server | Có, sau khi lùi dần; tin nhắn quá lớn thì không |
| `UNAVAILABLE` | mô hình đang nạp hoặc server đang tắt | Có, với lùi dần |
| `CANCELLED` | một bên chủ động hủy; hoặc lõi gRPC hủy vì ngưỡng số lượng hay giới hạn tuổi ở trên | Tùy lý do hủy |

## ONNX Runtime providers: CPU, CoreML và GPU

`providers` là danh sách nơi chạy phép tính theo thứ tự ưu tiên. `resolve_providers` giữ provider mà `onnxruntime` đang cài thật sự có và luôn thêm CPU cuối làm đường lùi: máy này có `CoreMLExecutionProvider`, `AzureExecutionProvider` và CPU, nên yêu cầu CUDA bị bỏ qua thay vì làm session lỗi. Bài **chưa đo GPU** (máy không có GPU NVIDIA); số dưới đây là CPU và CoreML. Việc vá chiều batch là điều kiện cho mọi provider: với file gốc, batch 8 bị từ chối cả trên CPU. Sau khi vá, `--providers` thử chuỗi cỡ batch 1, 8, 8, 3, 8, 8, 1:

Việc vá chiều batch ở trên là điều kiện cho mọi provider: với file gốc, batch 8 bị từ chối ngay cả trên CPU. Sau khi vá, `--providers` thử chuỗi cỡ batch 1, 8, 8, 3, 8, 8, 1:

```text
provider có trong bản onnxruntime này: ['CoreMLExecutionProvider', 'AzureExecutionProvider', 'CPUExecutionProvider']; tải máy lúc đo (load average 1 phút): 151.0
file gốc chưa vá, CPUExecutionProvider: batch 1 chạy được, batch 8 lỗi InvalidArgument: Got: 8 Expected: 1
file gốc chưa vá, CoreMLExecutionProvider: batch 1 chạy được, batch 8 lỗi InvalidArgument: Got: 8 Expected: 1
CPU: providers=['CPUExecutionProvider'], tạo session 20 ms
   cỡ batch -> ms: 1->144, 8->265, 8->357, 3->158, 8->335, 8->320, 1->37; cosine nhỏ nhất so với CPU: 1.000000
CoreML, hình dạng đổi tự do: providers=['CoreMLExecutionProvider', 'CPUExecutionProvider'], tạo session 840 ms
   cỡ batch -> ms: 1->59, 8->2085, 8->362, 3->1131, 8->3700, 8->262, 1->876; cosine nhỏ nhất so với CPU: 0.999997
CoreML, đệm batch tới 8: providers=['CoreMLExecutionProvider', 'CPUExecutionProvider'], tạo session 906 ms
   cỡ batch -> ms: 1->2018, 8->889, 8->542, 3->761, 8->984, 8->487, 1->349; cosine nhỏ nhất so với CPU: 0.999997
```

Tải máy lúc đo là 151 nên ta không so được tốc độ giữa các provider; phần đọc được là hình dạng thời gian của CoreML. Khi cỡ batch đổi tự do, mỗi lần đổi hình dạng tốn 1,1 đến 3,7 giây (8 tốn 2.085 ms, 3 tốn 1.131 ms, 8 lần nữa tốn 3.700 ms), còn lặp lại đúng cỡ vừa dùng chỉ 262 đến 362 ms. Với đệm tới 8, chỉ lần gọi đầu tốn 2.018 ms (cái giá warm-up trả lúc khởi động), sau đó không còn đợt tăng vọt theo cỡ; 349 đến 984 ms mỗi lần còn lại là nhiễu của máy bận. Hình dạng này khớp với việc biên dịch lại khi đầu vào đổi hình dạng; lab chỉ đo thời gian, không nhìn vào bên trong CoreML.

## Chạy và kiểm chứng

Cần uv và Python 3.14; cần mạng lần chạy đầu để tải SFace (38,7 MB), YuNet (0,2 MB) và Olivetti (tải 4,26 MB). Phiên bản đã dùng: Python 3.14.7, grpcio 1.85.1, grpcio-health-checking 1.85.1, protobuf 7.36.2, onnxruntime 1.31.1, onnx 1.23.2, numpy 2.5.4, Pillow 13.0.0, opencv-python-headless 5.0.0.93, scikit-learn 1.9.1. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/52-face-match-grpc-service examples/toi-noi-python/52-face-match-grpc-service/main.py --verify
```

```text
OK 56/56
```

`--verify` chỉ kiểm bất biến: file sinh khớp `.proto`; SFace gốc từ chối batch 8, bản vá thì không; batch cho kết quả như chạy lẻ; tiền xử lý khớp OpenCV; health đi qua `NOT_SERVING`, `SERVING`, cổng đóng; mã lỗi mỗi loại đầu vào sai, gồm PNG khai 20.000×20.000 điểm ảnh (trước đây trả `UNKNOWN` vì Pillow ném `DecompressionBombError`); số lần bỏ qua, suy luận phí, bị từ chối; lời gọi đang chạy hoàn tất khi tắt. Nó không kiểm tốc độ: deadline của phần gọi thật đặt 60 giây.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `UNAVAILABLE` ngay sau khi khởi động | health của service có phải `NOT_SERVING` (đang nạp, warm-up) không; chờ `SERVING` rồi mới gọi |
| `DEADLINE_EXCEEDED` hàng loạt dù mô hình nhanh | hàng chờ có dài không (`rejected_full`, `skipped_*` trong `Stats`); suy luận có chạy trong event loop không |
| `CANCELLED` nhưng handler không thấy | thời điểm hủy: gần t = 0 là ngưỡng số lượng, đúng mốc tuổi là giới hạn tuổi; giảm tải từ phía gọi |
| Batch 8 lỗi hình dạng đầu vào | khai báo `[1, 3, 112, 112]` của file ONNX gốc; vá chiều batch; thử `pad_to` nếu provider biên dịch lại |
| Điểm khác nhau giữa hai môi trường | `model_version`, provider, cách căn chỉnh và tiền xử lý (RGB, 0..255, 112×112) có giống nhau không |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Kênh không mã hóa (<code>insecure</code>): production dùng TLS hoặc mTLS. Ảnh là crop Olivetti (64×64, xám) căn chỉnh bằng YuNet; selfie thật cần phát hiện, căn chỉnh và kiểm chất lượng trước (bài 48, 50), và độ chính xác ở đây không phản ánh hệ thống thật. Giải mã ảnh chạy trong event loop vì ảnh nhỏ. Production cần nhiều bản sao service và đo lại <code>max_batch</code>, <code>max_wait_ms</code> trên phần cứng thật. Số đo lấy trên máy dùng chung rất bận, chỉ đọc theo bậc độ lớn; chưa đo GPU. Vector khuôn mặt là dữ liệu sinh trắc học: không ghi vào log (bài 53).</p></div>

**Bài tập:** Chạy `--bench --max-wait-ms 50` rồi so p50 ở C=1 với lần chạy mặc định (5 ms). Bạn kỳ vọng p50 đổi theo hướng nào và tối đa bao nhiêu, và vì sao tải thấp chịu thiệt nhiều nhất?

<details><summary>Xem lời giải</summary><p>Với C=1 mỗi lần chỉ có một yêu cầu trong cửa sổ, nên worker chờ hết <code>max_wait_ms</code> mới chạy batch một ảnh: mỗi lời gọi chậm thêm tối đa phần cửa sổ được nới (45 ms) mà không được gì. Tải thấp thiệt nhất vì cửa sổ chỉ có lợi khi có người đến cùng lúc. Với 5 ms, chi phí tối đa chỉ 5 ms, nhỏ hơn nhiều so với nhiễu của máy bận, nên không đọc được từ một lần đo.</p></details>

Bài trước: [bài 51](/articles/python-vector-search-pgvector-faiss-trung-tai-xe). Bài tiếp: [bài 53](/articles/python-face-matching-danh-gia-bias-privacy-consent).

## Tài liệu gốc

- [gRPC: Health Checking](https://grpc.io/docs/guides/health-checking/).
- [gRPC: Deadlines](https://grpc.io/docs/guides/deadlines/).
- [gRPC: Status codes and their use in gRPC](https://grpc.io/docs/guides/status-codes/).
- [gRPC Core: Channel argument keys (max_pending_requests)](https://grpc.github.io/grpc/core/group__grpc__arg__keys.html).
- [ONNX Runtime: Execution Providers](https://onnxruntime.ai/docs/execution-providers/).
- [OpenCV Zoo: SFace (face recognition)](https://github.com/opencv/opencv_zoo/tree/main/models/face_recognition_sface).
- [Python: asyncio.Queue](https://docs.python.org/3.14/library/asyncio-queue.html).
