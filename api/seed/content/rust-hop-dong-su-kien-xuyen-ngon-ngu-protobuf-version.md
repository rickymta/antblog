---
slug: rust-hop-dong-su-kien-xuyen-ngon-ngu-protobuf-version
title: "Hợp đồng sự kiện xuyên ngôn ngữ: protobuf, JSON Schema và version"
summary: "Bài 55: LedgerEntryPosted bằng protobuf và JSON Schema, consumer C#, Go, Python đọc cùng fixture do Rust sinh, int64 qua JSON, bảng đo kết quả của sáu thay đổi schema, kiểm tương thích bằng descriptor, thứ tự triển khai."
category: rust-advanced
level: advanced
tags: [rust, toi-noi, ledger, protobuf, schema-registry, kafka]
language: vi
---

Sau mỗi bút toán, ledger Rust phát `LedgerEntryPosted` lên Kafka ([bài 36](/articles/rust-kafka-rdkafka-consumer-producer-schema)). Theo định hướng thiết kế của Tới Nơi, các consumer đọc sự kiện này sẽ viết bằng nhiều ngôn ngữ: service C# như Payment, service thông báo bằng Go (báo tài xế `DR-001` ví vừa nhận 20.400 đồng), pipeline phân tích bằng Python. "Schema" của sự kiện lúc đó vẫn chỉ là một struct Rust, và một người đổi `idempotencyKey` thành `dedupKey` cho gọn thì không compiler nào của các nhóm bắt được lỗi đó.

Đây là bài 55 của lộ trình **Rust nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài này đưa sự kiện thành hợp đồng có hai bản mô tả, cho ba ngôn ngữ đọc thật, rồi đo sáu kiểu đổi schema. Bài này nối tiếp [bài 35](/articles/rust-tonic-grpc-get-balance-post-entry-interop-dotnet) và luồng thanh toán ở [bài 54](/articles/rust-use-case-thanh-toan-chuyen-end-to-end).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một đồng nghiệp đổi số field của <code>amount_dong</code> trong <code>Posting</code> từ 2 sang 3 "cho gọn". Consumer cũ đọc byte do producer mới ghi: consumer đó báo lỗi, hay đọc ra giá trị nào?</p>
<details><summary>Xem đáp án</summary><p>Không báo lỗi. Số 3 là trường lạ, số 2 vắng mặt, nên cả hai dòng đều có <code>amountDong = "0"</code>. Cả bốn thư viện đều như vậy; số đo nằm ở mục «Sáu thay đổi schema».</p></details>
</div>

## Một sự kiện, hai bản mô tả: protobuf và JSON Schema

Hợp đồng nằm trong hai tệp: `ledger_entry_posted.v1.proto` cho dạng nhị phân và proto3 JSON, `ledger_entry_posted.v1.schema.json` cho JSON camelCase mà loạt bài dùng trên Kafka (Hình 1).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 264" role="img" aria-label="Hình 1. Hai tệp hợp đồng, protobuf và JSON Schema, đi vào Rust; Rust sinh fixture nhị phân và JSON; consumer C#, Go và Python đọc cùng fixture."><defs><marker id="rust-55-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="250" height="100" rx="10" class="f-pk-soft s-pk"/><text x="36" y="48" class="t-b">protobuf v1</text><text x="36" y="70" class="t-small">schemas/*.v1.proto</text><text x="36" y="88" class="t-small">int64 amount_dong = 2</text><text x="36" y="106" class="t-small">số field là danh tính trên dây</text><rect x="20" y="150" width="250" height="100" rx="10" class="f-pk-soft s-pk"/><text x="36" y="178" class="t-b">JSON Schema v1</text><text x="36" y="200" class="t-small">schemas/*.schema.json</text><text x="36" y="218" class="t-small">int64 là số, tối đa 2^53 - 1</text><text x="36" y="236" class="t-small">trường lạ bị từ chối</text><rect x="340" y="70" width="250" height="130" rx="10" class="f-phys-soft s-phys"/><text x="356" y="98" class="t-b">Rust sinh fixture</text><text x="356" y="122" class="t-small">prost: sự kiện có kiểu</text><text x="356" y="140" class="t-small">prost-reflect: biến thể động</text><text x="356" y="158" class="t-small">fixtures/data/*.bin, *.json</text><text x="356" y="176" class="t-small">commit sẵn, test chống lệch</text><rect x="660" y="14" width="200" height="72" rx="10" class="f-warn-soft s-warn"/><text x="676" y="40" class="t-b">C# · Google.Protobuf</text><text x="676" y="60" class="t-small">mã sinh từ .proto</text><text x="676" y="76" class="t-small">Grpc.Tools khi build</text><path d="M590 135 L656 50" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-55-a)"/><rect x="660" y="98" width="200" height="72" rx="10" class="f-warn-soft s-warn"/><text x="676" y="124" class="t-b">Go · protobuf-go</text><text x="676" y="144" class="t-small">descriptor set</text><text x="676" y="160" class="t-small">protojson, dynamicpb</text><path d="M590 135 L656 134" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-55-a)"/><rect x="660" y="182" width="200" height="72" rx="10" class="f-warn-soft s-warn"/><text x="676" y="208" class="t-b">Python · protobuf</text><text x="676" y="228" class="t-small">descriptor set</text><text x="676" y="244" class="t-small">json_format, venv riêng</text><path d="M590 135 L656 218" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-55-a)"/><path d="M270 70 L336 115" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-55-a)"/><path d="M270 200 L336 155" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-55-a)"/></svg>
</div>
<figcaption>Hình 1. Hai tệp hợp đồng đi vào Rust; Rust sinh fixture; ba consumer đọc cùng fixture.</figcaption>
</figure>

```proto
// schemas/ledger_entry_posted.v1.proto (rút gọn)
syntax = "proto3";
package toinoi.events.v1;
import "google/protobuf/timestamp.proto";

message Posting {
  string account = 1;
  int64 amount_dong = 2;   // dương = vào tài khoản, âm = ra; số nguyên đồng
}

message LedgerEntryPosted {
  uint32 schema_version = 1;
  string entry_id = 2;
  string idempotency_key = 3;
  repeated Posting postings = 4;
  google.protobuf.Timestamp posted_at = 5;
}
```

```json
// schemas/ledger_entry_posted.v1.schema.json (rút gọn)
"schemaVersion": { "const": 1 },
"postings": { "type": "array", "minItems": 2, "items": { "$ref": "#/$defs/posting" } },
"amountDong": { "type": "integer", "minimum": -9007199254740991, "maximum": 9007199254740991 }
```

Chúng khác nhau ở một điểm: protobuf cho `int64` đủ 64 bit, còn JSON Schema giới hạn trong ±(2^53 - 1) để client đọc số JSON bằng `double` không mất chữ số. Schema đóng (`additionalProperties: false`), nên muốn thêm trường thì phải có tệp schema mới; tệp này dùng để kiểm đầu ra của producer, không phải để consumer đọc.

`build.rs` sinh các kiểu Rust tương ứng bằng `prost-build` và dựng một descriptor set chứa mọi schema của lab; `protoc` lấy từ `protoc-bin-vendored` nên máy không cần cài thêm gì. Rust (qua `prost-reflect`), Go và Python cùng đọc descriptor set đó.

```rust
// build.rs (rút gọn)
let mut config = prost_build::Config::new();
config.protoc_executable(&protoc);
config.compile_protos(&["schemas/ledger_entry_posted.v1.proto"], &["schemas"])?;

let mut cmd = Command::new(&protoc);   // descriptor set cho mọi schema
cmd.arg("--include_imports")
    .arg(format!("--descriptor_set_out={}", out_dir.join("schemas.binpb").display()));
```

Producer dựng sự kiện bằng kiểu prost, nên sai kiểu là lỗi biên dịch:

```rust
// src/typed.rs (rút gọn)
pub fn sample_capture() -> v1::LedgerEntryPosted {
    entry_posted("JE-000001", "trip:TN-0001:capture",
        &[("customer:CU-001", -25_500), ("psp:vixanh:clearing", 25_500)],
        POSTED_AT_SECONDS)   // giờ nghiệp vụ truyền vào, không đọc đồng hồ
}
```

`cargo run` in cùng một bút toán ở ba dạng:

```text
LedgerEntryPosted JE-000001 (trip:TN-0001:capture)
  protobuf nhị phân     : 100 byte
  JSON thuần (int64=số) : 231 byte
  proto3 JSON chuẩn     : 235 byte
```

Nhị phân nhỏ hơn JSON hơn hai lần; JSON đọc được bằng mắt.

## Ba consumer thật đọc cùng một fixture

Rust sinh `fixtures/data/*.bin` và `*.json` rồi commit. C# dùng mã sinh từ chính các tệp `.proto`, còn Go và Python đọc descriptor set và dựng message động. Cả ba nhận cùng danh sách test case, đọc bằng thư viện protobuf chính thức của ngôn ngữ mình, rồi in message đọc được dưới dạng proto3 JSON để Rust so với đáp án.

```go
// consumers/go/main.go (rút gọn)
msg := dynamicpb.NewMessage(md)
switch c.Format {
case "bin":
    err = proto.Unmarshal(data, msg)
case "json":
    err = protojson.UnmarshalOptions{}.Unmarshal(data, msg)
case "json-lenient":
    err = protojson.UnmarshalOptions{DiscardUnknown: true}.Unmarshal(data, msg)
}
```

```python
# consumers/python/consume.py (rút gọn)
if case["format"] == "bin":
    msg.ParseFromString(data)
elif case["format"] == "json":
    json_format.Parse(data.decode("utf-8"), msg)
else:
    json_format.Parse(data.decode("utf-8"), msg, ignore_unknown_fields=True)
```

```csharp
// consumers/dotnet/Program.cs (rút gọn)
IMessage msg = format switch
{
    "bin" => descriptor.Parser.ParseFrom(data),
    "json" => strict.Parse(Encoding.UTF8.GetString(data), descriptor),
    _ => lenient.Parse(Encoding.UTF8.GetString(data), descriptor),   // IgnoreUnknownFields
};
```

Với bản mẫu `TN-0001`, cả bốn reader (Rust, C#, Go, Python) đọc đúng cả ba dạng: nhị phân, proto3 JSON chuẩn và JSON thuần có `amountDong` là số.

## int64 trong JSON: vì sao là chuỗi, và số lớn đi đâu?

Ánh xạ JSON chuẩn của proto3 viết `int64` thành chuỗi (`"amountDong":"-25500"`), vì số JSON ở nhiều ngôn ngữ là `double`; parser phải nhận cả số lẫn chuỗi. Lab thử với `2^53 + 1 = 9007199254740993`, số nguyên dương nhỏ nhất mà `f64` không biểu diễn được; bút toán thật không gần mức đó, mục đích là xem thư viện làm gì.

| Cách đọc số 9007199254740993 | Rust | C# | Go | Python |
|---|---|---|---|---|
| nhị phân protobuf | đúng | đúng | đúng | đúng |
| proto3 JSON, số viết thành chuỗi | đúng | đúng | đúng | đúng |
| proto3 parser, số viết thành số JSON | đúng | **9007199254740992** | đúng | đúng |
| JSON generic, kiểu mặc định | `i64`, đúng | `JsonElement`, đúng | `float64`: **...992** | `int`, đúng |
| JSON generic, ép sang `double` | ...992 | ...992 | ...992 | ...992 |

Hai dòng đáng chú ý. Parser JSON của `Google.Protobuf` (C#) nhận số JSON không báo lỗi và làm tròn về 2^53, trong khi Go, Python và Rust giữ đúng. Còn `encoding/json` của Go đọc mọi số vào `any` thành `float64`, nên consumer viết nhanh với `map[string]any` mất chữ số mà không cảnh báo. JavaScript cũng vậy: `JSON.parse("9007199254740993")` trả `9007199254740992` (đã chạy trên `node` 26.8.1, ngoài lab).

Ba lớp phòng thủ: dùng protobuf nhị phân giữa service; nếu qua JSON thì dùng dạng chuẩn viết chuỗi; và để JSON Schema chặn số ngoài ±(2^53 - 1) ngay ở producer. Lab kiểm lớp cuối: bản JSON thuần của `2^53 + 1` bị schema từ chối, protobuf nhị phân giữ đúng. Dạng chuỗi chuẩn lại không hợp lệ với schema thuần (`type: integer` không nhận chuỗi), nên chọn dạng JSON trên Kafka là quyết định hợp đồng, không phải chi tiết thư viện.

## Sáu thay đổi schema, đo trên bốn reader

Mỗi thay đổi là một cặp schema cũ và mới (`schemas/experiments/`), đo hai chiều: BACKWARD là reader mới đọc dữ liệu của producer cũ, FORWARD là reader cũ đọc dữ liệu của producer mới (Hình 2). Ba cách đọc: nhị phân, JSON chặt (trường lạ là lỗi, mặc định của cả bốn thư viện) và JSON bỏ qua trường lạ. Mỗi ô là một lần đọc thật: `OK` (mọi trường reader biết đều đúng), `LOSS` (đọc được nhưng sai mà không báo) hoặc `ERR` (thư viện báo lỗi).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 252" role="img" aria-label="Hình 2. BACKWARD là consumer mới đọc dữ liệu của producer cũ, nên nâng consumer trước; FORWARD là consumer cũ đọc dữ liệu của producer mới, nên nâng producer trước; FULL là cả hai."><defs><marker id="rust-55-b" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="230" height="66" rx="10" class="f-phys-soft s-phys"/><text x="36" y="48" class="t-b">Producer cũ</text><text x="36" y="68" class="t-small">viết bằng schema A</text><rect x="420" y="20" width="230" height="66" rx="10" class="f-pk-soft s-pk"/><text x="436" y="48" class="t-b">Consumer mới</text><text x="436" y="68" class="t-small">đọc bằng schema B</text><path d="M254 53 L416 53" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-55-b)"/><text x="292" y="42" class="t-b">BACKWARD</text><text x="292" y="76" class="t-small">đọc dữ liệu cũ</text><text x="676" y="46" class="t-b">nâng consumer trước</text><text x="676" y="66" class="t-small">producer cũ vẫn ghi</text><rect x="20" y="110" width="230" height="66" rx="10" class="f-pk-soft s-pk"/><text x="36" y="138" class="t-b">Producer mới</text><text x="36" y="158" class="t-small">viết bằng schema B</text><rect x="420" y="110" width="230" height="66" rx="10" class="f-phys-soft s-phys"/><text x="436" y="138" class="t-b">Consumer cũ</text><text x="436" y="158" class="t-small">đọc bằng schema A</text><path d="M254 143 L416 143" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-55-b)"/><text x="292" y="132" class="t-b">FORWARD</text><text x="292" y="166" class="t-small">đọc dữ liệu mới</text><text x="676" y="136" class="t-b">nâng producer trước</text><text x="676" y="156" class="t-small">consumer cũ vẫn đọc</text><rect x="20" y="196" width="840" height="40" rx="10" class="f-warn-soft s-warn dash"/><text x="36" y="221" class="t-small">FULL = cả hai chiều đều đọc được, nâng theo thứ tự nào cũng được. Luôn nói rõ là áp dụng cho dạng nhị phân hay JSON.</text></svg>
</div>
<figcaption>Hình 2. Hai chiều tương thích. Tên BACKWARD và FORWARD đặt theo việc schema nào đọc dữ liệu của schema nào, không đặt theo thời gian.</figcaption>
</figure>

| Thay đổi | Chiều | Nhị phân | JSON chặt | JSON bỏ qua trường lạ |
|---|---|---|---|---|
| thêm `trace_id = 6`, `kind = 7` | BACKWARD | OK | OK | OK |
| | FORWARD | OK | ERR | OK |
| đổi tên `idempotency_key` thành `dedup_key` | BACKWARD | OK | ERR | **LOSS** |
| | FORWARD | OK | ERR | **LOSS** |
| đổi số `amount_dong` từ 2 sang 3 | BACKWARD | **LOSS** | OK | OK |
| | FORWARD | **LOSS** | OK | OK |
| gỡ `trace_id`, có `reserved` | BACKWARD | OK | ERR | OK |
| | FORWARD | OK | OK | OK |
| gỡ `trace_id`, dùng lại số 6 cho `region` | BACKWARD | **LOSS** | ERR | OK |
| | FORWARD | **LOSS** | ERR | OK |
| thêm `ENTRY_KIND_REFUND = 3` | BACKWARD | OK | OK | OK |
| | FORWARD | OK, số 3 | ERR | **LOSS** |

Cả bốn reader cho cùng mã kết quả ở cả 36 ô; chỉ câu chữ lỗi khác nhau (`unknown field "traceId"` của Go, `Unknown field: traceId` của C#).

**Đổi tên và đổi số là hai mặt đối xứng nhau.** Nhị phân nhận diện trường bằng số nên đổi tên không động tới dây; JSON nhận diện bằng tên nên đổi số không động tới JSON. "An toàn" phải nói rõ cho dạng nào.

**Chỗ nguy hiểm là ô `LOSS`, không phải `ERR`.** Đổi số làm số tiền thành 0 mà không thư viện nào báo lỗi; dùng lại số 6 khiến `trace_id` cũ hiện thành `region`, một chuỗi hợp lệ về kiểu. Đó là việc của `reserved`:

```proto
// schemas/experiments/s3.proto: gỡ trace_id và giữ chỗ
reserved 6;
reserved "trace_id";
```

Khi có hai dòng đó, `protoc` từ chối cho dùng lại số: `s3r.proto:23:12: Field "region" uses reserved number 6.` Bản `s3x` bỏ hai dòng `reserved` và thêm `string region = 6`; ô `LOSS` ở trên là hậu quả của việc đó.

**Thêm giá trị cho enum là cái bẫy phía consumer.** Ở nhị phân, enum proto3 là enum mở: consumer cũ giữ số lạ (`kind` hiện `3`, lab ghi `OK*`) và code của consumer phải có nhánh mặc định. JSON chặt thì lỗi; JSON bỏ qua trường lạ thì `kind` về `ENTRY_KIND_UNSPECIFIED`, mất ý nghĩa mà không báo.

## Schema registry làm gì, và lab thay nó bằng gì?

Schema registry là dịch vụ lưu schema theo subject (thường gắn với topic), cấp ID cho mỗi phiên bản, và từ chối đăng ký phiên bản vi phạm mức tương thích của subject. Producer ghi ID schema vào đầu mỗi message (Confluent: một byte magic và 4 byte ID, big-endian; riêng Protobuf có thêm message index ngay sau đó); consumer lấy schema theo ID để giải mã. Ba mức chính như ở Hình 2: BACKWARD (mặc định của Confluent), FORWARD, FULL. Các mức thường chỉ so với phiên bản ngay trước đó; biến thể `_TRANSITIVE` so với mọi phiên bản trước, cần khi consumer có thể replay từ đầu topic. Với Protobuf, tài liệu Confluent khuyên `BACKWARD_TRANSITIVE`.

**Lab này không dựng schema registry**, nên phần trên là mô tả, không phải kết quả đo. Lab làm phần cốt lõi của registry: so hai descriptor. `src/compat.rs` báo từng phát hiện, kèm chiều và dạng dữ liệu bị phá:

```rust
// tests/contract.rs (rút gọn)
assert_eq!(rules("s1", "s2"),  ["FIELD_ADDED", "FIELD_ADDED"]);
assert_eq!(rules("s2", "s2r"), ["NAME_CHANGED_AT_NUMBER"]);
assert_eq!(rules("s2", "s2n"), ["NUMBER_CHANGED"]);
assert_eq!(rules("s2", "s3"),  ["FIELD_REMOVED"]);
assert_eq!(rules("s2", "s2e"), ["ENUM_VALUE_ADDED"]);
```

Test đối chiếu checker với kết quả reader Rust đo thật trên sáu cặp: không có âm tính giả (thay đổi checker cho là ổn mà đo thấy hỏng) và đúng hai dương tính giả, cả hai là đổi tên ở nhị phân. Checker chặn đổi tên vì chỉ nhìn descriptor thì checker không phân biệt được "đổi tên" với "gỡ rồi dùng lại số" (`s2r` và `s3x` cùng dấu hiệu: tên khác ở cùng số). Luật là của lab, không phải của Confluent hay Buf. `buf breaking` chia luật thành bốn nhóm, mặc định `FILE`: `FILE` và `PACKAGE` giữ cả mã sinh ra, `WIRE_JSON` giữ nhị phân và JSON, `WIRE` chỉ giữ nhị phân. Đó cũng là câu «an toàn cho dạng nào» ở bảng trên; lab không chạy Buf.

## Đổi schema không dừng: thứ tự triển khai và version ở đâu

Hai cột thứ tự được rút ra từ chính bảng đo (hàm `matrix::rollout`, in ở bước 8 của `--verify`):

| Thay đổi | Nhị phân | JSON chặt | Cách làm không dừng |
|---|---|---|---|
| thêm trường | thứ tự nào cũng được | consumer trước | thêm trường, consumer chuyển sang JSON bỏ qua trường lạ |
| thêm giá trị enum | thứ tự nào cũng được | consumer trước | consumer có nhánh mặc định cho giá trị lạ trước khi producer dùng |
| gỡ trường | thứ tự nào cũng được | producer trước | producer thôi ghi, rồi mới gỡ khỏi consumer; `reserved` số và tên |
| đổi tên | thứ tự nào cũng được | không có thứ tự an toàn | thêm trường mới, ghi cả hai, chuyển consumer, `reserved` tên cũ |
| đổi số, đổi kiểu, dùng lại số | không có thứ tự an toàn | tuỳ | topic mới (Hình 3) |

"Thứ tự nào cũng được" ở nhị phân nghĩa là thư viện không báo lỗi, chứ không có nghĩa là nghiệp vụ không đổi: consumer cũ đọc giá trị mặc định cho trường chưa biết. Với JSON, nếu để consumer bỏ qua trường lạ ngay từ đầu thì ô `ERR` của thêm trường biến mất.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 282" role="img" aria-label="Hình 3. Đổi schema vỡ dây trên Kafka: bước một producer ghi song song hai topic, bước hai từng consumer chuyển sang topic mới, bước ba ngừng topic cũ sau thời gian retention."><defs><marker id="rust-55-c" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="270" height="190" rx="10" class="f-phys-soft s-phys"/><text x="36" y="50" class="t-b">Bước 1. Ghi song song</text><text x="36" y="82" class="t-small">producer ghi cả hai topic:</text><text x="36" y="106" class="t-small">ledger.entry-posted (A)</text><text x="36" y="130" class="t-small">ledger.entry-posted.v2 (B)</text><text x="36" y="154" class="t-small">consumer vẫn đọc topic cũ</text><text x="36" y="178" class="t-small">kiểm: hai topic cùng bút toán</text><rect x="305" y="20" width="270" height="190" rx="10" class="f-phys-soft s-phys"/><text x="321" y="50" class="t-b">Bước 2. Chuyển consumer</text><text x="321" y="82" class="t-small">mỗi consumer đổi sang topic v2</text><text x="321" y="106" class="t-small">một group một lần, theo dõi lag</text><text x="321" y="130" class="t-small">so kết quả hai bên trước khi tắt</text><text x="321" y="154" class="t-small">chỗ chưa đổi vẫn chạy trên topic cũ</text><rect x="590" y="20" width="270" height="190" rx="10" class="f-phys-soft s-phys"/><text x="606" y="50" class="t-b">Bước 3. Dọn topic cũ</text><text x="606" y="82" class="t-small">không còn group nào đọc topic cũ</text><text x="606" y="106" class="t-small">đã qua thời gian retention</text><text x="606" y="130" class="t-small">producer thôi ghi topic cũ</text><text x="606" y="154" class="t-small">reserved số field đã bỏ</text><path d="M291 115 L301 115" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-55-c)"/><path d="M576 115 L586 115" class="f-none s-phys" stroke-width="2" marker-end="url(#rust-55-c)"/><rect x="20" y="226" width="840" height="40" rx="10" class="f-warn-soft s-warn dash"/><text x="36" y="251" class="t-small">Dùng khi dây vỡ (đổi số field, đổi kiểu, đổi nghĩa): không thứ tự nâng cấp nào cứu được, nên chạy hai topic.</text></svg>
</div>
<figcaption>Hình 3. Ba bước khi dây vỡ: ghi song song, chuyển consumer, dọn topic cũ.</figcaption>
</figure>

**Version ở topic hay ở payload?** `schemaVersion` trong payload phù hợp với thay đổi cộng thêm: consumer đọc được message rồi mới rẽ nhánh theo nghĩa. Cách này không cứu được thay đổi vỡ dây, vì consumer chỉ thấy số version sau khi giải mã, mà lúc đó giải mã đã sai hoặc đã báo lỗi. Khi đó version nằm ở tên topic (quy ước của Tới Nơi có hậu tố `.v1`): hai topic chạy song song, từng consumer chuyển dần, và Kafka không bảo đảm thứ tự giữa hai topic. Quy tắc: số trong payload tăng khi nghĩa đổi mà dây vẫn đọc được; topic mới khi dây không còn đọc được; và message cũ còn tồn tại đến hết retention, nên consumer mới phải đọc được mọi phiên bản còn trong topic.

## Chạy ví dụ và kiểm chứng

Từ gốc repo. `--verify` và `cargo test` không cần .NET, Go, Python hay hạ tầng.

```bash
cargo run  --manifest-path examples/toi-noi-rust/55-event-contract/Cargo.toml -- --verify
cargo test --manifest-path examples/toi-noi-rust/55-event-contract/Cargo.toml
# consumer thật: cần .NET SDK 10, Go, và Python có protobuf (nên dùng venv riêng)
python3 -m venv ~/venv-lab55 && ~/venv-lab55/bin/pip install -r examples/toi-noi-rust/55-event-contract/consumers/python/requirements.txt
TOI_NOI_PYTHON=~/venv-lab55/bin/python \
  cargo run --manifest-path examples/toi-noi-rust/55-event-contract/Cargo.toml -- --with-consumers
```

Phần đầu output của `--verify` (dòng 4 có thông báo lỗi thật của JSON Schema):

```text
1. fixture commit sẵn khớp bản sinh lại (20 tệp)
2. nhị phân s1: 100 byte, giải mã bằng prost có kiểu và prost-reflect cho cùng kết quả, tổng dòng = 0
3. kích thước: nhị phân 100 B, JSON thuần 231 B, proto3 JSON chuẩn 235 B (int64 là số / là chuỗi)
4. JSON Schema v1: bản thuần hợp lệ; 2^53+1 bị từ chối (/postings/0/amountDong: -9007199254740993 is less than the minimum of -9007199254740991); trường lạ và dạng chuỗi bị từ chối
   protobuf nhị phân giữ đúng 9007199254740993 nhưng f64 chỉ giữ 9007199254740992
5. ma trận Rust (44 ca) khớp expected_matrix.json
6. checker descriptor: 0 âm tính giả; dương tính giả: ["rename/backward/bin", "rename/forward/bin"] (...)
7. protoc từ chối s3r.proto: s3r.proto:23:12: Field "region" uses reserved number 6.
...
VERIFY OK
```

`cargo test` chạy 12 contract test: fixture commit sẵn khớp bản sinh lại; byte của v1 được ghim (`08 01 12 09 JE-000001`, tổng 100 byte); consumer có kiểu của v1 đọc được byte của producer s2; sáu biến thể JSON bị schema từ chối. Đổi hợp đồng mà quên cập nhật fixture thì test đỏ.

`--with-consumers` chạy `dotnet run`, `go run` và `python` thật, đọc stdout tới EOF và so với `fixtures/expected_matrix.json`; thiếu công cụ, consumer lỗi hoặc kết quả lệch thì thoát với mã khác 0. Cuối output:

```text
big/plain  dotnet=LOSS go=OK python=OK rust=OK
CONSUMERS OK (44 ca x 4 reader khớp expected_matrix.json)
```

Build của .NET ghi `bin/` và `obj/` vào `target/dotnet/` (đã gitignore) nhờ `Directory.Build.props`. Phiên bản đo: `Google.Protobuf` 3.36.2 (.NET SDK 10.0.401), `google.golang.org/protobuf` 1.36.12 (Go 1.27.1), `protobuf` 6.33.6 (Python 3.9.6), `prost` 0.14.4 và `prost-reflect` 0.16.5. Phiên bản khác có thể cho kết quả khác.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ</span>
<p>Không có Kafka và không có schema registry ở lab này: sự kiện là tệp, và checker descriptor là luật bảo thủ của lab. Sáu thay đổi này là những kiểu hay gặp, không phải mọi kiểu thay đổi (chưa đo <code>oneof</code>, <code>map</code>, <code>optional</code>, đổi kiểu số). JavaScript và Java không nằm trong lab. Go và Python đọc qua descriptor set chứ không qua mã sinh sẵn như C#. Chưa có <code>event_id</code>, <code>trace_id</code> như hợp đồng đầy đủ của kiến trúc đích, chưa có DLQ.</p>
</div>

## Bài tập

Đổi `int64 amount_dong = 2` thành `sint64 amount_dong = 2` (cùng số field), với lý do "sint64 mã hoá số âm gọn hơn". Dự đoán `OK`, `LOSS` hay `ERR` ở nhị phân và JSON chặt, và checker của lab sẽ nói gì.

<details><summary>Xem lời giải</summary>
<p>Nhị phân: <code>LOSS</code> ở cả hai chiều, không có lỗi nào. Hai kiểu cùng kiểu dây (varint) nhưng <code>sint64</code> dùng zigzag: -25.500 ghi bằng <code>sint64</code> thành varint của 50.999, nên reader <code>int64</code> cũ thấy 50999; byte <code>int64</code> đọc bằng reader <code>sint64</code> mới cho ra 9223372036854763058 (hai số này lấy từ một script Python ngoài lab). JSON: tên và dạng chuỗi không đổi nên <code>OK</code>. Checker so kiểu của trường nên báo <code>TYPE_CHANGED</code> và chặn cả bốn ô, kể cả hai ô JSON vốn an toàn: dương tính giả, nhưng tốt hơn bỏ sót. Cách đúng: thêm trường mới <code>sint64 amount_zigzag = 3</code>, ghi cả hai, chuyển consumer, rồi <code>reserved</code> số cũ, hoặc mở topic <code>.v2</code>.</p>
</details>

Bài trước: [bài 54](/articles/rust-use-case-thanh-toan-chuyen-end-to-end). Bài tiếp: [bài 56](/articles/rust-feature-flag-config-migration-expand-contract).

## Tài liệu gốc

- [Protocol Buffers: cập nhật một message type](https://protobuf.dev/programming-guides/proto3/#updating) và [ánh xạ JSON](https://protobuf.dev/programming-guides/json/).
- [Confluent Schema Registry: schema evolution](https://docs.confluent.io/platform/current/schema-registry/fundamentals/schema-evolution.html).
- [Confluent: wire format của serializer](https://docs.confluent.io/platform/current/schema-registry/fundamentals/serdes-develop/index.html).
- [Buf: breaking change detection](https://buf.build/docs/breaking/).
- [Crate prost-reflect](https://docs.rs/prost-reflect).
- [Crate jsonschema](https://docs.rs/jsonschema).
