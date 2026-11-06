---
slug: location-hoc-diem-don-tu-lich-su-dbscan-python
title: "Học điểm đón từ lịch sử bằng DBSCAN: vòng tròn và kết quả rỗng"
summary: "Bài 39: DBSCAN trên đơn vị mét bằng Python và uv, xuất bảng hub cho Rust. Bảng học được không làm J tốt hơn (km rỗng −0,4 m [−1,5; +0,7]) vì lịch sử là vòng tròn và thưa. Trên dữ liệu mô phỏng, chỉ chứng minh phương pháp."
category: system-design
level: advanced
tags: [toi-noi, location, python, uv, dbscan, clustering, rust, simulation, bias]
language: vi
---

Một người vận hành nhìn bản đồ nhiệt của các điểm đón trong tháng và thấy vài đốm đỏ: nhiều khách hay được đón ở cùng một cổng, cùng một đoạn lề. Ý tưởng tự nhiên là biến các đốm đó thành một **bảng điểm đón** và đưa cho bộ chọn điểm đón như danh sách ứng viên đáng tin: nơi mà tài xế thật sự đã dừng, dễ đỗ, khách quen. Phân cụm theo mật độ là công cụ hiển nhiên cho việc này, và nó chạy tốt bằng Python trong vài chục dòng. Nhưng đây là bài phải dè chừng nhất trong cả phần, vì dữ liệu lịch sử của chúng ta không đến từ đời thật. Nó đến từ simulator.

Bài này làm ba việc. Dựng đường ống: xuất lịch sử điểm đón từ simulator, chạy DBSCAN bằng Python trong môi trường `uv`, đưa bảng hub về Rust. Đo xem bảng đó có làm hàm mục tiêu `J` tốt hơn không. Và, quan trọng nhất, nói rõ **vòng tròn** của việc học từ dữ liệu do chính mình sinh ra. Kết quả đo là **rỗng**: bảng hub không làm `J` tốt hơn. Phần lớn bài là giải thích vì sao, và việc đó dạy được nhiều hơn một con số đẹp.

Đây là bài 39 của lộ trình **Tối ưu quãng đường hai phía** qua [Tới Nơi](/articles/du-an-mau-toi-noi). Bài dựng trên bộ sinh ứng viên của [bài 33](/articles/location-sinh-ung-vien-diem-don-do-thi-di-bo), khoảng cách theo mét của [bài 04](/articles/location-khoang-cach-haversine-equirectangular-sai-so) và việc lưu lịch sử vị trí của [bài 20](/articles/location-lich-su-gps-parquet-retention-quyen-rieng-tu). So sánh cặp theo 20 seed đánh giá (20270001…20270020, khoảng tin cậy t 95%) như đã định nghĩa ở [bài 32](/articles/location-ham-muc-tieu-diem-hen-khach-va-tai-xe). Mọi số trong bài là **trên dữ liệu mô phỏng**; simulator không dự đoán thế giới thật, và bài này chỉ chứng minh **phương pháp**, không chứng minh hiệu quả ngoài đời.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn chạy DBSCAN trên 7.546 điểm đón lịch sử của baseline, được 62 hub, rồi thêm các hub đó làm ứng viên cho <code>J</code>. Trên 20 seed đánh giá, theo bạn km rỗng giảm bao nhiêu so với <code>J</code> không có hub? Và cụm học được có «khám phá» ra điểm đón nổi tiếng nào không?</p>
<details><summary>Xem đáp án</summary><p>Km rỗng đổi <b>−0,4 m [−1,5; +0,7]</b>: không khác biệt rõ, và mọi chỉ số khác cũng không. Hub chỉ có mặt trong tập ứng viên của 5,5% yêu cầu. Về «khám phá»: không. Cả 62 hub đều nằm trong 2σ của một điểm nóng đặt sẵn trong kịch bản, nên cụm học được là vùng nhu cầu đã biết, không phải điểm đón mới.</p></details>
</div>

## Dữ liệu vào: CSV điểm đón, không phải Parquet

Hai chi tiết về dữ liệu cần nói trước, vì người đọc loạt bài này sẽ hỏi.

**Lịch sử là CSV do simulator xuất, không phải Parquet.** [Bài 20](/articles/location-lich-su-gps-parquet-retention-quyen-rieng-tu) thiết kế lịch sử GPS lưu dạng Parquet. Bài này **không** dùng nó, vì hợp đồng Parquet của dự án (`proto/location/v1/gps-history-parquet.md`) mô tả điểm GPS lấy mẫu của tài xế, không có nhãn «đây là điểm đón». Thứ DBSCAN cần là **điểm dừng đón**: nơi một chuyến hoàn thành đã đón khách. Lab Rust xuất đúng thứ đó ra CSV với cột `seed, customer, lat, lng, pin_lat, pin_lng, outcome`. Trong thực tế bạn sẽ dựng điểm đón từ sự kiện chuyến (trạng thái «đã đón») ghép với vị trí tài xế lúc đó, có thể từ Parquet, và đó là việc của pipeline Python ở phần P06.

**Lịch sử có đúng 40 seed riêng.** Seed 20260301…20260340, không giao với dải chỉnh (20261201…20261210) và dải đánh giá (20270001…20270020). Lab chạy baseline (đón đúng ghim, tài xế gần nhất) trên 40 seed, được 8.723 dòng, trong đó 7.546 chuyến hoàn thành. Chỉ chuyến hoàn thành mới được học, vì ở chuyến hủy khách chưa thực sự được đón tại điểm đó.

## DBSCAN trên đơn vị mét

DBSCAN (Ester, Kriegel, Sander, Xu, KDD-96) gom các điểm theo mật độ: một điểm là **lõi** nếu có ít nhất `min_samples` điểm (kể cả nó) trong bán kính `eps`; các điểm lõi gần nhau nối thành cụm; điểm không thuộc cụm nào là **nhiễu**. Khác với k-means, nó không cần biết trước số cụm và để lại nhiễu. Với điểm đón, hai tính chất đó hợp lý: ta không biết có bao nhiêu điểm đón nổi tiếng, và phần lớn điểm đón chỉ là «khách đứng đâu đó».

Quan trọng là **đơn vị**. `eps` phải tính bằng mét; nếu đưa độ vĩ độ, kinh độ thẳng vào, `eps` sẽ lệch theo hướng vì một độ kinh ngắn hơn một độ vĩ theo cos(vĩ độ). Script chiếu equirectangular quanh trọng tâm (đủ chính xác trong bbox một thành phố, như bài 04), rồi chạy DBSCAN trên mét.

`examples/toi-noi-location/python/loc39/loc39_dbscan.py`

```python
def to_metres(lat: np.ndarray, lng: np.ndarray, lat0: float, lng0: float) -> np.ndarray:
    """Chiếu equirectangular quanh (lat0, lng0): x về đông, y về bắc, đơn vị mét. Đủ chính xác trong bbox một thành phố."""
    x = np.radians(lng - lng0) * math.cos(math.radians(lat0)) * R_EARTH
    y = np.radians(lat - lat0) * R_EARTH
    return np.column_stack([x, y])
```

Mỗi cụm cho một **hub**. Script chọn hub là **điểm thật của cụm gần tâm nhất** chứ không phải tâm trung bình, để hub luôn là một nơi đã có người đón (tâm trung bình có thể rơi vào giữa nhà):

`examples/toi-noi-location/python/loc39/loc39_dbscan.py`

```python
def fit_hubs(xy: np.ndarray, eps: float, min_samples: int) -> tuple[np.ndarray, np.ndarray]:
    """DBSCAN trên mét. Mỗi cụm cho một hub là ĐIỂM THẬT của cụm gần tâm nhất (medoid xấp xỉ), để hub luôn là nơi đã có người đón."""
    labels = DBSCAN(eps=eps, min_samples=min_samples).fit_predict(xy)
    hubs, counts = [], []
    for k in sorted(set(labels) - {-1}):
        pts = xy[labels == k]
        c = pts.mean(axis=0)
        hubs.append(pts[np.argmin(((pts - c) ** 2).sum(axis=1))])
        counts.append(len(pts))
    return (np.array(hubs) if hubs else np.zeros((0, 2))), np.array(counts)
```

## Chọn tham số: một quy tắc đặt trước, một cái so ngẫu nhiên

Chọn `eps` và `min_samples` bằng mắt là cách dễ nhất để «chỉnh tới khi đẹp». Script làm khác: chia 40 seed thành 30 seed huấn luyện và 10 seed kiểm định **theo seed, không xáo trộn**; học hub trên huấn luyện; chấm trên kiểm định bằng **độ phủ** (tỉ lệ điểm đón kiểm định cách một hub không quá 100 m); và so với một baseline ngẫu nhiên có cùng số hub chọn trong điểm đón huấn luyện. Quy tắc chọn được đặt trước: trong các cấu hình có từ 5 đến 150 hub và trung bình từ 10 điểm mỗi hub, chọn cấu hình có độ phủ **vượt** ngẫu nhiên lớn nhất; hòa thì ít hub hơn.

Để chọn lưới, script in khoảng cách từ mỗi điểm tới láng giềng thứ k (trên tập huấn luyện):

| k | p25 (m) | p50 (m) | p75 (m) | p90 (m) |
|---:|---:|---:|---:|---:|
| 5 | 115,0 | 171,4 | 292,4 | 482,6 |
| 10 | 173,8 | 245,2 | 429,7 | 686,0 |
| 20 | 251,3 | 348,0 | 602,0 | 983,7 |

Nguồn: `results/p04.md`, LOC-39. Láng giềng thứ 10 cách trung vị tới 245 m: điểm đón rất thưa so với mật độ mà «điểm đón nổi tiếng» cấp vài chục mét đòi hỏi. Một phần lưới quét (số hub, độ phủ so với ngẫu nhiên) cho thấy điều đó:

| `eps` (m) | `min_samples` | Số hub | Phủ | Ngẫu nhiên | Vượt | Ghi chú |
|---:|---:|---:|---:|---:|---:|---|
| 50 | 5 | 32 | 0,024 | 0,013 | +0,011 | loại: dưới 10 điểm mỗi hub |
| 100 | 5 | 154 | 0,083 | 0,060 | +0,023 | loại: quá nhiều hub |
| **100** | **10** | **31** | **0,032** | **0,012** | **+0,020** | **được chọn** |
| 150 | 10 | 47 | 0,027 | 0,025 | +0,003 | |
| 250 | 20 | 12 | 0,007 | 0,004 | +0,003 | |
| 400 | 40 | 5 | 0,002 | 0,002 | +0,000 | |

Nguồn: `results/p04.md`, LOC-39, quét tham số trên 24 cấu hình (sáu dòng được trích), chấm trên 10 seed kiểm định, trên dữ liệu mô phỏng. Cấu hình chọn theo quy tắc là `eps` = 100 m, `min_samples` = 10, cho 31 hub trên tập huấn luyện. Độ phủ của hub chỉ **3,2%** điểm đón kiểm định, so với 1,2% khi chọn hub ngẫu nhiên, và cả hai đều rất thấp: hầu hết điểm đón mới không rơi cạnh hub nào. Chênh +0,020 lại được chọn là lớn nhất trong 24 cấu hình trên chính tập kiểm định và so với một lần chọn ngẫu nhiên, nên nó lạc quan; không nên đọc là bằng chứng chắc rằng học hơn ngẫu nhiên. Hồ sơ cuối cùng, học trên cả 40 seed, có 62 hub và 84% điểm đón là nhiễu.

## Vòng tròn: học lại cái mình đã đặt

Đây là phần cốt lõi của bài.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 320" role="img" aria-label="Hình 1. Hai vòng tròn của việc học điểm đón từ lịch sử. Vòng thứ nhất đã đo: tám điểm nóng đặt tay sinh ra 7.546 điểm đón lịch sử, DBSCAN tìm ra 62 hub và cả 62 hub nằm trong 2σ của một điểm nóng đã đặt, tính bằng σ riêng của điểm nóng đó. Vòng thứ hai là cơ chế: nếu bảng hub được dùng thật thì nơi đón đổi, lịch sử đổi và bảng học lại chính nó; lab chưa đo vòng này. Đã đo: 541 trên 7.546 điểm đón lịch sử nằm ở chỗ ràng buộc cứng của J sẽ loại, và 3 trên 62 hub học được nằm trên cạnh bị cấm."><text x="36" y="28" class="t-b">Học từ lịch sử mô phỏng: đường đi của dữ liệu và hai vòng tròn</text><rect x="20" y="44" width="150" height="72" rx="8" class="f-surface s-line" stroke-width="1.5"/><text x="95" y="66" class="t-b" text-anchor="middle">8 điểm nóng</text><text x="95" y="84" class="t-small" text-anchor="middle">đặt tay trong</text><text x="95" y="102" class="t-small" text-anchor="middle">kịch bản TOML</text><path d="M173 80 L183.0 80.0" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><path d="M192 80 L183.0 84.5 L183.0 75.5 z" class="f-muted"/><rect x="195" y="44" width="150" height="72" rx="8" class="f-surface s-line" stroke-width="1.5"/><text x="270" y="66" class="t-b" text-anchor="middle">Simulator sinh</text><text x="270" y="84" class="t-small" text-anchor="middle">khách quanh</text><text x="270" y="102" class="t-small" text-anchor="middle">điểm nóng (Gauss)</text><path d="M348 80 L358.0 80.0" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><path d="M367 80 L358.0 84.5 L358.0 75.5 z" class="f-muted"/><rect x="370" y="44" width="150" height="72" rx="8" class="f-surface s-line" stroke-width="1.5"/><text x="445" y="66" class="t-b" text-anchor="middle">Baseline đón ở ghim</text><text x="445" y="84" class="t-small" text-anchor="middle">7.546 điểm đón</text><text x="445" y="102" class="t-small" text-anchor="middle">40 seed lịch sử</text><path d="M523 80 L533.0 80.0" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><path d="M542 80 L533.0 84.5 L533.0 75.5 z" class="f-muted"/><rect x="545" y="44" width="150" height="72" rx="8" class="f-surface s-line" stroke-width="1.5"/><text x="620" y="66" class="t-b" text-anchor="middle">DBSCAN trên mét</text><text x="620" y="84" class="t-small" text-anchor="middle">eps 100 m, min 10</text><text x="620" y="102" class="t-small" text-anchor="middle">→ 62 hub</text><path d="M698 80 L708.0 80.0" class="f-none s-muted" stroke-width="1.5" stroke-linecap="round"/><path d="M717 80 L708.0 84.5 L708.0 75.5 z" class="f-muted"/><rect x="720" y="44" width="150" height="72" rx="8" class="f-surface s-line" stroke-width="1.5"/><text x="795" y="66" class="t-b" text-anchor="middle">Bảng hub</text><text x="795" y="84" class="t-small" text-anchor="middle">thêm làm ứng viên</text><text x="795" y="102" class="t-small" text-anchor="middle">(snap ≤ 30 m)</text><path d="M795 120 L795 176" class="f-none s-warn dash" stroke-width="1.5" stroke-linecap="round"/><path d="M795 176 L445 176" class="f-none s-warn dash" stroke-width="1.5" stroke-linecap="round"/><path d="M445 176 L445.0 131.0" class="f-none s-warn" stroke-width="1.5" stroke-linecap="round"/><path d="M445 122 L449.5 131.0 L440.5 131.0 z" class="f-warn"/><text x="620" y="196" class="t-small" text-anchor="middle">vòng 2 (cơ chế, chưa đo): bảng đổi nơi đón → lịch sử đổi → bảng học lại</text><rect x="20" y="214" width="420" height="78" rx="8" class="f-pk-soft s-pk" stroke-width="1.5"/><text x="34" y="236" class="t-b">Đã đo, vòng 1: tìm lại cái đã đặt</text><text x="34" y="256" class="t-small">62/62 hub trong 2σ (σ riêng) của một điểm nóng; 52 trong 1σ</text><text x="34" y="274" class="t-small">cụm học được là vùng nhu cầu cho trước, không phải khám phá</text><rect x="460" y="214" width="400" height="78" rx="8" class="f-warn-soft s-warn" stroke-width="1.5"/><text x="474" y="236" class="t-b">Đã đo: học theo nơi baseline đã đón</text><text x="474" y="256" class="t-small">541/7.546 điểm đón lịch sử (7,2%) ở chỗ J sẽ loại</text><text x="474" y="274" class="t-small">3/62 hub học được cũng nằm trên cạnh bị cấm</text><text x="36" y="312" class="t-small">Trên dữ liệu mô phỏng. Số lấy từ results/p04.md, mục LOC-39.</text></svg>
</div>
<figcaption>Hình 1. Đường đi của dữ liệu và hai vòng tròn. Vòng 1 đã đo: tám điểm nóng đặt tay sinh ra lịch sử, DBSCAN tìm lại đúng vùng đó. Vòng 2 là cơ chế, lab chưa đo: nếu bảng hub được dùng thật thì nó đổi nơi tài xế đón, và lần học sau học lại chính nó. Trên dữ liệu mô phỏng.</figcaption>
</figure>

**Vòng 1: lịch sử sinh ra từ cái ta đã đặt.** Simulator sinh khách từ tám điểm nóng đặt sẵn trong `scenario-hcm-baseline.toml`; vị trí đứng của khách là Gauss quanh điểm nóng với độ lệch σ 700–1.000 m. Lịch sử điểm đón là hệ quả của kịch bản đó. Khi DBSCAN tìm ra «vùng đông điểm đón», nó tìm lại **chính các điểm nóng ta đã đặt**. Script in khoảng cách từ mỗi điểm nóng tới hub học được gần nhất:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 372" role="img" aria-label="Hình 2. Khoảng cách từ mỗi điểm nóng đã đặt tới hub học được gần nhất, mét: Chợ Bến Thành 64; Tân Sơn Nhất 151; Landmark 81 65; Ngã tư Hàng Xanh 117; Phú Mỹ Hưng 422; Đại học Bách Khoa 136; Gò Vấp 2.346; Thảo Điền 1.103. Sáu trong tám điểm nóng có hub gần nhất trong 422 mét; Gò Vấp, một trong hai điểm nóng trọng số thấp nhất, không có hub trong 2σ."><text x="36" y="28" class="t-b">Khoảng cách từ điểm nóng đã đặt tới hub học được gần nhất (m)</text><path d="M220 44 L220 316" class="f-none s-line" stroke-width="1" stroke-linecap="round"/><text x="640" y="60" class="t-small">khoảng cách · số hub trong 2σ · σ</text><text x="36" y="84">Chợ Bến Thành</text><rect x="220" y="70" width="10.3666" height="18" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><path d="M511.56 67 L511.56 91" class="f-none s-muted dash" stroke-width="1.5" stroke-linecap="round"/><text x="640" y="84" class="t-mono">64 m · 20 hub · σ 900 m</text><text x="36" y="114">Sân bay Tân Sơn Nhất</text><rect x="220" y="100" width="24.4587" height="18" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><path d="M446.769 97 L446.769 121" class="f-none s-muted dash" stroke-width="1.5" stroke-linecap="round"/><text x="640" y="114" class="t-mono">151 m · 5 hub · σ 700 m</text><text x="36" y="144">Landmark 81</text><rect x="220" y="130" width="10.5286" height="18" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><path d="M511.56 127 L511.56 151" class="f-none s-muted dash" stroke-width="1.5" stroke-linecap="round"/><text x="640" y="144" class="t-mono">65 m · 24 hub · σ 900 m</text><text x="36" y="174">Ngã tư Hàng Xanh</text><rect x="220" y="160" width="18.9514" height="18" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><path d="M479.165 157 L479.165 181" class="f-none s-muted dash" stroke-width="1.5" stroke-linecap="round"/><text x="640" y="174" class="t-mono">117 m · 21 hub · σ 800 m</text><text x="36" y="204">Phú Mỹ Hưng</text><rect x="220" y="190" width="68.3546" height="18" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><path d="M511.56 187 L511.56 211" class="f-none s-muted dash" stroke-width="1.5" stroke-linecap="round"/><text x="640" y="204" class="t-mono">422 m · 4 hub · σ 900 m</text><text x="36" y="234">Đại học Bách Khoa</text><rect x="220" y="220" width="22.029" height="18" rx="2" class="f-pk-soft s-pk" stroke-width="1.2"/><path d="M479.165 217 L479.165 241" class="f-none s-muted dash" stroke-width="1.5" stroke-linecap="round"/><text x="640" y="234" class="t-mono">136 m · 9 hub · σ 800 m</text><text x="36" y="264">Gò Vấp</text><rect x="220" y="250" width="380" height="18" rx="2" class="f-warn-soft s-warn" stroke-width="1.2"/><path d="M543.956 247 L543.956 271" class="f-none s-muted dash" stroke-width="1.5" stroke-linecap="round"/><text x="640" y="264" class="t-mono">2.346 m · 0 hub · σ 1.000 m</text><text x="36" y="294">Thảo Điền</text><rect x="220" y="280" width="178.662" height="18" rx="2" class="f-warn-soft s-warn" stroke-width="1.2"/><path d="M479.165 277 L479.165 301" class="f-none s-muted dash" stroke-width="1.5" stroke-linecap="round"/><text x="640" y="294" class="t-mono">1.103 m · 2 hub · σ 800 m</text><text x="36" y="340" class="t-small">Đường đứt đứng: 2σ của điểm nóng đó. Xanh: có từ 4 hub trong 2σ; cam: ít hơn.</text><text x="36" y="358" class="t-small">Trên dữ liệu mô phỏng, lịch sử 40 seed 20260301…20260340. Tên địa danh chỉ là nhãn của tâm nhu cầu.</text></svg>
</div>
<figcaption>Hình 2. Khoảng cách từ mỗi điểm nóng đã đặt tới hub học được gần nhất (mét), cùng số hub trong 2σ của điểm nóng đó. Đường đứt đứng đánh dấu 2σ. Sáu trong tám điểm nóng có hub gần nhất trong vòng 422 m; hai điểm nóng trọng số thấp nhất còn lại ít hoặc không có hub. Trên dữ liệu mô phỏng.</figcaption>
</figure>

Hình 2 đọc được hai điều. Các điểm nóng có trọng số cao (Chợ Bến Thành, Landmark 81: trọng số 3,0 và 2,0) có hub gần nhất chỉ 64–65 m và hàng chục hub quanh đó. Điểm nóng trọng số thấp, nhu cầu ít (Gò Vấp, trọng số 1,0) thì hub gần nhất cách 2.346 m, **không có hub nào** trong 2σ. DBSCAN không phát hiện «điểm đón mới», nó phát hiện **nơi simulator đã cho nhiều khách**. Script đo mỗi hub bằng σ riêng của điểm nóng: 62/62 hub nằm trong 2σ, 52 hub trong 1σ, xa nhất 1,5σ. Tám vòng 2σ chỉ phủ 18% diện tích bbox của lab, nên việc mọi hub rơi vào đó không phải chuyện ngẫu nhiên. Nhưng vòng 2σ vẫn rộng (bán kính 1.400–2.000 m): nó nói hub nằm trong vùng đã đặt, không nói hub trùng điểm nóng; bảng khoảng cách ở Hình 2 cho biết chi tiết đó. Cụm học được là vùng nhu cầu cho trước, không phải khám phá.

**Không có «điểm đón nổi tiếng» để học.** Vì khách đứng ngẫu nhiên và trơn quanh điểm nóng, mật độ điểm đón không có đỉnh nhọn cấp vài chục mét. Với `eps` từ 50 m trở xuống, chỉ `min_samples` = 5 còn cho cụm (32 hub ở `eps` 50 m, 3 hub ở 25 m), mỗi hub trung bình chưa tới 6 điểm; các `min_samples` lớn hơn cho 0 hub. Ngoài đời, những nơi như cổng một tòa nhà hay miệng hẻm thật sự có đỉnh nhọn. Mô phỏng này không có, nên nó không thể cho thấy DBSCAN học được gì từ chúng.

**Vòng 2: học từ nơi tài xế đã đón sẽ củng cố thiên lệch.** Đây là vòng mà lab **không** đo trực tiếp, nhưng nó là rủi ro chính khi đem phương pháp này ra thực tế. Lịch sử điểm đón là nơi tài xế **đã** dừng, tức nơi mà hệ thống hiện tại (ở đây là baseline, đón ở ghim đã snap) đã đưa họ tới. Nếu bảng học được được đưa lại vào bộ chọn điểm đón và bộ chọn thiên về các hub, tài xế sẽ đón nhiều hơn ở các hub; lần học sau, các hub đó lại đông hơn. Nơi chưa ai từng đón không bao giờ vào bảng, kể cả khi nó tốt hơn. Cái bạn học được là thói quen của hệ thống cũ, không phải điểm đón tốt.

Lab đo được **một nửa** của thiên lệch đó, ở bước học (chưa có vòng phản hồi):

- Trong 7.546 điểm đón lịch sử của baseline, **541 (7,2%)** nằm ở chỗ mà ràng buộc cứng của `J` sẽ loại: 355 trên cạnh quá ngắn, 137 trên cầu, hầm hay cầu vượt, 22 trên vòng xuyến, 27 trên đường cấm dừng. Lý do: baseline snap ghim lên đường gần nhất bất kể luật dừng.
- Trong 62 hub học được, **3 hub (4,8%)** nằm trên cạnh vi phạm ràng buộc cứng. Bảng học thẳng từ lịch sử học luôn cả các chỗ đó.

Lab chặn rủi ro này theo một cách đơn giản: hub chỉ được thêm làm **ứng viên** và vẫn phải qua ràng buộc cứng. Đoạn mã dưới đây là bước thêm hub trong bộ xếp hạng:

`examples/toi-noi-location/rust/crates/pickup/src/rank.rs`

```rust
        if !self.hubs.is_empty() {
            let view = engine.view(self.cfg.profile);
            for h in &self.hubs {
                if haversine_m(*h, req.from) > self.weights.max_walk_m {
                    continue;
                }
                let Ok(sn) = engine.snap_point(self.cfg.profile, *h, "điểm học được")
                else {
                    continue;
                };
                if sn.snap_dist_m > 30.0 {
                    continue;
                }
                let walk = Self::walk_m(engine, req.from, sn.point);
                if walk > self.weights.max_walk_m {
                    continue;
                }
                for p in sn.positions(view) {
                    if crate::candidates::stop_violation(engine, p.edge, 20.0).is_some() {
                        continue;
                    }
                    if elig
                        .iter()
                        .any(|c| c.pos.edge == p.edge && haversine_m(c.point, sn.point) < 10.0)
                    {
                        continue;
                    }
```

Hub bị bỏ nếu quá xa khách, không snap được trong 30 m, vi phạm ràng buộc cứng, hoặc trùng một ứng viên đã có trong 10 m. Điều đó trả lời câu hỏi «hub có thể đưa khách vào chỗ cấm dừng không»: không, trong lab. Nó cũng giải thích kết quả rỗng ở mục sau.

## Bảng học được có làm J tốt hơn không: không

Cách đo là so cặp ba lần trên cùng 20 seed đánh giá: `J` không hub so với baseline; `J` có hub so với baseline; và quan trọng nhất **`J` có hub so với `J` không hub**, tức tác dụng riêng của bảng.

| Chỉ số | `J` không hub | `J` có hub | Chênh cặp [KTC 95% t] |
|---|---:|---:|---:|
| `walk_m` trung bình | 68,2 | 68,2 | −0,0 [−0,1; +0,1] |
| `deadhead_m` trung bình | 2.250,0 | 2.249,6 | −0,4 [−1,5; +0,7] |
| `deadhead_m` p95 | 5.599,7 | 5.599,7 | +0,0 [+0,0; +0,0] |
| `pickup_eta_s` trung bình | 307,0 | 307,0 | −0,1 [−0,3; +0,1] |
| `meet_gap_s` trung bình | 264,2 | 264,2 | −0,0 [−0,2; +0,2] |
| `cancel_rate` | 0,1131 | 0,1131 | +0,0000 [+0,0000; +0,0000] |
| Cước trung bình (đồng) | 33.489,4 | 33.489,2 | −0,2 [−1,1; +0,7] |

Nguồn: `results/p04.md`, LOC-39 (3). Mọi khoảng tin cậy chứa 0 hoặc bằng 0; không chỉ số nào trong bảng đầy đủ (19 chỉ số) có khoảng loại 0. Trên dữ liệu mô phỏng.

Hub có mặt trong tập ứng viên của **5,5%** yêu cầu, trung bình 0,10 hub thêm cho mỗi yêu cầu. Vì sao ít vậy? Bộ sinh ứng viên của bài 33 đã **liệt kê mọi điểm dừng hợp lệ trong tầm đi bộ** của khách. Hub là một điểm dừng hợp lệ, nên thường đã có sẵn một ứng viên trong 10 m. Hub chỉ có ích khi nó mang thông tin mà `J` chưa có, ví dụ «điểm này dễ đỗ, khách quen», chứ không chỉ là một điểm dừng hợp lệ. Mô phỏng không có thông tin nào như vậy, và `J` không có số hạng nào cho độ phổ biến: thêm một hub chỉ thêm một chỗ được chấm như mọi chỗ khác.

**Phép thử lệch phân phối.** Lab dịch toàn bộ điểm nóng 0,02° (khoảng 2,2 km) về phía bắc nhưng giữ bảng hub học ở vị trí cũ. Nếu hub có ích thì lệch sẽ làm nó có hại (đưa khách tới chỗ cũ). Kết quả: hub chỉ có mặt trong **1,0%** yêu cầu (so với 5,5%), và không có tác dụng cũng không có hại: `deadhead_m` −0,0 [−0,1; +0,0]. Hub rơi khỏi tập ứng viên khi nhu cầu dịch đi; phép thử này cho thấy bảng học được **không theo kịp** thay đổi phân phối, một điểm yếu của bảng học từ lịch sử, nhưng vì hub vốn không có ích nên nó không đo được thiệt hại.

## Ai được lợi, ai chịu thiệt

Với bảng học được như một thành phần riêng, **không thấy ai được lợi hay chịu thiệt**: mọi khoảng tin cậy chứa 0. Ở đây «không khác biệt» đáng tin hơn thường lệ vì khoảng rất hẹp (km rỗng ±1,1 m), và lý do cũng đơn giản: hub hiếm khi đổi quyết định, nên phần lớn seed cho kết quả y hệt (ở `deadhead_m`, 17 trên 20 seed không đổi). Phần lợi ích của `J` so với baseline (ở hai dòng (1) và (2) trong output) là của chính `J`, không phải của bảng: `deadhead_m` −107,0 m so với −107,4 m có hub, như nhau.

Rủi ro, nếu đem phương pháp ra đời thật, thuộc về **khách ở những nơi hiếm người đón**: một bảng học từ lịch sử sẽ thiên về nơi đã đông. Nơi nhiều người đón (trung tâm) được hub thêm ứng viên tốt; nơi ít người đón (rìa) không có hub nào, và nếu hub còn tác động tới việc chọn điểm thì sự chênh lệch dịch vụ giữa hai khu tăng lên. Lab không đo điều này; đây là một rủi ro cần kiểm ngay khi có dữ liệu thật, theo tinh thần nguyên tắc «không phân biệt đối xử theo khu vực» của [bài 45](/articles/location-chi-phi-ghep-cong-bang-tai-xe-rui-ro-huy).

## Chạy ví dụ và kiểm chứng

Cần `data/hcm-graph.bin`, Rust (cargo) và `uv`. Project Python nằm riêng ở `examples/toi-noi-location/python/loc39` (Python ≥ 3.13, `numpy`, `scikit-learn`, phiên bản ghim trong `uv.lock`). Ba bước theo thứ tự, từ gốc repo:

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run --release -q --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc39_hoc_diem_don -- --export --jobs 8
(cd examples/toi-noi-location/python/loc39 && uv sync && uv run python loc39_dbscan.py)
cargo run --release -q --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc39_hoc_diem_don -- --eval --jobs 6
```

Bước 1 ghi `python/artifacts/loc39/pickups_history.csv` (thư mục này nằm trong `.gitignore`). Bước 2 đọc file đó, quét tham số, ghi `pickup_table.csv` với cột `hub_id, lat, lng, n_points, n_seeds, radius_m`. Bước 3 nạp bảng và đo `J` có hub so với không hub. Output thật của bước 1 và các dòng chính của bước 2 (Python 3.13.15, trên dữ liệu mô phỏng):

```text
đã ghi 8723 dòng (7546 chuyến hoàn thành) vào .../python/artifacts/loc39/pickups_history.csv

== LOC-39: DBSCAN trên mét ==
đầu vào: pickups_history.csv: 7546 điểm đón của chuyến hoàn thành, 40 seed lịch sử (20260301…20260340); dữ liệu mô phỏng
chia theo seed (không xáo trộn): huấn luyện 30 seed (5670 điểm), kiểm định 10 seed (1876 điểm)
→ chọn eps = 100 m, min_samples = 10: 31 hub, phủ kiểm định 0.032 so với ngẫu nhiên 0.012
hồ sơ cuối (học trên cả 40 seed): 62 hub, 0.84 điểm là nhiễu; cỡ cụm p50 14, max 97
```

Và đây là hai dòng của bước 3 nói lên kết quả rỗng:

```text
  (3) J có hub so với J không có hub (chênh cặp = tác dụng RIÊNG của bảng học được)
  deadhead_m trung bình                2250.0     2249.6       -0.4          [-1.5, +0.7]          [-1.5, +0.3]     1/2          2185.8→2180.8
  hub có mặt trong tập ứng viên của 5.5% yêu cầu (TB 0.10 hub thêm mỗi yêu cầu)
```

Kiểm tra nhanh không cần simulator: `uv run python loc39_dbscan.py --selftest` dựng ba đám mây điểm **bịa** và kiểm tra DBSCAN trên mét tìm lại đúng ba hub (output `selftest OK: 3 hub, cỡ cụm [60, 60, 61]`). `verify-p04.sh` chạy lệnh đó. Lab Rust chạy trên máy chịu tải rất nặng nên bài không dùng số thời gian.

## Giới hạn của ví dụ

- **Lịch sử là của simulator**, sinh từ tám điểm nóng đặt sẵn; cụm học được chỉ tìm lại chúng (vòng 1). Không có «điểm đón nổi tiếng» cấp mét để học.
- **Vòng 2 chưa đo.** Lab đo thiên lệch ở bước học (7,2% điểm lịch sử ở chỗ cấm; 3/62 hub), không mô phỏng vòng phản hồi nhiều thế hệ.
- **Hub chỉ là ứng viên**, không phải ưu tiên; `J` vẫn chọn theo chi phí. Một cách dùng khác (cộng điểm thưởng cho hub) chưa thử.
- **Dữ liệu vào không phải Parquet GPS** của hợp đồng lịch sử (xem trên); không dùng dữ liệu thật.
- **Chỉ một họ thuật toán** (DBSCAN, một lưới 24 cấu hình, một metric độ phủ 100 m). HDBSCAN hay OPTICS (mật độ thay đổi) không thử.
- **«62/62 hub trong 2σ»** là phép đo thô: vòng 2σ có bán kính 1.400–2.000 m. Nó xác nhận hub nằm trong vùng đã đặt, không nói gì về từng điểm dừng.
- **Seed lịch sử tách khỏi seed đánh giá**, nhưng cùng một kịch bản; phép thử lệch phân phối là một dịch chuyển duy nhất (0,02° về phía bắc).
- **Phiên bản thư viện** ghim trong `uv.lock`; chưa chạy lại trên phiên bản khác.

## Bài tập

**Bài 1.** Với `eps` = 100 m, `min_samples` = 5 độ phủ vượt ngẫu nhiên lớn nhất (+0,023) nhưng bị loại, còn `min_samples` = 10 (+0,020) được chọn. Vì sao quy tắc đặt trước loại cấu hình đầu, và đó có phải là «chọn cho đẹp» không?

<details><summary>Lời giải bài 1</summary>
<p>Cấu hình đầu cho 154 hub, vượt giới hạn 150 hub của quy tắc; quy tắc được viết trước khi chạy quét, nhằm tránh một bảng quá dày (gần như mọi điểm là hub, độ phủ tăng chỉ vì nhiều hub). Đó không phải chọn cho đẹp vì giới hạn có sẵn từ trước và áp dụng như nhau cho mọi dòng; nhưng nó vẫn là một lựa chọn của người viết, và con số 150 chưa được biện minh bằng dữ liệu.</p>
</details>

**Bài 2.** Thiết kế một thí nghiệm trong simulator để đo **vòng 2** (bảng hub làm đổi nơi đón và lần học sau học lại chính nó). Chỉ rõ cái gì thay đổi giữa các thế hệ và chỉ số nào báo thiên lệch tăng.

<details><summary>Lời giải bài 2</summary>
<p>Chạy theo thế hệ: thế hệ 0 học từ baseline; thế hệ 1 dùng bảng thế hệ 0 theo cách làm hub có trọng số (ví dụ cộng bớt vào <code>J</code> của hub) rồi xuất lịch sử điểm đón mới; học lại bảng; lặp vài thế hệ. Chỉ số báo thiên lệch: tỉ lệ điểm đón nằm ở hub, số hub, và độ chênh dịch vụ (ETA, quãng đi bộ) giữa khu đông điểm đón và khu thưa. Nếu thiên lệch tăng, tỉ lệ điểm đón ở hub tăng qua thế hệ trong khi khách ở khu thưa không được lợi tương ứng.</p>
</details>

**Bài 3.** Hub chỉ có mặt trong 5,5% yêu cầu. Nếu một hub luôn trùng một ứng viên có sẵn trong 10 m thì nó bị bỏ. Hãy chỉ ra một thay đổi **mà lab chưa làm** khiến bảng học được có thể giúp thật, và nói điều kiện dữ liệu cần có.

<details><summary>Lời giải bài 3</summary>
<p>Cho hub mang thông tin mà <code>J</code> không có: ví dụ một hệ số «độ dễ đỗ, độ tin cậy» học từ lịch sử thật (tỉ lệ đón thành công, thời gian khách chờ ở điểm đó, tỉ lệ khách không tìm thấy xe), dùng làm số hạng thưởng hoặc phạt trong <code>J</code>. Điều kiện: có nhãn kết quả thật cho từng điểm đón, mà simulator không có (khách đứng ngẫu nhiên, mọi điểm dừng hợp lệ như nhau). Ngoài đời, cần kiểm không phân biệt đối xử theo khu vực trước khi bật.</p>
</details>

Bài trước: [bài 38](/articles/location-toi-uu-diem-tra-cong-vao-chuyen-ke-tiep). Bài tiếp: [bài 40](/articles/location-cuoc-quote-giai-thich-doi-diem-don-cho-khach).

## Tài liệu gốc

- Ester, M., Kriegel, H.-P., Sander, J., Xu, X. (1996), «A Density-Based Algorithm for Discovering Clusters in Large Spatial Databases with Noise», KDD-96: [bản PDF trên AAAI](https://www.aaai.org/Papers/KDD/1996/KDD96-037.pdf). Tiêu đề và danh sách tác giả đã đối chiếu với siêu dữ liệu của tệp PDF; số trang chưa kiểm nên không ghi.
- [DBSCAN trong scikit-learn](https://scikit-learn.org/stable/modules/generated/sklearn.cluster.DBSCAN.html): tham số `eps`, `min_samples` và nhãn nhiễu `-1`.
- [Tài liệu uv](https://docs.astral.sh/uv/): `uv sync`, `uv run` và lockfile.
- [DBSCAN (Wikipedia)](https://en.wikipedia.org/wiki/DBSCAN): điểm lõi, điểm biên, nhiễu.
