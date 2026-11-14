---
slug: python-shap-explainability-audit-human-review
title: "SHAP, audit và quy trình người duyệt"
summary: "Bài 61: giải thích từng điểm gian lận bằng SHAP thành reasons[], đưa chuyến bị gắn cờ vào hàng đợi PostgreSQL, ghi nhật ký quyết định khó sửa, chỉ RV-001 được chốt, và đo công suất, SLA, dải điểm đáng duyệt."
category: python-advanced
level: advanced
tags: [python, toi-noi, fraud, shap, audit]
language: vi
---

Người duyệt của [Tới Nơi](/articles/du-an-mau-toi-noi) nhận một chuyến bị gắn cờ với p = 0,46 và phải quyết định giữ tiền hay thả. Con số không nói vì sao. Kiểm toán hỏi ngược lại, vài tháng sau: ai chốt chuyến này, theo mô hình và bộ rule phiên bản nào, đã thấy lý do gì, và có ai sửa quyết định về sau không. Bài 56 hứa nhật ký ghi phiên bản cấu hình rule; bài này làm phần đó cùng phần còn lại: giải thích **từng** dự đoán, hàng đợi người duyệt, và một nhật ký mà service chấm điểm không viết lại được.

Đây là bài 61 của lộ trình **Python nâng cao qua Tới Nơi**. Bạn cần quen mô hình đã hiệu chỉnh ở [bài 57](/articles/python-scikit-learn-lightgbm-imbalanced-cross-validation), rule engine ở [bài 56](/articles/python-rule-engine-scoring-strategy-explainability), phiên bản mô hình ở [bài 60](/articles/python-model-registry-ab-drift-retraining) và PostgreSQL ở [bài 24](/articles/python-sqlalchemy-psycopg-postgresql-mapping-migration). Chỉ `RV-001` mới chốt `confirmed_fraud` (xác nhận gian lận) hoặc `cleared` (bác đề xuất); ta không dùng `approved` vì ở bài 70 nó là hồ sơ KYC đạt. `RV-001` trong lab là **mô phỏng** (chốt theo nhãn cuối, luôn đúng), nên lab kiểm đường ống và ràng buộc, không đo độ chính xác của người thật. Hàng đợi dùng quy tắc chung của bài 59 đến 61: gắn cờ khi `p × (tiền chuyến + 20.000) > 8.000`. `shap` 0.54.1 chạy được trên Python 3.14.7 (kiểm ngày 07/11/2026).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>SHAP cho đóng góp của 30 đặc trưng cho một chuyến bị gắn cờ. Cộng chúng với hằng số nền, bạn được con số nào: xác suất p gửi cho người duyệt, log-odds của p, hay con số khác?</p>
<details><summary>Xem đáp án</summary><p><b>Một con số khác: điểm thô (margin) chưa hiệu chỉnh của LightGBM.</b> Ở chuyến ví dụ, điểm thô là −1,317 còn p gửi cho người duyệt là 0,4577, có log-odds −0,169. Với bộ hiệu chỉnh sigmoid của bài 57, log-odds của p bằng 1,073 + 0,943 × điểm thô (đúng tới 1e-12): nhân một đóng góp SHAP với 0,943 được đóng góp ở thang <b>log-odds của p</b> (không phải xác suất), và nền ở thang đó là 1,073 + 0,943 × base_value. Isotonic sẽ khác.</p></details>
</div>

## SHAP giải thích điều gì

`shap.TreeExplainer` tính giá trị Shapley của từng đặc trưng cho từng dự đoán của mô hình cây (TreeSHAP). Với mô hình nhị phân, đầu ra ở thang log-odds của **điểm thô**: tổng các đóng góp cộng hằng số nền (`base_value`) bằng điểm thô. Mô hình bài 57 là `CalibratedClassifierCV` bọc quanh `Pipeline` có LightGBM, nên ta lấy `booster` và bước chọn cột từ `ScoringModel` và bọc lại thành một lớp nhỏ (`explain.py`, trích):

```python
class Explainer:
    def __init__(self, scoring: ScoringModel) -> None:
        self.scoring = scoring
        self.calibrator = scoring.model.calibrated_classifiers_[0].calibrators[0]
        self.tree = shap.TreeExplainer(scoring.booster)
        self.base_value = float(np.ravel(self.tree.expected_value)[-1])

    def contributions(self, features):          # (n, 30), log-odds của điểm thô
        return np.asarray(self.tree.shap_values(self._matrix(features)))

    def calibrated(self, features):             # điểm thô -> p, phải bằng scoring.predict_proba
        return self.calibrator.predict(self.scoring.raw_margin(features))
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Sơ đồ: 30 đặc trưng vào LightGBM gốc cho điểm thô, qua hiệu chỉnh sigmoid thành p gửi người duyệt. SHAP đứng ở điểm thô, cho đóng góp từng đặc trưng cộng với hằng số nền bằng điểm thô, từ đó ra reasons. Hàng đợi và nhật ký ghi p, lý do, phiên bản mô hình và phiên bản rule."><defs><marker id="py61-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="40" width="150" height="62" rx="8" class="f-bg s-line"/><text x="34" y="66" class="t-b">30 đặc trưng</text><text x="34" y="86" class="t-small">của một chuyến</text><path d="M170 71 L206 71" class="f-none s-phys" stroke-width="2" marker-end="url(#py61-0-arrow)"/><rect x="210" y="40" width="190" height="62" rx="8" class="f-phys-soft s-phys"/><text x="224" y="66" class="t-b">LightGBM gốc</text><text x="224" y="86" class="t-small">điểm thô (margin)</text><path d="M400 71 L436 71" class="f-none s-phys" stroke-width="2" marker-end="url(#py61-0-arrow)"/><rect x="440" y="40" width="190" height="62" rx="8" class="f-warn-soft s-warn"/><text x="454" y="66" class="t-b">hiệu chỉnh sigmoid</text><text x="454" y="86" class="t-small">chạy trên điểm thô</text><path d="M630 71 L666 71" class="f-none s-phys" stroke-width="2" marker-end="url(#py61-0-arrow)"/><rect x="670" y="40" width="190" height="62" rx="8" class="f-pk-soft s-pk"/><text x="684" y="66" class="t-b">p gửi người duyệt</text><text x="684" y="86" class="t-small">xác suất đã hiệu chỉnh</text><text x="440" y="130" class="t-small">log-odds(p) = 1,073 + 0,943 x điểm thô</text><path d="M305 102 L305 176" class="f-none s-phys" stroke-width="2" marker-end="url(#py61-0-arrow)"/><rect x="210" y="180" width="190" height="84" rx="8" class="f-pk-soft s-pk"/><text x="224" y="206" class="t-b">SHAP</text><text x="224" y="228" class="t-small">đóng góp từng đặc trưng</text><text x="224" y="246" class="t-small">tổng + nền = điểm thô</text><path d="M400 222 L436 222" class="f-none s-phys" stroke-width="2" marker-end="url(#py61-0-arrow)"/><rect x="440" y="180" width="190" height="84" rx="8" class="f-pk-soft s-pk"/><text x="454" y="206" class="t-b">reasons[]</text><text x="454" y="228" class="t-small">top-3 đóng góp dương,</text><text x="454" y="246" class="t-small">kèm câu và số thật</text><path d="M630 222 L666 222" class="f-none s-phys" stroke-width="2" marker-end="url(#py61-0-arrow)"/><rect x="670" y="180" width="190" height="84" rx="8" class="f-warn-soft s-warn"/><text x="684" y="206" class="t-b">hàng đợi và log</text><text x="684" y="228" class="t-small">p, lý do, phiên bản</text><text x="684" y="246" class="t-small">mô hình và rule</text><path d="M765 102 L765 176" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py61-0-arrow)"/></svg>
</div>
<figcaption>Hình 1. SHAP giải thích điểm thô trước hiệu chỉnh, không giải thích trực tiếp p. Người duyệt thấy p và lý do; log giữ cả hai cùng phiên bản mô hình và phiên bản rule.</figcaption>
</figure>

Lab chạy trên ngày 50 đến 60 (33.416 chuyến, 494 gian lận). Quy tắc chi phí kỳ vọng cho 547 chuyến vào hàng đợi (ngưỡng p chung 0,149 sẽ cho 604, chỉ để so sánh). Các phép kiểm đầu tiên:

```text
33,416 chuyến x 30 đặc trưng trong 5.95 giây; base_value = -7.386 (log-odds)
base_value + tổng đóng góp so với điểm thô của LightGBM: lệch tối đa 6.6e-13
gói shap so với TreeSHAP gốc của LightGBM (ScoringModel.contributions, bài 57): lệch đóng góp tối đa 0.0e+00
điểm thô qua bộ hiệu chỉnh sigmoid so với predict_proba của mô hình lưu: lệch tối đa 0.0e+00
log-odds của p so với 1.073 + 0.943 x điểm thô: lệch tối đa 3.3e-13
một chuyến bị gắn cờ: điểm thô -1.317, p gửi cho người duyệt 0.4577 (log-odds -0.169); SHAP cộng ra điểm thô -1.317, không phải -0.169
```

Hai điều không hiển nhiên. Một: «lệch 0» giữa `shap` và `ScoringModel.contributions` không ngẫu nhiên, vì với LightGBM `shap` gọi chính `pred_contrib` của thư viện này; phép kiểm chỉ bảo đảm lớp bọc không làm sai cột, và bạn không cần cài `shap` chỉ để lấy lý do từ mô hình cây này. Hai: bộ hiệu chỉnh nhận điểm **thô** (`CalibratedClassifierCV` ưu tiên `decision_function`), nên log-odds của p là hàm affine của điểm thô; isotonic thì không. Ta chỉ giải thích các chuyến vào hàng đợi (1,6 % số chuyến):

```text
thời gian: từng chuyến 19.4 ms (trung bình 200 lần gọi); cả lô 547 chuyến bị gắn cờ 61 ms (load average 1/5/15 phút: 261.87 158.24 171.46)
6 đặc trưng đầu theo trung bình |SHAP| (cả cửa sổ): cust_prior_trips 0.81, cust_promo_share_30d 0.63, hour_local 0.28, route_ratio 0.27, duration_s 0.27, speed_kmh 0.20
6 đặc trưng đầu theo gain khi huấn luyện:             dev_customers_7d 16.9%, drv_jump_speed_kmh 14.6%, route_ratio 10.6%, start_pickup_gap_km 7.1%, duration_s 5.8%, dev_newuser_7d 5.0%
tương quan hạng giữa hai cách xếp (30 đặc trưng): 0.70
```

Hai bảng xếp hạng chỉ đồng ý ở mức khá (tương quan hạng 0,70); hai đặc trưng đầu bảng SHAP đều dựa vào lịch sử khách, và mục sau cho thấy một trong hai mang một tạo tác của dữ liệu.

## reasons[]: từ đóng góp đến câu cho người duyệt

Người duyệt không đọc 30 con số. Ta lấy tối đa ba đặc trưng đẩy điểm **lên** nhiều nhất (đóng góp dương, lớn hơn 0,05) và điền giá trị thật vào câu mẫu, chỉ từ số của đặc trưng, không bao giờ từ số giấy tờ. Có 30 câu mẫu, và test của lab kiểm mỗi đặc trưng có đúng một câu và một họ tín hiệu. Bốn chuyến bị gắn cờ:

```text
ví dụ ghost_trip, p = 0.857:
    +2.56  cặp khách-tài xế đã đi 1 chuyến trong 7 ngày trước
    +2.27  đường chim bay chỉ bằng 0.11 lần quãng đường khai báo
    +1.83  đặt lúc 1 giờ địa phương
ví dụ gps_spoof, p = 0.980:
    +4.52  GPS tài xế lúc bắt đầu cách điểm đón khai báo 104.0 km
    +3.24  nhảy vị trí tương đương 272 km/h giữa hai chuyến của tài xế
    +0.84  khách đã có 0 chuyến trước đó
ví dụ collusion, p = 0.457:
    +3.26  2 tài khoản khách khác nhau trên thiết bị trong 7 ngày (kể cả chuyến này)
    +2.49  cặp khách-tài xế đã đi 1 chuyến trong 7 ngày trước
    +0.72  đặt lúc 21 giờ địa phương
ví dụ chuyến sạch bị gắn cờ nhầm, p = 0.102:
    +2.35  đặt lúc 1 giờ địa phương
    +1.24  tốc độ trung bình trong chuyến 22 km/h
    +0.73  quãng đường khai báo 28.0 km
```

Chuyến `gps_spoof` có lý do rõ và đúng cơ chế; chuyến sạch bị gắn cờ không có dấu vết cụ thể nào, cũng là thông tin cho người duyệt. Lý do **mô tả điều mô hình dựa vào**, không phải điều gây ra gian lận.

Lý do có chỉ đúng cơ chế không? Ta so với bản đồ dấu vết của từng loại gian lận ở hình 1 bài 54, dịch thành tập đặc trưng mong đợi (bản «rộng» thêm vài đặc trưng phụ như `speed_kmh`, `amount_vnd`; bản «hẹp» chỉ gồm đặc trưng nêu thẳng trong hình). Với 407 chuyến gian lận bị gắn cờ, top-1 hoặc top-3 theo SHAP có nằm trong tập đó không, so với bộ top-1 và top-3 xếp theo gain (cố định cho mọi chuyến) và bốc ngẫu nhiên:

```text
lý do có chỉ đúng cơ chế? Chuyến gian lận bị gắn cờ: 407. Đặc trưng SHAP đứng đầu (top-1) hoặc trong ba đứng đầu (top-3) có nằm trong dấu vết mong đợi của loại đó (hình 1 bài 54)?
loại          chuyến  top-1 SHAP  top-1 gain  top-3 SHAP  top-3 gain  top-3 ngẫu nhiên
ghost_trip       218       75.2%          0 %       97.2%        100 %             67.2%
gps_spoof         79      100.0%          0 %      100.0%        100 %             43.3%
promo_abuse       89       98.9%        100 %      100.0%        100 %             56.4%
collusion         21       33.3%          0 %       90.5%          0 %             19.3%
«gain» là bộ đặc trưng cố định cho mọi chuyến: top-1 = dev_customers_7d; top-3 = dev_customers_7d, drv_jump_speed_kmh, route_ratio (xếp theo gain khi huấn luyện)
gộp bốn loại: top-1 SHAP 83.0% so với top-1 gain 21.9%, chênh +61.2% (khoảng tin cậy 95 % [+56.5%, +65.8%]); top-3 SHAP 98.0% so với top-3 gain 94.8%, chênh +3.2% ([+0.7%, +5.4%]) (bootstrap 1.000 lần)
dấu vết «mong đợi» là bản đồ ta viết từ hình 1 bài 54, rộng hơn hình ở vài đặc trưng phụ; kết quả với bản đồ hẹp in ngay dưới.
bản đồ HẸP (chỉ đặc trưng nêu thẳng trong hình 1 bài 54, bỏ speed_kmh, duration_s, amount_vnd...): top-1 SHAP 81.6%, top-3 SHAP 95.6%, top-3 ngẫu nhiên 37.5%
```

Ở top-1, SHAP trúng 83,0 % so với 21,9 % của bộ gain cố định, chênh lớn với khoảng tin cậy rất xa số 0. Ở top-3, bộ gain cố định trúng 94,8 % vì ba đặc trưng đầu theo gain tình cờ trùng dấu vết của ba trong bốn loại, nhưng 0 % ở `collusion` (21 chuyến); SHAP hơn nó 3,2 điểm phần trăm, khoảng tin cậy chỉ vừa qua số 0. Bản đồ hẹp cho top-3 95,6 % so với 37,5 % ngẫu nhiên. Giá trị của SHAP nằm ở **tính theo từng chuyến**, không ở một số trúng cao hơn hẳn. Xoá lý do có làm điểm tụt không? Ta đặt đặc trưng về trung vị dữ liệu huấn luyện rồi chấm lại, so với xoá đặc trưng ngẫu nhiên:

```text
xóa lý do (đặt về trung vị train), trên 407 chuyến gian lận bị gắn cờ; p trung bình trước khi xóa 0.875, còn bị gắn cờ 100%:
xóa                         p trung bình  còn gắn cờ
đặc trưng lý do số 1               0.461         57%
1 đặc trưng ngẫu nhiên             0.847         97%  (trung bình 10 lần bốc)
ba đặc trưng lý do đầu             0.058         10%
3 đặc trưng ngẫu nhiên             0.780         92%  (trung bình 10 lần bốc)
```

Xoá ba lý do đầu đưa p trung bình từ 0,875 xuống 0,058, trong khi xoá ba đặc trưng ngẫu nhiên chỉ xuống 0,780: lý do trỏ đúng chỗ mô hình dựa vào. Xoá riêng lý do số 1 chưa đủ để chuyến thoát cờ (57 % vẫn bị gắn); lab không đo vì sao. Họ tín hiệu có đóng góp lớn nhất của từng loại:

```text
họ tín hiệu có đóng góp lớn nhất, theo loại gian lận (số chuyến):
loại          hành trình  GPS tài xế  khuyến mãi    thiết bị    nhịp đặt
ghost_trip            94           0           0          61          63
gps_spoof              0          79           0           0           0
promo_abuse            0           0          86           2           1
collusion              0           0           0          18           3
```

Cuối cùng là phát hiện về chính mô hình: SHAP cho thấy **hướng và mức** ảnh hưởng của một đặc trưng theo từng chuyến, thứ mà gain không cho thấy. Bộ sinh có một tạo tác: khách gian lận không dùng mã khuyến mãi nào ngoài NEWUSER50, nên chuyến sạch hay có lịch sử khuyến mãi còn chuyến gian lận thì hầu như không:

```text
cust_promo_share_30d trong cửa sổ: > 0 ở 38.3% chuyến sạch và 3.4% chuyến gian lận (khách gian lận không dùng mã nào ngoài NEWUSER50 của promo_abuse: tạo tác của bộ sinh);
đóng góp SHAP trung bình của nó: -1.14 khi > 0, +0.32 khi = 0
```

Mô hình đã học lối tắt này: khi `cust_promo_share_30d > 0`, đóng góp trung bình là −1,14; khi bằng 0 là +0,32. Phát hiện này chỉ chứng minh cho `cust_promo_share_30d`, không phải cho mọi đặc trưng đứng đầu; trên dữ liệu thật, đặc trưng như thế đáng bị đặt câu hỏi trong review mô hình.

## Hàng đợi và nhật ký trên PostgreSQL

Mỗi chuyến vào hàng đợi mang theo điều người duyệt và kiểm toán cần: điểm, **phiên bản mô hình** và **phiên bản cấu hình rule**, lý do của mô hình (SHAP) cộng lý do của rule engine, mức ưu tiên và hạn duyệt. Ưu tiên là chi phí kỳ vọng của bỏ sót `p × (tiền chuyến + 20.000)`; hạn duyệt (SLA) là 24 giờ. Hàng đợi nằm ở schema `lab61` của PostgreSQL hạ tầng chung, tạo lúc đầu và xoá khi xong, kể cả khi lỗi (`review.py`, trích):

```sql
CREATE TABLE lab61.review_queue (
    queue_id bigserial PRIMARY KEY,
    trip_id text NOT NULL, model_version text NOT NULL, rules_version integer NOT NULL,
    score numeric(9,8) NOT NULL, priority numeric(14,2) NOT NULL, reasons jsonb NOT NULL,
    status text NOT NULL DEFAULT 'needs_review' CHECK (status IN ('needs_review', 'confirmed_fraud', 'cleared')),
    enqueued_at timestamptz NOT NULL, due_at timestamptz NOT NULL,
    decided_at timestamptz, decided_by text REFERENCES lab61.reviewer (reviewer_id), decision_note text,
    UNIQUE (trip_id, model_version),
    CHECK (decided_at IS NULL OR decided_at >= enqueued_at),
    CHECK ((status = 'needs_review') = (decided_by IS NULL)),
    CHECK ((decided_by IS NULL) = (decided_at IS NULL)));
CREATE TABLE lab61.decision_log (        -- trigger chặn UPDATE, DELETE, TRUNCATE và chặn dòng không khớp hàng đợi
    log_id bigserial PRIMARY KEY, at timestamptz NOT NULL,
    event text NOT NULL CHECK (event IN ('flagged', 'confirmed_fraud', 'cleared')),
    queue_id bigint NOT NULL REFERENCES lab61.review_queue (queue_id),
    trip_id text NOT NULL, actor text NOT NULL, model_version text NOT NULL, rules_version integer NOT NULL,
    score numeric(9,8) NOT NULL, rule text, reasons jsonb, note text,
    UNIQUE (queue_id, event),
    CHECK (event <> 'flagged' OR actor = 'svc:fraud-scoring'),
    CHECK (event = 'flagged' OR actor ~ '^RV-[0-9]{3}$'));
```

Việc ghi phải **idempotent**: consumer Kafka có thể nhận trùng (bài 39). `UNIQUE (trip_id, model_version)` cộng `ON CONFLICT DO NOTHING` làm việc đó; đề xuất và sự kiện `flagged` ghi trong một giao dịch:

```text
dựng 547 đề xuất: reasons của mô hình (SHAP) cộng lý do của rule engine bài 56 (658 lượt rule bắn, phiên bản cấu hình 1)
đã ghi 547 đề xuất và 547 sự kiện flagged (38.0 giây)
chạy lại 50 đề xuất đầu (như consumer nhận trùng): 50 bị bỏ qua, hàng đợi vẫn 547 dòng
một dòng review_queue (ưu tiên cao nhất, rút gọn lý do):
  queue_id=482 trip_id=TN-0192214 model=fraud-lgbm-f81991ad rules_version=1 score=0.84055653 priority=182283.09 status=needs_review
  enqueued_at=2026-04-28T10:55:40.017051+00:00 due_at=2026-04-29T10:55:40.017051+00:00
  model  +3.94 route_ratio=0.3936: đường chim bay chỉ bằng 0.39 lần quãng đường khai báo
  model  +3.51 speed_kmh=23.1163: tốc độ trung bình trong chuyến 23 km/h
  model  +0.57 distance_km=43.08: quãng đường khai báo 43.1 km
```

Lab ghi tuần tự cho đủ ví dụ; production gom theo lô. Rồi `RV-001` mô phỏng làm 70 đề xuất mỗi ngày 08:00 đến 17:00 (mọi ngày dương lịch, kể cả cuối tuần), lấy đề xuất ưu tiên cao nhất đã vào hàng đợi:

```text
RV-001 (mô phỏng, 70 đề xuất mỗi ngày làm việc 08:00-17:00, ưu tiên theo chi phí kỳ vọng) chốt 542 đề xuất trong 34.4 giây: {'confirmed_fraud': 406, 'needs_review': 5, 'cleared': 136}
```

## Ai được chốt: ràng buộc và che số giấy tờ

Quy tắc «chỉ người duyệt mới chốt» mà chỉ nằm trong mã Python thì một câu `UPDATE` viết vội phá được. Lab ép nó ở cơ sở dữ liệu: khoá ngoại `decided_by` tới bảng `reviewer` (chỉ có `RV-001`, trigger kiểm người đó đang hoạt động), CHECK ràng buộc trạng thái với người chốt và thời điểm chốt, trigger chặn INSERT đề xuất đã ở trạng thái chốt, sửa dòng đã chốt, xoá dòng và mọi thay đổi `decision_log`. Mỗi dòng log còn phải **khớp hàng đợi** (cùng chuyến, phiên bản, điểm; sự kiện chốt đúng với trạng thái, người chốt, thời điểm hiện có), nên không ghi được log «giả mạo». Hai mươi ca thử, mỗi ca phải bị chặn:

```text
  dịch vụ chấm điểm tự chốt confirmed_fraud: người duyệt svc:fraud-scoring không hoạt động
  người duyệt chưa đăng ký (RV-002) chốt cleared: người duyệt RV-002 không hoạt động
  người duyệt đã nghỉ (RV-009, active = false) chốt cleared: người duyệt RV-009 không hoạt động
  confirmed_fraud mà không ghi ai chốt: new row for relation "review_queue" violates check constraint "review_queue_check2"
  chốt với thời điểm trước lúc vào hàng đợi: new row for relation "review_queue" violates check constraint "review_queue_check1"
  sửa trực tiếp đề xuất đã chốt (confirmed_fraud -> cleared): quyết định 1 đã chốt, không sửa
  đổi điểm của đề xuất đang chờ: không đổi nội dung đề xuất của dịch vụ chấm điểm
  đổi mức ưu tiên của đề xuất đang chờ: không đổi nội dung đề xuất của dịch vụ chấm điểm
  xóa dòng review_queue: review_queue không cho xóa dòng
  INSERT thẳng một đề xuất đã ở trạng thái confirmed_fraud: đề xuất mới chỉ được vào ở needs_review, không phải confirmed_fraud
  sửa decision_log: decision_log chỉ ghi thêm (UPDATE bị chặn)
  xóa decision_log: decision_log chỉ ghi thêm (DELETE bị chặn)
  TRUNCATE decision_log: decision_log chỉ ghi thêm (TRUNCATE bị chặn)
  ghi sự kiện confirmed_fraud với actor là dịch vụ chấm điểm: sự kiện confirmed_fraud không khớp quyết định đang có trong hàng đợi (trạng thái, người chốt, thời điểm)
  ghi sự kiện flagged với actor là người duyệt: new row for relation "decision_log" violates check constraint "decision_log_check"
  ghi hai lần cùng sự kiện flagged cho một đề xuất: duplicate key value violates unique constraint "decision_log_queue_id_event_key"
  ghi log có điểm khác điểm trong hàng đợi (log giả mạo): log không khớp đề xuất 543 trong hàng đợi (chuyến, phiên bản, điểm)
  ghi log chốt cleared cho đề xuất đã là confirmed_fraud: sự kiện cleared không khớp quyết định đang có trong hàng đợi (trạng thái, người chốt, thời điểm)
  ghi log chốt cho đề xuất còn needs_review (chưa có quyết định trong hàng đợi): sự kiện cleared không khớp quyết định đang có trong hàng đợi (trạng thái, người chốt, thời điểm)
  hai lần chốt cùng đề xuất qua store.decide: AlreadyDecided: đề xuất 1 không còn ở needs_review
```

Bản DDL đầu của lab có hai lỗ mà ca thử bắt được. Một: CHECK viết `(status = 'needs_review') = (decided_by IS NULL AND decided_at IS NULL)`, và câu `UPDATE` đặt chốt với `decided_by` rỗng nhưng có `decided_at` lọt qua, vì vế phải `false` khớp vế trái `false`. Hai: thiếu trigger INSERT và kiểm khớp log. Giới hạn: người có quyền chủ bảng hoặc superuser vẫn `DISABLE TRIGGER`, `DROP TABLE` hay sửa dữ liệu được, nên «không viết lại được» chỉ đúng với người không có quyền đó. Production cần vai trò riêng cho service chấm điểm (chỉ INSERT), `REVOKE` các quyền còn lại, `reviewer_id` lấy từ phiên đăng nhập đã xác thực (bài 34), và một bản sao nhật ký ở kho chỉ ghi thêm ngoài tầm với của quản trị cơ sở dữ liệu.

Ghi chú tự do của người duyệt có thể chứa số giấy tờ mà người gõ dán vào. `pii.py` cất đi ngày giờ và tiền có dấu chấm ngăn nghìn (số bắt đầu bằng 0 không bao giờ là tiền), rồi tìm mọi dãy chữ số xen một dấu cách, chấm, gạch hoặc gạch chéo và che dãy từ 9 chữ số trở lên theo quy ước `099*******89`. Mọi trường văn bản đều qua hàm che, kể cả mã chuyến và tên phiên bản. Lab thử một ghi chú có hai kiểu số, rồi quét mọi trường văn bản của cả hai bảng bằng chính hàm che (còn che thêm được gì thì là rò):

```text
ghi chú người duyệt gõ:  CCCD 099123456789 dùng ở hai tài khoản, khách gọi từ 0900 000 001 hoặc 09 91 23 45 67 89, cước 125.000.000 VND ngày 2026-03-02 10:00
ghi chú được lưu:        CCCD 099*******89 dùng ở hai tài khoản, khách gọi từ 090*****01 hoặc 099*******89, cước 125.000.000 VND ngày 2026-03-02 10:00
quét 2,184 trường văn bản của cả hai bảng bằng chính hàm che (kể cả số xen dấu cách, chấm, gạch): 0 trường còn có thể che thêm
```

Đây là lớp chặn cuối. Giới hạn: số viết liền từ 9 chữ số trở lên luôn bị che kể cả khi là tiền (`125000000` thành `125****00`); số viết chữ hoặc tách bằng ký tự lạ lọt qua. Ở dữ liệu thật hãy dùng mã định danh giả (HMAC-SHA-256 có khoá, như bài 47) và đừng đưa số giấy tờ vào lý do ngay từ đầu.

## Công suất, SLA và tỉ lệ sai cho phép

«Tỉ lệ sai cho phép» rơi ra từ chi phí, công suất người duyệt và hạn duyệt. Phí duyệt (8.000 VND) trả cho **mọi** đề xuất, nên chuyến đáng duyệt khi `p × (tiền chuyến + 20.000) > 8.000`. Lab chia đề xuất theo dải điểm và đếm lãi ròng bằng nhãn cuối, với hai giả định: gắn cờ chặn hết thiệt hại (kể cả duyệt trễ), và `amount_vnd` đại diện cho thiệt hại (với promo_abuse thiệt hại thật là phần giảm):

```text
dải điểm của các đề xuất (nhãn cuối; phí duyệt 8,000 VND cho MỌI đề xuất, bỏ sót = tiền chuyến + 20,000 VND):
       dải p  đề xuất gian lận precision  tiền chặn được  chi phí duyệt    lãi ròng
 [0.069,0.3)       80       16     0.200         976,281        640,000     336,281
 [0.300,0.6)       88       41     0.466       2,025,139        704,000   1,321,139
 [0.600,1.0)      379      350     0.923      17,845,218      3,032,000  14,813,218
điểm hòa vốn p* = phí duyệt / (tiền chuyến + xử lý), vì phí duyệt trả cho mọi cờ: chuyến 30,000 VND -> 0.160, chuyến 120,000 VND -> 0.057, chuyến 500,000 VND -> 0.015
```

Dải thấp nhất (precision 0,200, 80 đề xuất) lãi ròng 336.281 VND trên 640.000 VND phí duyệt: 80 % đề xuất ở đó là gắn cờ nhầm mà tổng vẫn chưa lỗ; ở ba seed lãi ròng dải này là 0,34, 0,05 và 0,30 triệu VND, sát hoà vốn. Công suất người duyệt mới là giới hạn thật. Lab mô phỏng cùng 547 đề xuất với nhiều mức công suất và hai chính sách chọn việc (ưu tiên theo chi phí kỳ vọng, hoặc FIFO). «Tiền bắt được» giả sử duyệt trễ vẫn chặn đủ thiệt hại; «chờ p95» chỉ tính đề xuất đã được duyệt nên bị cắt khi còn tồn đọng:

```text
 công suất/ngày chính sách đã duyệt tồn đọng vi phạm SLA  chờ p95 đã duyệt (giờ) gian lận bắt tiền bắt được
             40   priority      400      147         212                   118.4    365/407         91.1%
             50   priority      500       47         127                   132.1    399/407         98.5%
             50       fifo      500       47         383                    46.4    384/407         94.4%
             65   priority      542        5          24                    22.2    406/407         99.8%
             65       fifo      542        5           5                    17.8    406/407         99.8%
             70   priority      542        5           8                    18.7    406/407         99.8%
            100   priority      542        5           0                    16.1    406/407         99.8%
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Biểu đồ cột số đề xuất vượt hạn 24 giờ theo công suất người duyệt, trên 547 đề xuất. Ưu tiên theo chi phí kỳ vọng: 212 ở 40 mỗi ngày, 127 ở 50, 24 ở 65, 8 ở 70, 0 ở 100. FIFO: 383 ở 50 mỗi ngày và 5 ở 65. Tiền bắt được: 91,1 phần trăm ở 40 mỗi ngày, 98,5 phần trăm ở 50 (FIFO 94,4), 99,8 phần trăm từ 65 trở lên."><defs></defs><path d="M80 230 L840 230" class="f-none s-line"/><rect x="80" y="14" width="14" height="14" rx="3" class="f-pk-soft s-pk"/><text x="102" y="26" class="t-small">ưu tiên theo chi phí kỳ vọng</text><rect x="320" y="14" width="14" height="14" rx="3" class="f-warn-soft s-warn"/><text x="342" y="26" class="t-small">FIFO</text><rect x="100" y="134" width="44" height="96" rx="3" class="f-pk-soft s-pk"/><text x="122" y="128" class="t-small" text-anchor="middle">212</text><rect x="240" y="173" width="44" height="57" rx="3" class="f-pk-soft s-pk"/><text x="262" y="167" class="t-small" text-anchor="middle">127</text><rect x="296" y="57" width="44" height="173" rx="3" class="f-warn-soft s-warn"/><text x="318" y="51" class="t-small" text-anchor="middle">383</text><rect x="380" y="219" width="44" height="11" rx="3" class="f-pk-soft s-pk"/><text x="402" y="213" class="t-small" text-anchor="middle">24</text><rect x="436" y="228" width="44" height="2" rx="1" class="f-warn-soft s-warn"/><text x="458" y="222" class="t-small" text-anchor="middle">5</text><rect x="548" y="226" width="44" height="4" rx="2" class="f-pk-soft s-pk"/><text x="570" y="220" class="t-small" text-anchor="middle">8</text><rect x="688" y="229" width="44" height="1" rx="1" class="f-pk s-pk"/><text x="710" y="223" class="t-small" text-anchor="middle">0</text><text x="122" y="250" class="t-small" text-anchor="middle">40/ngày</text><text x="122" y="268" class="t-small" text-anchor="middle">bắt 91,1 %</text><text x="290" y="250" class="t-small" text-anchor="middle">50/ngày</text><text x="290" y="268" class="t-small" text-anchor="middle">98,5 % | FIFO 94,4 %</text><text x="430" y="250" class="t-small" text-anchor="middle">65/ngày</text><text x="430" y="268" class="t-small" text-anchor="middle">99,8 % cả hai</text><text x="570" y="250" class="t-small" text-anchor="middle">70/ngày</text><text x="570" y="268" class="t-small" text-anchor="middle">99,8 %</text><text x="710" y="250" class="t-small" text-anchor="middle">100/ngày</text><text x="710" y="268" class="t-small" text-anchor="middle">99,8 %</text><text x="80" y="292" class="t-small">Cột: số đề xuất vượt hạn 24 giờ (trong 547). Dòng dưới nhãn: tiền bắt được (giả sử duyệt trễ vẫn chặn đủ).</text></svg>
</div>
<figcaption>Hình 2. Công suất 40 đến 100 đề xuất mỗi ngày với 547 đề xuất trong 10 ngày (một seed). Khi công suất thấp hơn lượng đề xuất, ưu tiên theo chi phí kỳ vọng bắt nhiều tiền hơn FIFO và ít đề xuất quá hạn hơn, nhưng đuôi chờ dài hơn nhiều.</figcaption>
</figure>

Lượng đề xuất trung bình là 54,7 mỗi ngày: ở 40 và 50 mỗi ngày tồn đọng tích lại, từ 65 trở lên chỉ còn 5. Khi thiếu công suất (50 mỗi ngày), ưu tiên bắt 98,5 % tiền so với 94,4 % của FIFO và chỉ 127 đề xuất quá hạn so với 383, nhưng đề xuất rẻ ở cuối hàng chờ rất lâu (p95 132,1 giờ so với 46,4). Khi công suất gần đủ (65 mỗi ngày), cả hai bắt cùng 99,8 % tiền và FIFO ít quá hạn hơn (5 so với 24). Một seed chưa đủ cho kết luận thứ bậc, nên lab lặp lại ở bảng cuối mục.

```text
lần chạy thật (70/ngày, ưu tiên; gồm 1 đề xuất chốt ở mục 5): 547 đề xuất, 543 đã chốt, tồn đọng 4; vi phạm SLA 24 giờ: 8 (1%); thời gian chờ của đề xuất đã chốt: trung vị 5.0 giờ, p95 18.7 giờ
đề xuất vào hàng đợi trung bình 54.7 mỗi ngày; công suất 70/ngày
ngoại suy thô theo tỉ lệ: ở 2 triệu chuyến/ngày, cùng tỉ lệ gắn cờ 1.64% cho 32,739 đề xuất/ngày, tức 2,728 giờ duyệt/ngày nếu mỗi đề xuất 5 phút (cửa sổ này có 3,342 chuyến/ngày)
```

Dòng cuối là cảnh báo về quy mô, không phải dự báo: cùng tỉ lệ gắn cờ ở 2 triệu chuyến mỗi ngày cần chừng 2.700 giờ duyệt mỗi ngày, nên tỉ lệ gắn cờ là tham số vận hành. Nhật ký cho **audit**: «ai chốt chuyến này theo mô hình nào» là một truy vấn. Lab chưa đo độ chính xác của chính người duyệt: cần mẫu đề xuất được duyệt lại độc lập và mẫu chuyến **không** gắn cờ để ước lượng bỏ sót.

Các phép đo chính được lặp lại với seed 42, 7 và 123 (dữ liệu và mô hình sinh lại, không dùng PostgreSQL):

```text
 seed    cờ gian lận precision top-1 SHAP top-1 gain            CI chênh xóa 3 lý do xóa 3 ngẫu nhiên  tiền 50/ngày ưu tiên   FIFO  tiền 65/ngày ưu tiên   FIFO
   42   547      407     0.744      83.0%      21.9%    [+56.0%, +65.6%]       0.058            0.785                 98.5%  94.4%                 99.8%  99.8%
    7   630      450     0.714      75.8%      23.3%    [+47.8%, +57.1%]       0.051            0.783                 94.5%  83.0%                 99.5%  99.5%
  123   601      455     0.757      93.4%      77.1%    [+11.9%, +21.1%]       0.093            0.792                 95.2%  85.7%                 99.7%  99.7%

SLA 24 giờ theo công suất (số đề xuất quá hạn; p95 chờ chỉ tính đề xuất đã được duyệt), ưu tiên / FIFO:
 seed  đề xuất/ngày  quá hạn 50 ưu tiên  FIFO  quá hạn 65 ưu tiên  FIFO  p95 chờ 65 ưu tiên   FIFO
   42          54.7                 127   383                  24     5                22.2   17.8
    7          63.0                 168   353                  42    44                44.5   25.5
  123          60.1                 170   450                  62    11                50.5   21.7

lãi ròng theo dải điểm (triệu VND, nhãn cuối): dải thấp nhất, 0,3 đến 0,6, từ 0,6
   42        0.34        1.32       14.81
    7        0.05        1.38       15.96
  123        0.30        1.41       16.83

bản đồ HẸP (chỉ đặc trưng nêu thẳng trong hình 1 bài 54): top-1 SHAP, top-3 SHAP, top-3 ngẫu nhiên, và top-3 SHAP theo bản đồ rộng để so:
   42     81.6%     95.6%     37.5%     98.0%
    7     75.3%     94.4%     36.7%     96.0%
  123     79.8%     96.0%     37.2%     99.1%
```

Ở cả ba seed top-1 SHAP trúng nhiều hơn top-1 gain cố định (khoảng tin cậy dưới lớn hơn 0), xoá ba lý do hạ p sâu hơn nhiều so với xoá ba đặc trưng ngẫu nhiên, và ở 50 mỗi ngày ưu tiên bắt nhiều tiền hơn FIFO (98,5, 94,5, 95,2 % so với 94,4, 83,0, 85,7 %). Ở seed 123 bộ gain cố định trúng tới 77,1 % nên chênh còn [+11,9 %, +21,1 %]: vững về chiều, không vững về độ lớn. SLA ở 65 mỗi ngày không đồng đều: FIFO ít quá hạn hơn rõ ở seed 42 (5 so với 24) và 123 (11 so với 62), nhưng ở seed 7 FIFO 44 và ưu tiên 42 (p95 chờ 25,5 so với 44,5 giờ). Công suất nên tính theo lượng đề xuất mỗi ngày (54,7, 63,0, 60,1 ở ba seed), không theo một con số cố định.

## Chạy và kiểm chứng

Cần PostgreSQL của hạ tầng chung, uv và Python 3.14; macOS cần `brew install libomp`; lần đầu cần mạng, rồi lab dựng đặc trưng và sinh lại dữ liệu cho seed 7 và 123. Từ thư mục gốc repository:

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
uv run --project examples/toi-noi-python/61-shap-audit-human-review examples/toi-noi-python/61-shap-audit-human-review/main.py
uv run --project examples/toi-noi-python/61-shap-audit-human-review examples/toi-noi-python/61-shap-audit-human-review/main.py --verify
uv run --project examples/toi-noi-python/61-shap-audit-human-review pytest examples/toi-noi-python/61-shap-audit-human-review -q
```

Không nối được PostgreSQL thì lab in `FAIL: không nối được PostgreSQL tại <url> (docker compose ...)` và thoát mã 1. Ba dòng sau lấy từ cùng lần chạy `--verify` (lần chạy này mất 2 phút 50 giây, load average 1 phút khoảng 261.87 trên 10 lõi) và `pytest`: dòng dọn schema, dòng kết quả, dòng pytest:

```text
Dọn: DROP SCHEMA lab61 CASCADE; schema còn tồn tại = False
Kiểm chứng: OK 61/61
30 passed in 15.95s
```

`--verify` kiểm 61 bất biến, không ghim số tuyệt đối hay tốc độ: SHAP cộng ra điểm thô, log-odds của p là hàm affine của điểm thô, hai mươi ca chặn của cơ sở dữ liệu, che số giấy tờ, nhật ký khớp hàng đợi, schema đã xoá, khoảng tin cậy top-1 ở ba seed.

| Hiện tượng | Kiểm tra trước |
|---|---|
| Tổng SHAP không khớp log-odds của p | SHAP giải thích điểm thô chưa hiệu chỉnh; đổi sang thang của p bằng bộ hiệu chỉnh, đừng cộng thẳng |
| Lý do trùng lặp hoặc đổi theo lần chạy | Đặc trưng tương quan có thể chia nhau công; giữ nền tham chiếu cố định và so lại trên cùng dữ liệu |
| Lý do dựa vào đặc trưng vô nghĩa với người duyệt | Có thể là tạo tác của dữ liệu; đưa ra review mô hình trước khi hiển thị |
| `UPDATE` chốt dòng nhưng nhật ký thiếu sự kiện, hoặc log không ghi được | Chốt và ghi log phải cùng một giao dịch, log phải khớp hàng đợi; kiểm CHECK và trigger bằng ca thử như ở mục «Ai được chốt» |
| Hàng đợi tồn đọng, nhiều đề xuất quá hạn | So công suất với lượng đề xuất mỗi ngày; chỉnh ngưỡng hoặc thêm người trước khi đổi chính sách chọn việc |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Dữ liệu là tổng hợp (200.000 chuyến, 60 ngày) và có hai tạo tác của bộ sinh: tỉ lệ NEWUSER50 của chuyến sạch giảm theo thời gian, và khách gian lận không dùng mã nào ngoài NEWUSER50 nên <code>cust_promo_share_30d &gt; 0</code> «tố» chuyến sạch, điều mô hình đã học. <code>RV-001</code> là mô phỏng, luôn đúng; lịch công suất coi cả cuối tuần là ngày làm việc. Mô hình chi phí có hai giả định: gắn cờ chặn hết thiệt hại (người duyệt không sai, duyệt trễ vẫn chặn đủ, dù chờ tới 132 giờ), và <code>amount_vnd</code> đại diện cho thiệt hại (với promo_abuse thiệt hại thật là phần giảm); 5 phút mỗi đề xuất cũng là giả định. «Dấu vết mong đợi» là bản đồ ta viết từ bài 54; nhãn đánh giá là nhãn cuối; khoảng tin cậy phụ thuộc bộ sinh số của numpy. Lab dùng một tài khoản cơ sở dữ liệu có toàn quyền, và <code>reviewer_id</code> do người gọi truyền vào. Đã chạy với Python 3.14.7, shap 0.54.1, lightgbm 4.7.0, scikit-learn 1.9.1, numpy 2.5.4, polars 2.1.0, psycopg 3.3.8, PostgreSQL 17.11.</p>
</div>

**Bài tập:** (1) Một chuyến 80.000 VND. Theo mô hình chi phí của bài, `p` tối thiểu để chuyến này đáng vào hàng đợi là bao nhiêu? (2) Tổng đóng góp SHAP của một chuyến là +3,0 và `base_value` là −7,386. Với bộ hiệu chỉnh của lab, p hiển thị xấp xỉ bao nhiêu, và vì sao không đọc «+3,0» thành «tăng xác suất 3 điểm»?

<details><summary>Xem lời giải</summary><p>(1) Gắn cờ khi <code>p × (80.000 + 20.000) &gt; 8.000</code>, tức <b>p &gt; 0,08</b>. (2) Điểm thô = −7,386 + 3,0 = −4,386; log-odds của p = 1,073 + 0,943 × (−4,386) ≈ −3,063, nên p ≈ <b>0,045</b>. «+3,0» là đóng góp vào điểm thô ở thang log-odds trước hiệu chỉnh (ở thang log-odds của p là 0,943 × 3,0 ≈ 2,83); sigmoid đổi log-odds thành xác suất nên cùng một mức cộng làm xác suất tăng rất khác nhau tuỳ điểm xuất phát (ở thang xác suất gốc: từ điểm thô −7,386 là từ 0,06 % lên 1,2 %; từ điểm thô 0 là từ 50 % lên 95 %).</p></details>

Bài trước: [bài 60](/articles/python-model-registry-ab-drift-retraining). Bài tiếp: [bài 62](/articles/python-eta-prediction-regression).

## Tài liệu gốc

- [SHAP: TreeExplainer](https://shap.readthedocs.io/en/latest/generated/shap.TreeExplainer.html).
- [LightGBM: Booster (pred_contrib)](https://lightgbm.readthedocs.io/en/stable/pythonapi/lightgbm.Booster.html).
- [scikit-learn: Probability calibration](https://scikit-learn.org/stable/modules/calibration.html).
- [PostgreSQL 17: Trigger Functions](https://www.postgresql.org/docs/17/plpgsql-trigger.html).
- [PostgreSQL 17: Constraints](https://www.postgresql.org/docs/17/ddl-constraints.html).
- [psycopg 3: tài liệu](https://www.psycopg.org/psycopg3/docs/).
