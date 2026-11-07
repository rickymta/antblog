---
slug: python-networkx-graph-cau-ket-tai-xe-khach
title: "Đồ thị với networkx: tìm cấu kết tài xế–khách"
summary: "Bài 58: dựng đồ thị khách, tài xế, thiết bị theo cửa sổ thời gian bằng networkx, tách nhóm bằng Louvain, gắn cờ nhóm cho người duyệt và đo trên dữ liệu tổng hợp; nói rõ giới hạn quy mô."
category: python-advanced
level: advanced
tags: [python, toi-noi, fraud, networkx, graph]
language: vi
---

Hai tài xế và năm tài khoản khách mới của [Tới Nơi](/articles/du-an-mau-toi-noi) chạy qua lại suốt vài tuần, mỗi chuyến 0,8 đến 3 km, GPS khớp điểm đón. Nhìn riêng, không chuyến nào đáng ngờ; đặt cạnh nhau mới thấy ngần ấy tài khoản chạy với ngần ấy tài xế cho đủ chỉ tiêu thưởng. Đó là `collusion` của bài 54, điểm yếu đã ghi sẵn: bộ chấm theo từng chuyến đạt PR-AUC 0,026; 30 đặc trưng cửa sổ của bài 55 kéo lên 0,651, nhưng trong 1 % chuyến điểm cao nhất vẫn chỉ có 36,1 % chuyến cấu kết.

Bài này thêm **đồ thị** (graph) gồm khách, tài xế và thiết bị, dựng bằng `networkx`. Ta chỉ **gắn cờ nhóm** để người duyệt xem, không kết luận cấu kết: khách quen của một tài xế hay gia đình dùng chung điện thoại cũng tạo hình dạng na ná. Dữ liệu là tổng hợp, nên số đo chỉ nói về bộ này.

Đây là bài 58 của lộ trình **Python nâng cao qua Tới Nơi**. Bạn cần dữ liệu và cách đo của [bài 54](/articles/python-fraud-toi-noi-bai-toan-nhan-du-lieu-metric), đặc trưng cửa sổ của [bài 55](/articles/python-feature-engineering-tu-event-kafka) và polars ở [bài 29](/articles/python-polars-pandas-parquet-phan-tich-chuyen).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ta dựng đồ thị khách–tài xế từ mọi chuyến trong 28 ngày rồi coi thành phần liên thông lớn nhất là «nhóm đáng ngờ» nhất. Cách này có dùng được không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Ở ảnh chụp ngày 50, đồ thị mọi cặp khách–tài xế có 88.100 cạnh nhưng chỉ 6 thành phần liên thông, lớn nhất 10.533 nút: một khách đi hai tài xế là đủ nối hai nhóm. Nó chứa 27 trong 60 khách cấu kết của cửa sổ, lẫn với hàng nghìn tài khoản thường. Phải lọc cạnh yếu và tách nhóm trước.</p></details>
</div>

## Điều đồ thị thấy mà từng chuyến không thấy

Cấu kết để lại hai dấu vết quan hệ: cặp khách–tài xế lặp lại và thiết bị dùng chung giữa nhiều tài khoản. Đặc trưng cửa sổ ở bài 55 đếm từng dấu vết riêng; đồ thị cho thấy **hình dạng** chung của chúng, thứ phân biệt nhóm cấu kết với tài xế đông khách quen. Lab có ba loại nút (khách `CU-…`, tài xế `DR-…`, thiết bị `DEV-…`); cạnh khách–tài xế mang trọng số là số chuyến (đồ thị hai phía, bipartite), cạnh tài khoản–thiết bị nối khách hoặc tài xế với máy họ dùng. Bốn khái niệm:

- **Thành phần liên thông** (connected component): tập nút đi được tới nhau.
- **Độ trung tâm** (centrality): bậc (số cạnh), core number (độ sâu của nút trong lõi dày nhất), betweenness (tỉ lệ đường đi ngắn nhất giữa mọi cặp nút đi qua nút đó; hub rất cao).
- **Mật độ** (density): số cạnh chia số cạnh tối đa.
- **Số chu trình độc lập** (cycle rank): E − N + C, C là số thành phần liên thông; 0 với cây, lớn khi có nhiều đường nối chéo.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 352" role="img" aria-label="Hai hình dạng đồ thị. Bên trái: một tài xế nối với năm khách quen, hình sao, sáu nút năm cạnh, không có chu trình. Bên phải: hai tài xế, bốn khách và hai thiết bị dùng chung, mỗi khách nối với cả hai tài xế, tám nút mười hai cạnh, năm chu trình độc lập."><rect x="20" y="30" width="410" height="264" rx="10" class="f-surface s-line"/><text x="36" y="56" class="t-b">Khách quen của một tài xế: hình sao</text><path d="M225 160 L225 80 M225 160 L301 135 M225 160 L272 225 M225 160 L178 225 M225 160 L149 135" class="f-none s-muted" stroke-width="1.5"/><circle cx="225" cy="160" r="24" class="f-phys-soft s-phys"/><circle cx="225" cy="80" r="15" class="f-pk-soft s-pk"/><circle cx="301" cy="135" r="15" class="f-pk-soft s-pk"/><circle cx="272" cy="225" r="15" class="f-pk-soft s-pk"/><circle cx="178" cy="225" r="15" class="f-pk-soft s-pk"/><circle cx="149" cy="135" r="15" class="f-pk-soft s-pk"/><text x="36" y="278" class="t-small">6 nút, 5 cạnh, 0 chu trình độc lập</text><rect x="450" y="30" width="410" height="264" rx="10" class="f-surface s-line"/><text x="466" y="56" class="t-b">Nhóm cấu kết: lưới nhiều đường nối chéo</text><path d="M610 105 L540 170 M610 105 L780 170 M710 105 L540 170 M710 105 L780 170 M610 235 L540 170 M610 235 L780 170 M710 235 L540 170 M710 235 L780 170" class="f-none s-muted" stroke-width="1.5"/><path d="M625 105 L642 105 M678 105 L695 105 M625 235 L642 235 M678 235 L695 235" class="f-none s-warn dash" stroke-width="1.5"/><circle cx="540" cy="170" r="24" class="f-phys-soft s-phys"/><circle cx="780" cy="170" r="24" class="f-phys-soft s-phys"/><circle cx="610" cy="105" r="15" class="f-pk-soft s-pk"/><circle cx="710" cy="105" r="15" class="f-pk-soft s-pk"/><circle cx="610" cy="235" r="15" class="f-pk-soft s-pk"/><circle cx="710" cy="235" r="15" class="f-pk-soft s-pk"/><rect x="642" y="95" width="36" height="20" rx="4" class="f-warn-soft s-warn dash"/><rect x="642" y="225" width="36" height="20" rx="4" class="f-warn-soft s-warn dash"/><text x="466" y="278" class="t-small">8 nút, 12 cạnh, 5 chu trình độc lập</text><circle cx="44" cy="326" r="8" class="f-phys-soft s-phys"/><text x="60" y="330" class="t-small">tài xế</text><circle cx="130" cy="326" r="8" class="f-pk-soft s-pk"/><text x="146" y="330" class="t-small">khách</text><rect x="210" y="318" width="22" height="16" rx="3" class="f-warn-soft s-warn dash"/><text x="240" y="330" class="t-small">thiết bị dùng chung</text><path d="M390 326 L420 326" class="f-none s-muted" stroke-width="1.5"/><text x="428" y="330" class="t-small">cặp từ 2 chuyến</text><path d="M560 326 L590 326" class="f-none s-warn dash" stroke-width="1.5"/><text x="598" y="330" class="t-small">tài khoản dùng thiết bị</text></svg>
</div>
<figcaption>Hình 1. Cùng là «tài xế có nhiều khách», nhưng khách quen tạo hình sao còn nhóm cấu kết tạo lưới. Số chu trình độc lập E − N + C là 0 với hình sao và 5 với lưới.</figcaption>
</figure>

Cách đo như bài 54 và 55: huấn luyện trước ngày 30, đo trên valid từ ngày 40 (66.704 chuyến, 155 chuyến `collusion`); PR-AUC theo loại là PR-AUC của loại đó so với chuyến sạch. Hai mốc cần vượt (bước 1, trích; máy đang bận):

```text
mô hình                                       PR-AUC  ROC-AUC  collusion PR-AUC  collusion recall@1%
hồi quy logistic, 9 đặc trưng theo từng chuyến   0.537    0.933             0.026                    -
HistGradientBoosting, 30 đặc trưng cửa sổ      0.886    0.995             0.651                36.1%
(huấn luyện HistGradientBoosting 62 giây; load average 1/5/15 phút: 127.61 106.14 106.98)
HistGradientBoosting: tối đa 150 vòng, dừng sớm ở 84 (n_iter_); dừng sớm dùng 10 % train tách ngẫu nhiên
tạo tác của bộ sinh: chuyến sạch dùng NEWUSER50: ngày 0-9 7.8%, ngày 50-59 0.9% (quần thể khách gần như cố định); chuyến gian lận không phải promo_abuse có mã khuyến mãi: 0.0%
```

HistGradientBoosting chạy tối đa 150 vòng và dừng sớm ở 84 vòng (`n_iter_`; dừng sớm dùng 10 % train tách ngẫu nhiên). 1 % điểm cao nhất của valid là khoảng 667 chuyến, số người duyệt xem được. Đồ thị thêm được bao nhiêu, nếu dựng đúng thời điểm?

## Dựng đồ thị theo cửa sổ thời gian

Quy tắc của bài 55 áp dụng nguyên: chuyến được chấm lúc `PaymentCaptured`, đặc trưng chỉ dùng thứ có trước đó. Dựng đồ thị cho từng chuyến là quá đắt, nên lab chụp **mỗi ngày một ảnh chụp**: lúc 00:00 giờ Việt Nam của ngày D, đồ thị gồm các chuyến đã thu tiền trong 28 ngày trước, và mọi chuyến thanh toán trong ngày D dùng ảnh chụp ấy. Không nhãn nào dùng để dựng đồ thị.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Ảnh chụp đồ thị lúc 00:00 giờ Việt Nam của ngày D. Cửa sổ 28 ngày trước đó gồm các chuyến đã thu tiền được dựng thành đồ thị; ngày D trở đi, kể cả chuyến đang chấm, chưa có trong đồ thị. Bốn bước: polars lọc cặp và thiết bị, networkx dựng đồ thị và tách nhóm, thống kê nhóm và nút, gắn 14 đặc trưng vào chuyến thanh toán trong ngày D."><defs><marker id="py58-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="140" y="56" width="440" height="76" rx="8" class="f-phys-soft s-phys"/><text x="156" y="88" class="t-b">Cửa sổ 28 ngày</text><text x="156" y="108" class="t-small">chuyến đã thu tiền trong [đầu ngày D - 28, đầu ngày D)</text><rect x="580" y="56" width="260" height="76" rx="8" class="f-warn-soft s-warn dash"/><text x="596" y="88" class="t-b">Từ đầu ngày D trở đi</text><text x="596" y="108" class="t-small">chưa vào đồ thị, kể cả chuyến đang chấm</text><path d="M580 40 L580 146" class="f-none s-muted dash" stroke-width="1.5"/><text x="596" y="34" class="t-small">ảnh chụp lúc 00:00 giờ Việt Nam</text><rect x="30" y="176" width="190" height="64" rx="8" class="f-surface s-line"/><text x="44" y="202" class="t-b">polars: lọc cạnh</text><text x="44" y="222" class="t-small">cặp từ 2 chuyến, thiết bị</text><path d="M220 208 L260 208" class="f-none s-phys" stroke-width="2" marker-end="url(#py58-1-arrow)"/><rect x="260" y="176" width="190" height="64" rx="8" class="f-surface s-line"/><text x="274" y="202" class="t-b">networkx: đồ thị</text><text x="274" y="222" class="t-small">thành phần, Louvain</text><path d="M450 208 L490 208" class="f-none s-phys" stroke-width="2" marker-end="url(#py58-1-arrow)"/><rect x="490" y="176" width="210" height="64" rx="8" class="f-surface s-line"/><text x="504" y="202" class="t-b">Thống kê nhóm và nút</text><text x="504" y="222" class="t-small">mật độ, chu trình, betweenness</text><path d="M700 208 L740 208" class="f-none s-phys" stroke-width="2" marker-end="url(#py58-1-arrow)"/><rect x="740" y="176" width="120" height="64" rx="8" class="f-pk-soft s-pk"/><text x="754" y="202" class="t-b">14 đặc trưng</text><text x="754" y="222" class="t-small">gắn vào chuyến</text><path d="M360 132 L360 176" class="f-none s-phys" stroke-width="2" marker-end="url(#py58-1-arrow)"/><text x="30" y="276" class="t-small">Chuyến thanh toán trong ngày D chỉ nhận đặc trưng từ ảnh chụp này, nên đồ thị luôn cũ tối đa một ngày.</text></svg>
</div>
<figcaption>Hình 2. Ảnh chụp của ngày D chỉ nhìn thấy 28 ngày đã qua; chuyến đang chấm không bao giờ nằm trong đồ thị phục vụ chính nó. Bốn bước từ chuyến thô đến đặc trưng.</figcaption>
</figure>

`window_slice` cắt cửa sổ nửa mở `[đầu ngày − 28, đầu ngày)`. Polars lọc cạnh trước khi vào networkx vì networkx tốn bộ nhớ theo số cạnh: chỉ giữ cặp khách–tài xế từ 2 chuyến (`STRONG_MIN_TRIPS`) và thiết bị có từ 2 tài khoản khác nhau (`graph.py`, trích):

```python
def window_slice(trips: pl.DataFrame, day: int, window_days: int = WINDOW_DAYS) -> pl.DataFrame:
    end = START + timedelta(days=day)
    return trips.filter((pl.col("paid_at") >= end - timedelta(days=window_days)) & (pl.col("paid_at") < end))

def edge_tables(w: pl.DataFrame, strong_min: int = STRONG_MIN_TRIPS) -> tuple[pl.DataFrame, pl.DataFrame]:
    # sort: group_by không giữ thứ tự, mà thứ tự thêm cạnh quyết định thứ tự nút trong networkx và kết quả Louvain
    pairs = w.group_by("customer_id", "driver_id").agg(trips=pl.len()).filter(pl.col("trips") >= strong_min).sort("customer_id", "driver_id")
    accounts = pl.concat([
        w.select(account="customer_id", device="device_id"),
        w.select(account="driver_id", device="driver_device_id"),
    ]).unique()
    shared = accounts.group_by("device").agg(n=pl.col("account").n_unique()).filter(pl.col("n") >= 2).select("device")
    return pairs, accounts.join(shared, on="device").sort("account", "device")
```

`.sort(...)` không để trang trí: `group_by` của polars không giữ thứ tự, mà thứ tự thêm cạnh quyết định thứ tự nút của networkx và kết quả phát hiện cộng đồng; bản đầu của lab thiếu dòng này. Lọc cạnh đáng giá đến đâu? Bước 2, ảnh chụp đầu ngày 50:

```text
giai đoạn                                          cạnh      nút  thành phần  lớn nhất  khách ring
1. mọi cặp khách-tài xế                          88,100   31,235           6    10,533          27
2. chỉ giữ cặp từ 2 chuyến                        3,132    3,180         434       425           4
3. thêm thiết bị dùng chung từ 2 tài khoản        5,018    5,218         792       552           7
4. tách 6 thành phần lớn hơn 40 nút bằng Louvain: 912 nhóm, nhóm lớn nhất 53 nút
```

Đồ thị thô có 6 thành phần, khớp 6 thành phố vì bộ sinh chỉ cho khách và tài xế gặp nhau trong cùng thành phố; thành phần lớn nhất 10.533 nút chứa 27 trên 60 khách cấu kết («khách ring» là khách thuộc nhóm cấu kết thật). Giữ cặp mạnh còn 3.132 cạnh nhưng vẫn có thành phần 425 nút; thêm thiết bị dùng chung đưa nó lên 552 nút. Phải tách tiếp.

## Nhóm, độ trung tâm và mật độ

Thành phần lớn hơn 40 nút (`MAX_GROUP_NODES`) được tách bằng **Louvain**, thuật toán phát hiện cộng đồng chia đồ thị thành các cụm nối dày bên trong, thưa với bên ngoài (`nx.community.louvain_communities`, `seed` cố định). Mỗi nhóm nhận một dòng thống kê (`graph.py`, trích):

```python
def ordered_subgraph(g: nx.Graph, nodes: Iterable[str]) -> nx.Graph:
    """Đồ thị con có thứ tự nút và cạnh đã sắp (g.subgraph(set) duyệt theo thứ tự của tập hợp)."""
    ordered = sorted(nodes)
    sub = nx.Graph()
    sub.add_nodes_from(ordered)
    sub.add_edges_from((u, v, g[u][v]) for u in ordered for v in sorted(g[u]) if u < v and v in sub)
    return sub

def split_groups(g: nx.Graph, max_nodes: int = MAX_GROUP_NODES) -> list[frozenset[str]]:
    groups: list[frozenset[str]] = []
    for comp in nx.connected_components(g):
        if len(comp) <= max_nodes:
            groups.append(frozenset(comp))
        else:
            parts = nx.community.louvain_communities(ordered_subgraph(g, comp), weight="weight", seed=LOUVAIN_SEED)
            groups.extend(frozenset(p) for p in parts)
    return sorted(groups, key=min)  # số hiệu nhóm ổn định giữa các lần chạy

def group_stats(g: nx.Graph, members: frozenset[str]) -> tuple[dict, dict[str, float]]:
    sg = g.subgraph(members)
    n, e = sg.number_of_nodes(), sg.number_of_edges()
    betw = nx.betweenness_centrality(sg, normalized=True) if n >= 3 else dict.fromkeys(members, 0.0)
    stats = {
        "size": n, "edges": e, "density": nx.density(sg) if n > 1 else 0.0,
        # số chu trình độc lập = E - N + số thành phần: cây (hub và nan hoa) bằng 0, lưới nhiều đường nối thì lớn
        "cycle_rank": e - n + nx.number_connected_components(sg),
        "top_betweenness": max(betw.values(), default=0.0),
        ...  # customers, drivers, devices, trips
    }
    return stats, betw
```

`ordered_subgraph` cũng có lý do: bản đầu đưa thẳng `g.subgraph(comp)` (`comp` là `set`) cho Louvain, và dù đã đặt `seed`, nhóm vẫn đổi giữa các lần chạy vì thứ tự duyệt của tập hợp đổi theo `PYTHONHASHSEED`; `--verify` dựng lại ảnh chụp ở hai tiến trình mới để bắt lỗi. Ở ngày 50, sau khi tách 6 thành phần lớn, đồ thị có 912 nhóm, lớn nhất 53 nút. Dùng nhãn (chỉ để đánh giá), ta tách nhóm «cấu kết thật» (từ một nửa khách và tài xế thuộc các nhóm cấu kết thật) khỏi phần còn lại; `top_betweenness` là betweenness lớn nhất trong nhóm, hình sao cho 1,0:

```text
loại nhóm         số nhóm             size        customers          drivers          devices          density       cycle_rank  top_betweenness
còn lại            33,365            3.000            2.000            1.000            1.000            0.667            0.000            1.000
cấu kết thật          474           11.000            5.000            2.000            4.000            0.280            6.000            0.577
```

Nhóm bình thường có trung vị 3 nút, 0 chu trình, betweenness lớn nhất 1,0. Nhóm cấu kết thật lớn hơn (11 nút: 5 khách, 2 tài xế, 4 thiết bị), 6 chu trình, betweenness lớn nhất 0,577, và mật độ thấp hơn (0,280 so với 0,667) vì nhóm tí hon luôn dày, nên mật độ chỉ làm sàn ở bước sau. Bảng chỉ tính nhóm từ 3 nút (33.839 trên 45.057 nhóm-ngày), trung vị theo nhóm-ngày.

## Gắn cờ nhóm cho người duyệt

Luật gắn cờ nằm trong một `NamedTuple` để cấu hình và quét lưới (`graph.py`, trích):

```python
class Rule(NamedTuple):
    min_drivers: int = 2
    min_customers: int = 3
    min_cycle_rank: int = 2
    min_density: float = 0.15
    max_top_betweenness: float = 0.6

def is_suspect(s: dict, rule: Rule = RULE) -> bool:
    """Nhóm có từ 2 tài xế, 3 khách, nhiều đường nối chéo (chu trình), đủ dày và không phải một hub duy nhất."""
    return (s["drivers"] >= rule.min_drivers and s["customers"] >= rule.min_customers and s["cycle_rank"] >= rule.min_cycle_rank
            and s["density"] >= rule.min_density and s["top_betweenness"] <= rule.max_top_betweenness)
```

`tune_rule` quét lưới trên nhóm của train (ảnh chụp ngày 7 đến 29) và chọn luật có F1 cao nhất trên nhóm-ngày; sự thật nền để chọn chỉ lấy từ chuyến thanh toán trước ngày 30 (nhãn cuối). Các cột cuối là kết quả của đúng luật đó trên valid. Nhóm-ngày «đúng» khi từ một nửa khách và tài xế của nó thuộc nhóm cấu kết thật:

```text
 cycle_rank  density  betw<=  F1 train  cờ valid   đúng  precision  recall  F1 valid
          2     0.15    0.60     0.529       131    113      0.863   0.589     0.700
          2     0.15    0.70     0.521       192    140      0.729   0.729     0.729
          2     0.10    0.60     0.512       147    113      0.769   0.589     0.667
luật cố định trong graph.py: Rule(min_drivers=2, min_customers=3, min_cycle_rank=2, min_density=0.15, max_top_betweenness=0.6)
độ nhạy của việc chọn: F1 train hạng 1 0.529, hạng 2 0.521 (hơn nhau 0.008); chọn chỉ trên ngày 7 đến 18, RULE đứng hạng 3/80; chỉ trên ngày 19 đến 29, hạng 1/80
```

Luật cố định trong `graph.py` đứng đầu trên train, nhưng lựa chọn này mong manh: F1 hơn hạng hai 0,008, chọn chỉ trên ngày 7 đến 18 thì RULE hạng 3 trên 80, và trên valid luật hạng hai (betweenness 0,7) có F1 cao hơn (0,729 so với 0,700) vì đổi precision lấy recall. Lab thêm lần lượt từng điều kiện, đổi một biến mỗi lần:

```text
bước thêm điều kiện (luật cố định)           cờ train   prec    rec  cờ valid   prec    rec
từ 2 tài xế và 3 khách                           1611  0.056  0.439      2435  0.063  0.797
+ cycle_rank >= 2                                 315  0.254  0.390       628  0.231  0.755
+ density >= 0.15                                 135  0.593  0.390       244  0.594  0.755
+ top_betweenness <= 0.6 (= RULE)                  86  0.895  0.376       131  0.863  0.589
precision nới trên valid (nhóm bị cờ có chứa từ 3 tài khoản cấu kết thật): 99.2%; độ thuần trung vị của nhóm bị cờ (tỉ lệ tài khoản thuộc nhóm cấu kết): 0.64
nhóm-ngày bị gắn cờ trong valid: 131, mỗi ngày trung bình 6.5, lớn nhất 10; nhưng chỉ 50 tập thành viên khác nhau, tức trung bình 2.5 tập mới mỗi ngày
```

Riêng «từ 2 tài xế và 3 khách» gắn cờ hàng nghìn nhóm-ngày ở precision cỡ 6 %. Thêm chu trình, mật độ, betweenness đẩy precision trên valid lên 86,3 % với recall 58,9 % (precision «chặt»). Cách «nới» (nhóm chứa ít nhất 3 tài khoản cấu kết thật, trong đó 2 khách) cho 99,2 %, nhưng độ thuần trung vị chỉ 0,64: tài xế của nhóm cấu kết vẫn chở khách thường nên nhóm bị cờ thường lẫn họ. Valid có 131 nhóm-ngày bị cờ (trung bình 6,5 mỗi ngày) nhưng chỉ 50 tập thành viên khác nhau. Recall là 0,376 ở train và 0,589 ở valid; thấp ở đầu kỳ, nguyên nhân chưa đo (cửa sổ đầu kỳ ngắn hơn 28 ngày):

```text
     ngày  nhóm-ngày thật    cờ  precision  recall
   7-14                86    10      1.000   0.116
  15-22               100    43      0.953   0.410
  23-29                87    33      0.788   0.299
  30-39               122    56      0.804   0.369
  40-49                91    64      0.906   0.637
  50-59               101    67      0.821   0.545
```

```text
     ngày  nhóm-ngày thật    cờ  precision  recall
   7-14                86    10      1.000   0.116
  15-22               100    43      0.953   0.410
  23-29                87    33      0.788   0.299
  30-39               122    56      0.804   0.369
  40-49                91    64      0.906   0.637
  50-59               101    67      0.821   0.545
```

Người vận hành quan tâm hơn: nhóm cấu kết thật bị phát hiện sau bao lâu? Lab nối 15 nhóm cấu kết thật (thành phần liên thông của các chuyến `collusion`) với nhóm bị cờ đầu tiên chứa từ 2 khách của chúng («ngày đầu» là ngày có chuyến cấu kết đầu tiên, tính từ 02/03/2026). Số nhóm do bộ sinh quyết định: nhóm cuối bị cắt bớt cho đủ tổng chuyến (nhóm 14 chỉ 1 khách, 2 chuyến, dùng chung thiết bị với nhóm 12), và seed khác cho số nhóm khác:

```text
 nhóm  khách  chuyến  ngày đầu  ngày cờ  trễ (ngày)  chuyến sau cờ  train  valid
    1      3      57         4       10           6             47     49      0
   10      5      37         5       12           7             24     37      0
    8      5      34         6       13           7             22     29      0
    2      4      40        10       16           6             32     24      4
    6      4      56        11       15           4             50     42      8
   13      6      41        11       16           5             29     41      0
    4      5      42        16        -           -              0     18     13
   12      4       9        19        -           -              0      6      3
    7      4      47        21       52          31              1     13     16
    9      4      52        24       41          17             17     13     18
    5      4      35        25       29           4             26     11      0
    3      6      46        27       34           7             34      3     22
   14      1       2        29        -           -              0      1      0
    0      6      44        30       37           7             29      0     22
   11      4      58        38       40           2             49      0     49
bắt được 12/15 nhóm (khoảng tin cậy Wilson 95 %: 0.548 đến 0.930); độ trễ trung vị 6 ngày; 360/600 chuyến cấu kết (60.0%) thanh toán từ ngày nhóm bị gắn cờ trở đi
nhóm đã có chuyến ở train (ngưỡng được chọn khi đã thấy chúng): 13/15; chưa có chuyến nào ở train: 2, bị bắt 2; nhóm còn hoạt động ở valid: 9, trong đó đã có ở train: 7
cặp nhóm cấu kết thật dùng chung ít nhất một thiết bị khách: [(12, 14)]
```

Luật bắt 12 trên 15 nhóm (khoảng tin cậy Wilson 95 %: 0,548 đến 0,930), độ trễ trung vị 6 ngày; 360 trên 600 chuyến cấu kết (60,0 %) thanh toán từ ngày nhóm bị cờ trở đi. Số này không phải held-out: 13 nhóm đã có chuyến ở train, nơi chọn ngưỡng; 2 nhóm chưa có chuyến nào ở train đều bị bắt; valid có 9 nhóm còn hoạt động, 7 đã có ở train. Với ba nhóm không bị cờ và hai nhóm bị cờ muộn, mỗi ngày ta lấy nhóm chứa nhiều khách của nó nhất và đếm điều kiện làm nó trượt:

```text
 nhóm  chuyến  ngày có nhóm  đạt luật  chỉ density  chỉ betw  density+betw  khác
    4      42            43         0           10         0            22    11
   12       9            28         0            0         0             7    21
    7      47            38         2            0        29             3     4
    9      52            33        13            6         5             6     3
   14       2             0         0            0         0             0     0
```

Nhóm 4 (5 khách, 42 chuyến) chưa lần nào bị cờ. Qua 43 ngày có nhóm chứa nó, mật độ trượt ở 32 ngày (10 chỉ mật độ, 22 cùng betweenness), betweenness một mình chưa lần nào: nguyên nhân chính là mật độ của nhóm Louvain đã gộp nó với khách thường, không phải ngưỡng betweenness. Nhóm 7 thì ngược lại: 29 trong 38 ngày chỉ trượt betweenness, nên ngưỡng 0,6 là cạnh sắc ở đó. Nhóm 12 (9 chuyến) và 14 (2 chuyến) quá nhỏ để thành lưới. Hạ ngưỡng không chữa được: betweenness 0,7 nâng recall trên valid lên 0,729 nhưng kéo precision từ 0,863 xuống 0,729.

Cuối cùng, hàng đợi người duyệt nhận bản ghi nêu bằng chứng và thành viên, không nêu kết luận (mã nhóm lấy theo thành viên có ID nhỏ nhất, vì số hiệu trong ảnh chụp đổi mỗi ngày). Đây là nhóm có nhiều chuyến nhất ở ảnh chụp ngày 59:

```json
{
  "group_id": "G-CU-06332",
  "snapshot_day": 59,
  "status": "needs_review",
  "members": {
    "customers": 7,
    "drivers": 2,
    "devices": 4
  },
  "member_ids": [
    "CU-06332",
    "CU-17713",
    "CU-22379",
    "CU-24708",
    "CU-27080",
    "CU-27832",
    "CU-42377",
    "DEV-01281",
    "DEV-07240",
    "DEV-28884",
    "DEV-38077",
    "DR-00361",
    "DR-01026"
  ],
  "trips_in_window": 65,
  "reasons": [
    "2 tài xế và 7 khách nối với nhau bằng các cặp từ 2 chuyến trở lên",
    "12 chu trình độc lập, mật độ 0.31, không phải một hub duy nhất (betweenness lớn nhất 0.23)",
    "4 thiết bị dùng chung từ 2 tài khoản"
  ],
  "note": "Gắn cờ nhóm để người duyệt xem; không phải kết luận cấu kết. Cần xem chuyến, thiết bị và nhãn của người duyệt trước khi hành động."
}
```

## Đưa đặc trưng đồ thị vào mô hình, và rò rỉ

`add_graph_features` gắn 14 đặc trưng vào mỗi chuyến từ ảnh chụp của ngày chuyến được thanh toán: trọng số cạnh khách–tài xế (`g_edge_trips`), cùng nhóm không, nhóm có bị cờ không, và với mỗi bên (`gc_` khách, `gd_` tài xế) kích thước nhóm, số chu trình, mật độ, core number, betweenness (tài xế thêm bậc). Nút chưa có trong đồ thị nhận 0. Trung bình trên valid:

```text
loại             gc_group_size     gc_cycle_rank    gd_betweenness      g_edge_trips   g_suspect_group
collusion               16.716            10.806             0.346             3.439             0.639
ghost_trip               3.718             0.779             0.439             0.859             0.000
gps_spoof                1.015             0.010             0.237             0.000             0.000
none                     4.689             0.234             0.371             0.189             0.000
promo_abuse              0.000             0.000             0.275             0.000             0.000
```

63,9 % chuyến `collusion` của valid nằm trong nhóm bị cờ, so với 26 trong 65.563 chuyến sạch. Cột của `promo_abuse` bằng 0 vì bộ sinh cho mỗi tài khoản dùng mã đúng một lần: lúc chuyến đó được chấm, tài khoản chưa có trong đồ thị. Thêm 14 cột vào mô hình (cùng siêu tham số và cách chia như bài 55):

```text
mô hình                             PR-AUC  collusion  recall@1%  promo_abuse  recall@1%  ghost_trip  gps_spoof
30 đặc trưng cửa sổ (mốc, bài 55)    0.886      0.651      36.1%        0.719      56.0%       0.873      0.787
+ 14 đặc trưng đồ thị                0.915      0.963      80.6%        0.774      55.7%       0.879      0.758
chỉ 14 đặc trưng đồ thị              0.235      0.951      96.8%        0.008       0.3%       0.173      0.006
luật nhóm một mình, không mô hình: gắn cờ 125 chuyến; 99/155 chuyến collusion (recall 63.9%); 79.2% số chuyến bị cờ là gian lận (mọi loại), 79.2% là collusion
```

PR-AUC tổng tăng từ 0,886 lên 0,915; với `collusion`, PR-AUC theo loại tăng từ 0,651 lên 0,963 và recall trong 1 % điểm cao nhất từ 36,1 % lên 80,6 % (hình 3). Đặc trưng đồ thị một mình cho `collusion` 0,951 nhưng chỉ 0,235 tổng thể: nó bổ sung, không thay thế. Luật nhóm chạy một mình gắn cờ 125 chuyến: 99 trên 155 chuyến `collusion` nằm trong đó (63,9 %), 26 chuyến còn lại là chuyến sạch, tức 79,2 % chuyến bị cờ là `collusion`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="PR-AUC của loại collusion trên valid và recall trong 1 % điểm cao nhất. Hồi quy logistic 0,026; 30 đặc trưng cửa sổ 0,651 và recall 36,1 %; thêm 14 đặc trưng đồ thị 0,963 và recall 80,6 %; chỉ 14 đặc trưng đồ thị 0,951 và recall 96,8 %."><text x="270" y="34" class="t-b">PR-AUC của collusion</text><text x="800" y="34" class="t-b">recall@1 %</text><path d="M270 46 L270 236" class="f-none s-muted" stroke-width="1.5"/><text x="20" y="79" class="t-small">logistic, 9 đặc trưng từng chuyến</text><rect x="270" y="60" width="12" height="28" rx="4" class="f-phys-soft s-phys"/><text x="290" y="79" class="t-mono">0,026</text><text x="800" y="79" class="t-mono">-</text><text x="20" y="125" class="t-small">30 đặc trưng cửa sổ (bài 55)</text><rect x="270" y="106" width="312" height="28" rx="4" class="f-phys-soft s-phys"/><text x="590" y="125" class="t-mono">0,651</text><text x="800" y="125" class="t-mono">36,1 %</text><text x="20" y="171" class="t-small">30 đặc trưng cửa sổ + 14 đồ thị</text><rect x="270" y="152" width="462" height="28" rx="4" class="f-pk-soft s-pk"/><text x="740" y="171" class="t-mono">0,963</text><text x="800" y="171" class="t-mono">80,6 %</text><text x="20" y="217" class="t-small">chỉ 14 đặc trưng đồ thị</text><rect x="270" y="198" width="456" height="28" rx="4" class="f-pk-soft s-pk"/><text x="734" y="217" class="t-mono">0,951</text><text x="800" y="217" class="t-mono">96,8 %</text><text x="20" y="262" class="t-small">Thanh dài hơn là tốt hơn; thang từ 0 đến 1. Đo trên valid từ ngày 40: 155 chuyến collusion trong 66.704 chuyến.</text></svg>
</div>
<figcaption>Hình 3. Với loại collusion, đặc trưng đồ thị nâng PR-AUC theo loại và recall trong 1 % điểm cao nhất; một mình chúng đã vượt 30 đặc trưng cửa sổ.</figcaption>
</figure>

Đồ thị rò rỉ rất dễ vì «dựng một lần cho cả kỳ» trông tự nhiên. Thứ nhất, phép thử cắt cụt như bài 55: ảnh chụp ngày D dựng từ event cắt ở đầu ngày D phải giống ảnh chụp dựng từ toàn bộ dữ liệu. Lab thử ba ngày, cả bảng chuyến bỏ cột nhãn, và một lỗi cố ý (cửa sổ dài thêm 3 ngày về tương lai):

```text
ảnh chụp ngày (45, 50, 55): dựng từ event cắt ở đầu ngày giống dựng từ toàn bộ? [True, True, True]
  giống nhau khi bỏ hết cột nhãn khỏi bảng chuyến? [True, True, True]
  nếu cửa sổ kết thúc muộn hơn 3 ngày (nhìn tương lai), phép cắt cụt có làm kết quả đổi (lộ lỗi)? [True, True, True]
ảnh chụp ngày 50 dựng lại trong tiến trình mới với PYTHONHASHSEED=1 và 2: dấu vân tay cd6eef9ea024a72c, cd6eef9ea024a72c; tiến trình này cd6eef9ea024a72c
```

Thứ hai, lỗi hay gặp nhất: dựng đồ thị MỘT lần trên cả 60 ngày rồi gắn cho mọi chuyến, đếm cả chuyến đang chấm và chuyến sau nó. Lab huấn luyện trên nó, đo offline (valid cũng dùng đồ thị toàn kỳ) và phục vụ (valid dùng ảnh chụp theo ngày):

```text
mô hình                                       PR-AUC  collusion PR-AUC
đồ thị theo ngày, đo như phục vụ (bước 5)      0.915             0.963
đồ thị toàn kỳ, offline (valid cũng toàn kỳ)   0.950             0.970
đồ thị toàn kỳ, phục vụ (valid theo ngày)      0.876             0.866
```

Số offline 0,950 đẹp hơn số trung thực 0,915, và khi phục vụ thật mô hình tụt xuống 0,876, thấp hơn cả mô hình dựng đúng. Riêng `collusion`: 0,970 offline, 0,866 khi phục vụ, 0,963 cho bản dựng đúng. Một seed chưa đủ cho kết luận thứ bậc, nên bước 9 chạy lại cả chuỗi ở seed 7 và 123 (nhãn cuối ở mọi bước):

```text
 seed nhóm thật  bắt RULE hạng prec valid rec valid  PR-AUC cửa sổ  +đồ thị  collusion cửa sổ  +đồ thị  toàn kỳ offline  phục vụ
   42        15   12         1      0.863     0.589          0.886    0.915             0.651    0.963            0.950    0.876
    7        14   14         9      0.778     0.551          0.882    0.908             0.506    0.885            0.948    0.852
  123        14   14         1      0.883     0.652          0.894    0.921             0.650    0.965            0.951    0.898
```

Đặc trưng đồ thị nâng PR-AUC theo loại của `collusion` ở cả ba seed (0,651 lên 0,963; 0,506 lên 0,885; 0,650 lên 0,965), và số offline của đồ thị toàn kỳ đẹp hơn số phục vụ ở cả ba, nhưng mức chênh dao động từ 0,053 đến 0,096. Số nhóm cấu kết thật là 15, 14, 14 và RULE đứng hạng 1, 9, 1 trên train: cùng một luật, độ nhạy cao.

## Quy mô: khi 730 triệu cạnh không vừa bộ nhớ

networkx giữ đồ thị bằng dict thuần Python: tiện cho thí nghiệm, nặng cho quy mô. Lab dựng đồ thị thô (mọi cặp khách–tài xế) ở bốn độ dài cửa sổ tính đến ngày 58, đúng như `build_graph` (chuỗi ID, thuộc tính `weight`, `kind`), đo thời gian và bộ nhớ Python (`tracemalloc`, chuỗi ID sinh trong phiên đo), rồi đo betweenness trên thành phần lớn nhất của ảnh chụp ngày 58:

```text
đồ thị dựng như build_graph (chuỗi ID, thuộc tính trên cạnh và nút), đo bộ nhớ bằng tracemalloc:
      đồ thị      cạnh      nút  dựng (giây)  thành phần (giây)  byte/cạnh  giữ lại
  thô 7 ngày    22,933   15,168        0.037              0.012        583     3.4%
 thô 14 ngày    45,065   22,519        0.552              0.321        535     4.2%
 thô 28 ngày    88,238   31,102        0.168              0.046        465     5.6%
 thô 56 ngày   171,486   39,717        0.805              0.084        416     7.1%
 lọc 28 ngày     4,913    5,165        0.039                  -        724   (giây dựng đo trong phiên tracemalloc)
ngoại suy tuyến tính, KHÔNG đo: cửa sổ 28 ngày ở 2 triệu chuyến/ngày tối đa 56 triệu cạnh ≈ 22 đến 24 GiB; 730 triệu cạnh/năm ≈ 283 đến 316 GiB (byte/cạnh của đồ thị thô 56 và 28 ngày)
đồ thị đã lọc: 3,118,022 cạnh sau lọc ở 56 triệu cạnh thô (giữ 5.6%) x 724 byte ≈ 2.1 GiB
tỉ lệ cạnh giữ lại theo cửa sổ: 7 ngày 3.4%, 14 ngày 4.2%, 28 ngày 5.6%, 56 ngày 7.1%
betweenness trên thành phần lớn nhất (472 nút, 514 cạnh): chính xác 0.98 giây, lấy mẫu k=50 0.35 giây; trùng 18/20 nút đứng đầu
```

Thời gian đo khi máy còn bận (load average 58,69 trên 10 lõi) nên chỉ để biết cỡ. Bộ nhớ là 416 đến 583 byte mỗi cạnh của đồ thị thô và 724 của đồ thị đã lọc. Bản đo đầu của lab chỉ đếm cấu trúc, không đếm chuỗi ID và thuộc tính, và báo 170 byte, thấp hơn thực tế gần ba lần. Ngoại suy (phép nhân, không phải phép đo): 2 triệu chuyến mỗi ngày cho cửa sổ 28 ngày tối đa 56 triệu cạnh, 22 đến 24 GiB; 730 triệu cạnh của cả năm, 283 đến 316 GiB; đồ thị đã lọc ở 56 triệu cạnh thô còn khoảng 3,1 triệu cạnh, cỡ 2,1 GiB.

Ba hướng thu nhỏ, theo thứ tự nên thử: lọc bằng polars trước networkx (giữ lại 3,4 đến 7,1 % tùy cửa sổ; tỉ lệ đó có giữ ở quy mô lớn hay không là giả định chưa kiểm), dựng riêng từng thành phố, và khi đồ thị không vừa một máy thì dùng cơ sở dữ liệu đồ thị (graph database) hay engine phân tán (lab không đo hướng này). Betweenness chính xác đắt nhất (thuật toán Brandes, O(V·E)): thành phần lớn nhất chỉ 472 nút đã mất 0,98 giây; lấy mẫu 50 nút nguồn mất 0,35 giây nhưng chỉ giữ 18 trong 20 nút đứng đầu, nên lab chỉ tính betweenness trong từng nhóm đã tách.

## Chạy và kiểm chứng

Cần uv và Python 3.14, không cần Docker; lần đầu cần mạng để tải numpy, polars, scikit-learn, networkx. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/58-networkx-graph-cau-ket examples/toi-noi-python/58-networkx-graph-cau-ket/main.py
uv run --project examples/toi-noi-python/58-networkx-graph-cau-ket examples/toi-noi-python/58-networkx-graph-cau-ket/main.py --verify
```

Hai hình mẫu dựng bằng tay (đáp án bài tập 1) và dòng cuối của lần chạy đầy đủ:

```text
Hai hình mẫu dựng bằng tay (không qua dữ liệu):
  2 tài xế x 3 khách, mỗi cặp 2 chuyến: 5 nút, 6 cạnh, mật độ 0.60, chu trình 2, betweenness lớn nhất 0.25 -> bị cờ: True
  1 tài xế x 5 khách, mỗi cặp 4 chuyến: 6 nút, 5 cạnh, mật độ 0.33, chu trình 0, betweenness lớn nhất 1.00 -> bị cờ: False

Kiểm chứng: OK 53/53
```

`--verify` in `OK 53/53`, kiểm bất biến chứ không kiểm tốc độ: thành phần khổng lồ của đồ thị thô, lọc cạnh dưới 10 %, ảnh chụp không đổi khi cắt event, bỏ cột nhãn hay đổi `PYTHONHASHSEED`, `g_edge_trips` khớp phép đếm lại độc lập, luật cố định là luật tốt nhất của lưới trên train, và ở cả ba seed đặc trưng đồ thị nâng `collusion`, đồ thị toàn kỳ đẹp offline hơn phục vụ. Lần chạy `--verify` cuối: 58 phút 8 giây (load average 279,91 327,13 347,87 lúc kết thúc).

| Hiện tượng | Kiểm tra trước |
|---|---|
| Một thành phần liên thông chứa hàng nghìn nút | Còn cạnh yếu (cặp 1 chuyến) hoặc thiết bị chung của cả quán; tăng `STRONG_MIN_TRIPS`, tách bằng Louvain |
| Hai lần dựng cho nhóm khác nhau | Thứ tự thêm cạnh (`.sort(...)`), `seed` Louvain, đồ thị con dựng từ `set` (`PYTHONHASHSEED`) |
| PR-AUC offline đẹp, chạy thật tụt | Phép thử cắt cụt; đồ thị dựng một lần cho cả kỳ; cửa sổ kết thúc sau thời điểm chấm |
| Nhóm bị cờ toàn khách quen hợp lệ | `cycle_rank`, `top_betweenness`; thiết bị và độ mới của tài khoản |
| Hết bộ nhớ khi dựng đồ thị | Đếm cạnh trước `add_edges_from`; lọc bằng polars; dựng từng thành phố |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Dữ liệu là tổng hợp, và bộ sinh có hai tạo tác ảnh hưởng mọi valid của phần 07: (a) tỉ lệ chuyến sạch dùng `NEWUSER50` giảm từ 7,8 % (ngày 0 đến 9) xuống 0,9 % (ngày 50 đến 59) vì quần thể khách gần như cố định; (b) chuyến gian lận không phải `promo_abuse` có mã khuyến mãi ở 0,0 % trường hợp, nên đặc trưng như `cust_promo_share_30d` lớn hơn 0 tố «sạch» dễ hơn ngoài đời. Chỉ có 15 nhóm cấu kết, do chính ta sinh theo một khuôn (2 đến 3 tài xế, khách mới, thiết bị dùng chung), một nhóm bị cắt cụt: «bắt được 12» có khoảng tin cậy rộng và luật chọn trên chúng sẽ không giữ nguyên với cấu kết thật. Precision đếm theo nhóm-ngày. Cấu kết bắt đầu trong ngày chỉ được thấy từ hôm sau; đồ thị nằm trong RAM của một tiến trình; lab dùng nhãn cuối, ngoài đời nhãn đến trễ hàng tuần. Đồ thị tài khoản–thiết bị là dữ liệu cá nhân, cần cơ sở pháp lý và retention riêng. Nhóm bị cờ là gợi ý cho người duyệt, không phải kết luận, và không tự khóa tài khoản.</p>
</div>

**Bài tập:** (1) Hai tài xế và ba khách, mỗi khách đi mỗi tài xế đúng 2 chuyến trong 28 ngày, không thiết bị chung: tính số nút, cạnh, chu trình độc lập, mật độ; `RULE` có gắn cờ không, và điều đó nói gì về luật? (2) Với 2 triệu chuyến mỗi ngày, cửa sổ 28 ngày, mỗi chuyến một cặp mới, 465 byte mỗi cạnh (đồ thị thô 28 ngày của lab), networkx cần bao nhiêu GiB; nếu polars giữ 5,6 % cạnh với 724 byte mỗi cạnh thì sao?

<details><summary>Xem lời giải</summary><p>(1) 5 nút, 6 cạnh, chu trình độc lập 6 − 5 + 1 = <b>2</b>, mật độ 6 / 10 = <b>0,6</b>, betweenness lớn nhất 0,25. Cả năm điều kiện của <code>RULE</code> đều đạt nên nhóm <b>bị gắn cờ</b> (lab dựng đúng ca này, kết quả ở trên), dù đó cũng có thể là hai tài xế có ba khách quen chung hợp lệ: luật chỉ nói «hãy xem», phân biệt cần tín hiệu khác. Hình sao 1 tài xế × 5 khách có 0 chu trình nên không bị cờ. (2) 56 triệu cạnh × 465 byte ≈ 2,6 × 10<sup>10</sup> byte, khoảng <b>24 GiB</b>; giữ 5,6 % còn khoảng 3,14 triệu cạnh × 724 byte ≈ <b>2,1 GiB</b>, với điều kiện dữ liệu thật cũng giữ tỉ lệ cạnh lặp lại như dữ liệu tổng hợp và byte mỗi cạnh không đổi khi đồ thị dày hơn.</p></details>

Bài trước: [bài 57](/articles/python-scikit-learn-lightgbm-imbalanced-cross-validation). Bài tiếp: [bài 59](/articles/python-real-time-scoring-feature-store-redis-latency).

## Tài liệu gốc

- [NetworkX documentation](https://networkx.org/documentation/stable/): tài liệu chính thức, gồm mô hình đồ thị và danh sách thuật toán.
- [`connected_components`](https://networkx.org/documentation/stable/reference/algorithms/generated/networkx.algorithms.components.connected_components.html) và [`core_number`](https://networkx.org/documentation/stable/reference/algorithms/generated/networkx.algorithms.core.core_number.html).
- [`betweenness_centrality`](https://networkx.org/documentation/stable/reference/algorithms/generated/networkx.algorithms.centrality.betweenness_centrality.html): tham số `k` và `seed` cho bản lấy mẫu.
- [`louvain_communities`](https://networkx.org/documentation/stable/reference/algorithms/generated/networkx.algorithms.community.louvain.louvain_communities.html): phát hiện cộng đồng, tham số `seed`.
- [`tracemalloc`](https://docs.python.org/3.14/library/tracemalloc.html): đo bộ nhớ do Python cấp phát.
- [scikit-learn `average_precision_score`](https://scikit-learn.org/stable/modules/generated/sklearn.metrics.average_precision_score.html): định nghĩa PR-AUC dạng average precision dùng trong bài.
