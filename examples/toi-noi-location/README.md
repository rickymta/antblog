# Service định vị bản đồ qua Tới Nơi

Khung biên tập và bản đồ lab của loạt bài. Danh sách 64 bài, «Cần chứng minh», phụ thuộc và trạng thái nằm ở [`docs/location-master-curriculum.md`](../../docs/location-master-curriculum.md). Hồ sơ hệ thống: bài `du-an-mau-toi-noi` và `docs/toi-noi-overall-architecture.md`.

## Câu hỏi xuyên suốt

Làm sao tổng quãng đường và thời gian của **cả khách lẫn tài xế** nhỏ nhất. Mỗi bài tối ưu phải nói rõ bên nào được lợi, bên nào chịu thiệt. Hàm mục tiêu và các chỉ số ở §3.4 của curriculum.

## Cấu trúc lab

```text
examples/toi-noi-location/
├── data/        bản đồ tải về, gitignore. fetch.sh tải OSM Việt Nam; lab LOC-06 cắt ra hcm.pbf
├── fixtures/    đồ thị tổng hợp nhỏ và kịch bản simulator, commit, dùng cho test và CI
├── infra/       Redis (127.0.0.1:6391), Kafka (127.0.0.1:9096)
├── proto/       hợp đồng gRPC và Kafka dùng chung cho Rust, Go, C#
├── results/     pNN.md: lệnh và output thật của từng bài, nguồn duy nhất cho số trong bài
├── rust/        Cargo workspace
│   ├── crates/  geo-core, road-graph, driver-store, map-match, route-engine, simulator, pickup, matcher, geo-engine
│   └── labs/    một binary cho mỗi bài: src/bin/locNN_<ten>.rs
├── go/          Go module: cmd/location-ingest, cmd/tracking-hub, cmd/loadgen, cmd/locNN_<ten>
├── python/      project uv, Python 3.13: locNN_<ten>.py
├── tools/       check_articles.py, sync_series.py
└── verify-pNN.sh  chạy build, test, lab không cần mạng của phần NN
```

Lab tích lũy: bài sau dùng crate của bài trước. Không phá API đã công bố ở bài trước; nếu buộc phải đổi thì sửa cả bài cũ và ghi vào `results/`.

## Môi trường trên máy viết

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"   # cargo không có sẵn trên PATH
```

- Rust 1.98.1 edition 2024, stable, không nightly. Go 1.27. `uv` với Python 3.13; Python hệ thống 3.9 không dùng.
- .NET SDK có sẵn cho client C# trong các bài interop.
- Docker dùng được với image đã có: `redis:7`, `apache/kafka:4.1.0`. Pull image mới từ Docker Hub có thể treo; đừng để bài phụ thuộc vào việc đó.
- Bản đồ: `data/vietnam-latest.osm.pbf` đã có trên máy viết. Bbox lab là nội thành TP.HCM: lat 10.70–10.86, lng 106.60–106.78.

## Quy tắc số liệu

1. Số trong bài phải giống output trong `results/pNN.md`, và output đó phải do lệnh ghi trong bài sinh ra. Không làm tròn đẹp, không đoán, không viết «khoảng» cho số chưa đo.
2. Ghi cấu hình đo (Apple Silicon, `--release`, kích thước đồ thị, seed) gần mọi bảng số. Hiệu năng là của máy viết, chỉ so sánh tương đối.
3. Số tối ưu (`walk_m`, `deadhead_m`, …) luôn so với baseline, kèm seed, kịch bản, và cụm «trên dữ liệu mô phỏng».
4. Việc không chạy được thì viết thẳng là chưa chạy và vì sao. Không bao giờ giả vờ đã thành công.
5. Không có GPS người thật. Dữ liệu chuyến, GPS, tắc nghẽn đều do simulator sinh ra. Không gán số cho công ty thật nếu không có nguồn công bố. Không ghi giá tiền provider hay cloud.
6. Trích dẫn bài báo, văn bản pháp luật chỉ sau khi kiểm tra được nguồn; không chắc thì viết rõ là cần kiểm lại, không tự chế số hiệu, năm, tác giả.

## Cấu trúc một bài

File `api/seed/content/<slug>.md`, viết tiếng Việt có dấu, văn phong rõ ràng như loạt Rust. Xem mẫu ở bài `du-an-mau-toi-noi`.

1. Frontmatter đúng các khóa, mỗi khóa một dòng:

   ```yaml
   ---
   slug: location-...
   title: "..."
   summary: "Bài NN: ..."        # tối đa 234 ký tự
   category: system-design
   level: intermediate           # P01–P03; advanced cho P04–P07
   tags: [toi-noi, location, rust, ...]   # tối đa 10
   language: vi
   ---
   ```

2. Mở bằng tình huống cụ thể của Tới Nơi (1–2 đoạn). Sau đó câu: `Đây là bài NN của lộ trình **<tên lộ trình>** qua [Tới Nơi](/articles/du-an-mau-toi-noi).` Tên lộ trình: «Nền tảng định vị», «Tối ưu quãng đường hai phía», «Dữ liệu và vận hành».
3. Callout `Thử trước khi đọc`, có câu hỏi và `<details>` đáp án:

   ```html
   <div class="tkh-callout tkh-callout--example">
   <span class="tkh-callout-title">Thử trước khi đọc</span>
   <p>...</p>
   <details><summary>Xem đáp án</summary><p>...</p></details>
   </div>
   ```

4. 4–7 mục `##` giải thích cơ chế. Ít nhất một hình:

   ```html
   <figure class="tkh-fig">
   <div class="tkh-fig-box">
   <svg viewBox="0 0 880 300" role="img" aria-label="...">...</svg>
   </div>
   <figcaption>Hình 1. ...</figcaption>
   </figure>
   ```

   Chỉ dùng các lớp: `f-bg f-surface f-muted f-none f-dead f-phys f-phys-soft f-pk f-pk-soft f-warn f-warn-soft s-line s-muted s-dead s-phys s-pk s-warn t-b t-big t-small t-mono t-phys t-pk t-warn t-dead dash`. Escape `&` thành `&amp;`, `<` thành `&lt;` trong SVG. Hình dùng dữ liệu OSM ghi «© OpenStreetMap contributors». Kết quả thuật toán vẽ từ output của lab, không vẽ tay.
5. Code trích đúng từ lab, ghi tên file ngay trên khối code, chú thích tiếng Việt. Không dán nguyên file dài; chọn đoạn nói lên cơ chế.
6. Mục `## Chạy ví dụ và kiểm chứng` với lệnh chạy từ gốc repo và kết quả thật. Hai bài không có code (LOC-01, LOC-02) bỏ mục này.
7. Mục hoặc callout có chữ `Giới hạn của ví dụ`: lab chưa có gì so với production.
8. Mục `## Bài tập` có `<details>` lời giải, ít nhất 2 câu.
9. Dòng điều hướng ngay trước `## Tài liệu gốc`; tool `sync_series.py` sẽ tự viết lại nên chỉ cần một dòng bắt đầu bằng `Bài trước:`.
10. `## Tài liệu gốc`: tài liệu chính thức, docs.rs, go.dev, bài báo gốc. Chỉ link URL chắc chắn đúng.

Link nội bộ chỉ tới bài đã có trong `api/seed/content`. Các loạt Rust, Python, Flutter chưa merge vào nhánh này nên nhắc bằng chữ thường, ví dụ «loạt Rust qua Tới Nơi, bài về Send và Sync», không tạo link. Loạt C#, AWS, Azure đã có; kiểm tra slug trước khi link.

Tránh văn sáo và câu mở đầu rỗng. Giải thích cơ chế bằng tình huống đặt xe cụ thể: hẻm, cầu vượt, dải phân cách, khách đứng sai phía đường. Dùng đúng thuật ngữ: điểm đón, điểm trả, km rỗng, bám đường (map matching), ghép chuyến.

## Kiểm tra

```bash
python3 examples/toi-noi-location/tools/check_articles.py --from 1 --to 10   # bài, frontmatter, SVG, link, cấu trúc
bash examples/toi-noi-location/verify-p01.sh                                 # lab
```
