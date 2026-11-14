---
slug: location-cuoc-quote-giai-thich-doi-diem-don-cho-khach
title: "Đổi điểm đón thì cước ra sao, nói với khách thế nào, và tổng kết P04"
summary: "Bài 40: cước 12.000 + 4.500/km, quote bất biến, không tăng giá lén, câu chữ cho khách và tài xế; rồi tổng kết P04 trên 20 seed, dải xác nhận và độ nhạy: cái gì có ích, cái gì không, ai trả. Trên dữ liệu mô phỏng."
category: location-routing
level: advanced
tags: [toi-noi, location, rust, pricing, quote, fare, sensitivity-analysis, simulation, summary]
language: vi
---

Chuyến TN-0001 dài 5 km. Theo mô hình cước của Tới Nơi, 12.000 VND cho 2 km đầu và 4.500 VND mỗi km vượt, khách thấy trên màn hình **25.500 đồng** và bấm đặt xe. Sau đó bộ chọn điểm đón đưa điểm hẹn sang một đoạn lề khác, vì chỗ ghim ban đầu nằm sát một cầu vượt. Quãng có khách đổi, và vì cước chỉ phụ thuộc quãng có khách, **cước cũng đổi**. Mỗi km thêm là 4.500 đồng. Không có gì sai về mặt thuật toán, nhưng khách đã đồng ý với một con số. Nếu điểm hẹn dịch đi mà khách thấy hoá đơn khác, đó là một lần mất niềm tin, dù chênh chỉ vài nghìn đồng.

Bài này làm hai việc. Thứ nhất, dựng hợp đồng về cước: quote là một ảnh chụp bất biến, đổi điểm đón chỉ tạo một **đề xuất**, và cước tăng thì khách phải đồng ý, nếu không khách trả đúng cước đã báo (**không tăng giá lén**). Kèm theo là câu chữ cho khách và cho tài xế. Thứ hai, vì đây là bài cuối của phần P04, nó **tổng kết cả phần**: cái gì có ích, cái gì không, ai trả giá, và kết luận đúng dưới giả định nào.

Đây là bài 40 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi), bài cuối của phần P04. Bài dùng điểm hẹn động của [bài 36](/articles/location-diem-hen-dong-cap-nhat-khi-hai-ben-di-chuyen) và điểm trả của [bài 38](/articles/location-toi-uu-diem-tra-cong-vao-chuyen-ke-tiep), hàm mục tiêu và phương pháp so sánh cặp của [bài 32](/articles/location-ham-muc-tieu-diem-hen-khach-va-tai-xe) (20 seed đánh giá 20270001…20270020, khoảng tin cậy t 95%, bên cạnh dải xác nhận 20270101…20270120). Mọi số trong bài là **trên dữ liệu mô phỏng**, kịch bản `hcm-baseline`, đồ thị HCM (126.699 nút, 319.074 cạnh, © OpenStreetMap contributors). Simulator không dự đoán thế giới thật. Mô hình cước ở đây là cùng mô hình với các loạt khác của dự án, như [bài về `Money` và `Quote`](/articles/csharp-record-struct-money-quote) trong loạt C#.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Điểm đón mới của <code>J</code> làm quãng có khách của chuyến thay đổi. Trên 3.780 chuyến ghép cặp theo khách, bạn đoán cước tăng ở bao nhiêu phần trăm chuyến, giảm ở bao nhiêu phần trăm? Và nếu nền tảng không bao giờ tính khách hơn cước đã báo, nền tảng chịu bao nhiêu?</p>
<details><summary>Xem đáp án</summary><p>Cước <b>giảm ở 60,3%</b> chuyến, bằng nhau ở 12,3%, <b>tăng ở 27,4%</b>. Vì <code>J</code> kéo điểm đón về phía đích, quãng có khách ngắn đi ở số đông, nên cước trung bình giảm 354 đồng mỗi chuyến. Phần chênh nền tảng chịu ở các chuyến tăng cước là 587.859 đồng, tức 0,46% tổng quote. Một điểm cần nhớ: cải thiện km rỗng của tài xế được trả bằng doanh thu mỗi chuyến thấp hơn khoảng 1%.</p></details>
</div>

## Mô hình cước: số nguyên, theo quãng có khách

Cước chỉ phụ thuộc quãng có khách, không phụ thuộc quãng tài xế chạy tới đón. Công thức dùng số nguyên đồng, cộng 500 trước khi chia 1.000 để làm tròn tới đồng gần nhất:

`examples/toi-noi-location/rust/crates/pickup/src/fare.rs`

```rust
pub const BASE_FARE_VND: u64 = 12_000;
pub const BASE_METERS: u64 = 2_000;
pub const PER_KM_VND: u64 = 4_500;

/// Cước theo quãng đường (mét): `12000 + (extraMeters·4500 + 500) / 1000` với `extraMeters = max(0, m − 2000)`.
pub fn fare_vnd(distance_m: u64) -> u64 {
    let extra = distance_m.saturating_sub(BASE_METERS);
    BASE_FARE_VND + (extra * PER_KM_VND + 500) / 1000
}
```

Lab in vài giá trị để kiểm tay:

| Quãng có khách (m) | Cước (đồng) |
|---:|---:|
| 1.500 | 12.000 |
| 2.000 | 12.000 |
| 2.001 | 12.005 |
| 3.000 | 16.500 |
| 5.000 | 25.500 |
| 6.392 | 31.764 |
| 12.133 | 57.599 |

Nguồn: `results/p04.md`, LOC-40 mục 1. Dòng 5.000 m cho 25.500 đồng chính là chuyến TN-0001 ở đầu bài. Đây là mô hình cước tối giản của loạt bài: không có giờ cao điểm, phụ phí hay khuyến mãi, và **không phải bảng giá của công ty nào**.

## Quote là snapshot bất biến

Lúc khách đặt xe, hệ thống tạo một **snapshot quote**: những gì khách đã thấy và đồng ý. Nó chứa điểm đón, điểm trả, quãng ước tính, cước, và `policy_version` của chính sách chọn điểm đón. Snapshot này không bao giờ bị sửa sau khi tạo.

`examples/toi-noi-location/rust/crates/pickup/src/fare.rs`

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuoteSnapshot {
    pub quote_id: String,
    pub pickup: [f64; 2],
    pub dropoff: [f64; 2],
    pub est_distance_m: u64,
    pub fare_vnd: u64,
    pub policy: PolicySnapshot,
}
```

Output thật của lab cho một quote ví dụ (`quote_id` `Q-20270001-0000`) cùng chính sách đi kèm (có mã băm FNV-1a `ed054ea27b008e90`, để kiểm sau này rằng chính sách trong snapshot là chính sách đã dùng):

```text
  chính sách điểm đón đi kèm quote: {"policy_version":"p04-v1","weights":[1.0,1.0,1.0,1.0],"max_walk_m":250.0,"penalties":{"wrong_side_s":30.0,"tail_walk_s_per_100m":20.0,"comfort_walk_m":150.0,"alley_stop_s":20.0}}
  mã băm chính sách (FNV-1a): ed054ea27b008e90
  ví dụ snapshot quote (bất biến, lưu lúc đặt xe): {"quote_id":"Q-20270001-0000","pickup":[10.764697563766253,106.68217252368436],"dropoff":[10.79380962338618,106.68761215365417],"est_distance_m":6392,"fare_vnd":31764,"policy":{"policy_version":"p04-v1","weights":[1.0,1.0,1.0,1.0],"max_walk_m":250.0,"penalties":{"wrong_side_s":30.0,"tail_walk_s_per_100m":20.0,"comfort_walk_m":150.0,"alley_stop_s":20.0}}}
```

Hợp đồng với Pricing (C#) có ba điểm:

1. **Rust đề xuất, C# quyết định.** `geo-engine` trả về một `Proposal` (cước cũ, cước mới, chênh, quãng đi bộ thêm, `policy_version`); Pricing và Dispatch ở C# là nơi **lưu** quote và quyết định số tiền khách thật sự phải trả. Đây là cùng ranh giới của [bài 02](/articles/location-adr-rust-go-python-csharp-ranh-gioi-service).
2. **Đồng ý của khách gắn với `quote_id`.** Một lần đồng ý chỉ áp dụng cho đúng đề xuất đó; để an toàn khi gọi lại, bản ghi đồng ý nên idempotent theo `quote_id` (cùng hướng với hợp đồng của [bài 46](/articles/location-goi-y-sang-dispatch-csharp-grpc-lease-idempotency)).
3. **Chính sách đi theo quote.** `policy_version` và mã băm lưu cùng snapshot, để khi có khiếu nại ta biết điểm đón đã được chọn theo chính sách nào.

Lab chỉ có các hàm thuần của Rust (`fare_vnd`, `propose_move`, `settle`, hai hàm sinh câu chữ) và test của chúng. **Lab chưa có Pricing C# thật**: chưa có bảng lưu quote, chưa có luồng đồng ý qua mạng, chưa chia phần chênh giữa nền tảng và tài xế. Ba điểm trên là hợp đồng đề xuất, chưa kiểm trong một hệ thống thật.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 1. Hợp đồng quote khi đổi điểm đón. Quote là snapshot bất biến lúc đặt xe. Đổi điểm đón chỉ tạo đề xuất với cước mới tính theo quãng mới. Cước giảm hoặc bằng thì áp dụng tự động và báo khách. Cước tăng cần khách đồng ý: đồng ý thì trả cước mới, từ chối hoặc im lặng thì trả cước đã báo và nền tảng chịu phần chênh."><rect x="20" y="50" width="200" height="90" rx="8" class="f-surface s-line" stroke-width="1.5"/><text x="120" y="74" class="t-b" text-anchor="middle">Quote snapshot (bất biến)</text><text x="120" y="92" class="t-small" text-anchor="middle">pickup, dropoff</text><text x="120" y="110" class="t-small" text-anchor="middle">est_distance_m, fare_vnd</text><text x="120" y="128" class="t-small" text-anchor="middle">policy_version</text><rect x="262" y="50" width="200" height="90" rx="8" class="f-surface s-line" stroke-width="1.5"/><text x="362" y="74" class="t-b" text-anchor="middle">Đề xuất đổi điểm đón</text><text x="362" y="92" class="t-small" text-anchor="middle">propose_move: cước mới</text><text x="362" y="110" class="t-small" text-anchor="middle">theo quãng mới</text><text x="362" y="128" class="t-small" text-anchor="middle">(Proposal)</text><rect x="520" y="20" width="340" height="62" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="690" y="44" class="t-b" text-anchor="middle">Cước giảm hoặc bằng</text><text x="690" y="62" class="t-small" text-anchor="middle">áp dụng tự động, báo cho khách</text><rect x="520" y="106" width="340" height="62" rx="8" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="690" y="130" class="t-b" text-anchor="middle">Cước tăng (NeedsConsent)</text><text x="690" y="148" class="t-small" text-anchor="middle">hỏi khách, chưa tính tiền</text><rect x="520" y="206" width="160" height="78" rx="8" class="f-surface s-line" stroke-width="1.5"/><text x="600" y="230" class="t-b" text-anchor="middle">Khách đồng ý</text><text x="600" y="248" class="t-small" text-anchor="middle">trả cước mới</text><rect x="700" y="206" width="160" height="78" rx="8" class="f-surface s-line" stroke-width="1.5"/><text x="780" y="230" class="t-b" text-anchor="middle">Từ chối hoặc im lặng</text><text x="780" y="248" class="t-small" text-anchor="middle">trả cước đã báo</text><text x="780" y="266" class="t-small" text-anchor="middle">nền tảng chịu chênh</text><path d="M224 95 L249.0 95.0" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><path d="M258 95 L249.0 99.5 L249.0 90.5 z" class="f-muted"/><path d="M466 80 L508.2 55.5" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><path d="M516 51 L510.5 59.4 L506.0 51.6 z" class="f-muted"/><path d="M466 110 L507.6 126.7" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><path d="M516 130 L506.0 130.8 L509.3 122.5 z" class="f-muted"/><path d="M600 172 L600.0 193.0" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><path d="M600 202 L595.5 193.0 L604.5 193.0 z" class="f-muted"/><path d="M780 172 L780.0 193.0" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><path d="M780 202 L775.5 193.0 L784.5 193.0 z" class="f-muted"/><text x="36" y="312" class="t-small">Hình minh hoạ hợp đồng của crate pickup::fare. Lab chưa có Pricing C# thật; Pricing vẫn là nơi lưu quote.</text></svg>
</div>
<figcaption>Hình 1. Vòng đời của một quote khi điểm đón đổi. Quote không bao giờ bị sửa; đổi điểm đón chỉ tạo đề xuất. Cước giảm áp dụng tự động; cước tăng cần khách đồng ý, không thì khách trả cước đã báo. Hình minh hoạ hợp đồng, không phải một hệ thống đã chạy.</figcaption>
</figure>

## Không tăng giá lén: quy tắc nằm ở một hàm

Quy tắc «không tăng giá lén» gói gọn trong hàm tính tiền cuối chuyến:

`examples/toi-noi-location/rust/crates/pickup/src/fare.rs`

```rust
pub fn settle(q: &QuoteSnapshot, actual_distance_m: u64, consent: Option<bool>) -> Settlement {
    let actual = fare_vnd(actual_distance_m);
    match consent {
        Some(true) => Settlement {
            charged_vnd: actual,
            absorbed_vnd: 0,
            moved: true,
        },
        _ => {
            let charged = actual.min(q.fare_vnd);
            Settlement {
                charged_vnd: charged,
                absorbed_vnd: actual - charged,
                moved: consent.is_some_and(|c| c),
            }
        }
    }
}
```

Nếu khách đồng ý rõ ràng (`Some(true)`), khách trả cước mới. Nếu khách từ chối (`Some(false)`) hoặc **chưa trả lời** (`None`), khách trả `min(cước thật, cước đã báo)`: **không bao giờ hơn** cước đã báo. Phần chênh (`absorbed_vnd`) là của bên khác. Im lặng không phải là đồng ý, và đó là điểm chính của quy tắc.

Với quote ví dụ ở trên (6.392 m, 31.764 đồng), lab tạo một đề xuất khi quãng mới là 6.850 m:

```text
  đề xuất đổi điểm đón: {"quote_id":"Q-20270001-0000","old_fare_vnd":31764,"new_fare_vnd":33825,"delta_vnd":2061,"extra_walk_m":90,"effect":"NeedsConsent","policy_version":"p04-v1"}
  hợp đồng: khách không đồng ý/chưa trả lời → trả 31.764 đồng (cước đã báo), nền tảng chịu 2.061 đồng; đồng ý → trả 33.825 đồng
```

Cước giảm thì áp dụng tự động (`FareEffect::Lower`). Quy tắc của lab chỉ nói về **tiền**: cước giảm nhưng khách đi bộ thêm 90 m vẫn là một thay đổi khách nên biết, và nếu đổi sau khi đặt xe thì nên được hỏi. Câu chữ ở mục sau luôn nói cả hai.

## Câu chữ cho khách và cho tài xế

Hai hàm sinh câu chữ nằm cạnh hàm tính cước. Nguyên tắc: nói rõ **cước và quãng đi bộ**, không dùng từ ép buộc, mặc định là «không đổi» nếu khách không làm gì.

`examples/toi-noi-location/rust/crates/pickup/src/fare.rs`

```rust
        FareEffect::NeedsConsent => format!(
            "Điểm đón mới làm cước tăng từ {} lên {} (bạn đi bộ thêm {} m). Bạn chỉ bị tính cước mới nếu bấm đồng ý; nếu không, cước vẫn là {}.",
            vnd(p.old_fare_vnd),
            vnd(p.new_fare_vnd),
            p.extra_walk_m.max(0),
            vnd(p.old_fare_vnd)
        ),
```

Output thật cho khách và tài xế (từ quote ví dụ):

```text
  câu cho khách (cước tăng): Điểm đón mới làm cước tăng từ 31.764 đồng lên 33.825 đồng (bạn đi bộ thêm 90 m). Bạn chỉ bị tính cước mới nếu bấm đồng ý; nếu không, cước vẫn là 31.764 đồng.
  câu cho khách (cước giảm): Chúng tôi đề xuất đổi điểm đón, bạn đi bộ thêm 90 m. Cước giảm từ 31.764 đồng còn 29.550 đồng.
  câu cho tài xế: Điểm đón đã đổi: bạn chạy ít hơn 180 m tới chỗ khách.
```

Ba nguyên tắc rút từ các câu này:

- **Nêu cả hai con số cạnh nhau**: cước cũ và cước mới, và quãng đi bộ thêm. Giấu quãng đi bộ khi cước giảm là cách làm khách thấy mình bị chọn thay.
- **Khẳng định hậu quả của im lặng**: «nếu không, cước vẫn là …». Khách không cần đoán điều gì xảy ra nếu bỏ qua.
- **Câu cho tài xế chỉ nói quãng chạy**, không nói về cước của khách. Tài xế cần biết mình chạy ít hơn hay nhiều hơn bao nhiêu mét tới khách; cước của khách là việc giữa khách và nền tảng.

Lab **chưa** thử các câu này với người dùng thật; chúng là bản nháp có số liệu đúng.

## Điểm đón đổi thì cước đổi bao nhiêu

Lab so, cho từng khách hoàn thành ở cả hai lần chạy, cước theo quãng có khách của baseline (đón đúng ghim, đóng vai «quote») với cước theo quãng của chuyến dùng điểm đón `J`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 2. Khi điểm đón đổi theo J p04-v1, cước thay đổi thế nào so với quote theo quãng baseline, trên 3.780 chuyến ghép cặp. Cước giảm 60,3%, bằng nhau 12,3%, tăng 27,4%. Tổng cước theo quote 127.779.528 đồng; theo quãng thật 126.439.610 đồng; khách trả khi không ai đồng ý cước tăng 125.851.751 đồng; nền tảng chịu 587.859 đồng."><text x="36" y="28" class="t-b">Cước thay đổi thế nào khi điểm đón đổi (3.780 chuyến ghép cặp, 20 seed)</text><rect x="36" y="44" width="487.224" height="40" rx="4" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="279.612" y="69" class="t-b" text-anchor="middle">giảm 60,3%</text><rect x="523.224" y="44" width="99.384" height="40" rx="4" class="f-surface s-line" stroke-width="1.5"/><text x="572.916" y="69" class="t-b" text-anchor="middle">bằng 12,3%</text><rect x="622.608" y="44" width="221.392" height="40" rx="4" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="733.304" y="69" class="t-b" text-anchor="middle">tăng 27,4%</text><text x="36" y="112" class="t-mono">Chênh theo chuyến (đồng): TB −354, p5 −2.164, p50 −67, p95 +778, min −24.079, max +32.841</text><text x="36" y="142">Tổng cước theo quote (quãng baseline)</text><text x="380" y="142" class="t-mono">127.779.528 đồng</text><text x="36" y="164">Tổng cước theo quãng thật, nếu tính hết</text><text x="380" y="164" class="t-mono">126.439.610 đồng (−1,05%)</text><text x="36" y="186">Khách trả khi cước tăng chưa được đồng ý</text><text x="380" y="186" class="t-mono">125.851.751 đồng (−1,51%)</text><text x="36" y="208">Nền tảng chịu phần chênh</text><text x="380" y="208" class="t-mono">587.859 đồng (0,46% tổng quote)</text><text x="36" y="230">Nếu hỏi khách (mô hình đồng ý giả định)</text><text x="380" y="230" class="t-mono">126.312.400 đồng (−1,15%); 140 chuyến bị từ chối</text><text x="36" y="262" class="t-small">Trên dữ liệu mô phỏng, seed 20270001…20270020. Mô hình đồng ý của khách: 0,95 − 0,004 × mét đi thêm (giả định).</text></svg>
</div>
<figcaption>Hình 2. Cước thay đổi thế nào khi điểm đón đổi theo <code>p04-v1</code>, trên 3.780 chuyến ghép cặp theo khách (20 seed): đa số giảm, hơn một phần tư tăng. Dưới: tổng tiền theo từng cách tính. Trên dữ liệu mô phỏng.</figcaption>
</figure>

Hình 2 nói ba điều. Một: đa số cước **giảm** (60,3%) vì `J` kéo điểm đón về phía đích, và trung bình giảm 354 đồng mỗi chuyến. Hai: đuôi dài, p95 +778 đồng nhưng max +32.841 đồng; một chuyến có thể tăng hơn ba mươi nghìn đồng, và đó là loại chuyến mà quy tắc «khách đồng ý trước» phải bắt. Ba: tổng cước theo quãng thật thấp hơn quote 1,05%, **nền tảng và tài xế thu ít hơn** vì quãng có khách ngắn đi. Khi nền tảng không tính khách hơn cước đã báo, khách trả tổng cộng ít hơn quote 1,51%, và nền tảng chịu thêm 587.859 đồng (0,46% tổng quote). Nếu hỏi khách bằng mô hình đồng ý giả định (`0,95 − 0,004 × mét đi thêm`, **là giả định**), 140 chuyến tăng cước bị từ chối.

Ai được lợi và ai chịu thiệt: khách đang trả cước thấp hơn (đa số) được lợi về tiền, nhưng cũng là bên đi bộ thêm; tài xế và nền tảng thu ít hơn khoảng 1% **trên mỗi chuyến**, vì quãng có khách ngắn đi. Tổng tiền thì chưa chắc giảm: số chuyến hoàn thành mỗi seed tăng từ 192,2 lên 195,4 vì ít huỷ hơn, nên nhân hai trung bình cho khoảng 6,51 triệu so với 6,54 triệu đồng mỗi seed (phép nhân từ bảng, lab không in và không có khoảng tin cậy). Lab không chia phần chênh 587.859 đồng giữa nền tảng và tài xế; đó là một quyết định kinh doanh phải ghi vào hợp đồng với Pricing, không phải của thuật toán.

Gợi ý thiết kế (chưa đo): tính quote từ **điểm đón đã chọn**, và hiện điểm đón trước khi khách xác nhận; khi đó đề xuất đổi cước chỉ xuất hiện khi điểm đón đổi thật sau lúc đặt, như ở bài 36.

## Tổng kết P04: cái gì có ích, cái gì không, ai trả

Phần P04 chạy trên một nguyên tắc: không chỉnh tới khi đẹp. Trọng số chọn theo quy tắc nêu trước. Quy tắc này có kẽ hở: bản v1 chỉ có cổng p95 của `walk_m`, nên nó chọn một cấu hình làm khách điển hình đi bộ thêm 132 m trong khi p95 vẫn «tốt hơn» (vì đuôi baseline do ca ghim sai quyết định). Cổng thứ hai (chênh `walk_m` trung bình ≤ +25 m, quy tắc v2) được thêm **sau khi thấy** kết quả đánh giá của cấu hình mà v1 chọn. Vì thế số của `p04-v1` trên dải đánh giá không còn «sạch» cho quyết định chọn trọng số, nên lab đo lại trên dải xác nhận 20270101…20270120, chưa từng dùng khi chọn. Dải đó nhân đôi bằng chứng cho đúng một cấu hình; nó không xoá lối rẽ (bài 32 giải thích).

**Lưu ý đọc `walk_m`.** Baseline có một đuôi đi vòng, gắn với cách simulator sinh khách và với đồ thị đi bộ (nguyên nhân cụ thể chưa xác minh): 11 trong 227 khách (seed 20261106) có `walk_m` trên 300 m, đều có chim bay dưới 150 m, chiếm 64,4% tổng quãng đi bộ của baseline. Bỏ khách đuôi và so cặp theo seed thì `walk_m` trung bình +21,2 m [+19,4; +23,1], p50 +23,3 m [+19,9; +26,8], và p95 **đổi dấu** thành +14,7 m [+4,9; +24,5] (dải xác nhận: +22,2 [+18,7; +25,7], +25,1 [+21,4; +28,7], +15,7 [+0,6; +30,7]). Tức `walk_m` p95 và max «giảm» là bỏ các ca ghim sai, không phải thắng thật.

Bảng tổng hợp dưới đây chỉ dùng số trong `results/p04.md`. Mọi số **trên dữ liệu mô phỏng**, 20 seed đánh giá, chênh cặp so baseline, khoảng tin cậy t 95% trong ngoặc.

| Việc | Kết quả đo | Ai được lợi | Ai trả | Dưới giả định nào còn đúng |
|---|---|---|---|---|
| Điểm đón `J` `p04-v1` (bài 32–35) | `deadhead_m` −107,0 m [−151,6; −62,4] (2.357,0 → 2.250,0, −4,5%); `pickup_eta_s` −21,7 s [−26,6; −16,8]; `meet_gap_s` −37,1 s [−41,8; −32,3]; `u_turns` 1,6 → 1,3; `cancel_rate` −0,0141 [−0,0210; −0,0072] | tài xế (km rỗng, quay đầu), khách chờ xe (ETA, huỷ) | khách điển hình: `walk_m` p50 +26,5 m [+23,1; +30,0] (14,3 → 40,9), bỏ khách đuôi trung bình +21,2 m [+19,4; +23,1]; nền tảng: cước −368 đồng [−484,7; −251,3] mỗi chuyến | ETA, `meet_gap_s`, quay đầu, huỷ, chuyến hoàn thành: đúng ở cả 6 bộ giả định. Km rỗng: không khác biệt rõ khi chỉ có 60 xe. `walk_m` p50 xấu hơn ở 5/6 bộ |
| Điểm hẹn động, hysteresis đặt trước (bài 36) | đổi 0,00–0,06 lần mỗi ca; không chỉ số nào có khoảng loại 0, trừ một ô `walk_m` −10,4 m ở khách đi chậm | không ai (đứng im) | không ai | kết quả rỗng; ngưỡng đặt trước quá chặt (quét bỏ cổng 10% đổi 0,29–0,86 lần mỗi ca, chưa kiểm định cặp) |
| Điểm hẹn động, «đổi ngay» (bài 36) | tài xế trễ 120 s: `t_pick_s` −22,5 s [−32,1; −12,9], `drive_m` −93,5 m [−121,5; −65,5] | tài xế, và khách khi tài xế trễ | khách: `walk_m` +59,0 m [+40,8; +77,1], 0,64 lần đổi qua đổi lại mỗi ca | khi khách đi chậm thì `meet_gap_s` +22,5 s [+12,2; +32,8] và tài xế chạy thêm 24,8 m |
| Nơi khó: cổng, hẻm, vùng bắt buộc (bài 37) | 0,7% khách mẫu có cổng OSM trong 80 m; hẻm: ô tô đi bộ TB 45 m so với xe máy 3 m; vùng bắt buộc (dữ liệu **bịa**) +415 m và +303 m | cơ chế an toàn (chuỗi lùi ghi cảnh báo) | khách trong hẻm đi ô tô; khách trong vùng bắt buộc | vùng bắt buộc là dữ liệu bịa và chưa cắm vào `JPickup`, nên không có trong các dòng khác của bảng; «lùi về ghim» phụ thuộc ghim lệch chỉ 12 m; một seed, không khoảng tin cậy |
| Điểm trả riêng (bài 38) | `deadhead_m` −16,3 m [−51,5; +18,9] (không khác biệt rõ); `pickup_eta_s` −5,6 s [−9,3; −1,9] | chưa chứng minh | khách đi bộ thêm +10,6 m [+9,0; +12,2] ở điểm trả | chuyến kế chỉ là tâm điểm nóng; thước đi bộ không gồm đoạn vào tới đích; không đo thời gian băng đường |
| Điểm đón + điểm trả (bài 38, 40) | `deadhead_m` −129,4 m [−171,1; −87,8]; dải xác nhận −129,5 m [−172,0; −86,9] | tài xế | khách: `walk_m` p50 +26,4 m [+22,7; +30,1] và đi bộ ở điểm trả +11,9 m [+9,4; +14,4] | phần cộng thêm của điểm trả so với điểm đón riêng chưa kiểm định trực tiếp |
| Hub học bằng DBSCAN (bài 39) | `deadhead_m` −0,4 m [−1,5; +0,7]; hub có mặt trong 5,5% yêu cầu | không ai | không ai | lịch sử là vòng tròn, thưa; chỉ chứng minh phương pháp |
| Cước và quote (bài 40) | cước tăng 27,4%, bằng 12,3%, giảm 60,3% chuyến; TB −354 đồng mỗi chuyến; nền tảng chịu 587.859 đồng (0,46%) | khách (cước thấp hơn ở số đông) | nền tảng và/hoặc tài xế (doanh thu mỗi chuyến −1,05%); lab không chia | quy tắc «không tăng giá lén» đúng theo thiết kế; mô hình đồng ý là giả định |

Ba điều đọc từ bảng. **Một**: trên mô phỏng này, lợi ích bền nhất nằm ở **ETA, quay đầu, huỷ** của điểm đón `J`: chúng tốt hơn ở cả sáu bộ giả định. Mô hình huỷ gốc chỉ phụ thuộc ETA, nên «huỷ giảm» là hệ quả của ETA ngắn hơn. Bộ «huỷ theo đi bộ» là một **giả định thay thế**, không phải hiệu chuẩn. **Hai**: ba ý tưởng cho kết quả rỗng hoặc chưa chứng minh được (hysteresis đặt trước, điểm trả riêng, hub DBSCAN). **Ba**: mọi lợi ích đều có người trả, và người trả gần như luôn là **khách điển hình** ở quãng đi bộ.

Nếu chỉ chọn một việc để làm trước, nên chọn điểm đón `J`, nhưng không nên giới thiệu nó là «giảm km rỗng»: ở 60 xe câu đó chưa chứng minh được. Câu đúng hơn, trên mô phỏng: «ETA, quay đầu và huỷ tốt hơn ở mọi giả định đã thử, km rỗng tốt hơn ở 5/6, và khách điển hình đi bộ thêm 20–30 m, gần 50 m nếu GPS nhiễu 15 m». Đó là nhận định từ các số trên, không phải một kết quả đo.

### Dải xác nhận: kết quả lặp lại, và một chỉ số xấu hơn rõ

Trên 20 seed 20270101…20270120 (chưa từng nhìn khi chọn trọng số, cùng cấu hình `p04-v1`, không chỉnh gì thêm):

| Chỉ số | Baseline | `p04-v1` | Chênh cặp [KTC 95% t] |
|---|---:|---:|---:|
| `deadhead_m` trung bình | 2.368,2 | 2.253,2 | −115,0 [−152,8; −77,2] |
| `pickup_eta_s` trung bình | 328,4 | 306,7 | −21,7 [−25,8; −17,6] |
| `cancel_rate` | 0,1415 | 0,1277 | −0,0138 [−0,0194; −0,0082] |
| `walk_m` p50 | 13,8 | 42,2 | +28,4 [+24,5; +32,3] |
| `walk_m` trung bình | 61,3 | 68,7 | **+7,3** [+2,7; +12,0] |

Nguồn: `results/p04.md`, LOC-40 mục 5. Hai điểm đáng chú ý. `walk_m` trung bình ở dải đánh giá là +3,5 m [−1,1; +8,1], không khác biệt rõ, nhưng ở dải xác nhận là **+7,3 m [+2,7; +12,0], xấu hơn rõ**: kết luận «không đổi» ở dải đầu là thiếu độ phân giải, không phải sự thật. Và `deadhead_m` p95 không khác biệt rõ ở dải đánh giá (−99,7 m [−344,6; +145,2]) nhưng giảm rõ ở dải xác nhận. Cùng cấu hình, khác dải seed, hai chỉ số đổi nhãn. Điều đó không có nghĩa là hiệu ứng đã đổi: các khoảng của hai dải chồng nhau rộng. Nó có nghĩa là «không khác biệt rõ» không bao giờ nên đọc là «không có tác động», và một khoảng vừa đủ loại 0 cũng không nên đọc là chắc chắn. Hiệu chỉnh Bonferroni cho 19 chỉ số (bài 32) đưa cả hai dòng này về chứa 0, trong khi `deadhead_m` trung bình, ETA, `walk_m` p50 và huỷ vẫn loại 0 ở cả hai dải.

## Độ nhạy: kết luận chỉ đúng ở một bộ giả định thì không bền

Mỗi bộ đổi **một** giả định và chạy lại cả baseline với cùng giả định: khách đi bộ chậm 0,9 m/s; mô hình huỷ có tính quãng đi bộ; 60 tài xế; 240 tài xế; nhiễu GPS 15 m.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 469" role="img" aria-label="Hình 3. Độ nhạy của kết luận điểm đón p04-v1 dưới sáu bộ giả định, 13 chỉ số, 20 seed. Dấu cộng là tốt hơn baseline, dấu trừ là xấu hơn, số 0 là không khác biệt rõ. Năm chỉ số tốt hơn ở mọi bộ giả định: ETA trung bình, khoảng lệch giờ gặp trung bình, quay đầu, huỷ, số chuyến hoàn thành. Khách điển hình đi bộ thêm (walk_m p50 xấu hơn) ở 5 trong 6 bộ. Km rỗng trung bình không khác biệt rõ khi chỉ có 60 xe."><text x="36" y="28" class="t-b">Điểm đón J p04-v1 so với baseline dưới sáu bộ giả định</text><text x="336" y="56" class="t-small" text-anchor="middle">gốc</text><text x="428" y="56" class="t-small" text-anchor="middle">đi bộ chậm</text><text x="520" y="56" class="t-small" text-anchor="middle">huỷ theo đi bộ</text><text x="612" y="56" class="t-small" text-anchor="middle">ít xe (60)</text><text x="704" y="56" class="t-small" text-anchor="middle">nhiều xe (240)</text><text x="796" y="56" class="t-small" text-anchor="middle">GPS nhiễu 15 m</text><text x="36" y="83">walk_m trung bình</text><rect x="292" y="67" width="88" height="22" class="f-surface s-line"/><text x="336" y="83" class="t-b" text-anchor="middle">0</text><rect x="384" y="67" width="88" height="22" class="f-pk-soft s-pk"/><text x="428" y="83" class="t-b" text-anchor="middle">+</text><rect x="476" y="67" width="88" height="22" class="f-warn-soft s-warn"/><text x="520" y="83" class="t-b" text-anchor="middle">−</text><rect x="568" y="67" width="88" height="22" class="f-surface s-line"/><text x="612" y="83" class="t-b" text-anchor="middle">0</text><rect x="660" y="67" width="88" height="22" class="f-surface s-line"/><text x="704" y="83" class="t-b" text-anchor="middle">0</text><rect x="752" y="67" width="88" height="22" class="f-warn-soft s-warn"/><text x="796" y="83" class="t-b" text-anchor="middle">−</text><text x="36" y="108">walk_m p50</text><rect x="292" y="92" width="88" height="22" class="f-warn-soft s-warn"/><text x="336" y="108" class="t-b" text-anchor="middle">−</text><rect x="384" y="92" width="88" height="22" class="f-pk-soft s-pk"/><text x="428" y="108" class="t-b" text-anchor="middle">+</text><rect x="476" y="92" width="88" height="22" class="f-warn-soft s-warn"/><text x="520" y="108" class="t-b" text-anchor="middle">−</text><rect x="568" y="92" width="88" height="22" class="f-warn-soft s-warn"/><text x="612" y="108" class="t-b" text-anchor="middle">−</text><rect x="660" y="92" width="88" height="22" class="f-warn-soft s-warn"/><text x="704" y="108" class="t-b" text-anchor="middle">−</text><rect x="752" y="92" width="88" height="22" class="f-warn-soft s-warn"/><text x="796" y="108" class="t-b" text-anchor="middle">−</text><text x="36" y="133">walk_m p95</text><rect x="292" y="117" width="88" height="22" class="f-pk-soft s-pk"/><text x="336" y="133" class="t-b" text-anchor="middle">+</text><rect x="384" y="117" width="88" height="22" class="f-pk-soft s-pk"/><text x="428" y="133" class="t-b" text-anchor="middle">+</text><rect x="476" y="117" width="88" height="22" class="f-pk-soft s-pk"/><text x="520" y="133" class="t-b" text-anchor="middle">+</text><rect x="568" y="117" width="88" height="22" class="f-pk-soft s-pk"/><text x="612" y="133" class="t-b" text-anchor="middle">+</text><rect x="660" y="117" width="88" height="22" class="f-pk-soft s-pk"/><text x="704" y="133" class="t-b" text-anchor="middle">+</text><rect x="752" y="117" width="88" height="22" class="f-surface s-line"/><text x="796" y="133" class="t-b" text-anchor="middle">0</text><text x="36" y="158">walk_m max</text><rect x="292" y="142" width="88" height="22" class="f-pk-soft s-pk"/><text x="336" y="158" class="t-b" text-anchor="middle">+</text><rect x="384" y="142" width="88" height="22" class="f-pk-soft s-pk"/><text x="428" y="158" class="t-b" text-anchor="middle">+</text><rect x="476" y="142" width="88" height="22" class="f-pk-soft s-pk"/><text x="520" y="158" class="t-b" text-anchor="middle">+</text><rect x="568" y="142" width="88" height="22" class="f-pk-soft s-pk"/><text x="612" y="158" class="t-b" text-anchor="middle">+</text><rect x="660" y="142" width="88" height="22" class="f-pk-soft s-pk"/><text x="704" y="158" class="t-b" text-anchor="middle">+</text><rect x="752" y="142" width="88" height="22" class="f-surface s-line"/><text x="796" y="158" class="t-b" text-anchor="middle">0</text><text x="36" y="183">deadhead_m trung bình</text><rect x="292" y="167" width="88" height="22" class="f-pk-soft s-pk"/><text x="336" y="183" class="t-b" text-anchor="middle">+</text><rect x="384" y="167" width="88" height="22" class="f-pk-soft s-pk"/><text x="428" y="183" class="t-b" text-anchor="middle">+</text><rect x="476" y="167" width="88" height="22" class="f-pk-soft s-pk"/><text x="520" y="183" class="t-b" text-anchor="middle">+</text><rect x="568" y="167" width="88" height="22" class="f-surface s-line"/><text x="612" y="183" class="t-b" text-anchor="middle">0</text><rect x="660" y="167" width="88" height="22" class="f-pk-soft s-pk"/><text x="704" y="183" class="t-b" text-anchor="middle">+</text><rect x="752" y="167" width="88" height="22" class="f-pk-soft s-pk"/><text x="796" y="183" class="t-b" text-anchor="middle">+</text><text x="36" y="208">deadhead_m p95</text><rect x="292" y="192" width="88" height="22" class="f-surface s-line"/><text x="336" y="208" class="t-b" text-anchor="middle">0</text><rect x="384" y="192" width="88" height="22" class="f-pk-soft s-pk"/><text x="428" y="208" class="t-b" text-anchor="middle">+</text><rect x="476" y="192" width="88" height="22" class="f-surface s-line"/><text x="520" y="208" class="t-b" text-anchor="middle">0</text><rect x="568" y="192" width="88" height="22" class="f-surface s-line"/><text x="612" y="208" class="t-b" text-anchor="middle">0</text><rect x="660" y="192" width="88" height="22" class="f-pk-soft s-pk"/><text x="704" y="208" class="t-b" text-anchor="middle">+</text><rect x="752" y="192" width="88" height="22" class="f-surface s-line"/><text x="796" y="208" class="t-b" text-anchor="middle">0</text><text x="36" y="233">pickup_eta_s trung bình</text><rect x="292" y="217" width="88" height="22" class="f-pk-soft s-pk"/><text x="336" y="233" class="t-b" text-anchor="middle">+</text><rect x="384" y="217" width="88" height="22" class="f-pk-soft s-pk"/><text x="428" y="233" class="t-b" text-anchor="middle">+</text><rect x="476" y="217" width="88" height="22" class="f-pk-soft s-pk"/><text x="520" y="233" class="t-b" text-anchor="middle">+</text><rect x="568" y="217" width="88" height="22" class="f-pk-soft s-pk"/><text x="612" y="233" class="t-b" text-anchor="middle">+</text><rect x="660" y="217" width="88" height="22" class="f-pk-soft s-pk"/><text x="704" y="233" class="t-b" text-anchor="middle">+</text><rect x="752" y="217" width="88" height="22" class="f-pk-soft s-pk"/><text x="796" y="233" class="t-b" text-anchor="middle">+</text><text x="36" y="258">pickup_eta_s p95</text><rect x="292" y="242" width="88" height="22" class="f-surface s-line"/><text x="336" y="258" class="t-b" text-anchor="middle">0</text><rect x="384" y="242" width="88" height="22" class="f-pk-soft s-pk"/><text x="428" y="258" class="t-b" text-anchor="middle">+</text><rect x="476" y="242" width="88" height="22" class="f-surface s-line"/><text x="520" y="258" class="t-b" text-anchor="middle">0</text><rect x="568" y="242" width="88" height="22" class="f-surface s-line"/><text x="612" y="258" class="t-b" text-anchor="middle">0</text><rect x="660" y="242" width="88" height="22" class="f-pk-soft s-pk"/><text x="704" y="258" class="t-b" text-anchor="middle">+</text><rect x="752" y="242" width="88" height="22" class="f-pk-soft s-pk"/><text x="796" y="258" class="t-b" text-anchor="middle">+</text><text x="36" y="283">meet_gap_s trung bình</text><rect x="292" y="267" width="88" height="22" class="f-pk-soft s-pk"/><text x="336" y="283" class="t-b" text-anchor="middle">+</text><rect x="384" y="267" width="88" height="22" class="f-pk-soft s-pk"/><text x="428" y="283" class="t-b" text-anchor="middle">+</text><rect x="476" y="267" width="88" height="22" class="f-pk-soft s-pk"/><text x="520" y="283" class="t-b" text-anchor="middle">+</text><rect x="568" y="267" width="88" height="22" class="f-pk-soft s-pk"/><text x="612" y="283" class="t-b" text-anchor="middle">+</text><rect x="660" y="267" width="88" height="22" class="f-pk-soft s-pk"/><text x="704" y="283" class="t-b" text-anchor="middle">+</text><rect x="752" y="267" width="88" height="22" class="f-pk-soft s-pk"/><text x="796" y="283" class="t-b" text-anchor="middle">+</text><text x="36" y="308">u_turns mỗi chuyến</text><rect x="292" y="292" width="88" height="22" class="f-pk-soft s-pk"/><text x="336" y="308" class="t-b" text-anchor="middle">+</text><rect x="384" y="292" width="88" height="22" class="f-pk-soft s-pk"/><text x="428" y="308" class="t-b" text-anchor="middle">+</text><rect x="476" y="292" width="88" height="22" class="f-pk-soft s-pk"/><text x="520" y="308" class="t-b" text-anchor="middle">+</text><rect x="568" y="292" width="88" height="22" class="f-pk-soft s-pk"/><text x="612" y="308" class="t-b" text-anchor="middle">+</text><rect x="660" y="292" width="88" height="22" class="f-pk-soft s-pk"/><text x="704" y="308" class="t-b" text-anchor="middle">+</text><rect x="752" y="292" width="88" height="22" class="f-pk-soft s-pk"/><text x="796" y="308" class="t-b" text-anchor="middle">+</text><text x="36" y="333">cancel_rate</text><rect x="292" y="317" width="88" height="22" class="f-pk-soft s-pk"/><text x="336" y="333" class="t-b" text-anchor="middle">+</text><rect x="384" y="317" width="88" height="22" class="f-pk-soft s-pk"/><text x="428" y="333" class="t-b" text-anchor="middle">+</text><rect x="476" y="317" width="88" height="22" class="f-pk-soft s-pk"/><text x="520" y="333" class="t-b" text-anchor="middle">+</text><rect x="568" y="317" width="88" height="22" class="f-pk-soft s-pk"/><text x="612" y="333" class="t-b" text-anchor="middle">+</text><rect x="660" y="317" width="88" height="22" class="f-pk-soft s-pk"/><text x="704" y="333" class="t-b" text-anchor="middle">+</text><rect x="752" y="317" width="88" height="22" class="f-pk-soft s-pk"/><text x="796" y="333" class="t-b" text-anchor="middle">+</text><text x="36" y="358">chuyến hoàn thành</text><rect x="292" y="342" width="88" height="22" class="f-pk-soft s-pk"/><text x="336" y="358" class="t-b" text-anchor="middle">+</text><rect x="384" y="342" width="88" height="22" class="f-pk-soft s-pk"/><text x="428" y="358" class="t-b" text-anchor="middle">+</text><rect x="476" y="342" width="88" height="22" class="f-pk-soft s-pk"/><text x="520" y="358" class="t-b" text-anchor="middle">+</text><rect x="568" y="342" width="88" height="22" class="f-pk-soft s-pk"/><text x="612" y="358" class="t-b" text-anchor="middle">+</text><rect x="660" y="342" width="88" height="22" class="f-pk-soft s-pk"/><text x="704" y="358" class="t-b" text-anchor="middle">+</text><rect x="752" y="342" width="88" height="22" class="f-pk-soft s-pk"/><text x="796" y="358" class="t-b" text-anchor="middle">+</text><text x="36" y="383">walk_m + deadhead_m mỗi chuyến</text><rect x="292" y="367" width="88" height="22" class="f-pk-soft s-pk"/><text x="336" y="383" class="t-b" text-anchor="middle">+</text><rect x="384" y="367" width="88" height="22" class="f-pk-soft s-pk"/><text x="428" y="383" class="t-b" text-anchor="middle">+</text><rect x="476" y="367" width="88" height="22" class="f-pk-soft s-pk"/><text x="520" y="383" class="t-b" text-anchor="middle">+</text><rect x="568" y="367" width="88" height="22" class="f-surface s-line"/><text x="612" y="383" class="t-b" text-anchor="middle">0</text><rect x="660" y="367" width="88" height="22" class="f-pk-soft s-pk"/><text x="704" y="383" class="t-b" text-anchor="middle">+</text><rect x="752" y="367" width="88" height="22" class="f-pk-soft s-pk"/><text x="796" y="383" class="t-b" text-anchor="middle">+</text><rect x="36" y="402" width="14" height="12" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><text x="58" y="413" class="t-small">+ tốt hơn baseline (KTC t 95% không chứa 0)</text><rect x="330" y="402" width="14" height="12" rx="2" class="f-warn-soft s-warn" stroke-width="1.2"/><text x="352" y="413" class="t-small">− xấu hơn baseline</text><rect x="480" y="402" width="14" height="12" rx="2" class="f-surface s-line" stroke-width="1.2"/><text x="502" y="413" class="t-small">0 không khác biệt rõ</text><text x="36" y="437" class="t-small">Mỗi bộ đổi một giả định và chạy lại baseline với cùng giả định. Trên dữ liệu mô phỏng, seed 20270001…20270020.</text><text x="36" y="453" class="t-small">Số lấy từ results/p04.md, mục LOC-40, «tóm tắt độ nhạy».</text></svg>
</div>
<figcaption>Hình 3. Điểm đón <code>p04-v1</code> so với baseline dưới sáu bộ giả định, 13 chỉ số. Cộng: tốt hơn rõ. Trừ: xấu hơn rõ. Số 0: không khác biệt rõ. Năm chỉ số tốt hơn ở mọi bộ; km rỗng không bền ở 60 xe; <code>walk_m</code> p50 xấu hơn ở 5/6 bộ. Trên dữ liệu mô phỏng.</figcaption>
</figure>

Các dòng quyết định, chênh cặp so baseline (20 seed, KTC t 95%), trên dữ liệu mô phỏng:

| Chỉ số | Gốc | Đi bộ chậm | Huỷ theo đi bộ | 60 xe | 240 xe | GPS nhiễu 15 m |
|---|---:|---:|---:|---:|---:|---:|
| `deadhead_m` trung bình (m) | −107,0 [−151,6; −62,4] | −133,6 [−176,3; −91,0] | −116,6 [−154,3; −78,9] | **−111,5 [−289,6; +66,7]** | −141,8 [−162,3; −121,4] | −129,6 [−173,6; −85,7] |
| `walk_m` p50 (m) | +26,5 [+23,1; +30,0] | −5,3 [−7,7; −2,9] | +24,6 [+21,0; +28,2] | +23,6 [+19,6; +27,6] | +26,5 [+22,5; +30,5] | +48,5 [+44,6; +52,5] |
| `pickup_eta_s` trung bình (s) | −21,7 [−26,6; −16,8] | −19,3 [−23,8; −14,8] | −21,9 [−26,2; −17,7] | −20,9 [−39,4; −2,3] | −26,1 [−28,7; −23,4] | −23,6 [−28,9; −18,3] |
| `cancel_rate` | −0,0141 [−0,0210; −0,0072] | −0,0126 [−0,0180; −0,0072] | −0,0153 [−0,0220; −0,0085] | −0,0182 [−0,0265; −0,0100] | −0,0126 [−0,0167; −0,0084] | −0,0144 [−0,0189; −0,0098] |

Nguồn: `results/p04.md`, LOC-40 mục 6; cột «Gốc» là 20 seed đánh giá. Ở **60 xe**, chênh −111,5 m có khoảng [−289,6; +66,7] chứa 0: chưa chứng minh được, và cũng chưa loại được một mức giảm lớn. `walk_m` + `deadhead_m` cũng không khác biệt rõ. Ở bộ này 26% lượt chọn không có tài xế rảnh nên `J` dùng phân bố quanh đó (bài 35). Giả thuyết, lab chưa kiểm: khi ít xe, ghép chuyến bị chi phối bởi việc **có xe hay không** hơn là xe nào, nên điểm đón khó đổi km rỗng. Thêm: `walk_m` p50 chỉ khác ở bộ «đi bộ chậm» (−5,3 m, vì `J` dùng tốc độ đi bộ nên tự giảm đi bộ khi khách chậm); và `walk_m` trung bình xấu hơn khi huỷ có tính quãng đi bộ (+13,7 m) và khi GPS nhiễu 15 m (+36,5 m, p50 +48,5 m). Với GPS nhiễu, giả thuyết là chiến lược thấy chỗ đứng lệch nên chọn điểm xa hơn cho khách (baseline không dùng GPS). Với bộ huỷ theo đi bộ, giả thuyết là khách đuôi của baseline hay huỷ hơn nên rời khỏi mẫu hoàn thành (`walk_m` trung bình của baseline là 48,2 m thay vì 64,7 m).

Kết luận của độ nhạy gọn trong một câu: **chỉ năm chỉ số tốt hơn baseline ở mọi bộ giả định** (ETA trung bình, `meet_gap_s` trung bình, quay đầu, huỷ, chuyến hoàn thành), còn «tài xế chạy ít hơn» đúng ở 5/6 bộ và «khách điển hình đi bộ thêm» đúng ở 5/6 bộ.

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin` (hoặc thêm `-- --fixture --skip-sensitivity` để chạy nhanh trên đồ thị tổng hợp). Từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run --release -q --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc40_cuoc_quote_tong_hop -- --jobs 8
```

Output thật (rút gọn, kịch bản `hcm-baseline`, 20 seed 20270001…20270020):

```text
-- 2. đổi điểm đón làm cước đổi bao nhiêu (J p04-v1 so với baseline, cặp theo khách, 20 seed) --
  3780 chuyến ghép cặp: cước tăng 27.4%, bằng nhau 12.3%, giảm 60.3%; chênh theo chuyến: TB -354, p5 -2164, p50 -67, p95 +778, min -24079, max +32841 đồng
  tổng cước theo quote (baseline) 127.779.528 đồng; tổng cước theo quãng thật nếu tính hết 126.439.610 đồng (-1.05%)
  KHÔNG TĂNG GIÁ LÉN: khách chưa đồng ý chỉ trả tối đa cước đã báo; phần chênh nền tảng chịu = 587.859 đồng (0.46% tổng quote); tổng khách trả = 125.851.751 đồng (-1.51% so với quote)

  chỉ số tốt hơn baseline ở MỌI bộ giả định: ["pickup_eta_s trung bình", "meet_gap_s trung bình", "u_turns mỗi chuyến", "cancel_rate", "chuyến hoàn thành"]
  chỉ số xấu hơn baseline ở ÍT NHẤT một bộ giả định: ["walk_m trung bình", "walk_m p50"]
```

Máy đang chịu tải rất nặng khi đo, nên bài không dùng số giây chạy. Số mô phỏng tái lập được: chạy lại toàn bộ lab sau khi sửa nhánh phân bố quanh đó (bài 35) cho output giống từng chữ số, trừ bộ «ít xe» đi qua nhánh đó. `bash examples/toi-noi-location/verify-p04.sh` chạy test của `pickup::fare` cùng các lab của P04 trên fixture; nó bỏ qua độ nhạy giả định và dải xác nhận.

## Giới hạn của ví dụ

- **Mọi số là của simulator.** Nhu cầu (điểm nóng Gauss), huỷ chuyến (chỉ theo ETA ở mô hình gốc), nhiễu tốc độ, giao thông tự do, tốc độ đi bộ, vị trí khách đứng và ghim đều là giả định của lab. Không có GPS người thật.
- **Cước:** chỉ có mô hình 12.000 + 4.500/km; không có giờ cao điểm, phụ phí, khuyến mãi; không có hợp đồng thật với Pricing C#; chưa chia ai chịu phần chênh.
- **Mô hình đồng ý của khách** là giả định (`0,95 − 0,004 × mét đi thêm`); câu chữ chưa được thử với người dùng, chỉ có số liệu đúng.
- **Quote trong lab là cước theo quãng baseline** (đón đúng ghim), để có thước so; hệ thống thật nên tính quote từ điểm đón đã chọn.
- **Simulator không đo hai thứ P04 muốn tối ưu:** thời gian băng đường của khách và quãng đi bộ sau khi xuống xe (đo riêng bằng nhật ký, không gồm đoạn từ đường vào tới đích).
- **Dải xác nhận** chỉ có cho điểm đón `p04-v1` (và điểm đón cộng điểm trả), không cho các lab khác; độ nhạy chạy trên 20 seed đánh giá, không chạy lại trên dải xác nhận.

## Bài tập

**Bài 1.** Tính cước cho quãng 5.800 m bằng công thức của lab và so với quote 5.000 m (25.500 đồng). Cước đổi bao nhiêu, và đó là loại đề xuất cần khách đồng ý hay áp dụng tự động?

<details><summary>Lời giải bài 1</summary>
<p>Quãng vượt 2 km là 3.800 m; 3.800 × 4.500 + 500 = 17.100.500, chia nguyên cho 1.000 là 17.100; cước 12.000 + 17.100 = 29.100 đồng, tăng 3.600 đồng so với 25.500. Cước tăng nên cần khách đồng ý (<code>NeedsConsent</code>); nếu khách không trả lời, trả 25.500 đồng và nền tảng chịu 3.600 đồng. Đây cũng là tình huống của một test đơn vị trong <code>fare.rs</code> (quote 5.000 m, đề xuất 5.800 m).</p>
</details>

**Bài 2.** Phần chênh 587.859 đồng (0,46% tổng quote) ở các chuyến tăng cước do ai chịu: nền tảng, tài xế, hay chia? Nêu hai lựa chọn và hệ quả cho tài xế.

<details><summary>Lời giải bài 2</summary>
<p>Lab không quyết định; nó chỉ ghi <code>absorbed_vnd</code>. Lựa chọn A: nền tảng chịu hết, tài xế vẫn nhận thu nhập theo quãng thật; tài xế không bị phạt vì thuật toán, nhưng nền tảng có động cơ chỉnh để ít đề xuất tăng cước. Lựa chọn B: trừ vào phần tài xế, thì thuật toán đẩy rủi ro cho đúng bên đã chịu km rỗng và doanh thu thấp hơn; cần minh bạch rõ và dễ bị coi là không công bằng. Quyết định thuộc hợp đồng với Pricing.</p>
</details>

**Bài 3.** Nhìn Hình 3, một đồng nghiệp muốn viết «điểm đón `J` giảm km rỗng ở mọi kịch bản». Chỉ ra ô nào phản bác và đề xuất câu viết đúng.

<details><summary>Lời giải bài 3</summary>
<p>Ô <code>deadhead_m</code> trung bình ở cột «60 xe» là 0 (−111,5 m [−289,6; +66,7], khoảng chứa 0). Câu đúng: «km rỗng giảm ở 5 trong 6 bộ giả định đã thử; khi chỉ có 60 xe, chênh lệch không khác biệt rõ.» Nên nêu thêm những chỉ số bền ở cả 6 bộ (ETA, <code>meet_gap_s</code>, quay đầu, huỷ, chuyến hoàn thành) và chi phí của khách (<code>walk_m</code> p50 xấu hơn ở 5/6 bộ).</p>
</details>

Bài trước: [bài 39](/articles/location-hoc-diem-don-tu-lich-su-dbscan-python). Bài tiếp: [bài 41](/articles/location-gan-nhat-sai-o-dau-do-bang-mo-phong).

## Tài liệu gốc

- [Sensitivity analysis (Wikipedia)](https://en.wikipedia.org/wiki/Sensitivity_analysis): phân tích độ nhạy, nền của phép thử «đổi một giả định mỗi lần».
- [Student's t-test (Wikipedia)](https://en.wikipedia.org/wiki/Student%27s_t-test): mục kiểm định mẫu ghép cặp, nền của khoảng tin cậy trên hiệu từng seed.
- [Bản quyền và giấy phép của OpenStreetMap](https://www.openstreetmap.org/copyright): điều kiện ghi công cho dữ liệu OSM của đồ thị HCM.
