---
slug: python-rule-engine-scoring-strategy-explainability
title: "Rule engine và điểm rủi ro: Strategy, cấu hình và giải thích được"
summary: "Bài 56: rule engine chấm điểm rủi ro gian lận: rule là hàm trả điểm kèm lý do, cấu hình YAML kiểm khi nạp, ba strategy gộp điểm, kiểm thử bằng pytest, đo từng rule và chỉ ra khi nào rule hết đủ. Số đo thật."
category: programming
level: advanced
tags: [python, toi-noi, fraud, rule-engine, strategy]
language: vi
---

Hàm `score_trip` của service fraud-scoring ở [bài 37](/articles/python-grpc-fraud-scoring-service) có ba luật viết thẳng bằng `if`, mỗi luật cộng một hằng số `W_...` vào điểm. Với [Tới Nơi](/articles/du-an-mau-toi-noi) thế là đủ để minh họa hợp đồng gRPC, chưa đủ để vận hành. Sau bài 55 ta có 30 đặc trưng và hơn chục luật đáng thử, vận hành muốn nâng điểm luật khuyến mãi vào tuần khuyến mãi lớn mà không chờ phát hành bản mới, và người duyệt muốn đọc được «vì sao chuyến này vào hàng đợi». Ba yêu cầu cùng chỉ về một hình dạng: mỗi luật là một giá trị có tên, có điểm, có lý do, đọc từ cấu hình.

Đây là bài 56 của lộ trình **Python nâng cao qua Tới Nơi**. Bạn cần Strategy ở [bài 19](/articles/python-design-pattern-strategy-state-adapter), 30 đặc trưng ở [bài 55](/articles/python-feature-engineering-tu-event-kafka) và dữ liệu cùng metric ở [bài 54](/articles/python-fraud-toi-noi-bai-toan-nhan-du-lieu-metric). Dữ liệu là tổng hợp nên số đo chỉ nói về bộ này; điểm rủi ro là đầu vào cho người duyệt, và engine cố ý không có dải nào tự khóa tài khoản.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Rule «thiết bị đã dùng <code>NEWUSER50</code> ít nhất một lần trong 7 ngày» có precision 0,097 trên train. Trên valid, precision của đúng rule đó sẽ gần 0,1, thấp hơn, hay cao hơn hẳn?</p>
<details><summary>Xem đáp án</summary><p><b>Cao hơn hẳn: 0,490.</b> Train có 3.391 chuyến bị gắn cờ, valid chỉ 406: trong bộ sinh, số «chuyến đầu của khách» giảm từ 17.192 (10 ngày đầu) xuống 1.869 (10 ngày cuối) vì quần thể khách gần như cố định. Đó là giả định của bộ sinh, và mọi rule dựa vào «khách mới» chịu ảnh hưởng; ngưỡng từ 2 lần thì precision là 1,000 ở cả hai tập.</p></details>
</div>

## Rule là một hàm trả điểm kèm lý do

Một rule nhận đặc trưng của **một** chuyến và trả `None` (không bắn) hoặc `RuleHit` (điểm và lý do). Kiểu của nó chỉ là một hàm, đúng tinh thần Strategy ở bài 19: không cần interface hay một lớp cho mỗi luật. `rules.py` (trích):

```python
@dataclass(frozen=True, slots=True)
class RuleHit:
    rule_id: str
    points: int
    reason: str            # câu đã điền số thật của chuyến này

type Features = Mapping[str, float | None]
type Rule = Callable[[Features], RuleHit | None]
type Aggregator = Callable[[Sequence[RuleHit]], float]

def build_rule(spec: RuleSpec) -> Rule:
    def rule(features: Features) -> RuleHit | None:
        values = [features[c.feature] for c in spec.when]   # lấy MỌI giá trị trước khi so
        if all(c.holds(v) for c, v in zip(spec.when, values, strict=True)):
            return RuleHit(spec.id, spec.points, spec.reason.format_map(features))
        return None
    return rule
```

`build_rule` là một closure giữ `spec` lại. Hai kiểu thiếu dữ liệu được phân biệt có chủ ý. Thiếu **khóa** đặc trưng thì `features[...]` ném `KeyError`: lược đồ đã đổi, lỗi phải thấy ngay. Dòng «lấy mọi giá trị trước» là có lý do: với `all(c.holds(features[c.feature]) ...)`, điều kiện đầu sai làm `all()` dừng và không bao giờ đụng khóa thứ hai, nên khóa bị đổi tên làm rule im lặng (một ca test canh việc này). Giá trị `None` hoặc `NaN` nghĩa là «không biết», và `Condition.holds` trả `False`: rule im lặng thay vì đoán (NaN phải loại riêng vì `nan != 1` là `True`). Bài 59 sẽ gặp trường hợp này.

Engine chỉ cần một danh sách callable, một aggregator và hai ngưỡng dải:

```python
class RuleEngine:
    def score(self, features: Features) -> Verdict:
        hits = tuple(h for rule in self.rules if (h := rule(features)) is not None)
        score = self.aggregate(hits)
        band = "review" if score >= self.review_at else "watch" if score >= self.watch_at else "none"
        return Verdict(score, band, hits)
```

Dải `review` là «đưa vào hàng đợi người duyệt», `watch` là «ghi nhận». Không có dải `block`: quyết định khóa thuộc về người duyệt và quy trình ở bài 61.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Đặc trưng của một chuyến đi vào mười một rule, mỗi rule là một hàm trả điểm kèm lý do hoặc None. Cấu hình rules.yaml cấp điểm, điều kiện và lý do cho các rule. Aggregator là một strategy gộp điểm của các rule đã bắn. Kết quả là Verdict gồm điểm, dải và lý do, đi vào hàng đợi người duyệt, không có đường tự khóa."><defs><marker id="py56-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="110" width="170" height="90" rx="10" class="f-phys-soft s-phys"/><text x="36" y="142" class="t-b">Đặc trưng</text><text x="36" y="162" class="t-small">một chuyến, 30 cột</text><text x="36" y="180" class="t-small">None = không biết</text><rect x="240" y="14" width="250" height="52" rx="8" class="f-warn-soft s-warn"/><text x="256" y="38" class="t-b">rules.yaml</text><text x="256" y="56" class="t-small">điểm, điều kiện, lý do, strategy</text><path d="M365 66 L365 98" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py56-0-arrow)"/><rect x="240" y="100" width="250" height="40" rx="8" class="f-pk-soft s-pk"/><text x="254" y="117" class="t-b">gps_jump_impossible</text><text x="254" y="133" class="t-small">(features) -> RuleHit | None</text><rect x="240" y="150" width="250" height="40" rx="8" class="f-pk-soft s-pk"/><text x="254" y="167" class="t-b">promo_farm</text><text x="254" y="183" class="t-small">im lặng thì trả None</text><rect x="240" y="200" width="250" height="40" rx="8" class="f-pk-soft s-pk"/><text x="254" y="217" class="t-b">... 11 rule</text><text x="254" y="233" class="t-small">mỗi rule một hàm, thêm không sửa engine</text><path d="M190 140 L240 120" class="f-none s-phys" stroke-width="2" marker-end="url(#py56-0-arrow)"/><path d="M190 155 L240 170" class="f-none s-phys" stroke-width="2" marker-end="url(#py56-0-arrow)"/><path d="M190 170 L240 220" class="f-none s-phys" stroke-width="2" marker-end="url(#py56-0-arrow)"/><rect x="550" y="100" width="150" height="140" rx="10" class="f-phys-soft s-phys"/><text x="566" y="128" class="t-b">Aggregator</text><text x="566" y="146" class="t-small">Strategy (bài 19)</text><text x="566" y="176" class="t-mono">sum</text><text x="566" y="194" class="t-mono">max</text><text x="566" y="212" class="t-mono">noisy_or</text><path d="M490 120 L550 150" class="f-none s-phys" stroke-width="2" marker-end="url(#py56-0-arrow)"/><path d="M490 170 L550 170" class="f-none s-phys" stroke-width="2" marker-end="url(#py56-0-arrow)"/><path d="M490 220 L550 190" class="f-none s-phys" stroke-width="2" marker-end="url(#py56-0-arrow)"/><rect x="740" y="50" width="120" height="90" rx="10" class="f-pk-soft s-pk"/><text x="756" y="80" class="t-b">Verdict</text><text x="756" y="100" class="t-small">điểm 0..100</text><text x="756" y="118" class="t-small">dải, lý do</text><path d="M700 150 L740 110" class="f-none s-phys" stroke-width="2" marker-end="url(#py56-0-arrow)"/><rect x="740" y="190" width="120" height="76" rx="10" class="f-warn-soft s-warn dash"/><text x="756" y="220" class="t-b">Người duyệt</text><text x="756" y="240" class="t-small">không tự khóa</text><path d="M800 140 L800 190" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py56-0-arrow)"/><text x="240" y="268" class="t-small">Điểm và lý do đi cùng nhau: người duyệt thấy rule nào bắn, với số liệu nào.</text></svg>
</div>
<figcaption>Hình 1. Rule engine chạy một chuyến qua mọi rule, gộp điểm bằng strategy chọn trong cấu hình, rồi trả điểm, dải và lý do cho người duyệt. Không có đường nào dẫn thẳng tới khóa tài khoản.</figcaption>
</figure>

## Cấu hình YAML, kiểm khi nạp

Cấu hình mô tả rule bằng dữ liệu: `id`, `points`, điều kiện `when` (tất cả phải thỏa) và câu `reason` có chỗ điền số. `rules.yaml` (trích hai trong 11 rule):

```yaml
version: 1
review_at: 40   # từ mức này đưa vào hàng đợi người duyệt
watch_at: 15    # từ mức này chỉ ghi nhận, chưa duyệt
aggregate: noisy_or   # sum | max | noisy_or

rules:
  - id: gps_jump_impossible
    points: 90
    when: ["drv_jump_speed_kmh > 120", "drv_jump_km >= 5"]
    reason: "tài xế nhảy {drv_jump_km:.0f} km giữa hai chuyến, tương đương {drv_jump_speed_kmh:.0f} km/h"
  - id: promo_farm
    points: 95
    when: ["dev_newuser_7d >= 2"]
    reason: "mã NEWUSER50 đã dùng {dev_newuser_7d:.0f} lần trên thiết bị này trong 7 ngày"
```

Mỗi điều kiện là một chuỗi `tên_đặc_trưng toán_tử số`, tách bằng biểu thức chính quy rồi tra bảng toán tử. Không dùng `eval`: file cấu hình là dữ liệu, không phải mã. Luật phức tạp hơn thì viết thành hàm Python và đưa vào engine như một callable.

Nguy cơ của cấu hình là lỗi gõ lúc 2 giờ sáng (đặc trưng đổi tên, `points` thành 150, `id` trùng), nên `parse_config` kiểm khi nạp, và mọi cấu hình hỏng đều ra `ConfigError` chứ không bao giờ một ngoại lệ lạ (bộ test có 12 file hỏng kiểu đó). Thử với một file nhiều lỗi (trích):

```text
một file cấu hình có nhiều lỗi: ConfigError báo 10 lỗi một lượt:
  - aggregate 'average' không thuộc ['max', 'noisy_or', 'sum']
  - watch_at phải nhỏ hơn review_at
  - rules[0] (speed_impossible): khóa không biết 'point' (gõ nhầm?)
  - rules[0] (speed_impossible): points phải là số nguyên 1..100, nhận None
  - rules[1] (speed_impossible): id trùng
  - rules[1] (speed_impossible): points phải là số nguyên 1..100, nhận 150
  - rules[1] (speed_impossible): đặc trưng 'speed_kph' không có trong FEATURES
  - rules[1] (speed_impossible): điều kiện 'dev_trips_1h >=> 2' không có dạng «tên_đặc_trưng toán_tử số»
  - rules[1] (speed_impossible): reason dùng {dev_trips_1h} nhưng đó không phải đặc trưng trong when
  - rules[1] (speed_impossible): reason dùng {speed_kmh} nhưng đó không phải đặc trưng trong when
```

Hai chi tiết về YAML. File được đọc bằng lớp con của `SafeLoader` (như `yaml.safe_load`), nên thẻ `!!python/object/apply:...` bị từ chối và khóa trùng cũng bị từ chối (`version` hai lần ra `ConstructorError: khóa trùng 'version'`). Còn `reason` là chuỗi `format_map`: parser chỉ nhận tên đặc trưng trần, không truy cập thuộc tính hay lồng trường, và chỉ đặc trưng có trong `when` của chính rule, để một `None` không làm `:.0f` nổ giữa luồng chấm điểm. Chi tiết nằm trong README của lab.

Đổi hành vi bằng cấu hình không đụng code. Lab chấm lại valid với bốn tổ hợp của `aggregate` và `review_at` (đổi từng biến một):

```text
  aggregate / review_at     gắn cờ   bắt precision
  noisy_or / 40  (gốc)         999   667     0.668
  noisy_or / 60                766   607     0.792
  max / 40                     992   665     0.670
  max / 60  (đã đổi)           763   604     0.792
```

Chính `review_at` quyết định hàng đợi: nâng từ 40 lên 60 làm hàng đợi ngắn đi 233 chuyến và chính xác hơn, đổi lại bắt ít hơn 60 chuyến gian lận; đổi `aggregate` chỉ dịch vài chuyến. Đó là quyết định về công suất người duyệt.

## Strategy gộp điểm: sum, max hay noisy-or

Khi nhiều rule cùng bắn, điểm cuối là gì? Mỗi cách trả lời là một hàm cùng chữ ký `Sequence[RuleHit] -> float`, tức một Strategy kiểu bài 19, và cấu hình chọn theo tên:

```python
def sum_capped(hits):   # cộng điểm, chặn ở 100
    return float(min(100, sum(h.points for h in hits)))

def max_points(hits):   # chỉ lấy rule mạnh nhất
    return float(max((h.points for h in hits), default=0))

def noisy_or(hits):     # điểm / 100 coi như xác suất, các rule độc lập: 1 - tích(1 - p)
    return 100.0 * (1.0 - math.prod(1.0 - h.points / 100.0 for h in hits))

AGGREGATORS = {"sum": sum_capped, "max": max_points, "noisy_or": noisy_or}
```

Với một chuyến có hai rule 35 và 15 điểm, `sum` cho 50, `max` cho 35 và `noisy_or` cho 44,75. Trên valid, với cùng 11 rule (chi phí theo mô hình ở cuối mục, chọn ngưỡng trên nửa đầu valid, báo cáo trên nửa sau):

```text
aggregator  PR-AUC  ROC-AUC số mức điểm  top 1% bắt  ngưỡng chi phí  gắn cờ   TP    FP  chi phí (VND)
sum          0.533    0.810          13         553            40.0     494  305   189     13,370,933
max          0.567    0.810           9         553            40.0     490  304   186     13,407,968
noisy_or     0.565    0.810          56         553            40.0     494  305   189     13,370,933
(top 1% = 667 chuyến điểm cao nhất; chi phí tính trên nửa sau valid, ngày 50 trở đi, với 33,416 chuyến)
không làm gì: chi phí 24,495,068 VND trên cùng nửa sau
```

Chi phí là mô hình của bài 54, đã sửa: **8.000 VND × số chuyến gắn cờ (kể cả gian lận thật) + (tiền chuyến + 20.000 VND) × gian lận bỏ sót**, giả định gắn cờ chặn hết thiệt hại và `amount_vnd` đại diện cho thiệt hại. Số đo không phân thắng bại: ROC-AUC y hệt 0,810, top 1 % bắt cùng 553 chuyến, chi phí chênh dưới 1 %. Lựa chọn dựa vào hình dạng điểm: `max` bỏ thông tin «nhiều rule cùng bắn» (một rule bắn: gian lận 28,3 %; hai rule: 81,7 %, mục sau), `sum` vượt trần, `noisy_or` tăng chậm dần, không vượt 100 và cho 56 mức điểm (9 của `max`). Nó vẫn tính hai lần khi hai rule nhìn cùng một dấu vết, chỉ chậm hơn `sum`.

Điểm cũng không phải xác suất. Tỉ lệ gian lận thật theo khoảng điểm `noisy_or`:

```text
 khoảng điểm   chuyến  gian lận   tỉ lệ
           0   65,109       425    0.7%
     (0, 20)      410        44   10.7%
    [20, 40)      186         5    2.7%
    [40, 60)      233        60   25.8%
    [60, 70)      134        70   52.2%
    [70, 80)       69        32   46.4%
   [80, 100]      563       505   89.7%
```

Tỉ lệ không đơn điệu ([0, 20) cao hơn [20, 40); [60, 70) cao hơn [70, 80)), nên điểm 70 không có nghĩa «70 %»: đọc điểm như thứ tự thô; muốn xác suất thì phải hiệu chỉnh (bài 57). Ngưỡng `review_at: 40` chọn như bài 54, trên nửa đầu valid (ngày 40 đến 50). Trên nửa sau, nửa không dùng để chọn, dải review gắn cờ 494 chuyến, bắt 305 trong 494 gian lận (precision 0,617, recall 0,617), tốn 13,37 triệu VND so với 24,50 triệu nếu không làm gì.

## Đo từng rule: điểm theo precision, ngưỡng chọn trên train

`points` bằng precision đo trên train, làm tròn xuống bội 5 (0,991 thành 95, 0,452 thành 40); `--verify` đòi nó không vượt 100 lần precision đó quá 10 điểm. Lab in cả hai tập và một cột đo bằng nhãn biết ở ngày 40:

```text
rule                        điểm train: bắn precision (nhãn ngày 40) valid: bắn precision  recall
gps_jump_impossible           90        171     0.924          0.842         86     0.919   0.069
speed_impossible              70        236     0.746          0.678        128     0.672   0.075
pickup_far                    65        478     0.661          0.607        259     0.591   0.134
trip_not_moving               95        234     0.991          0.962        147     1.000   0.129
device_many_accounts_24h      90        182     0.923          0.863        133     0.955   0.111
device_night_activity         70        207     0.725          0.681        140     0.729   0.089
route_short_repeat            35        440     0.393          0.375        280     0.332   0.082
promo_farm                    95        221     1.000          1.000        144     1.000   0.126
promo_shared_device           85        349     0.897          0.897        207     0.957   0.174
pair_repeat                   15        562     0.185          0.165        474     0.211   0.088
driver_device_booking         40        431     0.452          0.385        302     0.440   0.117
điểm vượt 100 x precision theo nhãn biết ở ngày 40: 6/11 rule; bảng chọn điểm bằng nhãn cuối cùng
```

Ba điều đọc ra. Một: điểm đặt từ nhãn **cuối cùng**, làm tròn xuống bội 5 và **trần ở 95** (precision 1,000 vẫn chỉ 95, vì 100 nghĩa là chắc chắn tuyệt đối; `--verify` kiểm trần này); với nhãn biết ở ngày 40 (thứ ta có lúc huấn luyện, bài 54), 6 trong 11 rule có điểm vượt precision đo được. Hai: **mỗi rule rất hẹp**, recall cao nhất chỉ 0,174; gộp mười một rule thì dải review mới bắt 667 trong 1.141 chuyến gian lận (58,5 %). Ba: precision 1,000 của `promo_farm` và `trip_not_moving` là đặc điểm của bộ sinh (không phải mọi chuyến ảo «không đi đâu» như vậy); người gian lận thật biết luật sẽ tránh chúng.

Số rule cùng bắn trên một chuyến là tín hiệu mạnh hơn bất kỳ rule riêng nào:

```text
 số rule   chuyến  gian lận   tỉ lệ
       0   65,109       425    0.7%
       1    1,156       327   28.3%
       2      229       187   81.7%
    >= 3      210       202   96.2%
```

Chuyến không rule nào bắn vẫn chứa 425 chuyến gian lận: điểm 0 là các luật im lặng, không phải chuyến sạch.

Ngưỡng của từng điều kiện cũng chọn trên train. Lab quét `dev_customers_24h` (số khách khác nhau trên một thiết bị trong 24 giờ) và `dev_newuser_7d` (số lần dùng `NEWUSER50` trên thiết bị trong 7 ngày):

```text
điều kiện                  train: bắn precision  recall valid: bắn precision  recall
dev_customers_24h >= 2            847     0.587   0.247        533     0.644   0.301
dev_customers_24h >= 3            182     0.923   0.084        133     0.955   0.111
dev_customers_24h >= 4             58     0.983   0.028         46     1.000   0.040
dev_newuser_7d >= 1             3,391     0.097   0.163        406     0.490   0.174
dev_newuser_7d >= 2               221     1.000   0.110        144     1.000   0.126
```

Nâng ngưỡng số khách thì precision tăng, recall giảm; lab chọn 3. Dòng `dev_newuser_7d >= 1` là đáp án câu hỏi đầu bài. Chuyến sạch theo giai đoạn cho thấy vì sao:

```text
   giai đoạn  chuyến sạch  chuyến đầu của khách  có NEWUSER50   tỉ lệ
    0 đến 10       32,793                17,192         2,534   14.7%
   10 đến 20       32,584                 7,781         1,151   14.8%
   20 đến 30       32,634                 4,752           742   15.6%
   30 đến 40       32,426                 3,169           459   14.5%
   40 đến 50       32,641                 2,450           399   16.3%
   50 đến 60       32,905                 1,869           298   15.9%
```

Tỉ lệ chuyến đầu dùng mã gần như đứng yên (14,5 đến 16,3 %), còn số chuyến đầu giảm gần 10 lần: quần thể khách của bộ sinh gần như cố định nên «khách mới» cạn dần. Đó là giả định của bộ sinh, không phải hiệu ứng của cửa sổ chưa đầy, và nó làm precision của rule dựa vào «khách mới» đổi giữa train và valid. Ngưỡng `>= 2` an toàn hơn `>= 1`.

## Giải thích được và kiểm thử được

`Verdict.reasons` xếp lý do theo điểm giảm dần, mỗi dòng có tên rule, câu đã điền số thật và điểm của rule. Lab in năm chuyến của valid (trích: chuyến đầu có sáu lý do, ở đây giữ ba dòng; `promo_abuse` giữ hai):

```text
TN-0173322  fraud_type=ghost_trip  điểm 100.0  dải review
    - [trip_not_moving] đồng hồ cước báo 10.7 km nhưng điểm cuối cách điểm đầu 0.07 km (rule 95)
    - [device_many_accounts_24h] 3 tài khoản khách khác nhau trên một thiết bị trong 24 giờ (rule 90)
    - [device_night_activity] thiết bị đã có 1 chuyến trong giờ trước, và chuyến này đặt ban đêm (rule 70)
TN-0136703  fraud_type=promo_abuse  điểm 100.0  dải review
    - [promo_farm] mã NEWUSER50 đã dùng 3 lần trên thiết bị này trong 7 ngày (rule 95)
    - [device_many_accounts_24h] 3 tài khoản khách khác nhau trên một thiết bị trong 24 giờ (rule 90)
TN-0134529  fraud_type=gps_spoof  điểm 99.0  dải review
    - [gps_jump_impossible] tài xế nhảy 581 km giữa hai chuyến, tương đương 705 km/h (rule 90)
    - [speed_impossible] tốc độ trung bình trong chuyến 2028 km/h (rule 70)
    - [pickup_far] GPS tài xế lúc bắt đầu cách điểm đón khách khai báo 579.4 km (rule 65)
TN-0133533  fraud_type=none  điểm 70.0  dải review
    - [device_night_activity] thiết bị đã có 1 chuyến trong giờ trước, và chuyến này đặt ban đêm (rule 70)
TN-0133408  fraud_type=collusion  điểm 0.0  dải none
    - (không rule nào bắn: engine không có gì để nói)
```

Chuyến `gps_spoof` có ba dòng cùng kể một sự việc từ ba góc; chuyến `TN-0133533` vào hàng đợi chỉ vì một dòng, một bằng chứng mỏng, nhưng rule đó có precision 0,725 ở train nên không vô hại; chuyến `collusion` điểm 0 nhắc rằng lý do trống không phải bằng chứng vô tội. Lý do của rule nói **luật nào bắn**, không chứng minh vì sao đó là gian lận, và khi nhiều rule nhìn cùng một dấu vết (bốn trong sáu lý do của chuyến đầu đều về một thiết bị) danh sách dài hơn thông tin của nó: giao diện duyệt chỉ cần hiện ba lý do đầu. Giải thích điểm của mô hình học là việc của SHAP, bài 61.

Rule cần kiểm thử như mã, nhất là biên. `tests/test_rules.py` có hai tầng. Tầng một sinh một ca cho **mỗi điều kiện của mỗi rule** bằng `pytest.mark.parametrize` (như bài 10): mọi điều kiện ở mép trong thì rule phải bắn, đẩy riêng một điều kiện ra mép ngoài thì phải im.

```python
@pytest.mark.parametrize(("spec", "cond"), BOUNDARY_CASES, ids=...)
def test_each_condition_flips_exactly_at_its_threshold(spec, cond):
    rule = build_rule(spec)
    assert rule(features_with(spec.when)) is not None          # mọi điều kiện thỏa: bắn
    assert rule(features_with(spec.when, flip=cond)) is None   # riêng điều kiện này qua biên: im
```

Tầng này có điểm mù: ca sinh từ **chính cấu hình**, nên không kiểm ngưỡng đã chốt. Tầng hai là bảng ca vàng viết tay, độc lập với YAML: điểm, một ca «phải bắn», một ca «phải im» cho **từng điều kiện**, và dải khi rule đứng một mình.

```python
GOLDEN = {   # điểm, ca phải bắn, một ca im cho mỗi điều kiện, dải khi đứng một mình (review_at 40, watch_at 15)
    "gps_jump_impossible": (90, {"drv_jump_speed_kmh": 121.0, "drv_jump_km": 5.0},
                            [{"drv_jump_speed_kmh": 120.0}, {"drv_jump_km": 4.9}], "review"),
    "pair_repeat": (15, {"pair_trips_7d": 3.0}, [{"pair_trips_7d": 2.0}], "watch"),
    # ... đủ 11 rule
}
```

Ba lỗi gõ nhầm trên bản sao YAML, chạy cùng bộ test qua biến môi trường `RULES_YAML`: `>= 3` thành `>= 4` ở `device_many_accounts_24h`; điểm của `gps_jump_impossible` từ 90 xuống 85; `is_night == 1` thành `is_night == 0`. Dòng cuối của mỗi lần:

```text
2 failed, 81 passed in 11.66s    # >= 4: ca vàng device_many_accounts_24h và một test viết tay dùng giá trị 3
1 failed, 82 passed in 6.23s     # 90 -> 85: ca vàng gps_jump_impossible (điểm)
1 failed, 82 passed in 6.81s     # is_night == 0: ca vàng device_night_activity
```

Ba lần đều chỉ ca vàng (và một test viết tay) đỏ; tầng một vẫn xanh.

## Khi rule đủ, khi cần học máy

Lab huấn luyện `HistGradientBoostingClassifier` (tối đa 150 vòng, dừng sớm ở 84 theo `n_iter_` bằng 10 % train tách ngẫu nhiên; 30 đặc trưng) trên cùng train. Trước hết ở cùng công suất người duyệt: dải review của engine gắn cờ 999 chuyến (1,5 %), mô hình lấy đúng 999 chuyến điểm cao nhất.

```text
PR-AUC trên valid: rule engine 0.565, mô hình 0.886 (mốc cơ sở 0.0171)

cùng công suất người duyệt, toàn valid: dải review (điểm >= 40) gắn cờ 999 chuyến (1.5%); mô hình lấy 999 chuyến điểm cao nhất
               rule engine   mô hình
bắt được               667       859   (trong 1,141 chuyến gian lận)
precision            0.668     0.860
recall theo loại:
  ghost_trip         62.4%     84.7%   (439 chuyến)
  gps_spoof          75.0%     82.4%   (204 chuyến)
  promo_abuse        57.7%     59.8%   (343 chuyến)
  collusion          27.1%     73.5%   (155 chuyến)
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 350" role="img" aria-label="Recall theo loại gian lận khi cả hai bên gắn cờ cùng 999 chuyến trên 66.704 chuyến valid. Rule engine và mô hình học: ghost_trip 62,4 và 84,7 phần trăm; gps_spoof 75,0 và 82,4; promo_abuse 57,7 và 59,8; collusion 27,1 và 73,5."><rect x="260" y="14" width="14" height="14" class="f-phys-soft s-phys"/><text x="282" y="26" class="t-small">rule engine, 11 rule</text><rect x="520" y="14" width="14" height="14" class="f-pk-soft s-pk"/><text x="542" y="26" class="t-small">mô hình học, 30 đặc trưng</text><text x="20" y="78" class="t-small">ghost_trip (439)</text><rect x="200" y="52" width="262" height="18" rx="3" class="f-phys-soft s-phys"/><text x="470" y="65" class="t-small">62,4 %</text><rect x="200" y="76" width="356" height="18" rx="3" class="f-pk-soft s-pk"/><text x="564" y="89" class="t-small">84,7 %</text><text x="20" y="148" class="t-small">gps_spoof (204)</text><rect x="200" y="122" width="315" height="18" rx="3" class="f-phys-soft s-phys"/><text x="523" y="135" class="t-small">75,0 %</text><rect x="200" y="146" width="346" height="18" rx="3" class="f-pk-soft s-pk"/><text x="554" y="159" class="t-small">82,4 %</text><text x="20" y="218" class="t-small">promo_abuse (343)</text><rect x="200" y="192" width="242" height="18" rx="3" class="f-phys-soft s-phys"/><text x="450" y="205" class="t-small">57,7 %</text><rect x="200" y="216" width="251" height="18" rx="3" class="f-pk-soft s-pk"/><text x="459" y="229" class="t-small">59,8 %</text><text x="20" y="288" class="t-small">collusion (155)</text><rect x="200" y="262" width="114" height="18" rx="3" class="f-phys-soft s-phys"/><text x="322" y="275" class="t-small">27,1 %</text><rect x="200" y="286" width="309" height="18" rx="3" class="f-pk-soft s-pk"/><text x="517" y="299" class="t-small">73,5 %</text><text x="20" y="336" class="t-small">Cùng công suất người duyệt: 999 chuyến (1,5 %). Số trong ngoặc là số chuyến gian lận của loại đó.</text></svg>
</div>
<figcaption>Hình 2. Recall theo loại gian lận khi rule engine và mô hình học cùng gắn cờ 999 chuyến. Ở điểm vận hành này rule gần mô hình ở gps_spoof và promo_abuse, kém xa ở collusion.</figcaption>
</figure>

Đó mới là một điểm vận hành; ở cùng số chuyến sạch bị gắn cờ nhầm thì khác:

```text
cùng số chuyến sạch bị gắn cờ nhầm (FP = 332): rule engine bắt 667; mô hình gắn cờ 1,271 chuyến, bắt 939; recall theo loại: ghost_trip 87.7%, gps_spoof 87.7%, promo_abuse 70.0%, collusion 87.1%
PR-AUC theo loại (loại đó so với chuyến sạch; lúc chấm điểm hệ thống không biết loại):
  ghost_trip   rule engine 0.558  mô hình 0.873
  gps_spoof    rule engine 0.540  mô hình 0.787
  promo_abuse  rule engine 0.542  mô hình 0.719
  collusion    rule engine 0.050  mô hình 0.651
```

Với cùng 332 chuyến sạch bị xem nhầm, mô hình bắt 939 so với 667, và hơn ở **mọi** loại; khoảng cách ở `gps_spoof` và `promo_abuse` chỉ trông nhỏ ở công suất 999 chuyến. `collusion` là chỗ rule gần như vô dụng (PR-AUC 0,050 so với 0,651), và loại này chịu ảnh hưởng của tạo tác của bộ sinh (cuối bài). Chi phí trên nửa sau valid: 13,37 triệu VND cho engine, 8,59 triệu cho mô hình (gắn cờ 695, bắt 426) và 24,50 triệu nếu không làm gì.

Đổi từng biến để tách nguồn lợi thế: cùng mô hình nhưng chỉ cộng tính (`max_depth=1`) còn PR-AUC 0,785 so với 0,886, nên tương tác có giá trị thật; học trên đúng 11 cờ rule thay vì 30 đặc trưng chỉ đạt 0,580, nên phần lớn lợi thế đến từ chính các đặc trưng. Hồi quy logistic trên 11 cờ rule cũng cho 0,580 (trọng số tay: 0,565), và cho `speed_impossible` hệ số âm (-0,84) dù rule này đứng riêng có precision 0,746: trên train, cả 236 chuyến mà nó bắn đều bị `pickup_far` bắn (168 bị cả `gps_jump_impossible`). Bỏ từng rule trùng rồi hồi quy lại:

```text
  hệ số của speed_impossible khi bỏ không rule nào: -0.84
  hệ số của speed_impossible khi bỏ gps_jump_impossible: +0.68
  hệ số của speed_impossible khi bỏ pickup_far: +3.53
  hệ số của speed_impossible khi bỏ gps_jump_impossible, pickup_far: +5.65
```

Dấu âm đến từ cặp trùng chứ không phải bản thân rule, nên trọng số học không còn đọc được thành «rule này đáng nghi bao nhiêu» (một seed). Đường chấm từng chuyến và đường theo lô (polars) dùng chung định nghĩa điều kiện, và lab so chúng trên 66.704 chuyến valid: chênh lớn nhất 0 với `sum`, `max`, 1,42e-14 với `noisy_or`. Thời gian chấm từng chuyến chỉ để biết cỡ (100 micro giây mỗi chuyến, load average 422) và chưa tính việc lấy đặc trưng, phần chiếm gần hết ngân sách ở bài 59.

## Chạy và kiểm chứng

Không cần Docker. Cần uv và Python 3.14; lần đầu cần mạng và dựng đặc trưng từ event (lưu `out/table.parquet`). Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/56-rule-engine-scoring examples/toi-noi-python/56-rule-engine-scoring/main.py
uv run --project examples/toi-noi-python/56-rule-engine-scoring examples/toi-noi-python/56-rule-engine-scoring/main.py --verify
uv run --project examples/toi-noi-python/56-rule-engine-scoring pytest examples/toi-noi-python/56-rule-engine-scoring -q
```

Dòng cuối của `--verify` và của `pytest` (bước huấn luyện mô hình để so sánh là phần lâu nhất, 342 giây trong lần chạy này, khi load average lên tới 429):

```text
Kiểm chứng: OK 50/50
83 passed in 3.58s
```

`--verify` kiểm 50 bất biến, viết bằng biên chứ không ghim số phụ thuộc seed: `points` không vượt 100 lần precision train quá 10 điểm và không quá 95; càng nhiều rule cùng bắn càng chắc gian lận; ba aggregator trùng nhau ở chuyến có đúng một rule bắn; engine rẻ hơn không làm gì; mô hình học thắng engine với biên lớn, cả ở cùng FP; cấu hình hỏng, khóa trùng và thẻ YAML nguy hiểm bị từ chối; hai đường Python và polars khớp. Có ca sai thì lab in tên ca, `FAIL n/m` và thoát với mã 1.

| Hiện tượng | Kiểm tra trước |
|---|---|
| Rule dựa vào «khách mới» bắn nhiều bất thường ở những tuần đầu của dữ liệu | Quần thể khách cố định làm «khách mới» cạn dần; đo precision trên giai đoạn sau, không chỉ train |
| `ConfigError` khi nạp YAML | Đọc cả danh sách lỗi: thường là tên đặc trưng gõ sai, khóa trùng, hoặc `reason` dùng đặc trưng không có trong `when` |
| `KeyError` tên đặc trưng khi chấm điểm | Lược đồ đặc trưng đã đổi tên cột; sửa nguồn đặc trưng hoặc cấu hình, đừng bắt rồi bỏ qua |
| Điểm theo lô khác điểm từng chuyến | So `None` và `NaN`: cả hai đường coi là không bắn; chạy lại ca so hai đường |
| Người duyệt kêu lý do trùng lặp | Nhiều rule cùng nhìn một dấu vết; hiện ba lý do đầu hoặc gộp rule theo nhóm |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Dữ liệu là tổng hợp (200.000 chuyến trong 60 ngày) và có hai tạo tác của bộ sinh ảnh hưởng mọi valid của phần 07: chuyến sạch dùng <code>NEWUSER50</code> giảm từ 7,8 % xuống 0,9 % theo thời gian vì quần thể khách gần như cố định, và tài khoản gian lận không dùng mã nào ngoài <code>NEWUSER50</code> của <code>promo_abuse</code>, nên <code>cust_promo_share_30d</code> lớn hơn 0 là dấu hiệu «sạch», một lợi thế giả cho mô hình học. Ngưỡng và điểm trong <code>rules.yaml</code> khớp với bộ sinh, nên precision 1,000 của một số rule và recall 58,5 % của cả engine không đại diện cho dữ liệu thật; người gian lận thật đổi hành vi khi biết luật. Số trong bài là của seed 42; <code>--verify</code> dùng biên và cũng đạt khi thay <code>SEED</code> trong <code>prepare.py</code> bằng 1 hoặc 2 (kiểm một lần). Chi phí 8.000 và 20.000 VND là giả định của bài 54, kèm hai giả định ở mục Strategy; <code>review_at: 40</code> chọn trên nửa đầu valid; nhãn đặt điểm là nhãn cuối cùng. Engine nạp cấu hình một lần lúc khởi động; production cần phiên bản cho cấu hình, review khi sửa <code>rules.yaml</code> như sửa mã và nhật ký quyết định (bài 61) ghi phiên bản đó. Đã chạy với Python 3.14.7, polars 2.1.0, numpy 2.5.4, scikit-learn 1.9.1, PyYAML 6.0.3 và pytest 9.2.0.</p>
</div>

**Bài tập:** (1) Với `review_at: 40` và `watch_at: 15`, hai chuyến có các rule bắn như sau: chuyến A gồm `trip_not_moving` (95) và `driver_device_booking` (40); chuyến B gồm `pair_repeat` (15) và `route_short_repeat` (35). Tính điểm và dải của từng chuyến theo ba aggregator. (2) Rule `dev_customers_24h >= 2` có precision 0,587 trên train (bảng ở mục «Đo»). Theo quy ước làm tròn xuống bội 5, `points` là bao nhiêu? Verify còn xanh đến mức điểm nào?

<details><summary>Xem lời giải</summary><p>(1) Chuyến A: <code>sum</code> = min(100, 135) = <b>100</b>, <code>max</code> = <b>95</b>, <code>noisy_or</code> = 100 × (1 - 0,05 × 0,60) = <b>97</b>; cả ba vào dải <code>review</code>. Chuyến B: <code>sum</code> = <b>50</b> (review), <code>max</code> = <b>35</b> (watch), <code>noisy_or</code> = 100 × (1 - 0,85 × 0,65) = <b>44,75</b> (review). (2) 100 × 0,587 = 58,7, làm tròn xuống bội 5 cho <b>55</b>. Verify cho phép vượt precision tối đa 10 điểm, tức đến 68,7: đặt 60 vẫn xanh, đặt 70 thì ca về biên 10 điểm đỏ và <code>--verify</code> thoát với mã 1.</p></details>

Bài trước: [bài 55](/articles/python-feature-engineering-tu-event-kafka). Bài tiếp: [bài 57](/articles/python-scikit-learn-lightgbm-imbalanced-cross-validation).

## Tài liệu gốc

- [PyYAML: documentation](https://pyyaml.org/wiki/PyYAMLDocumentation).
- [Python: string.Formatter](https://docs.python.org/3.14/library/string.html#string.Formatter).
- [pytest: How to parametrize fixtures and test functions](https://docs.pytest.org/en/stable/how-to/parametrize.html).
- [PEP 695: Type Parameter Syntax (câu lệnh type)](https://peps.python.org/pep-0695/).
- [Polars: all_horizontal](https://docs.pola.rs/api/python/stable/reference/expressions/api/polars.all_horizontal.html).
