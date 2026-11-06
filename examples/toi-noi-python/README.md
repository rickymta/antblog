# Python qua dự án Tới Nơi

Khung biên tập cho loạt bài Python của Tech Knowledge Hub. Danh sách 79 bài, phụ thuộc và trạng thái nằm trong [`docs/python-master-curriculum.md`](../../docs/python-master-curriculum.md). Mỗi bài có câu hỏi mở đầu, hình SVG giải thích cơ chế, ví dụ chạy được, lỗi thường gặp, bài tập có đáp án và nguồn chính thức. Tách rõ code minh họa với giải pháp production. Không tạo link cho bài chưa viết.

Đối tượng cấp cơ bản: người mới biết dùng terminal, chưa cần biết Python. Người đã đọc loạt C# sẽ gặp lại cùng mô hình cước, cùng chuyến xe để đối chiếu hai ngôn ngữ.

## Vai trò của Python trong Tới Nơi

C#/.NET giữ các backend nghiệp vụ chính (đặt chuyến, đơn hàng, điều phối). Python giữ **OCR giấy tờ định danh**, **so khớp khuôn mặt**, **chấm điểm gian lận** và **phân tích/dự đoán**. Các service Python nhận event từ Kafka, trả kết quả qua gRPC/REST, sở hữu dữ liệu riêng (PostgreSQL, MongoDB cho document OCR, Redis cho feature online). Kết quả OCR và điểm fraud là đầu vào cho quy trình nghiệp vụ, không tự động là quyết định xác minh hay kết luận gian lận. Xem bài `du-an-mau-toi-noi`, mục «Đa ngôn ngữ theo trách nhiệm service».

## Quy ước xuyên suốt

- **Python 3.14** quản lý bằng **uv**: `uv python install 3.14`, mỗi bài là một project độc lập có `pyproject.toml` (`requires-python = ">=3.14"`), chạy bằng `uv run`. Không cài gì vào Python hệ thống.
- Không dùng tính năng còn ở bản rc. Free-threaded build chỉ nhắc như thử nghiệm ở bài 22.
- Tiền VND dùng `Decimal` với `ROUND_HALF_UP`; thời điểm aware và lưu UTC; dữ liệu test giả. Mô hình cước học tập giống loạt C#: giá cơ bản 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt; không phụ phí, thuế, giảm giá.
- Mã chuyến `TN-0001`, tài xế `DR-001`, khách `CU-001`, thành phố theo hồ sơ Tới Nơi (6 thành phố). 8 trạng thái chuyến: requested, searching, accepted, arriving, picked_up, in_transit, completed, cancelled.
- Code trong bài là code trong thư mục lab tương ứng, có chú thích tiếng Việt, ghi tên file. Mỗi project cấp cơ bản có chế độ `--verify` (hoặc `pytest` từ bài 10) kiểm các kết quả kỳ vọng in trong bài.
- Ảnh giấy tờ, khuôn mặt (từ phần 05) là dữ liệu tổng hợp sinh bằng code. Không commit ảnh người thật, không commit file mô hình lớn, không có credential thật.
- Hình minh họa: SVG nội tuyến dùng bộ lớp `packages/ui/src/article.css` (`f-phys-soft s-phys t-b t-small f-pk-soft s-pk f-warn-soft s-warn f-bg s-line dash`), `viewBox` rộng 880, `role="img"`, `aria-label`, caption «Hình N. …». Ảnh bìa khai báo trong `apps/web/src/lib/article-covers.json`, sinh bằng `npm run covers:generate -w @tkh/web`.

## Cấu trúc một bài

1. Frontmatter: `slug`, `title`, `summary` («Bài NN: …», ≤ 234 ký tự), `category: programming`, `level`, `tags` (`python`, `toi-noi`, thêm chủ đề), `language: vi`.
2. Mở bài bằng một tình huống của Tới Nơi, câu «Đây là bài NN của lộ trình **Python cơ bản qua Tới Nơi**» (hoặc trung bình / nâng cao) với link `[Tới Nơi](/articles/du-an-mau-toi-noi)`.
3. Callout «Thử trước khi đọc» với `<details>` đáp án.
4. 4–7 mục `##`, ít nhất một hình SVG, code chạy được, bảng «Hiện tượng / Kiểm tra trước» khi phù hợp.
5. Đoạn hoặc callout nói rõ giới hạn của ví dụ so với production.
6. **Bài tập** có `<details>` lời giải.
7. Dòng «Bài trước: … Bài tiếp: …» chỉ link bài đã viết; mục «Tài liệu gốc» trỏ docs.python.org, PEP hoặc tài liệu chính thức của thư viện.

## P01 · Nền tảng Python

01. Python 3.14 và uv: từ source code đến chương trình tính cước — **đã viết**
02. Biến, kiểu dữ liệu và Decimal: biểu diễn tiền và khoảng cách — **đã viết**
03. Điều kiện, vòng lặp và hàm: tách quy tắc tính cước — **đã viết**
04. Chuỗi, f-string, ngày giờ và múi giờ: hiển thị đúng, lưu đúng — **đã viết**
05. List, dict, set, tuple: quản lý chuyến và tài xế trong bộ nhớ — **đã viết**
06. Comprehension, iterator và generator: dòng vị trí tài xế không cần nạp hết — **đã viết**
07. Module, package và import: tổ chức mã tính cước thành gói — **đã viết**
08. Exception, debug và logging: lần theo một báo giá sai — **đã viết**
09. File, JSON, CSV và pathlib: đọc bộ dữ liệu mẫu Tới Nơi — **đã viết**
10. Unit test với pytest: kiểm thử chính sách cước — **đã viết**

Các phần P02–P10 (bài 11–79) xem master curriculum.

## Chạy lab phần 01

Cần uv và Python 3.14 (`uv python install 3.14`). Mỗi thư mục `NN-…` có README ghi lệnh chạy, lệnh `--verify` và kết quả kỳ vọng; lab không cần mạng khi chạy (bài 10 cần mạng ở lần resolve pytest đầu tiên).

```bash
uv run --project examples/toi-noi-python/01-python-va-uv examples/toi-noi-python/01-python-va-uv/main.py
uv run --project examples/toi-noi-python/01-python-va-uv examples/toi-noi-python/01-python-va-uv/main.py --verify
sh examples/toi-noi-python/verify-01-10.sh
```

`verify-01-10.sh` chạy `--verify` của 9 lab console và pytest của bài 10. Mọi `--verify` in `OK n/n` khi đạt, `FAIL n/m` và exit 1 khi lệch; các con số in trong bài là output của chính lab.
