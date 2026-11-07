---
slug: python-fraud-toi-noi-bai-toan-nhan-du-lieu-metric
title: "Gian lận trong Tới Nơi: loại hình, nhãn, dữ liệu và metric"
summary: "Bài 54: bốn loại gian lận của Tới Nơi và dấu vết của chúng, bộ dữ liệu tổng hợp dùng chung cả phần 07, nhãn trễ, chia train/valid theo thời gian, PR-AUC so với ROC-AUC và chi phí hai chiều. Có số đo thật."
category: programming
level: advanced
tags: [python, toi-noi, fraud, metric, imbalanced-data]
language: vi
---

Service fraud-scoring của [Tới Nơi](/articles/du-an-mau-toi-noi) đã có đường vào: Dispatch gọi nó qua gRPC ([bài 37](/articles/python-grpc-fraud-scoring-service)) và event đi qua Kafka ([bài 38](/articles/python-kafka-aiokafka-producer-consumer-schema)). Còn thiếu phần khó nhất, không phải mô hình: thế nào là gian lận, dữ liệu nào dạy và đo được điều đó, con số nào đáng tin. Tài xế dựng chuyến không có khách thật để lấy thưởng, giả GPS, một người mở hàng chục tài khoản ăn mã chào khách mới, vài tài xế và khách chạy qua lại với nhau. Chưa viết dòng mô hình nào, ta đã vướng ba cái bẫy: nhãn đến rất muộn, 98 % chuyến là sạch nên accuracy vô nghĩa, và «sai» có hai giá khác nhau.

Đây là bài 54 của lộ trình **Python nâng cao qua Tới Nơi**, bài mở đầu phần 07. Bạn cần quen worker nền ([bài 40](/articles/python-worker-arq-scheduling-retry-dead-letter)), event Kafka (bài 38), polars và Parquet ([bài 29](/articles/python-polars-pandas-parquet-phan-tich-chuyen)). Dữ liệu trong bài là **tổng hợp**, sinh bằng seed cố định, kể cả tỉ lệ gian lận; số đo chỉ nói về bộ này. Điểm rủi ro của phần 07 là **đầu vào cho người duyệt**, không phải quyết định khóa tài khoản.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một bộ chấm điểm gian lận đạt ROC-AUC 0,933 trên tập valid, nghe gần như hoàn hảo. Tập valid có khoảng 2 % chuyến gian lận. Nếu gắn cờ mọi chuyến có điểm từ 0,5 trở lên, nó bắt được khoảng bao nhiêu phần trăm số chuyến gian lận?</p>
<details><summary>Xem đáp án</summary><p><b>Khoảng 27 %.</b> Ở ngưỡng 0,5 bộ chấm bắt 304 trong 1.141 chuyến gian lận (recall 0,266), dù 80 % số chuyến nó gắn cờ là gian lận thật (precision 0,798); PR-AUC của chính bộ điểm đó chỉ 0,537. ROC-AUC không nói sai, nhưng không trả lời câu cần hỏi: trong số chuyến bị gắn cờ bao nhiêu là thật, và ta bỏ sót bao nhiêu.</p></details>
</div>

## Bốn loại gian lận và dấu vết mỗi loại để lại

Bộ dữ liệu có bốn loại, mỗi loại là một **kiểu hành vi**, không phải một thuộc tính của chuyến:

| `fraud_type` | Người gian lận làm gì (trong bộ tổng hợp) | Dấu vết |
|---|---|---|
| `ghost_trip` | Chuyến ảo: một người điều khiển cả tài xế lẫn vài tài khoản khách mới, chạy từng đợt 3 đến 9 chuyến, nhiều chuyến gần như không đi | Đường chim bay ngắn hơn hẳn quãng đường khai báo; chuyến dồn trên một thiết bị |
| `gps_spoof` | GPS giả: tài xế bật ứng dụng giả vị trí theo phiên 4 đến 9 chuyến, điểm bắt đầu bị dịch vài km hoặc hàng trăm km | GPS tài xế cách điểm đón; nhảy vị trí so với chuyến trước |
| `promo_abuse` | Lạm dụng khuyến mãi: một người mở 3 đến 12 tài khoản mới, có khi chung điện thoại, mỗi tài khoản dùng `NEWUSER50` một lần | Mã giảm một nửa cước; nhiều tài khoản chung một máy |
| `collusion` | Cấu kết: 2 đến 3 tài xế và 3 đến 6 tài khoản khách mới chạy qua lại 2 đến 5 tuần để nhận thưởng | Cặp khách-tài xế lặp lại; đôi khi khách đặt bằng máy của tài xế |

Dấu vết chia hai tầng. Tầng một nằm trong event của **một** chuyến: tọa độ, quãng đường, mã khuyến mãi, thiết bị. Tầng hai chỉ hiện khi nhìn **nhiều** chuyến: cùng thiết bị trong một giờ, một cặp khách-tài xế lặp lại. Tầng hai đắt hơn (cửa sổ thời gian, trạng thái, đồ thị) nên để bài 55 và 58.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 366" role="img" aria-label="Bốn loại gian lận ở cột trái, mỗi loại nối với dấu vết đọc được trên một chuyến ở cột giữa và dấu vết chỉ hiện khi nhìn nhiều chuyến ở cột phải."><defs><marker id="py54-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="26" class="t-b">Loại gian lận</text><text x="300" y="26" class="t-b">Dấu vết trên một chuyến</text><text x="630" y="26" class="t-b">Dấu vết cần nhiều chuyến</text><rect x="20" y="40" width="220" height="62" rx="10" class="f-pk-soft s-pk"/><text x="34" y="66" class="t-b">ghost_trip</text><text x="34" y="86" class="t-small">chuyến ảo, không đi thật</text><path d="M240 71 L300 71" class="f-none s-phys" stroke-width="2" marker-end="url(#py54-0-arrow)"/><rect x="300" y="40" width="270" height="62" rx="10" class="f-phys-soft s-phys"/><text x="314" y="66" class="t-small">đường chim bay ngắn hơn hẳn</text><text x="314" y="86" class="t-small">quãng đường khai báo</text><path d="M570 71 L630 71" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py54-0-arrow)"/><rect x="630" y="40" width="230" height="62" rx="10" class="f-warn-soft s-warn"/><text x="644" y="66" class="t-small">nhiều chuyến dồn dập trên</text><text x="644" y="86" class="t-small">một thiết bị, một cặp</text><rect x="20" y="116" width="220" height="62" rx="10" class="f-pk-soft s-pk"/><text x="34" y="142" class="t-b">gps_spoof</text><text x="34" y="162" class="t-small">giả vị trí tài xế</text><path d="M240 147 L300 147" class="f-none s-phys" stroke-width="2" marker-end="url(#py54-0-arrow)"/><rect x="300" y="116" width="270" height="62" rx="10" class="f-phys-soft s-phys"/><text x="314" y="142" class="t-small">GPS tài xế cách điểm đón khách</text><text x="314" y="162" class="t-small">khai báo hơn 1 km</text><path d="M570 147 L630 147" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py54-0-arrow)"/><rect x="630" y="116" width="230" height="62" rx="10" class="f-warn-soft s-warn"/><text x="644" y="142" class="t-small">nhảy vị trí so với chuyến</text><text x="644" y="162" class="t-small">trước của cùng tài xế</text><rect x="20" y="192" width="220" height="62" rx="10" class="f-pk-soft s-pk"/><text x="34" y="218" class="t-b">promo_abuse</text><text x="34" y="238" class="t-small">nhiều tài khoản ăn mã</text><path d="M240 223 L300 223" class="f-none s-phys" stroke-width="2" marker-end="url(#py54-0-arrow)"/><rect x="300" y="192" width="270" height="62" rx="10" class="f-phys-soft s-phys"/><text x="314" y="218" class="t-small">mã NEWUSER50, giảm một nửa</text><text x="314" y="238" class="t-small">cước của chuyến</text><path d="M570 223 L630 223" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py54-0-arrow)"/><rect x="630" y="192" width="230" height="62" rx="10" class="f-warn-soft s-warn"/><text x="644" y="218" class="t-small">nhiều tài khoản mới trên</text><text x="644" y="238" class="t-small">cùng một thiết bị</text><rect x="20" y="268" width="220" height="62" rx="10" class="f-pk-soft s-pk"/><text x="34" y="294" class="t-b">collusion</text><text x="34" y="314" class="t-small">tài xế và khách thông đồng</text><path d="M240 299 L300 299" class="f-none s-phys" stroke-width="2" marker-end="url(#py54-0-arrow)"/><rect x="300" y="268" width="270" height="62" rx="10" class="f-phys-soft s-phys"/><text x="314" y="294" class="t-small">khách đặt bằng chính điện</text><text x="314" y="314" class="t-small">thoại của tài xế</text><path d="M570 299 L630 299" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py54-0-arrow)"/><rect x="630" y="268" width="230" height="62" rx="10" class="f-warn-soft s-warn"/><text x="644" y="294" class="t-small">cặp khách-tài xế lặp lại</text><text x="644" y="314" class="t-small">nhiều lần trong vài ngày</text><text x="300" y="354" class="t-small">Cột giữa đọc được từ một chuyến; cột phải cần cửa sổ thời gian (bài 55).</text></svg>
</div>
<figcaption>Hình 1. Mỗi loại gian lận để lại dấu vết ở hai tầng. Tầng một rẻ nhưng chỉ bắt được một phần; tầng hai bắt thêm nhưng phải giữ trạng thái theo thời gian.</figcaption>
</figure>

Lab đo thẳng điều đó: mỗi loại một luật đơn giản dùng một cột của event. «Sạch bị bắt» là số chuyến sạch mà luật cũng gắn cờ (output dùng dấu phẩy ngăn nghìn của Python):

```text
loại        luật                             chuyến loại  bắt được sạch bị bắt precision
ghost_trip  route_ratio < 0.4                      1,400     57.4%       9,855      7.5%
gps_spoof   start_pickup_gap_km > 1                  800     93.5%         956     43.9%
promo_abuse promo_code == NEWUSER50                1,200    100.0%       5,622     17.6%
collusion   device_id == driver_device_id            600     24.2%         484     23.1%
```

Không luật nào hoàn hảo, mỗi cái hỏng một kiểu: luật GPS bắt 93,5 % `gps_spoof` nhưng gắn cờ 956 chuyến sạch (GPS nhảy vài km); luật khuyến mãi bắt 100 % `promo_abuse` mà precision 17,6 %, vì khách mới hợp lệ cũng dùng `NEWUSER50` ở chuyến đầu (5.622 chuyến); luật tỉ lệ đường chim bay bắt 57,4 % chuyến ảo và gắn nhầm 9.855 chuyến đi đường vòng; luật «cùng điện thoại» chỉ thấy 24,2 % `collusion`. Tầng hai khác hẳn. Đếm trên cả tập, 90,0 % chuyến `collusion` thuộc cặp khách-tài xế có từ 3 chuyến (chuyến sạch: 5,2 %, vì có khách quen một tài xế) và 61,3 % chuyến `promo_abuse` nằm trên thiết bị có từ 3 khách (chuyến sạch: 0,5 %). Hai số này dùng cả tương lai của bộ dữ liệu nên chỉ để mô tả; bài 55 tính lại đúng theo từng thời điểm.

## Bộ dữ liệu tổng hợp: schema và vì sao nhãn nằm riêng

`synthetic_fraud.py` là nguồn dữ liệu của bảy bài sau; mỗi lab copy nguyên file này. Hàm vào là `make_events(seed, n_trips)`:

```python
data = make_events(seed=42, n_trips=200_000)   # FraudData(events, labels)
data.events   # log sự kiện, 600.000 dòng, sắp theo occurred_at tăng dần
data.labels   # nhãn, 200.000 dòng, một dòng mỗi chuyến
```

Bộ sinh dựng 60 ngày của sáu thành phố (từ 02/03/2026, giờ Việt Nam), cao điểm sáng và chiều, 8 % khách có tài xế quen, vài phần trăm khách dùng chung thiết bị, khoảng 0,4 % chuyến sạch thuộc đợt đặt xe cho cả nhóm. Chuyến sạch cũng nhiễu: GPS sai vài chục mét, khoảng 0,55 % chuyến lệch vị trí từ nửa km đến hàng trăm km. Mỗi chuyến phát ba event, theo envelope của từ điển miền:

```text
{"event_id": "75004414-…", "event_type": "BookingCreated", "occurred_at": "2026-03-01T17:06:12.547557+00:00", "version": 1, "payload": {…}}
```

Bảng `events` có 22 cột; mỗi loại event điền một nhóm và để null phần còn lại. Không có cột nhãn nào:

| Nhóm cột | Kiểu | Có ở event |
|---|---|---|
| `event_id` (UUID v4 giả), `event_type`, `trip_id` (`TN-0000001`), `customer_id` (`CU-00001`) | String | cả ba |
| `occurred_at` | Datetime[us, UTC], tăng dần | cả ba |
| `version` | Int8 (luôn 1) | cả ba |
| `device_id` (`DEV-00001`), `service`, `city`, `promo_code` | String | `BookingCreated` |
| `pickup_lat`, `pickup_lng` | Float64 | `BookingCreated` |
| `driver_id` (`DR-00001`), `driver_device_id` | String | `TripCompleted` |
| `start_lat`, `start_lng`, `end_lat`, `end_lng`, `distance_km` | Float64 | `TripCompleted` |
| `duration_s` | Int32 | `TripCompleted` |
| `amount_vnd`, `discount_vnd` | Int64 (VND nguyên) | `PaymentCaptured` |

Mã dùng độ rộng lớn (`TN-0000001`, `CU-00001`...) để chứa 200.000 chuyến; các bài trước dùng mã ngắn như `TN-0001`. Bảng `labels` có `trip_id`, `fraud_type` (`none` hoặc một trong bốn loại), `is_fraud` (Boolean) và `label_known_at` (Datetime UTC). Nhãn tách riêng vì pipeline đặc trưng chỉ nhận `events` nên không thể vô tình thấy nhãn (lab có ca kiểm), và vì nhãn thật chỉ **biết sau chuyến**. Số trong ID đã xáo ngẫu nhiên: dùng số trong `customer_id` làm điểm gian lận cho ROC-AUC 0,502, tức là không biết gì.

## Nhãn trễ: khi ta biết điều mình đang dạy

Chuyến ảo bị phát hiện khi có khiếu nại hoặc kiểm toán, cấu kết cần điều tra, khuyến mãi bị quét hàng loạt: không loại nào có nhãn ngay lúc thu tiền. Bộ dữ liệu mô hình hóa điều đó bằng `label_known_at`; chuyến sạch chỉ được «xác nhận sạch» khi hết hạn khiếu nại 30 ngày. Độ trễ đo từ lúc thu tiền:

```text
loại         trung vị (ngày)  p90 (ngày)  <= 7 ngày  <= 14 ngày  <= 30 ngày
ghost_trip               6.1        14.6      56.8%       89.3%       99.1%
gps_spoof                8.1        19.7      42.4%       79.4%       96.5%
promo_abuse              3.0         6.3      92.7%       99.6%      100.0%
collusion               16.1        34.9       7.3%       39.5%       84.2%
none                    30.0   (chuyến sạch chỉ được xác nhận sạch khi hết hạn khiếu nại 30 ngày)
```

Các tham số độ trễ do bộ sinh chọn; điều đáng học là hệ quả. Giả sử ta huấn luyện vào ngày 40 (hình 2): nhãn của chuyến thanh toán ngày 33 đến sau khoảng 3 ngày với `promo_abuse` nhưng 16 ngày với `collusion`, nên lúc huấn luyện nhãn loại sau chưa có.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Trục 60 ngày chia thành train từ ngày 0 đến 30, khoảng chờ nhãn từ 30 đến 40 và valid từ 40 đến 60. Một chuyến thanh toán ngày 33 nhận nhãn promo_abuse sau 3,0 ngày và ghost_trip sau 6,1 ngày, trước ngày huấn luyện 40; nhãn collusion đến sau 16,1 ngày và nhãn sạch sau 30 ngày, đều sau ngày 40."><defs><marker id="py54-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py54-1-arrow-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="40" y="50" width="400" height="44" rx="6" class="f-phys-soft s-phys"/><text x="56" y="77" class="t-small">train: thanh toán ngày 0 đến 30</text><rect x="440" y="50" width="133" height="44" rx="6" class="f-warn-soft s-warn dash"/><text x="452" y="77" class="t-small">chờ nhãn</text><rect x="573" y="50" width="267" height="44" rx="6" class="f-pk-soft s-pk"/><text x="589" y="77" class="t-small">valid: ngày 40 đến 60</text><text x="36" y="114" class="t-small">0</text><text x="432" y="114" class="t-small">30</text><text x="565" y="114" class="t-small">40</text><text x="826" y="114" class="t-small">60</text><path d="M573 34 L573 292" class="f-none s-muted dash" stroke-width="1.5"/><text x="581" y="40" class="t-small">ngày huấn luyện = 40</text><circle cx="480" cy="146" r="4" class="f-phys"/><text x="40" y="150" class="t-small">chuyến thanh toán ngày 33; nhãn đến sau:</text><path d="M480 176 L520 176" class="f-none s-phys" stroke-width="2" marker-end="url(#py54-1-arrow)"/><text x="40" y="180" class="t-small">promo_abuse, trung vị 3,0 ngày</text><path d="M480 210 L561 210" class="f-none s-phys" stroke-width="2" marker-end="url(#py54-1-arrow)"/><text x="40" y="214" class="t-small">ghost_trip, trung vị 6,1 ngày</text><path d="M480 244 L695 244" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py54-1-arrow-warn)"/><text x="40" y="248" class="t-small">collusion, trung vị 16,1 ngày</text><path d="M480 278 L836 278" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py54-1-arrow-warn)"/><text x="40" y="282" class="t-small">chuyến sạch, xác nhận sau 30 ngày</text><text x="40" y="318" class="t-small">Mũi tên đi qua đường dọc là nhãn chưa tồn tại lúc huấn luyện.</text></svg>
</div>
<figcaption>Hình 2. Chia theo thời gian có khoảng chờ. Chuyến càng gần ngày huấn luyện thì càng ít nhãn gian lận được biết, và loại điều tra lâu bị thiếu nhiều nhất.</figcaption>
</figure>

Số nhãn biết được ở ngày 40 tính bằng `known_label` (chuyến gian lận chỉ được tính khi `label_known_at` đã tới):

```python
def known_label(labels: pl.DataFrame, cutoff_day: float) -> pl.DataFrame:
    """Nhãn nhìn thấy được vào ngày cutoff: chuyến gian lận mà label_known_at chưa tới thì vẫn là 0."""
    return labels.with_columns(
        known_fraud=pl.col("is_fraud") & (pl.col("label_known_at") <= pl.lit(at_day(cutoff_day)))
    )
```

```text
chuyến gian lận thanh toán trước ngày 30: 2,011; biết được vào ngày 40: 1,830 (91.0%)
chuyến gian lận thanh toán trước ngày 40: 2,859; biết được vào ngày 40: 2,172 (76.0%)
chuyến gian lận của 7 ngày ngay trước ngày 40: 564; biết được: 156 (27.7%)
train (trước ngày 30), độ chín của nhãn gian lận ở ngày 40 theo loại: ghost_trip 95.1%, gps_spoof 92.0%, promo_abuse 100.0%, collusion 61.7%
chuyến sạch chưa hết hạn khiếu nại 30 ngày vẫn mang nhãn 0 lúc đó, nên khoảng chờ 10 ngày không chữa loại điều tra lâu
```

Bảy ngày gần nhất chỉ có 27,7 % nhãn, và khoảng chờ 10 ngày không chữa đều: với train trước ngày 30, nhãn `collusion` mới chín 61,7 % ở ngày 40, `promo_abuse` đã đủ; chuyến sạch chưa hết hạn 30 ngày vẫn mang nhãn 0. Hệ quả thứ nhất là **đo sai**. Tập valid có 1.141 chuyến gian lận thật, nhưng khi dữ liệu kết thúc (ngày 60,3) mới biết 763 (66,9 %). Chuyến gian lận chưa có nhãn bị tính là sạch nên gắn cờ đúng lại thành dương tính giả. Giữ nguyên mô hình, chỉ đổi nhãn chấm:

```text
nhãn dùng để đo           PR-AUC  precision@0.5  recall@0.5
nhãn cuối cùng             0.537          0.798       0.266
nhãn biết đến lúc đó       0.374          0.525       0.262
```

PR-AUC tụt từ 0,537 xuống 0,374 và precision từ 0,798 xuống 0,525 chỉ vì nhãn chưa chín; dashboard tính trên nhãn mới nhất sẽ báo «mô hình xuống cấp» khi chưa có gì xuống cấp. Hệ quả thứ hai là **huấn luyện sai**: chuyến gian lận chưa có nhãn bị dạy như chuyến sạch. Với hồi quy logistic của lab tác động này nhỏ. Lab huấn luyện bốn lần, có hoặc không có khoảng chờ, nhãn cuối hoặc nhãn biết ở ngày 40, rồi chấm trên valid từ ngày 50 bằng nhãn cuối:

```text
PR-AUC trên valid ngày 50 trở đi (nhãn cuối), khi huấn luyện bằng:
  train < ngày 30, nhãn cuối               0.560
  train < ngày 30, nhãn biết ở ngày 40     0.569
  train < ngày 40, nhãn cuối               0.563
  train < ngày 40, nhãn biết ở ngày 40     0.556
```

Bốn kết quả cách nhau 0,013. Đừng suy ra nhãn trễ vô hại: mô hình dùng lịch sử gian lận làm đặc trưng nhạy hơn nhiều, và bài 55 đo trường hợp đó (rò rỉ nhãn). Khoảng chờ trong hình 2 loại chuyến chưa chín khỏi train, nhưng không chữa được loại điều tra lâu.

## Chia train/valid theo thời gian

Chia ngẫu nhiên trả lời «mô hình có tốt trên một chuyến bất kỳ cùng giai đoạn không», còn ta cần «mô hình hôm nay có tốt trên chuyến ngày mai không». Lab chia theo `paid_at`, thời điểm chuyến được chấm điểm:

```python
TRAIN_END_DAY = 30   # train: chuyến thanh toán trước ngày 30 (tính từ START)
VALID_START_DAY = 40 # valid: từ ngày 40; ngày 30 đến 40 là khoảng chờ cho nhãn chín
```

```text
train: thanh toán trước ngày 30 (02/03/2026 00:00 đến 01/04/2026 00:00): 100,022 chuyến, 2,011 gian lận (2.01%)
valid: từ ngày 40 (11/04/2026 00:00): 66,704 chuyến, 1,141 gian lận (1.71%); ngày 30 đến 40 bỏ trống làm khoảng chờ nhãn
```

Chia ngẫu nhiên làm số đẹp giả vì người gian lận lặp lại: một điện thoại xuất hiện trong nhiều chuyến. Thử một «bộ phát hiện» chỉ biết nhớ: thiết bị nào từng có gian lận ở train thì gắn cờ mọi chuyến của nó ở test.

```python
def blacklist_eval(train: pl.DataFrame, test: pl.DataFrame) -> tuple[int, float, float]:
    """Danh sách đen thiết bị: thiết bị từng có gian lận ở train thì gắn cờ mọi chuyến của nó ở test."""
    bad = set(train.filter(pl.col("is_fraud"))["device_id"])
    flag = test["device_id"].is_in(list(bad)).to_numpy()
    y = test["is_fraud"].to_numpy()
    tp, fp = int((flag & y).sum()), int((flag & ~y).sum())
    return len(bad), tp / max(1, int(y.sum())), tp / max(1, tp + fp)
```

```text
cùng cỡ: train 133,296 chuyến, test 66,704 chuyến
cách chia                      thiết bị đen   recall  precision  PR-AUC (LR)
ngẫu nhiên                            1,262    64.4%      51.7%        0.488
theo thời gian                        1,289    39.4%      35.7%        0.543
theo thời gian, train < 30              961    24.2%      31.5%             
... nhãn chỉ biết ở ngày 40             909    21.8%      31.0%             
gian lận ở valid: 60.6% nằm trên thiết bị không có trong danh sách đen; 47.2% nằm trên thiết bị chưa từng xuất hiện trước ngày 40
```

Hai cách chia dùng cùng cỡ train (133.296 chuyến trước ngày 40) và cùng test: danh sách đen 1.262 và 1.289 thiết bị, recall 64,4 % khi chia ngẫu nhiên, 39,4 % khi chia theo thời gian. Hai dòng sau là biến thể của chia theo thời gian: thêm khoảng chờ (train trước ngày 30) thì recall 24,2 %, nhãn chỉ biết đến ngày 40 thì 21,8 %. Ở valid, 60,6 % chuyến gian lận nằm trên thiết bị không có trong danh sách đen, và 47,2 % nằm trên thiết bị chưa từng xuất hiện trước ngày 40.

Cột cuối đi ngược chiều: PR-AUC của hồi quy logistic 0,488 khi chia ngẫu nhiên, 0,543 khi chia theo thời gian. Đây là tạo tác của bộ sinh: quần thể khách gần như cố định, nên tỉ lệ chuyến sạch dùng `NEWUSER50` giảm dần theo thời gian và valid về sau dễ hơn:

```text
tỉ lệ chuyến sạch dùng NEWUSER50 và PR-AUC (LR, train < 30 chấm trên từng giai đoạn 10 ngày):
  ngày  0 đến 10: sạch dùng NEWUSER50   7.8%, PR-AUC 0.428
  ngày 10 đến 20: sạch dùng NEWUSER50   3.6%, PR-AUC 0.485
  ngày 20 đến 30: sạch dùng NEWUSER50   2.3%, PR-AUC 0.500
  ngày 30 đến 40: sạch dùng NEWUSER50   1.4%, PR-AUC 0.562
  ngày 40 đến 50: sạch dùng NEWUSER50   1.2%, PR-AUC 0.523
  ngày 50 đến 60: sạch dùng NEWUSER50   0.9%, PR-AUC 0.560
bỏ hai cột has_newuser_promo, discount_ratio: PR-AUC (LR) ngẫu nhiên 0.418, theo thời gian 0.408
```

Bỏ hai cột `has_newuser_promo` và `discount_ratio`, hai cách chia cho 0,418 và 0,408, chiều đảo biến mất. Hồi quy logistic không nhớ định danh nên không được lợi từ chia ngẫu nhiên, và 0,488 so với 0,543 không chứng minh chia ngẫu nhiên bi quan. Mọi valid của phần 07 hưởng cùng dịch phân phối này.

## Metric trên dữ liệu mất cân bằng

Với khoảng 1,7 % chuyến gian lận ở valid, bộ chấm luôn trả «sạch» đạt accuracy 0,9829 và recall 0,00, nên accuracy không dùng được. Hai con số thay thế, cho một ngưỡng gắn cờ:

- **precision**: trong các chuyến bị gắn cờ, bao nhiêu phần gian lận thật. Thấp nghĩa là người duyệt xem nhiều chuyến sạch.
- **recall**: trong các chuyến gian lận, bao nhiêu phần bị gắn cờ. Thấp nghĩa là gian lận lọt qua.

Đổi ngưỡng thì hai số đổi ngược chiều. **PR-AUC** (ở đây là average precision) gom cả đường đó thành một số: trung bình precision tại mọi ngưỡng, trọng số theo mức tăng recall. Lab tự viết nó, rồi đối chiếu scikit-learn:

```python
def average_precision(y: np.ndarray, score: np.ndarray) -> float:
    """PR-AUC dạng average precision: sum_k (R_k - R_{k-1}) * P_k, k chạy qua các ngưỡng khác nhau."""
    y = np.asarray(y, dtype=bool)
    order = np.argsort(-score, kind="stable")
    ys, ss = y[order], score[order]
    tp = np.cumsum(ys)
    seen = np.arange(1, len(ys) + 1)
    last = np.r_[ss[1:] != ss[:-1], True]  # mỗi nhóm điểm bằng nhau chỉ lấy dòng cuối
    precision = tp[last] / seen[last]
    recall = tp[last] / max(1, int(y.sum()))
    return float(np.sum(np.diff(np.r_[0.0, recall]) * precision))
```

**ROC-AUC** là xác suất một chuyến gian lận được chấm cao hơn một chuyến sạch. Trên tập valid (hồi quy logistic trên đặc trưng theo từng chuyến, huấn luyện ở train):

```text
luôn đoán «sạch»:  accuracy 0.9829, recall 0.00 (tỉ lệ gian lận 0.0171)
điểm ngẫu nhiên:  PR-AUC 0.0171, ROC-AUC 0.5041
hồi quy logistic:  PR-AUC 0.5374, ROC-AUC 0.9332
đối chiếu scikit-learn: average_precision 0.5374, roc_auc 0.9332
ngưỡng 0.5: TP 304, FP 77, FN 837, TN 65,486; precision 0.798, recall 0.266, F1 0.399, accuracy 0.9863
```

Điểm ngẫu nhiên cho PR-AUC xấp xỉ tỉ lệ gian lận (0,0171), còn ROC-AUC 0,504: PR-AUC có **mốc cơ sở bằng tỉ lệ dương**, nên 0,537 là gấp khoảng 31 lần mốc đó, trong khi ROC-AUC 0,933 trông «gần tối đa». ROC-AUC lạc quan vì chia số dương tính giả cho tổng số chuyến sạch: 77 chuyến sạch bị gắn cờ nhầm chỉ là 0,12 % mẫu số. Thử bỏ 90 % chuyến sạch khỏi tập valid, giữ nguyên mọi chuyến gian lận và điểm số:

```text
cùng điểm, chỉ giữ 10% chuyến sạch (7,717 dòng, tỉ lệ gian lận 14.8%): PR-AUC 0.8243, ROC-AUC 0.9322
```

ROC-AUC gần như không đổi (0,9322), PR-AUC nhảy từ 0,537 lên 0,824 mà mô hình không khác. PR-AUC phụ thuộc tỉ lệ dương, nên so hai mô hình phải so trên cùng một tập; với người vận hành, precision và recall ở ngưỡng thật nói nhiều hơn một con số. Tách theo loại thì thấy chỗ hổng mà con số tổng che mất:

```text
loại          chuyến  recall@0.5  recall@0.1   PR-AUC
ghost_trip       439       45.8%       60.4%    0.551
gps_spoof        204       42.2%       67.6%    0.446
promo_abuse      343        2.9%       44.6%    0.178
collusion        155        4.5%       24.5%    0.026
```

Bộ chấm chỉ có đặc trưng theo từng chuyến nên gần như mù với `collusion` (PR-AUC 0,026) và kém với `promo_abuse` (0,178). Đó là việc của bài 55 và bài 58.

## Chi phí hai chiều, ngưỡng và người duyệt

Bỏ sót một chuyến gian lận và gắn cờ nhầm một chuyến sạch không đắt như nhau. Lab tính chi phí = số chuyến bị gắn cờ × 8.000 VND (phí duyệt, tính cho mọi chuyến gắn cờ, kể cả gian lận thật) + số chuyến gian lận bị bỏ sót × (tiền chuyến + 20.000 VND xử lý). Đây là giả định minh họa, với hai giả định ngầm: gắn cờ là chặn hết thiệt hại (người duyệt không sai), và `amount_vnd` đại diện cho thiệt hại (với `promo_abuse` thiệt hại thật là phần giảm giá). Lab chọn ngưỡng trên nửa đầu của valid (ngày 40 đến 50) rồi báo cáo trên nửa sau:

```text
chọn ngưỡng trên valid nửa đầu (40 đến 50): 0.065; báo cáo trên nửa sau: 33,416 chuyến, 494 gian lận
chính sách                  gắn cờ    TP     FP    FN   chi phí (VND)
không gắn cờ gì                  0     0      0   494      24,495,068
gắn cờ tất cả               33,416   494  32922     0     267,328,000
ngưỡng 0.5                     189   155     34   339      17,168,653
ngưỡng chọn theo chi phí 0.065     885   335    550   159      15,094,547
mốc lý thuyết nếu điểm là xác suất đã hiệu chỉnh: phí duyệt / chi phí bỏ sót trung bình = 8,000 / 49,585 = 0.161; điểm của hồi quy logistic ở đây chưa hiệu chỉnh (điểm trung bình 0.0158, tỉ lệ gian lận 0.0171)
(đối chiếu, ngưỡng tốt nhất nếu chọn ngay trên nửa sau: 0.071, chi phí 14,767,209; không dùng được ngoài đời)
người duyệt chỉ xem được 1% chuyến (334 chuyến điểm cao nhất): bắt 205 trong 494 gian lận, precision 61.4%
```

Ngưỡng chọn theo chi phí, 0,065, gắn cờ 885 chuyến thay vì 189, bắt 335 chuyến gian lận thay vì 155, với 550 chuyến sạch bị xem nhầm thay vì 34. Chi phí 15,09 triệu VND, thấp hơn 17,17 triệu ở ngưỡng 0,5 và 24,50 triệu nếu không làm gì; «gắn cờ tất cả» tốn 267,33 triệu. Ngưỡng 0,065 xa mốc lý thuyết 0,161 (8.000 chia 49.585, chi phí bỏ sót trung bình) vì điểm của hồi quy logistic chưa hiệu chỉnh. Dòng cuối là hướng thực tế hơn: người duyệt có công suất cố định, nên câu hỏi là «với 1 % chuyến mỗi ngày, 334 chuyến đầu bắt được bao nhiêu», ở đây 205 trên 494 gian lận, precision 61,4 %. Điểm xếp hàng cho người duyệt, người quyết định. Hàng đợi, tỉ lệ sai cho phép và thời hạn duyệt là việc của [bài 61](/articles/python-shap-explainability-audit-human-review).

## Chạy và kiểm chứng

Cần uv và Python 3.14, không cần Docker; lần đầu cần mạng để tải numpy, polars, scikit-learn. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/54-fraud-bai-toan-nhan-du-lieu examples/toi-noi-python/54-fraud-bai-toan-nhan-du-lieu/main.py
uv run --project examples/toi-noi-python/54-fraud-bai-toan-nhan-du-lieu examples/toi-noi-python/54-fraud-bai-toan-nhan-du-lieu/main.py --verify
```

Đầu ra của bước 1 (trích, bỏ dòng danh sách cột; load average đọc lúc chạy, máy đang bận):

```text
make_events(seed=42, n_trips=200,000) mất 18.5 giây; load average 1/5/15 phút: 387.57 285.09 190.87
events: 600,000 dòng x 22 cột; labels: 200,000 dòng x 4 cột
occurred_at: 2026-03-01 17:06 đến 2026-04-30 23:44 UTC; tăng dần: True
số event theo loại: {'BookingCreated': 200000, 'PaymentCaptured': 200000, 'TripCompleted': 200000}
khách 39,299; tài xế 1,176; thiết bị khách 38,266
fraud_type     chuyến    tỉ lệ
none          196,000   98.00%
ghost_trip      1,400    0.70%
gps_spoof         800    0.40%
promo_abuse     1,200    0.60%
collusion         600    0.30%
tỉ lệ gian lận chung: 2.00%
envelope đầu tiên (topic booking.created.v1):
{"event_id": "75004414-96e1-4b5a-9296-86e793dae771", "event_type": "BookingCreated", "occurred_at": "2026-03-01T17:06:12.547557+00:00", "version": 1, "payload": {"trip_id": "TN-0000001", "customer_id": "CU-30159", "device_id": "DEV-17871", "service": "ride", "city": "TP.HCM", "pickup_lat": 10.738870307881399, "pickup_lng": 106.77413235402456}}
ROC-AUC khi dùng số thứ tự trong customer_id làm điểm gian lận: 0.502 (0.5 là không biết gì)
```

`--verify` kiểm 43 bất biến và in `OK 43/43`: schema khớp, `events` không có cột nhãn, thời gian tăng dần, tỉ lệ gian lận 1,8 đến 2,2 %, hai hàm metric khớp scikit-learn, ROC-AUC cao hơn hẳn PR-AUC, nhãn chưa chín làm PR-AUC thấp hơn, chia ngẫu nhiên làm danh sách đen đẹp hơn. Nó không kiểm số tuyệt đối hay tốc độ.

| Hiện tượng | Kiểm tra trước |
|---|---|
| PR-AUC trên valid cao khác thường so với train | Kiểm chia theo thời gian, có chuyến cùng thiết bị hay khách ở cả hai bên không |
| PR-AUC tụt dần khi theo dõi hàng tuần dù mô hình không đổi | Nhãn của tuần gần nhất chưa chín; so trên nhãn đã chín, hoặc đo độ chín trước |
| ROC-AUC 0,97 nhưng người duyệt kêu quá nhiều chuyến sạch | Xem precision ở ngưỡng đang dùng, đừng chỉ nhìn ROC-AUC |
| Hai mô hình có PR-AUC khác nhau trên hai tập khác nhau | PR-AUC phụ thuộc tỉ lệ dương; so trên cùng một tập |
| Ngưỡng chọn trên valid, báo cáo trên chính valid | Chọn ngưỡng trên một nửa, báo cáo trên nửa còn lại |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Mọi số đo trên dữ liệu tổng hợp: tỉ lệ 2 %, bốn loại gian lận, độ trễ nhãn, chi phí 8.000 và 20.000 VND đều là giả định của bộ sinh; bộ này nhỏ hơn nhiều so với 2 triệu chuyến mỗi ngày của Tới Nơi. Gian lận thật thích nghi với cách ta bắt, và nhãn thật nhiễu. Hai tạo tác của bộ sinh ảnh hưởng mọi con số: tỉ lệ chuyến sạch dùng <code>NEWUSER50</code> giảm từ 7,8 % xuống 0,9 % theo thời gian (valid dễ hơn train), và tài khoản gian lận không dùng mã nào ngoài <code>NEWUSER50</code> của <code>promo_abuse</code>. Các kết luận thứ bậc đo ở một seed (42); chạy lại <code>--verify</code> ở seed 1, 2, 7 vẫn đạt. Một lần chia theo thời gian là quá ít cho production. Bộ sinh số của numpy không cam kết giữ dòng số giữa các phiên bản (đã chạy với Python 3.14.7, numpy 2.5.4, polars 2.1.0, scikit-learn 1.9.1).</p>
</div>

**Bài tập:** (1) Một bộ phát hiện bắt 50 % chuyến gian lận và gắn cờ nhầm 1 % chuyến sạch. Trên 100.000 chuyến có 2 % gian lận, precision là bao nhiêu? Nếu chỉ 0,2 % gian lận, với cùng hai tỉ lệ đó thì sao? (2) Một chuyến `collusion` thanh toán ngày 33 có `label_known_at` vào ngày 49. Huấn luyện vào ngày 40, nhãn của nó là gì, và đưa nó vào train thì mô hình học điều gì?

<details><summary>Xem lời giải</summary><p>(1) Với 2 %: 2.000 gian lận, 98.000 sạch; TP = 1.000, FP = 980; precision = 1.000 / 1.980 = <b>50,5 %</b>. Với 0,2 %: 200 gian lận, 99.800 sạch; TP = 100, FP = 998; precision = 100 / 1.098 = <b>9,1 %</b>. Cùng một điểm trên đường ROC mà precision tụt từ 50 % xuống 9 %. (2) Nhãn là <b>0 (sạch)</b>, vì ngày 49 chưa tới ngày 40; đưa vào train như chuyến sạch là dạy mô hình rằng dấu vết của <code>collusion</code> bình thường. Cách phòng: khoảng chờ, loại chuyến chưa chín khỏi train, hoặc đánh dấu «chưa biết» thay vì «sạch».</p></details>

Bài trước: [bài 53](/articles/python-face-matching-danh-gia-bias-privacy-consent). Bài tiếp: [bài 55](/articles/python-feature-engineering-tu-event-kafka).

## Tài liệu gốc

- [scikit-learn: Metrics and scoring](https://scikit-learn.org/stable/modules/model_evaluation.html).
- [scikit-learn: average_precision_score](https://scikit-learn.org/stable/modules/generated/sklearn.metrics.average_precision_score.html).
- [scikit-learn: Precision-Recall](https://scikit-learn.org/stable/auto_examples/model_selection/plot_precision_recall.html).
- [scikit-learn: TimeSeriesSplit](https://scikit-learn.org/stable/modules/generated/sklearn.model_selection.TimeSeriesSplit.html).
- [scikit-learn: precision_recall_curve](https://scikit-learn.org/stable/modules/generated/sklearn.metrics.precision_recall_curve.html).
