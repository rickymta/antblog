# Lab 51 · Vector search với pgvector và FAISS

Đi kèm bài `python-vector-search-pgvector-faiss-trung-tai-xe`. Có thêm bước 3b: bẫy `WHERE` với chỉ mục gần đúng (iterative scan, giới hạn `hnsw.max_scan_tuples`, HNSW một phần, B-tree). Lưu embedding khuôn mặt trong PostgreSQL (`lab51.driver_faces`, cột `vector(128)`), tìm top-10 «cùng người khác tài khoản» bằng pgvector (quét tuần tự, HNSW, IVFFlat) và FAISS (`IndexFlatIP`, `IndexHNSWFlat`), rồi so recall, độ trễ và kích thước chỉ mục với đáp án chính xác từ NumPy.

**Phương án embedding (tự chứa, không dùng lab 49):** PCA 128 chiều fit trên 200 ảnh Olivetti (ảnh 0-4 của mỗi người), chuẩn hóa L2. Đây KHÔNG phải mô hình nhận diện khuôn mặt hiện đại; nó chỉ cho vector có cấu trúc thật. **Tài khoản nền** (mặc định 100.000) là vector TỔNG HỢP lấy mẫu Gaussian cùng trung bình/hiệp phương sai với embedding thật, chỉ để đủ quy mô đo; chúng không phải người thật. Bộ khuôn mặt Olivetti do AT&T Laboratories Cambridge thu thập; xem tài liệu scikit-learn. Không có ảnh người nào được commit.

- `embedding.py`: Olivetti → PCA → vector; `synthetic_distractors`.
- `store.py`: kết nối, schema `lab51`, `COPY` nhị phân, truy vấn `<=>`, dựng HNSW / IVFFlat, `EXPLAIN`.
- `ann.py`: NumPy chính xác, FAISS, recall, thời gian từng truy vấn.
- `main.py`: demo 5 bước và `--verify`; tùy chọn `--distractors N`, `--keep`.

## Chạy

Cần hạ tầng chung (PostgreSQL có pgvector, xem `../infra/README.md`) và **mạng ở lần chạy đầu** (scikit-learn tải Olivetti 4,26 MB (bản cache 1,4 MB); uv tải faiss-cpu ~5 MB). Từ thư mục gốc repository:

```bash
docker compose -f examples/toi-noi-python/infra/compose.yml up -d --wait
uv run --project examples/toi-noi-python/51-vector-search-pgvector-faiss examples/toi-noi-python/51-vector-search-pgvector-faiss/main.py
uv run --project examples/toi-noi-python/51-vector-search-pgvector-faiss examples/toi-noi-python/51-vector-search-pgvector-faiss/main.py --verify
```

Lab đo với một luồng ở mọi phía (`max_parallel_workers_per_gather = 0`, `omp_set_num_threads(1)`, `threadpool_limits(1)` cho NumPy). Lab tạo schema `lab51` và xóa khi xong (trừ `--keep`). Không nối được PostgreSQL thì `--verify` in `FAIL: không nối được PostgreSQL tại <url> (docker compose ...)` và thoát mã 1. Một lượt mặc định mất cỡ hai đến ba phút (dựng HNSW cho 100.200 dòng chiếm phần lớn); `--distractors 5000` mất cỡ nửa phút.

Dựng chỉ mục đặt `max_parallel_maintenance_workers = 0`: `/dev/shm` của container PostgreSQL mặc định chỉ 64 MB, và khi thử dựng song song với `maintenance_work_mem = 512MB` PostgreSQL báo «No space left on device».

## Kết quả kỳ vọng

- Lặp lại được: số dòng, recall của quét chính xác, FAISS và HNSW pgvector (`ef_search=40` từ 0,968 đến 0,971). **Dao động**: recall của IVFFlat đổi theo từng lần dựng vì k-means chọn tâm ngẫu nhiên (`probes=1` từ 0,39 đến 0,45, `probes=10` từ 0,963 đến 0,978, `top1` cũng đổi), và số dòng 'bắt' ở bước 3b có thể lệch 1. **Thời gian không ổn định**: p50 của quét tuần tự pgvector đo được 15 đến 36 ms qua các lần chạy lúc load average từ 22 đến hơn 130 (lúc chưa ép một luồng), 36,0 và 36,6 ms ở hai lần cuối (một luồng). Chỉ thứ bậc lớn đáng tin (FAISS HNSW trong tiến trình vài chục micro giây đến dưới một mili giây; pgvector vài mili giây đến hơn chục; quét tuần tự cỡ ba chục).
- `--verify` kết thúc bằng `OK 27/27` (exit code 0) với cấu hình mặc định; lệch thì in `SAI: <tên ca>` rồi `FAIL n/27` với n là số ca đạt. Với bảng nhỏ (ví dụ `--distractors 5000`) PostgreSQL tự chọn quét tuần tự cho truy vấn có `WHERE`, nên bảy ca về bẫy lọc bị bỏ qua và tổng là 20. Không ca nào dùng ngưỡng tốc độ.
- Đã chạy với Python 3.14.7, faiss-cpu 1.15.1, psycopg 3.3.8, pgvector (Python) 0.5.1, scikit-learn 1.9.1, numpy 2.5.4; PostgreSQL 17.11 với extension pgvector 0.8.6 (`pgvector/pgvector:pg17`).
