---
slug: python-vector-search-pgvector-faiss-trung-tai-xe
title: "Vector search với pgvector và FAISS: phát hiện một người nhiều tài khoản"
summary: "Bài 51: lưu embedding khuôn mặt trong PostgreSQL pgvector và FAISS, tìm top-k cùng người khác tài khoản. So HNSW, IVFFlat về recall, độ trễ, kích thước; bẫy khi kèm WHERE. Có số đo thật."
category: python-advanced
level: advanced
tags: [python, toi-noi, pgvector, faiss, vector-search]
language: vi
---

[Tới Nơi](/articles/du-an-mau-toi-noi) có 60.000 tài xế đăng ký, và một tài xế bị khoá vì gian lận quay lại sau vài tuần, đăng ký bằng số điện thoại và CCCD khác. Không ai nhớ nổi 60.000 gương mặt, nhưng một cơ sở dữ liệu thì hỏi được: trong các tài khoản đã có, ảnh nào gần ảnh này nhất? Bài này lưu embedding khuôn mặt và trả lời câu hỏi đó bằng pgvector trong PostgreSQL và bằng FAISS, đo xem chỉ mục gần đúng đổi độ chính xác lấy thời gian thế nào, và chỉ ra chỗ nó trả thiếu mà không báo lỗi.

Đây là bài 51 của lộ trình **Python nâng cao qua Tới Nơi**. Bạn cần biết embedding và cosine ([bài 49](/articles/python-face-embedding-cosine-threshold-far-frr)) và cách làm việc với PostgreSQL qua psycopg ([bài 24](/articles/python-sqlalchemy-psycopg-postgresql-mapping-migration)).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Tìm 10 vector gần nhất trong 100.000 vector 128 chiều bằng NumPy: vét cạn, không chỉ mục, mỗi lần một truy vấn. Mất cỡ micro giây, mili giây hay giây?</p>
<details><summary>Xem đáp án</summary><p><b>Cỡ mili giây.</b> Lab đo p50 là 2,76 ms (một luồng BLAS) trên một máy đang rất bận; các lần chạy khác cho 1,97 đến 2,88 ms. Ở quy mô này chỉ mục gần đúng không phải để cứu thời gian; lý do dùng pgvector là dữ liệu đã nằm trong PostgreSQL, và lý do dùng chỉ mục là khi số dòng lớn hơn nhiều. Phần so sánh bên dưới cho thấy chỉ mục còn làm gì cho recall.</p></details>
</div>

## Từ khuôn mặt tới vector: phương án embedding của lab

Lab này tự chứa, không dùng kết quả lab 49. Embedding là **PCA** (kiểu «eigenfaces») 128 chiều, fit trên 200 ảnh Olivetti (ảnh 0 đến 4 của mỗi người trong 40 người), rồi chuẩn hoá L2. Đây không phải mô hình nhận diện khuôn mặt hiện đại như InsightFace; nó chỉ cho vector có cấu trúc thật để tìm kiếm. Độ chính xác nhận diện trong bài là của PCA. Bộ khuôn mặt Olivetti do AT&T Laboratories Cambridge thu thập; xem tài liệu scikit-learn (tải về 4,26 MB, bản cache 1,4 MB). Chỉ với 200 tài khoản thật, tìm chính xác cho `top1` cùng người là 0,855.

Năm ảnh đầu mỗi người là 5 «tài khoản» đã đăng ký; năm ảnh còn lại là 200 truy vấn, mỗi truy vấn là ảnh của một lần đăng ký mới cần kiểm tra trùng. Để có quy mô, lab thêm 100.000 **tài khoản nền** tổng hợp: vector Gaussian cùng trung bình và hiệp phương sai với embedding thật, chuẩn hoá L2. Chúng không phải người thật, chỉ để đủ số dòng đo thời gian và recall.

```python
# embedding.py
def synthetic_distractors(real: np.ndarray, n: int, seed: int = 51) -> np.ndarray:
    rng = np.random.default_rng(seed)
    mean = real.mean(axis=0)
    values, vectors = np.linalg.eigh(np.cov(real.T))
    root = vectors * np.sqrt(np.clip(values, 0, None))
    z = mean + rng.standard_normal((n, real.shape[1])).astype(np.float32) @ root.T.astype(np.float32)
    ...  # chuẩn hóa L2
```

Vì mọi vector có độ dài 1, cosine bằng tích vô hướng, và toán tử `<=>` của pgvector (khoảng cách cosine) bằng 1 trừ cosine; `--verify` kiểm đẳng thức này trên một truy vấn. Mã tài khoản dùng 6 chữ số (`DR-000001`) vì lab cần hơn trăm nghìn dòng, dài hơn `DR-001` của các bài trước.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 210" role="img" aria-label="Ảnh đăng ký mới thành embedding 128 chiều, đi vào pgvector hoặc FAISS để lấy top-10 kèm khoảng cách, rồi qua ngưỡng tới người duyệt."><defs><marker id="py51-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="45" width="140" height="120" rx="10" class="f-phys-soft s-phys"/><text x="34" y="78" class="t-b">Ảnh đăng ký</text><text x="34" y="104" class="t-small">ảnh 5-9 mỗi người</text><text x="34" y="124" class="t-small">64 x 64, xám</text><path d="M160 105 L190 105" class="f-none s-phys" stroke-width="2" marker-end="url(#py51-0-arrow)"/><rect x="190" y="45" width="140" height="120" rx="10" class="f-phys-soft s-phys"/><text x="204" y="78" class="t-b">Embedding</text><text x="204" y="104" class="t-small">PCA 128 chiều</text><text x="204" y="124" class="t-small">chuẩn hoá L2</text><path d="M330 105 L360 70" class="f-none s-phys" stroke-width="2" marker-end="url(#py51-0-arrow)"/><path d="M330 105 L360 150" class="f-none s-phys" stroke-width="2" marker-end="url(#py51-0-arrow)"/><rect x="360" y="25" width="210" height="72" rx="10" class="f-pk-soft s-pk"/><text x="374" y="55" class="t-b">pgvector (PostgreSQL)</text><text x="374" y="78" class="t-small">HNSW hoặc IVFFlat, có SQL</text><rect x="360" y="113" width="210" height="72" rx="10" class="f-pk-soft s-pk"/><text x="374" y="143" class="t-b">FAISS (trong bộ nhớ)</text><text x="374" y="166" class="t-small">Flat hoặc HNSW, không SQL</text><path d="M570 61 L600 95" class="f-none s-phys" stroke-width="2" marker-end="url(#py51-0-arrow)"/><path d="M570 149 L600 115" class="f-none s-phys" stroke-width="2" marker-end="url(#py51-0-arrow)"/><rect x="600" y="45" width="130" height="120" rx="10" class="f-phys-soft s-phys"/><text x="614" y="78" class="t-b">Top-10</text><text x="614" y="104" class="t-small">id và khoảng</text><text x="614" y="124" class="t-small">cách cosine</text><path d="M730 105 L750 105" class="f-none s-phys" stroke-width="2" marker-end="url(#py51-0-arrow)"/><rect x="750" y="45" width="110" height="120" rx="10" class="f-warn-soft s-warn"/><text x="764" y="78" class="t-b">Ngưỡng</text><text x="764" y="104" class="t-small">rồi người</text><text x="764" y="124" class="t-small">duyệt</text></svg>
</div>
<figcaption>Hình 1. Đường đi của một lần đăng ký mới. Tìm kiếm chỉ cho ra ứng viên; quyết định thuộc về ngưỡng và người duyệt.</figcaption>
</figure>

## pgvector và FAISS: hai cách dựng chỉ mục

**pgvector** thêm kiểu `vector(N)` và các toán tử khoảng cách vào PostgreSQL. Bảng trong lab (schema `lab51`) có cột `person` là nhãn kiểm tra của lab (không có trong production) và cột `status` cho tài khoản đã bị khoá:

```python
# store.py
conn.execute(
    f"""CREATE TABLE {TABLE} (
        driver_id text PRIMARY KEY,
        person    int,                  -- nhãn kiểm tra của lab (Olivetti); NULL với tài khoản nền
        status    text NOT NULL,        -- 'active' hoặc 'banned' (tài khoản đã bị khóa)
        embedding vector({dim}) NOT NULL
    )"""
)
TOPK_SQL = f"SELECT driver_id, embedding <=> %s AS dist FROM {TABLE} ORDER BY embedding <=> %s LIMIT %s"
```

Nạp 100.200 dòng bằng `COPY` nhị phân mất 3,0 s, truy vấn là một câu `ORDER BY … LIMIT` thông thường. Không có chỉ mục, PostgreSQL quét tuần tự và cho đáp án chính xác. Có hai loại chỉ mục gần đúng:

- **HNSW**: đồ thị nhiều tầng, mỗi vector có tối đa `m` kết nối ở mỗi tầng. Khi dựng, `ef_construction` quyết định chất lượng đồ thị; khi truy vấn, `hnsw.ef_search` (mặc định 40) là số ứng viên được xét, lớn hơn thì chính xác hơn và chậm hơn.
- **IVFFlat**: chia vector vào `lists` ô bằng k-means, truy vấn chỉ xét `ivfflat.probes` ô gần nhất (mặc định 1). Phải dựng sau khi đã có dữ liệu; tài liệu pgvector gợi ý khởi đầu `rows / 1000` ô cho tới 1 triệu dòng.

```sql
-- SQL mà store.py gửi tới PostgreSQL (build_hnsw, build_ivfflat)
CREATE INDEX driver_faces_hnsw ON lab51.driver_faces USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 64)
CREATE INDEX driver_faces_ivf ON lab51.driver_faces USING ivfflat (embedding vector_cosine_ops)
    WITH (lists = 100)
```

Một bẫy hạ tầng đo được khi viết lab: dựng chỉ mục song song với `maintenance_work_mem = 512MB` làm PostgreSQL báo `No space left on device`, vì `/dev/shm` của container Docker mặc định chỉ 64 MB. Cách sửa gốc là tăng `/dev/shm` của container (tuỳ chọn `--shm-size` hoặc `shm_size` trong compose, lớn hơn `maintenance_work_mem`); lab không đổi compose dùng chung nên chọn cách lách `max_parallel_maintenance_workers = 0` (xem `prepare_index_build`), chậm hơn dựng song song.

**FAISS** là thư viện chạy trong tiến trình Python, không phải cơ sở dữ liệu: không SQL, không transaction, không lọc theo cột, lưu ra đĩa chỉ khi ta tự gọi. Đổi lại nó nhanh. Với vector đã chuẩn hoá, độ đo tích vô hướng cho cùng thứ hạng cosine:

```python
# ann.py
index = faiss.IndexHNSWFlat(base.shape[1], m, faiss.METRIC_INNER_PRODUCT)
index.hnsw.efConstruction = ef_construction
index.add(base)
# truy vấn: index.hnsw.efSearch = ef; index.search(q[None], 10)
```

## So sánh: recall, độ trễ, kích thước

Mỗi cấu hình trả lời 200 truy vấn, từng truy vấn một như yêu cầu đăng ký thật. **recall@10** là tỉ lệ top-10 của cấu hình trùng với top-10 chính xác tính bằng NumPy. Lab chạy với PostgreSQL 17.11 và pgvector 0.8.6 trong Docker, mọi phía một luồng tính toán: `max_parallel_workers_per_gather = 0` cho PostgreSQL, `omp_set_num_threads(1)` cho faiss-cpu 1.15.1, `threadpool_limits(1)` cho BLAS của NumPy. Output (trích):

```text
  cấu hình                            p50 ms  p95 ms recall@10 found@10    top1
  NumPy brute force (1 luồng BLAS)      2.76   12.01     1.000    0.299   0.670
  pgvector quét tuần tự (chính xác)    35.98  104.50     1.000    0.299   0.670
  [pgvector IVFFlat lists=100: dựng 2.0 s, chỉ mục 55.2 MB; kế hoạch: Index Scan using driver_faces_ivf on driver_faces]
  pgvector IVFFlat probes=1             4.49   15.54     0.425    0.139   0.410
  pgvector IVFFlat probes=10            6.69   16.44     0.970    0.287   0.635
  pgvector IVFFlat probes=100 (hết)    33.20   65.21     1.000    0.299   0.670
  [pgvector HNSW m=16 ef_construction=64: dựng 49.3 s, chỉ mục 83.4 MB; kế hoạch: Index Scan using driver_faces_hnsw on driver_faces]
  pgvector HNSW ef_search=10            1.68    3.28     0.819    0.248   0.530
  pgvector HNSW ef_search=40            3.30    5.21     0.970    0.282   0.615
  pgvector HNSW ef_search=400           9.31   14.10     0.999    0.297   0.660
  FAISS IndexFlatIP (chính xác)         1.66    4.31     1.000    0.299   0.670
  [FAISS HNSW M=16 efConstruction=64: dựng 23.4 s, cỡ 65.8 MB khi tuần tự hóa]
  FAISS HNSW efSearch=10                0.03    0.04     0.763    0.249   0.550
  FAISS HNSW efSearch=40                0.07    0.15     0.945    0.276   0.620
  FAISS HNSW efSearch=400               0.71    1.31     0.998    0.297   0.660
  load average lúc kết thúc phép đo 137.40 151.23 180.93
```

Cách đọc. **Recall đổi bằng thời gian ở cả hai họ chỉ mục**: HNSW pgvector từ 0,819 (`ef_search=10`) lên 0,970 (40) rồi 0,999 (400); IVFFlat từ 0,425 (`probes=1`) lên 0,970 (10) rồi 1,000 khi quét hết 100 ô, lúc này nó chậm cỡ quét tuần tự (33,20 so với 35,98 ms). **Chi phí dựng và kích thước khác hẳn**: IVFFlat dựng trong 2,0 s và nặng 55,2 MB, HNSW dựng 49,3 s và nặng 83,4 MB; qua các lần chạy HNSW luôn lâu hơn IVFFlat hàng chục lần. **Thứ bậc thời gian**: FAISS HNSW trong tiến trình cỡ vài chục micro giây đến dưới một mili giây, pgvector (HNSW, IVFFlat) cỡ vài đến hơn chục mili giây, quét tuần tự cỡ ba chục mili giây. Khoảng cách với pgvector gồm cả vòng đi về qua mạng và giải mã kết quả, nên đây không phải so thuật toán thuần.

**Mức dao động giữa các lần chạy.** Recall của HNSW pgvector gần như đứng yên (`ef_search=40` từ 0,968 đến 0,971 qua các lần chạy); FAISS và quét chính xác hoàn toàn lặp lại. IVFFlat thì không: k-means của nó chọn tâm ngẫu nhiên nên mỗi lần dựng ra một phân ô khác, `probes=1` có recall từ 0,39 đến 0,45 và `probes=10` từ 0,963 đến 0,978, `top1` cũng dịch theo. Về thời gian, máy lúc đo có load average hơn 100: quét tuần tự pgvector đo được 36,0 và 36,6 ms ở hai lần chạy cuối (một luồng), còn 15 đến 36 ms ở các lần chạy trước khi tắt song song; HNSW `ef_search=40` từ 1,9 đến 6,5 ms. Ta chỉ khẳng định thứ bậc lớn; IVFFlat `probes=10` và HNSW `ef_search=40` cùng recall cỡ 0,97 và thời gian chồng lên nhau nên không xếp hạng. NumPy vét cạn (2,76 ms) cùng bậc với FAISS `IndexFlatIP` (1,66 ms): ở 100.000 vector, vét cạn trong bộ nhớ là một đối thủ nghiêm túc.

Cột `found@10` là chỉ số khác hẳn: tỉ lệ 5 tài khoản cùng người nằm trong top-10. Chỉ với 200 tài khoản thật, tìm chính xác cho 0,668 và `top1` 0,855; thêm 100.000 tài khoản nền, ngay cả đáp án chính xác (NumPy, quét tuần tự, `probes=100`) chỉ còn 0,299 và 0,670. Chỉ mục gần đúng chỉ tiến sát đáp án chính xác (recall → 1), không vượt được nó. Phần còn lại phụ thuộc chất lượng embedding, tức là bài 49.

## Lọc kèm vector search: bẫy WHERE

Bài toán thật của Tới Nơi hẹp hơn «ai giống ai»: đối chiếu ảnh đăng ký mới với danh sách **tài khoản đã bị khoá**. Lab đánh dấu 1,0% số dòng (1.020 dòng) là `banned`, gồm toàn bộ tài khoản thật của 4 người đầu, và so khớp 200 truy vấn chỉ trong nhóm này:

```sql
SELECT driver_id, embedding <=> %s AS dist FROM lab51.driver_faces
WHERE status = 'banned' ORDER BY embedding <=> %s LIMIT 10
```

Với chỉ mục gần đúng, điều kiện `WHERE` được áp *sau khi* quét chỉ mục: HNSW lấy `ef_search` ứng viên gần nhất rồi mới lọc. Nếu 1% ứng viên thoả điều kiện thì 40 ứng viên chỉ còn khoảng 0,4 dòng. Từ pgvector 0.8.0 có `hnsw.iterative_scan` để quét tiếp chỉ mục khi chưa đủ dòng, nhưng không quét vô hạn: nó dừng ở `hnsw.max_scan_tuples` (mặc định 20.000) và ở giới hạn bộ nhớ `hnsw.scan_mem_multiplier` (bội số của `work_mem`, mặc định 1).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Mặc định: chỉ mục trả 40 ứng viên, WHERE giữ khoảng một phần trăm, kết quả trung bình 0,61 dòng. Với iterative scan: quét theo lô, lọc rồi quét tiếp cho tới khi đủ dòng hoặc chạm giới hạn hnsw.max_scan_tuples, kết quả 10 dòng."><defs><marker id="py51-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="22" class="t-b">Mặc định</text><rect x="20" y="34" width="250" height="84" rx="10" class="f-pk-soft s-pk"/><text x="36" y="62" class="t-b">Quét chỉ mục</text><text x="36" y="86" class="t-small">lấy 40 ứng viên gần nhất</text><text x="36" y="104" class="t-small">hnsw.ef_search = 40</text><path d="M270 76 L315 76" class="f-none s-phys" stroke-width="2" marker-end="url(#py51-1-arrow)"/><rect x="315" y="34" width="250" height="84" rx="10" class="f-pk-soft s-pk"/><text x="331" y="62" class="t-b">Lọc sau</text><text x="331" y="86" class="t-small">WHERE status = 'banned'</text><text x="331" y="104" class="t-small">giữ khoảng 1% ứng viên</text><path d="M565 76 L610 76" class="f-none s-phys" stroke-width="2" marker-end="url(#py51-1-arrow)"/><rect x="610" y="34" width="250" height="84" rx="10" class="f-warn-soft s-warn"/><text x="626" y="62" class="t-b">Kết quả</text><text x="626" y="86" class="t-small">trung bình 0,61 dòng</text><text x="626" y="104" class="t-small">yêu cầu 10 dòng, recall 0,061</text><text x="20" y="144" class="t-b">Có hnsw.iterative_scan = relaxed_order</text><rect x="20" y="156" width="250" height="84" rx="10" class="f-pk-soft s-pk"/><text x="36" y="184" class="t-b">Quét chỉ mục</text><text x="36" y="208" class="t-small">lấy ứng viên theo từng lô</text><text x="36" y="226" class="t-small">tiếp tục, có giới hạn</text><path d="M270 198 L315 198" class="f-none s-phys" stroke-width="2" marker-end="url(#py51-1-arrow)"/><rect x="315" y="156" width="250" height="84" rx="10" class="f-pk-soft s-pk"/><text x="331" y="184" class="t-b">Lọc rồi lặp</text><text x="331" y="208" class="t-small">thiếu 10 dòng thì quét</text><text x="331" y="226" class="t-small">thêm lô kế tiếp</text><path d="M565 198 L610 198" class="f-none s-phys" stroke-width="2" marker-end="url(#py51-1-arrow)"/><rect x="610" y="156" width="250" height="84" rx="10" class="f-phys-soft s-phys"/><text x="626" y="184" class="t-b">Kết quả</text><text x="626" y="208" class="t-small">10,00 dòng, recall 0,996</text><text x="626" y="226" class="t-small">thứ tự có thể lệch nhẹ</text></svg>
</div>
<figcaption>Hình 2. Cùng một câu truy vấn có WHERE, hai hành vi. Kế hoạch thực thi dùng chỉ mục trong cả hai trường hợp, nên không có cảnh báo nào.</figcaption>
</figure>

```text
== 3b. Kèm WHERE status = 'banned' (1020 dòng, 1.0%), yêu cầu top-10; 'bắt' = số trong 20 truy vấn của người bị khóa có tài khoản cùng người trong top-10
  cấu hình                                             dòng TB  recall@10    bắt   p50 ms
  quét tuần tự (chính xác)                               10.00      1.000  16/20    22.97
  HNSW ef_search=40, mặc định (lọc sau khi quét chỉ mục)    0.61      0.061  10/20     3.37
    + hnsw.iterative_scan = relaxed_order                10.00      0.996  16/20    17.11
    + hnsw.iterative_scan = strict_order                 10.00      0.971  14/20    21.01
    + relaxed_order, hnsw.max_scan_tuples = 200 (mặc định 20000)    8.80      0.501  12/20     4.52
  B-tree trên status, rồi sắp xếp chính xác              10.00      1.000  16/20     4.72
  HNSW một phần WHERE status='banned' (dựng 0.31 s, 0.85 MB)   10.00      1.000  16/20     2.68
  (đối chứng) HNSW mặc định, WHERE status = 'active' (99%)   10.00      0.966      -     3.85
  (kế hoạch với HNSW toàn bảng: Index Scan using driver_faces_hnsw on driver_faces; B-tree: Bitmap Heap Scan on driver_faces; một phần: Index Scan using driver_faces_hnsw_banned on driver_faces)
```

Cách đọc (hai lần chạy cuối đều cho 10 trên 20 ở hàng HNSW mặc định; các lần chạy trước cho 9 hoặc 10). HNSW mặc định trả trung bình 0,61 dòng thay vì 10 (kỳ vọng ngây thơ 40 × 1% = 0,4), recall 0,061, và bắt được người bị khoá ở 10 trên 20 truy vấn của họ, trong khi tìm chính xác bắt 16 trên 20. Bật `iterative_scan` đưa lại 10,00 dòng với recall 0,996 (`relaxed_order`) hoặc 0,971 (`strict_order`, đúng thứ tự, bắt 14 trên 20). Giới hạn là có thật: với `hnsw.max_scan_tuples = 200` chỉ còn 8,80 dòng và recall 0,501. Nhưng với bài toán này iterative scan không phải khuyến nghị chính. Danh sách khoá nhỏ và ổn định, nên chỉ mục hợp với đúng tập đó tốt hơn: **HNSW một phần** (`CREATE INDEX … WHERE status = 'banned'`) dựng 0,31 s, trả đủ 10 dòng với recall 1,000; hoặc **B-tree trên `status` rồi sắp xếp chính xác** (1.020 dòng là rẻ) cũng recall 1,000. Iterative scan hợp hơn khi điều kiện lọc đa dạng đến mức không dựng nổi chỉ mục riêng.

Muốn thứ tự chặt với `relaxed_order`, README của pgvector gợi ý CTE `AS MATERIALIZED` rồi `ORDER BY distance + 0` (`+ 0` cần cho PostgreSQL 17 trở lên). Điều kiện kém chọn lọc thì không sao: hàng đối chứng `status = 'active'` (99% số dòng) với HNSW mặc định vẫn trả đủ 10,00 dòng, recall 0,966 so với đáp án chính xác trên tập đó. Cái bẫy là điều kiện hẹp mà không ai kiểm: kế hoạch thực thi dùng `Index Scan` trong cả hai trường hợp nên không có cảnh báo, lỗi không ném exception, chỉ làm hệ thống bỏ sót người. Ngược lại, với bảng nhỏ hoặc điều kiện rất hẹp, planner có thể tự chọn `Seq Scan` (lab gặp ở `--distractors 5000`) và kết quả lại chính xác.

## Từ top-k tới quyết định: ngưỡng và người duyệt

Top-k luôn trả đủ k dòng dù không dòng nào đáng nghi, nên phải có ngưỡng cosine. Bước 4 của lab đếm, trên 200 truy vấn và tìm chính xác:

```text
   ngưỡng   tìm được cùng người    tài khoản nền vượt ngưỡng/truy vấn  người khác (thật)/truy vấn
      0.5                 0.595                                2967.5                        6.50
      0.7                 0.324                                 129.6                        0.64
      0.8                 0.197                                   4.9                        0.12
```

Ở ngưỡng 0,7 chỉ còn tìm được 0,324 số tài khoản cùng người, mà vẫn có trung bình 129,6 tài khoản nền vượt ngưỡng mỗi truy vấn. Đừng đọc số này như tỉ lệ chấp nhận nhầm của hệ thống thật: 100.000 vector nền là Gaussian cùng phân bố với PCA 128 chiều, chưa đo với embedding thật nên không biết chúng khó hay dễ hơn người thật. Điều đáng mang đi là cơ chế: quy mô nhân số ứng viên đáng ngờ, nên cần embedding tách người tốt hơn (bài 49) và ngưỡng chọn từ ROC trên dữ liệu thật, không thể chọn bằng cảm giác.

Quy trình hợp lý: lấy top-k, giữ các dòng vượt ngưỡng, đẩy sang hàng đợi người duyệt cùng ảnh và lý do, không tự động khoá hay từ chối. Khoảng cách cosine là một điểm số để người duyệt xem, không phải kết luận về danh tính. Vector khuôn mặt là dữ liệu sinh trắc, thuộc dữ liệu cá nhân nhạy cảm; lưu, xoá khi rút lại đồng ý và đánh giá theo nhóm là chủ đề của bài 53.

## Chạy và kiểm chứng

Cần hạ tầng chung (PostgreSQL có pgvector) và mạng ở lần chạy đầu để tải Olivetti (4,26 MB) và faiss-cpu. Từ gốc worktree:

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
uv run --project examples/toi-noi-python/51-vector-search-pgvector-faiss examples/toi-noi-python/51-vector-search-pgvector-faiss/main.py --verify
```

Lab tạo schema `lab51` và xoá khi chạy xong (chạy lại không để lại gì). Lần chạy cuối kết thúc bằng:

```text
OK 27/27
```

27 ca là bất biến, không có ngưỡng tốc độ. Các ca chính: kế hoạch thực thi thật sự dùng `driver_faces_hnsw` / `driver_faces_ivf` và quét tuần tự khi tắt chỉ mục; pgvector chính xác, FAISS flat và IVFFlat quét hết đều recall 1,000 so với NumPy; recall HNSW tăng theo `ef_search`; `probes=1` thấp hơn quét hết; các ca về `WHERE` (HNSW mặc định trả ít hơn nửa số dòng, `iterative_scan` trả đủ dòng nhưng bị `max_scan_tuples` chặn được, chỉ mục một phần và B-tree cho recall 1,000); thêm tài khoản nền làm giảm `found@10`; khoảng cách pgvector bằng 1 trừ tích vô hướng; schema bị xoá; và không nối được PostgreSQL thì in `FAIL: không nối được PostgreSQL tại <url> (docker compose …)` rồi thoát mã 1 (ca này chạy lại lab với URL sai).

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Truy vấn có `WHERE` trả ít hơn `LIMIT` dòng | `EXPLAIN` có `Index Scan` không; bật `hnsw.iterative_scan` hoặc dựng chỉ mục riêng cho tập con |
| Dựng chỉ mục lỗi `No space left on device` | `/dev/shm` của container; giảm `maintenance_work_mem` hoặc đặt `max_parallel_maintenance_workers = 0` |
| Recall IVFFlat thấp bất thường | Chỉ mục dựng khi bảng còn trống hay ít dữ liệu; `lists` và `probes` có hợp lý không |
| Có chỉ mục mà truy vấn vẫn quét tuần tự | Toán tử trong `ORDER BY` có khớp `vector_cosine_ops` không (`<=>` cho cosine) |
| Cùng truy vấn chậm nhanh thất thường | Máy bận (xem load average); đo nhiều lần, so thứ bậc chứ không so từng ms |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Embedding là PCA trên 200 ảnh, tài khoản nền là vector tổng hợp, nên mọi con số về nhận diện (<code>top1</code>, <code>found@10</code>, số tài khoản nền vượt ngưỡng) chỉ minh hoạ cơ chế, không phải độ chính xác của hệ thống. Dữ liệu 128 chiều với cấu trúc Gaussian có thể dễ hơn hoặc khó hơn vector 512 chiều từ mô hình thật cho chỉ mục gần đúng; ta chưa đo nên không khẳng định. Production còn phải lo ghi liên tục vào chỉ mục (IVFFlat cần dựng lại khi dữ liệu trôi), xoá vector khi người dùng rút lại đồng ý, mã hoá và phân quyền cột embedding, và đo trên dữ liệu thật theo từng nhóm người dùng.</p>
</div>

**Bài tập 1:** Chạy `main.py --distractors 5000`. So `found@10` của tìm chính xác và recall của HNSW `ef_search=10` với lần chạy 100.000. Chỉ mục gần đúng còn đáng để dựng không?

**Bài tập 2:** Lọc `status = 'active'` (99% số dòng) với HNSW mặc định có bị thiếu dòng như `banned` không? Đo bằng cách nào?

<details><summary>Xem lời giải</summary><p><b>Bài tập 1.</b> Với 5.200 dòng, tìm chính xác cho <code>found@10</code> 0,427 (so với 0,299 ở 100.200 dòng và 0,668 khi chỉ có 200 tài khoản thật): thêm càng nhiều tài khoản nền thì càng giảm đều. HNSW <code>ef_search=10</code> có recall 0,968 (so với 0,824 ở 100.200 dòng), nên chỉ mục gần đúng ít mất mát khi bảng nhỏ. Cũng ở 5.200 dòng (load average hơn 140 lúc đo), NumPy mất 0,09 ms, nhanh hơn cả pgvector quét tuần tự (3,28 ms) lẫn HNSW (2,47 đến 3,41 ms). Chỉ mục chưa cần thiết ở quy mô này. <b>Bài tập 2.</b> Dựng HNSW, đặt <code>hnsw.ef_search = 40</code>, chạy 200 truy vấn với <code>WHERE status = 'active'</code> và đếm số dòng trả về; lab đã có sẵn hàng đối chứng này: trung bình 10,00 dòng, recall 0,966 (bảng 100.200 dòng). Điều kiện giữ 99% ứng viên nên 40 ứng viên còn khoảng 39, đủ cho <code>LIMIT 10</code>. Chỉ điều kiện hẹp (như 1% ở trên) mới thiếu dòng.</p></details>

Bài trước: [bài 50](/articles/python-liveness-anti-spoofing-chat-luong-anh). Bài tiếp: [bài 52](/articles/python-face-match-grpc-service-batching-model-loading).

## Tài liệu gốc

- [pgvector: README (HNSW, IVFFlat, lọc, iterative scan)](https://github.com/pgvector/pgvector).
- [PostgreSQL 17: CREATE INDEX](https://www.postgresql.org/docs/17/sql-createindex.html).
- [psycopg 3: Using COPY TO and COPY FROM](https://www.psycopg.org/psycopg3/docs/basic/copy.html).
- [FAISS: các loại chỉ mục](https://github.com/facebookresearch/faiss/wiki/Faiss-indexes).
- [Malkov, Yashunin: Hierarchical Navigable Small World graphs](https://arxiv.org/abs/1603.09320).
- [scikit-learn: PCA](https://scikit-learn.org/stable/modules/generated/sklearn.decomposition.PCA.html).
