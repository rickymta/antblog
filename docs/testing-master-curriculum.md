# Master Curriculum — Kiểm thử phần mềm theo ISTQB qua dự án Tới Nơi

Bản quản lý toàn bộ chuyên mục Testing của Tech Knowledge Hub (antblog.xyz), dựng theo §13 của tài liệu định hướng `toi-noi-testing-antblog.md`. Gồm **162 bài** trong **12 series** và **1 bài thực hành cuối Series 01**, tổng thời lượng đọc dự kiến khoảng **47 giờ 51 phút**.

- Bảng đầy đủ (20 cột, mở được bằng Excel): `testing-master-curriculum.csv` — mô tả, mục tiêu học tập, LO, bài thực hành, công cụ của từng bài nằm ở đó.
- Tài liệu này: cách dùng, quy ước, bảng tóm tắt từng series, chuỗi bài cần đọc trước, các chỗ chồng lấn và điểm cần chủ blog quyết.

## 1. Cách dùng

1. **Người viết**: chọn bài theo Phase, đọc dòng tương ứng trong CSV (Description + Learning Objectives + Practical Exercise) làm đề cương, rồi viết theo template 8 phần ở §6 của tài liệu định hướng. Đối chiếu cột ISTQB LO với syllabus trước khi xuất bản; ô ghi «Đối chiếu khi viết» nghĩa là chưa kiểm được mã LO chính thức.
2. **Người đọc / lộ trình học**: đi theo cột «Bài cần đọc trước». Mọi tham chiếu tạo thành một đồ thị không chu trình (DAG — đã kiểm bằng script), nên luôn có thứ tự đọc hợp lệ.
3. **Theo dõi tiến độ**: cập nhật cột Publication Status trong CSV (`Kế hoạch` → `Đang viết` → `Đã viết` → `Đã xuất bản`). Đối chiếu repo ngày 07/11/2026: S01–S05 và hai bài đầu S06 (TEST-055–056) đã có nội dung; «Đã viết» không khẳng định đã triển khai lên website.

## 2. Quy ước

| Mục | Quy ước |
|---|---|
| Article ID | `TEST-001` … `TEST-162`, tăng dần theo thứ tự series. Bài thực hành cuối Series 01 mang mã riêng `TEST-S01-LAB` (không chiếm số). |
| Series ID | `S01` … `S12`, giữ đúng thứ tự và danh sách chủ đề của tài liệu định hướng (10, 10, 10, 12, 12, 10, 14, 14, 19, 15, 18, 18 bài). |
| Slug | Chữ thường ASCII, gạch nối, ≤ 60 ký tự, duy nhất. Slug Series 01 cố định theo bài đang viết. |
| Knowledge Level | L1 – Beginner · L2 – Intermediate · L3 – Advanced · L4 – Expert · L5 – QA Architect (§3.1 tài liệu định hướng). Mức có thể lệch nhẹ so với mức chung của series khi bài thuộc syllabus nâng cao. |
| Test Level | Component · Component Integration · System · System Integration · Acceptance · «Không áp dụng» (bài khái niệm, quản lý). |
| Test Type | Functional · Non-functional · Black-box · White-box · Confirmation & Regression · «Không áp dụng»; có thể kết hợp, ngăn bằng `;`. |
| Prerequisites | Mã bài cần đọc trước, ngăn bằng `;`. Chỉ ghi phụ thuộc trực tiếp, không ghi bắc cầu. |
| Estimated Reading Time | Phút; bài thường 12–25 phút, bài thực hành cuối series dài hơn. |
| Phase | Phase 1 = S01–S04 · Phase 2 = S05–S08 · Phase 3 = S09–S11 · Phase 4 = S12 (§10 tài liệu định hướng). |

### 2.1. ISTQB và nội dung mở rộng

- Cột **ISTQB Syllabus** chỉ ghi syllabus khi nội dung bài thực sự nằm trong syllabus đó. Phiên bản dùng: CTFL v4.0.1, CTAL-TA v4.0, CTAL-TTA v4.0, CTAL-TAE v2.0, CTAL-TM v3.0, CT-PT v1.0 (2018); các module chuyên biệt CT-SEC, CT-MAT, CT-AcT, CT-UT và CTEL-TM chỉ ghi tên, chưa đối chiếu phiên bản.
- Kỹ thuật thực tiễn ngành không có trong syllabus (test doubles, loop testing, mutation testing, contract testing chuyên sâu, Saga, chaos engineering, observability-driven testing…) ghi **«Mở rộng ngoài ISTQB»**, hoặc «<syllabus gần nhất> (mở rộng)» khi syllabus có nhắc tới bối cảnh nhưng không có LO. Nguyên tắc 4 của tài liệu định hướng.
- **ISTQB Chapter** chỉ ghi số mục khi đã kiểm trong syllabus. **ISTQB LO** ghi mã + mức K khi đã kiểm; nếu không ghi «Đối chiếu khi viết». Ở CTAL-TTA chương 5, syllabus in mã dạng «TTA 5.1.1» (không gạch nối) — bảng ghi thống nhất `TTA-5.1.1`.

### 2.2. Nguồn đã đối chiếu (04/11/2026)

Mã LO và số mục đã kiểm trực tiếp từ bản PDF chính thức trên istqb.org (trích văn bản bằng `pdftotext`):

| Syllabus | Bản đã kiểm | Phạm vi kiểm |
|---|---|---|
| CTFL | Foundation Level Syllabus **v4.0.1** (2024) | Toàn bộ LO FL-1.1.1 … FL-6.2.1 và số mục chương 1–6 |
| CTAL-TA | Advanced Level Test Analyst **v4.0** (GA 2025/05/02) | Toàn bộ LO TA-1.1.1 … TA-5.3.2 và số mục |
| CTAL-TTA | Advanced Level Technical Test Analyst **v4.0** (2021-06-30) | Toàn bộ LO TTA-1.2.1 … TTA-6.2.6 và số mục |
| CTAL-TAE | Advanced Level Test Automation Engineering **v2.0** (2024/05/03) | Toàn bộ LO TAE-1.1.1 … TAE-8.1.4 |
| CTAL-TM | Advanced Level Test Management **v3.0** (2024/05/03) | LO chương 1–3 (TM-1.1.1 … TM-3.2.2) và số mục |
| CT-PT | Foundation Level Specialist Performance Testing **v1.0 (2018)** | Toàn bộ LO PTFL-1.1.1 … PTFL-5.2.1 và số mục |
| CT-SEC, CT-MAT, CT-AcT, CT-UT, CTEL-TM | Chưa kiểm | Mọi ô LO của các module này ghi «Đối chiếu khi viết» |

### 2.3. Hệ thống mẫu và phòng lab

- Bối cảnh nghiệp vụ: hồ sơ `du-an-mau-toi-noi.md` (8 trạng thái chuyến, 70 chuyến/giây cao điểm…) và kiến trúc mục tiêu `toi-noi-overall-architecture.md` (microservices theo bounded context, REST + gRPC + Kafka với outbox/inbox, Redis, Saga, Realtime Gateway, SLO §14.1, chiến lược test §17).
- Phòng lab `examples/toi-noi-testing` (TypeScript, đang xây) là **modular monolith kiểu MVP**: domain cước/bưu kiện/đồ ăn/voucher/phí huỷ/máy trạng thái chuyến, API Express, repository PGlite, cổng VíXanh giả, web UI tối giản; công cụ vitest, fast-check, Stryker, Playwright, autocannon, supertest, zod. Test basis là `SPEC.md` v1.0 — bài tập trích mã yêu cầu `R-…` của file này.
- Lab **không có** Kafka, Redis, gRPC, nhiều instance hay app di động. Bài về các chủ đề đó ghi rõ là bài thiết kế/phân tích hoặc cần môi trường riêng (Docker Compose) — cột «Thực hành» bên dưới cho biết loại.

## 3. Lộ trình theo phase

| Phase | Series | Số bài | Thời lượng đọc | Kết quả mong muốn |
|---|---|---:|---:|---|
| Phase 1 | S01–S04 | 42 (+1 bài thực hành) | 12 giờ 13 phút | Nền tảng Tester và Test Design |
| Phase 2 | S05–S08 | 50 | 14 giờ 39 phút | Kỹ năng test kỹ thuật và hệ thống |
| Phase 3 | S09–S11 | 52 | 15 giờ 35 phút | Năng lực Automation và Advanced Testing |
| Phase 4 | S12 | 18 | 5 giờ 24 phút | QA Lead và QA Architect |

## 4. Bảng theo series

Cột «Thực hành»: **Lab (code)** = làm trên phòng lab · **Lab + thiết kế** = một phần trên lab, phần còn lại là thiết kế · **Môi trường riêng** = cần dựng Kafka/Redis/PostgreSQL… ngoài lab · **Thiết kế/phân tích** = bài giấy dựa trên kiến trúc · **Giấy/bảng tính** = bài phân tích không cần code. Nội dung bài tập đầy đủ ở cột Practical Exercise của CSV.

### S01 — Fundamentals of Software Testing

10 bài · Phase 1 · 204 phút đọc · mức L1

| ID | Tiêu đề | Level | ISTQB ref | Bài cần đọc trước | Thực hành | Trạng thái |
|---|---|---|---|---|---|---|
| TEST-001 | Software Testing là gì? Testing khác Debugging như thế nào? | L1 | CTFL v4.0.1 — 1.1 (1.1.1, 1.1.2) | — | Lab (code) | Đã viết |
| TEST-002 | Vì sao cần kiểm thử? (Why is Testing Necessary?) | L1 | CTFL v4.0.1 — 1.2.1 | 001 | Giấy/bảng tính | Đã viết |
| TEST-003 | Bảy nguyên lý kiểm thử (Seven Testing Principles) | L1 | CTFL v4.0.1 — 1.3 | 001, 002 | Lab (đọc SPEC) | Đã viết |
| TEST-004 | Error, Defect, Failure và Root Cause | L1 | CTFL v4.0.1 — 1.2.3 | 001 | Lab (code) | Đã viết |
| TEST-005 | Verification và Validation | L1 | CTFL v4.0.1 — 1.1; 1.3 (nguyên lý 7) | 001, 003 | Lab (code) | Đã viết |
| TEST-006 | Mục tiêu kiểm thử và Quality Assurance | L1 | CTFL v4.0.1 — 1.1.1; 1.2.2 | 002 | Giấy/bảng tính | Đã viết |
| TEST-007 | Quy trình kiểm thử (Test Process): từ STLC đến CTFL v4 | L1 | CTFL v4.0.1 — 1.4.1; 1.4.2 | 001, 006 | Lab (đọc SPEC) | Đã viết |
| TEST-008 | Kiểm thử trong SDLC: Waterfall, Agile và DevOps | L1 | CTFL v4.0.1 — 2.1.1; 2.1.2; 2.1.4 | 007 | Giấy/bảng tính | Đã viết |
| TEST-009 | Vai trò trong kiểm thử: Developer, Tester, QA, Test Lead | L1 | CTFL v4.0.1 — 1.4.5; 1.5.2; 1.5.3 | 007, 008 | Giấy/bảng tính | Đã viết |
| TEST-010 | Testware và các sản phẩm kiểm thử | L1 | CTFL v4.0.1 — 1.4.3; 1.4.4 | 007 | Lab (code) | Đã viết |
| TEST-S01-LAB | Thực hành Series 01: phân tích tính năng đặt xe từ điểm đón đến khi hoàn thành chuyến | L1 | CTFL v4.0.1 — 1.1; 1.4; 2.1 | 003, 004, 006, 007, 010 | Lab (code) | Đã viết |

### S02 — Test Levels & Test Types

10 bài · Phase 1 · 155 phút đọc · mức L1–L2

| ID | Tiêu đề | Level | ISTQB ref | Bài cần đọc trước | Thực hành | Trạng thái |
|---|---|---|---|---|---|---|
| TEST-011 | Tổng quan 5 test level của ISTQB | L1 | CTFL v4.0.1 — 2.2.1 | 007, 008 | Giấy/bảng tính | Đã viết |
| TEST-012 | Component Testing (Unit Testing): góc nhìn tổng quan | L1 | CTFL v4.0.1 — 2.2.1 | 011 | Lab (đọc code) | Đã viết |
| TEST-013 | Component Integration Testing | L1 | CTFL v4.0.1 — 2.2.1 | 011, 012 | Lab (code) | Đã viết |
| TEST-014 | System Testing | L1 | CTFL v4.0.1 — 2.2.1 | 011 | Lab (code) | Đã viết |
| TEST-015 | System Integration Testing | L1 | CTFL v4.0.1 — 2.2.1 | 013, 014 | Lab (code) | Đã viết |
| TEST-016 | Acceptance Testing: UAT, OAT, hợp đồng, alpha/beta | L1 | CTFL v4.0.1; tham khảo CT-AcT — 2.2.1 | 011 | Giấy/bảng tính | Đã viết |
| TEST-017 | Functional Testing | L1 | CTFL v4.0.1 — 2.2.2 | 011 | Lab (code) | Đã viết |
| TEST-018 | Non-functional Testing: tổng quan theo ISO/IEC 25010 | L2 | CTFL v4.0.1 — 2.2.2 | 017 | Lab (đọc SPEC) | Đã viết |
| TEST-019 | Confirmation Testing và Regression Testing | L2 | CTFL v4.0.1 — 2.2.3 | 004, 017 | Lab (code) | Đã viết |
| TEST-020 | Maintenance Testing | L2 | CTFL v4.0.1 — 2.3 | 019 | Lab (đọc SPEC) | Đã viết |

### S03 — Static Testing & Review Techniques

10 bài · Phase 1 · 150 phút đọc · mức L2

| ID | Tiêu đề | Level | ISTQB ref | Bài cần đọc trước | Thực hành | Trạng thái |
|---|---|---|---|---|---|---|
| TEST-021 | Static Testing là gì? | L2 | CTFL v4.0.1 — 3.1.1; 3.1.2 | 004, 007 | Lab (đọc SPEC) | Đã viết |
| TEST-022 | Static Testing và Dynamic Testing | L2 | CTFL v4.0.1 — 3.1.3 | 021 | Lab (code) | Đã viết |
| TEST-023 | Informal Review | L2 | CTFL v4.0.1 — 3.2.4 | 021 | Giấy/bảng tính | Đã viết |
| TEST-024 | Walkthrough | L2 | CTFL v4.0.1 — 3.2.4 | 023 | Lab (đọc SPEC) | Đã viết |
| TEST-025 | Technical Review: review API contract | L2 | CTFL v4.0.1; CTAL-TTA v4.0 — CTFL 3.2.4; TTA 5.1 | 023 | Lab (code) | Đã viết |
| TEST-026 | Inspection: review hình thức nhất | L2 | CTFL v4.0.1; CTAL-TA v4.0 — CTFL 3.2.4; TA 5.2 | 024, 025, 027 | Lab (đọc SPEC) | Đã viết |
| TEST-027 | Review Process và các vai trò | L2 | CTFL v4.0.1 — 3.2.2; 3.2.3; 3.2.5 | 021 | Lab (đọc SPEC) | Đã viết |
| TEST-028 | Checklist-based Review | L2 | CTFL v4.0.1; CTAL-TA v4.0; CTAL-TTA v4.0 — CTFL 3.2.4; TA 5.2; TTA 5.2 | 027 | Lab (đọc SPEC) | Đã viết |
| TEST-029 | Static Analysis: phân tích code tĩnh | L2 | CTAL-TTA v4.0; CTFL v4.0.1 — TTA 3.2; CTFL 3.1, 6.1 | 021, 022 | Lab (code) | Đã viết |
| TEST-030 | Shift-left Testing | L2 | CTFL v4.0.1 — 2.1.3; 2.1.5; 3.2.1 | 008, 021 | Lab (code) | Đã viết |

### S04 — Black-box Test Design Techniques

12 bài · Phase 1 · 224 phút đọc · mức L2–L3

| ID | Tiêu đề | Level | ISTQB ref | Bài cần đọc trước | Thực hành | Trạng thái |
|---|---|---|---|---|---|---|
| TEST-031 | Equivalence Partitioning | L2 | CTFL v4.0.1; CTAL-TA v4.0 — CTFL 4.2.1; TA 3.1.1 | 010, 017 | Lab (code) | Đã viết |
| TEST-032 | Boundary Value Analysis | L2 | CTFL v4.0.1; CTAL-TA v4.0 — CTFL 4.2.2; TA 3.1.1 | 031 | Lab (code) | Đã viết |
| TEST-033 | Decision Table Testing | L2 | CTFL v4.0.1; CTAL-TA v4.0 — CTFL 4.2.3; TA 3.3.1 | 031 | Lab (code) | Đã viết |
| TEST-034 | State Transition Testing (cơ bản) | L2 | CTFL v4.0.1; CTAL-TA v4.0 — CTFL 4.2.4; TA 3.2.2 | 031 | Lab (code) | Đã viết |
| TEST-035 | Use Case Testing và Scenario-based Testing | L2 | CTAL-TA v4.0 — 3.2.3 | 034 | Lab (đọc SPEC) | Đã viết |
| TEST-036 | Classification Tree Method | L2 | CTAL-TA v4.0 — 3.1.2 | 031 | Giấy/bảng tính | Đã viết |
| TEST-037 | Pairwise Testing | L2 | CTAL-TA v4.0 — 3.1.2 | 036 | Giấy/bảng tính | Đã viết |
| TEST-038 | Combinatorial Testing | L2 | CTAL-TA v4.0 — 3.1.2 | 037 | Lab (code) | Đã viết |
| TEST-039 | CRUD Testing | L2 | CTAL-TA v4.0 — 3.2.1 | 034 | Lab (code) | Đã viết |
| TEST-040 | Data-based test techniques: domain testing nhiều biến | L3 | CTAL-TA v4.0 — 3.1.1 | 032, 038 | Lab (code) | Kế hoạch |
| TEST-041 | Random Testing | L3 | CTAL-TA v4.0 — 3.1.3 | 031, 040 | Lab (code) | Đã viết |
| TEST-042 | Metamorphic Testing | L3 | CTAL-TA v4.0 — 3.3.2 | 041 | Lab (code) | Đã viết |

### S05 — White-box & Experience-based Testing

12 bài · Phase 2 · 208 phút đọc · mức L3

| ID | Tiêu đề | Level | ISTQB ref | Bài cần đọc trước | Thực hành | Trạng thái |
|---|---|---|---|---|---|---|
| TEST-043 | Statement Coverage | L3 | CTFL v4.0.1; CTAL-TTA v4.0 — CTFL 4.3.1; TTA 2.2 | 012, 031 | Lab (code) | Đã viết |
| TEST-044 | Branch Coverage / Decision Coverage | L3 | CTFL v4.0.1; CTAL-TTA v4.0 — CTFL 4.3.2; TTA 2.3 | 043 | Lab (code) | Đã viết |
| TEST-045 | Condition Coverage | L3 | CTAL-TTA v4.0 (mở rộng) — TTA 2.4–2.5 (bối cảnh) | 044 | Lab (code) | Đã viết |
| TEST-046 | Multiple Condition Testing | L3 | CTAL-TTA v4.0 — 2.5 | 045 | Lab (code) | Đã viết |
| TEST-047 | MC/DC Testing | L3 | CTAL-TTA v4.0 — 2.4 | 046 | Lab (code) | Đã viết |
| TEST-048 | Path Testing | L3 | CTAL-TTA v4.0 (mở rộng) — TTA 2.6; TTA 3.2.1 | 044 | Lab (code) | Đã viết |
| TEST-049 | Loop Testing | L3 | Mở rộng ngoài ISTQB | 044 | Lab (code) | Đã viết |
| TEST-050 | API Testing từ góc nhìn code | L3 | CTAL-TTA v4.0 — 2.7 | 044 | Lab (code) | Đã viết |
| TEST-051 | Error Guessing | L3 | CTFL v4.0.1 — 4.4.1 | 017 | Lab (code) | Đã viết |
| TEST-052 | Exploratory Testing | L3 | CTFL v4.0.1; CTAL-TA v4.0 — CTFL 4.4.2; TA 3.4.1 | 051 | Lab (code) | Kế hoạch |
| TEST-053 | Checklist-based Testing | L3 | CTFL v4.0.1; CTAL-TA v4.0 — CTFL 4.4.3; TA 3.4.2 | 028, 051 | Lab (code) | Đã viết |
| TEST-054 | Defect-based Testing | L3 | CTAL-TA v4.0 (mở rộng) — TA 5.3 | 004, 051 | Giấy/bảng tính | Đã viết |

### S06 — Unit Testing & Component Testing

10 bài · Phase 2 · 180 phút đọc · mức L3

| ID | Tiêu đề | Level | ISTQB ref | Bài cần đọc trước | Thực hành | Trạng thái |
|---|---|---|---|---|---|---|
| TEST-055 | Unit Testing Fundamentals | L3 | CTFL v4.0.1; mở rộng Engineering Practice — CTFL 2.1.3; 2.2.1 | 012, 031 | Lab (code) | Đã viết |
| TEST-056 | Unit Test Design cho Pricing Engine | L3 | CTFL v4.0.1; mở rộng Engineering Practice — CTFL 4.2 | 055, 032, 033 | Lab (code) | Đã viết |
| TEST-057 | Test Doubles: Dummy, Stub, Fake, Mock, Spy | L3 | Mở rộng ngoài ISTQB | 055 | Lab (code) | Kế hoạch |
| TEST-058 | Dependency Isolation | L3 | Mở rộng ngoài ISTQB | 057 | Lab (code) | Kế hoạch |
| TEST-059 | Parameterized Testing | L3 | Mở rộng ngoài ISTQB (liên quan CTAL-TAE v2.0 3.1.4) | 055, 031 | Lab (code) | Kế hoạch |
| TEST-060 | Data-driven Unit Testing | L3 | Mở rộng ngoài ISTQB (liên quan CTAL-TTA v4.0 6.1) — TTA 6.1 (bối cảnh) | 059 | Lab (code) | Kế hoạch |
| TEST-061 | Exception Testing | L3 | Mở rộng ngoài ISTQB | 056 | Lab (code) | Kế hoạch |
| TEST-062 | Code Coverage trong thực hành | L3 | CTFL v4.0.1; CTAL-TTA v4.0 — CTFL 4.3; TTA 2 | 043, 044, 056 | Lab (code) | Kế hoạch |
| TEST-063 | Mutation Testing | L3 | Mở rộng ngoài ISTQB (gần CTAL-TTA v4.0 6.2.1 fault seeding) — TTA 6.2.1 (bối cảnh) | 062 | Lab (code) | Kế hoạch |
| TEST-064 | Testability và Testable Architecture | L3 | Mở rộng ngoài ISTQB (liên quan CTAL-TTA v4.0 4.6) — TTA 4.6 (bối cảnh) | 058 | Lab (code) | Kế hoạch |

### S07 — Integration Testing & API Testing

14 bài · Phase 2 · 234 phút đọc · mức L3

| ID | Tiêu đề | Level | ISTQB ref | Bài cần đọc trước | Thực hành | Trạng thái |
|---|---|---|---|---|---|---|
| TEST-065 | Integration Testing Fundamentals | L3 | CTFL v4.0.1; mở rộng — 2.2.1 | 013, 015 | Lab (code) | Kế hoạch |
| TEST-066 | Top-down Integration Testing | L3 | Mở rộng ngoài ISTQB | 065, 057 | Lab (code) | Kế hoạch |
| TEST-067 | Bottom-up Integration Testing | L3 | Mở rộng ngoài ISTQB | 065 | Lab (code) | Kế hoạch |
| TEST-068 | Big-bang Integration Testing | L3 | Mở rộng ngoài ISTQB | 065 | Thiết kế/phân tích | Kế hoạch |
| TEST-069 | Incremental Integration Testing | L3 | Mở rộng ngoài ISTQB | 066, 067, 068 | Thiết kế/phân tích | Kế hoạch |
| TEST-070 | API Testing Fundamentals | L3 | CTAL-TTA v4.0; CTAL-TAE v2.0 — TTA 2.7; TAE 5.1.3 | 050, 065 | Lab (code) | Kế hoạch |
| TEST-071 | API Positive & Negative Testing | L3 | CTFL v4.0.1 — 4.2 | 070, 031, 032 | Lab (code) | Kế hoạch |
| TEST-072 | API Schema Validation | L3 | Mở rộng ngoài ISTQB | 070 | Lab (code) | Kế hoạch |
| TEST-073 | API Authentication & Authorization | L3 | CTAL-TTA v4.0; mở rộng (OWASP API Security Top 10) — TTA 4.3 (bối cảnh) | 070 | Lab (code) | Kế hoạch |
| TEST-074 | API Contract Testing: tổng quan | L3 | CTAL-TAE v2.0 — 5.1.3 | 072 | Thiết kế/phân tích | Kế hoạch |
| TEST-075 | Database Integration Testing | L3 | Mở rộng ngoài ISTQB | 065 | Lab (code) | Kế hoạch |
| TEST-076 | Message Queue Integration Testing | L3 | Mở rộng ngoài ISTQB | 065, 072 | Môi trường riêng | Kế hoạch |
| TEST-077 | Third-party Integration Testing | L3 | CTAL-TA v4.0; mở rộng — TA 4.4 | 015, 070 | Lab + thiết kế | Kế hoạch |
| TEST-078 | Integration Failure Testing: timeout, retry, duplicate request | L3 | Mở rộng ngoài ISTQB (liên quan CTAL-TTA v4.0 4.4.4) — TTA 4.4.4 (bối cảnh) | 075, 077 | Lab (code) | Kế hoạch |

### S08 — System Testing & End-to-end Testing

14 bài · Phase 2 · 257 phút đọc · mức L3–L4

| ID | Tiêu đề | Level | ISTQB ref | Bài cần đọc trước | Thực hành | Trạng thái |
|---|---|---|---|---|---|---|
| TEST-079 | System Testing Strategy | L3 | CTFL v4.0.1; CTAL-TA v4.0 — CTFL 2.2.1; 5.1.3 | 014, 018, 069 | Lab + thiết kế | Kế hoạch |
| TEST-080 | End-to-end Testing | L3 | CTFL v4.0.1; mở rộng — 2.2.1 | 079, 070 | Lab (code) | Kế hoạch |
| TEST-081 | Business Flow Testing | L3 | CTAL-TA v4.0 — 3.2.3 | 035, 080 | Lab (code) | Kế hoạch |
| TEST-082 | State Transition Testing nâng cao | L4 | CTAL-TA v4.0 — 3.2.2 | 034 | Lab (code) | Kế hoạch |
| TEST-083 | Cross-platform Testing | L3 | CT-MAT; mở rộng — Đối chiếu khi viết | 080, 037 | Lab (code) | Kế hoạch |
| TEST-084 | Multi-user Scenario Testing | L3 | Mở rộng ngoài ISTQB | 080 | Lab (code) | Kế hoạch |
| TEST-085 | Concurrent Testing: hai tài xế cùng nhận một chuyến | L4 | Mở rộng ngoài ISTQB | 084, 075 | Lab (code) | Kế hoạch |
| TEST-086 | Race Condition Testing | L4 | Mở rộng ngoài ISTQB | 085, 082 | Lab (code) | Kế hoạch |
| TEST-087 | Negative Flow Testing | L3 | CTFL v4.0.1 — 4.2 | 081, 071 | Lab (code) | Kế hoạch |
| TEST-088 | Recovery Testing cấp hệ thống | L3 | CTAL-TTA v4.0; mở rộng — TTA 4.4.5 | 080, 078 | Lab (code) | Kế hoạch |
| TEST-089 | Compatibility Testing: thiết bị và hệ điều hành | L3 | CTAL-TA v4.0; CT-MAT — TA 4.4 | 083, 037 | Thiết kế/phân tích | Kế hoạch |
| TEST-090 | Usability Testing: đánh giá trải nghiệm đặt xe | L3 | CTAL-TA v4.0; CT-UT — TA 4.2 | 014 | Lab (code) | Kế hoạch |
| TEST-091 | Accessibility Testing | L3 | CTAL-TA v4.0 (mở rộng); CT-UT — TA 4.2 | 090 | Lab (code) | Kế hoạch |
| TEST-092 | Acceptance Testing: nghiệm thu toàn bộ nghiệp vụ | L4 | CTFL v4.0.1; CT-AcT — CTFL 4.5.2; 4.5.3 | 016, 081 | Lab (code) | Kế hoạch |

### S09 — Non-functional Testing

19 bài · Phase 3 · 335 phút đọc · mức L4

| ID | Tiêu đề | Level | ISTQB ref | Bài cần đọc trước | Thực hành | Trạng thái |
|---|---|---|---|---|---|---|
| TEST-093 | Non-functional Testing Fundamentals | L4 | CTAL-TTA v4.0 — 4.1; 4.2; 4.9 | 018, 079 | Lab (đọc SPEC) | Kế hoạch |
| TEST-094 | Performance Testing Fundamentals | L4 | CT-PT v1.0 (2018); CTAL-TTA v4.0 — CT-PT 1.1; 2.1–2.2; TTA 4.5 | 093, 070 | Lab (code) | Kế hoạch |
| TEST-095 | Load Testing | L4 | CT-PT v1.0 (2018) — 1.2; 4.2 | 094 | Lab (code) | Kế hoạch |
| TEST-096 | Stress Testing | L4 | CT-PT v1.0 (2018) — 1.2 | 095 | Lab (code) | Kế hoạch |
| TEST-097 | Spike Testing | L4 | CT-PT v1.0 (2018) — 1.2 | 095 | Lab (code) | Kế hoạch |
| TEST-098 | Endurance / Soak Testing | L4 | CT-PT v1.0 (2018) — 1.2 | 095 | Lab (code) | Kế hoạch |
| TEST-099 | Scalability Testing | L4 | CT-PT v1.0 (2018) — 1.2 | 095 | Môi trường riêng | Kế hoạch |
| TEST-100 | Capacity Testing | L4 | CT-PT v1.0 (2018); CTAL-TTA v4.0 — CT-PT 1.2; TTA 4.5.4 | 096, 099 | Lab (code) | Kế hoạch |
| TEST-101 | Security Testing Fundamentals | L4 | CTAL-TTA v4.0; CT-SEC — TTA 4.3 | 093 | Thiết kế/phân tích | Kế hoạch |
| TEST-102 | Authentication & Authorization Testing | L4 | CT-SEC; mở rộng (OWASP ASVS) — Đối chiếu khi viết | 073, 101 | Lab + thiết kế | Kế hoạch |
| TEST-103 | Input Validation & Injection Testing | L4 | CT-SEC; mở rộng (OWASP) — Đối chiếu khi viết | 101 | Lab (code) | Kế hoạch |
| TEST-104 | Session & Token Testing | L4 | CT-SEC; mở rộng — Đối chiếu khi viết | 102 | Thiết kế/phân tích | Kế hoạch |
| TEST-105 | Reliability Testing | L4 | CTAL-TTA v4.0 — 4.4 | 093, 098 | Lab (code) | Kế hoạch |
| TEST-106 | Availability Testing | L4 | CTAL-TTA v4.0 — 4.4.3 | 105 | Lab (code) | Kế hoạch |
| TEST-107 | Recovery Testing: khả năng phục hồi | L4 | CTAL-TTA v4.0 — 4.4.5 | 088, 105 | Lab (code) | Kế hoạch |
| TEST-108 | Failover Testing | L4 | CTAL-TTA v4.0; mở rộng — TTA 4.4.4 | 107 | Môi trường riêng | Kế hoạch |
| TEST-109 | Compatibility Testing: co-existence và interoperability | L4 | CTAL-TTA v4.0; CTAL-TA v4.0 — TTA 4.8; TA 4.4 | 089, 093 | Thiết kế/phân tích | Kế hoạch |
| TEST-110 | Maintainability Testing | L4 | CTAL-TTA v4.0 — 4.6 | 029, 064 | Lab (code) | Kế hoạch |
| TEST-111 | Usability & Accessibility: đặc tính chất lượng | L4 | CTAL-TA v4.0; CT-UT — TA 4.2 | 090, 091 | Lab (code) | Kế hoạch |

### S10 — Automation Testing Engineering

15 bài · Phase 3 · 258 phút đọc · mức L4

| ID | Tiêu đề | Level | ISTQB ref | Bài cần đọc trước | Thực hành | Trạng thái |
|---|---|---|---|---|---|---|
| TEST-112 | Automation Testing Fundamentals | L4 | CTAL-TAE v2.0; CTFL v4.0.1 — TAE 1.1; CTFL 6.2 | 019, 055 | Giấy/bảng tính | Kế hoạch |
| TEST-113 | Automation Test Strategy | L4 | CTAL-TAE v2.0 — 1.2; 2.2 | 112 | Lab (đọc SPEC) | Kế hoạch |
| TEST-114 | Test Pyramid và Testing Quadrants | L4 | CTFL v4.0.1 — 5.1.6; 5.1.7 | 113 | Lab (code) | Kế hoạch |
| TEST-115 | Test Automation Architecture | L4 | CTAL-TAE v2.0 — 3.1 | 113 | Lab (code) | Kế hoạch |
| TEST-116 | Data-driven Automation | L4 | CTAL-TAE v2.0; CTAL-TTA v4.0 — TAE 3.1.4; TTA 6.1 | 060, 115 | Lab (code) | Kế hoạch |
| TEST-117 | Keyword-driven Automation | L4 | CTAL-TA v4.0; CTAL-TTA v4.0; CTAL-TAE v2.0 — TA 1.3; TTA 6.1.2; TAE 3.1.4 | 115 | Lab (code) | Kế hoạch |
| TEST-118 | Page Object Model | L4 | CTAL-TAE v2.0 — 3.1.5 | 080, 115 | Lab (code) | Kế hoạch |
| TEST-119 | API Automation | L4 | CTAL-TAE v2.0 — 5.1.3 | 070, 116 | Lab (code) | Kế hoạch |
| TEST-120 | UI Automation | L4 | CTAL-TAE v2.0; CTAL-TTA v4.0 — TAE 3.1.4; TTA 6.2.3 | 118 | Lab (code) | Kế hoạch |
| TEST-121 | Mobile Automation | L4 | CT-MAT; CTAL-TTA v4.0 — TTA 6.2.6 | 083, 120 | Thiết kế/phân tích | Kế hoạch |
| TEST-122 | CI/CD Testing Integration | L4 | CTAL-TAE v2.0; CTFL v4.0.1 — TAE 5.1; CTFL 2.1.4 | 114, 119 | Lab (đọc SPEC) | Kế hoạch |
| TEST-123 | Test Execution & Reporting | L4 | CTAL-TAE v2.0 — 6.1 | 122 | Lab (code) | Kế hoạch |
| TEST-124 | Flaky Test Management | L4 | CTAL-TAE v2.0; mở rộng — TAE 7.1 | 120, 123 | Lab (code) | Kế hoạch |
| TEST-125 | Test Maintenance | L4 | CTAL-TAE v2.0 — 4.3; 8.1 | 118, 124 | Lab (code) | Kế hoạch |
| TEST-126 | Automation ROI | L4 | CTAL-TM v3.0; CTAL-TAE v2.0 — TM 1.6.3; TAE 1.1 | 113, 125 | Giấy/bảng tính | Kế hoạch |

### S11 — Advanced Testing for Distributed Systems

18 bài · Phase 3 · 342 phút đọc · mức L4–L5

| ID | Tiêu đề | Level | ISTQB ref | Bài cần đọc trước | Thực hành | Trạng thái |
|---|---|---|---|---|---|---|
| TEST-127 | Distributed System Testing | L4 | Mở rộng ngoài ISTQB | 078, 085 | Thiết kế/phân tích | Kế hoạch |
| TEST-128 | Microservices Testing Strategy | L4 | Mở rộng ngoài ISTQB (liên quan CTAL-TAE v2.0 5.1) — TAE 5.1 (bối cảnh) | 127, 114 | Thiết kế/phân tích | Kế hoạch |
| TEST-129 | Contract Testing chuyên sâu | L4 | CTAL-TAE v2.0 (mở rộng) — TAE 5.1.3 | 074, 128 | Thiết kế/phân tích | Kế hoạch |
| TEST-130 | Consumer-driven Contract Testing | L4 | CTAL-TAE v2.0 (mở rộng) — TAE 5.1.3 | 129 | Môi trường riêng | Kế hoạch |
| TEST-131 | Event-driven Testing | L4 | Mở rộng ngoài ISTQB | 076, 127 | Môi trường riêng | Kế hoạch |
| TEST-132 | Message Ordering Testing | L4 | Mở rộng ngoài ISTQB | 131, 082 | Thiết kế/phân tích | Kế hoạch |
| TEST-133 | Duplicate Message Testing | L4 | Mở rộng ngoài ISTQB | 131 | Thiết kế/phân tích | Kế hoạch |
| TEST-134 | Idempotency Testing | L4 | Mở rộng ngoài ISTQB | 133, 078 | Lab + thiết kế | Kế hoạch |
| TEST-135 | Eventual Consistency Testing | L4 | Mở rộng ngoài ISTQB | 131 | Môi trường riêng | Kế hoạch |
| TEST-136 | Distributed Transaction Testing: Saga | L5 | Mở rộng ngoài ISTQB | 134, 135 | Thiết kế/phân tích | Kế hoạch |
| TEST-137 | Chaos Engineering | L5 | Mở rộng ngoài ISTQB | 094, 105, 065, 127, 138, 143 | Môi trường riêng | Kế hoạch |
| TEST-138 | Fault Injection Testing | L4 | CTAL-TTA v4.0; mở rộng — TTA 6.2.2; 4.4.4 | 078, 127 | Lab + thiết kế | Kế hoạch |
| TEST-139 | Resilience Testing: circuit breaker, retry, fallback | L4 | Mở rộng ngoài ISTQB | 138 | Lab + thiết kế | Kế hoạch |
| TEST-140 | Database Concurrency Testing: deadlock, lock contention | L4 | Mở rộng ngoài ISTQB | 086, 075 | Môi trường riêng | Kế hoạch |
| TEST-141 | Cache Testing: invalidation và cache stampede | L4 | Mở rộng ngoài ISTQB | 094, 135 | Môi trường riêng | Kế hoạch |
| TEST-142 | Data Consistency Testing: đối soát giữa service | L4 | Mở rộng ngoài ISTQB | 135, 136 | Thiết kế/phân tích | Kế hoạch |
| TEST-143 | Observability-driven Testing | L4 | Mở rộng ngoài ISTQB | 128, 123 | Môi trường riêng | Kế hoạch |
| TEST-144 | Disaster Recovery Testing | L5 | Mở rộng ngoài ISTQB (liên quan CTAL-TTA v4.0 4.4.5) — TTA 4.4.5 (bối cảnh) | 107, 108, 142 | Thiết kế/phân tích | Kế hoạch |

### S12 — Test Management & QA Architecture

18 bài · Phase 4 · 324 phút đọc · mức L5

| ID | Tiêu đề | Level | ISTQB ref | Bài cần đọc trước | Thực hành | Trạng thái |
|---|---|---|---|---|---|---|
| TEST-145 | Test Strategy và Test Plan | L5 | CTFL v4.0.1; CTAL-TM v3.0 — CTFL 5.1.1; TM 1.4 | 079, 113 | Thiết kế/phân tích | Kế hoạch |
| TEST-146 | Risk-based Testing | L5 | CTFL v4.0.1; CTAL-TM v3.0 — CTFL 5.2; TM 1.3 | 145 | Giấy/bảng tính | Kế hoạch |
| TEST-147 | Risk Identification & Analysis | L5 | CTAL-TM v3.0; CTFL v4.0.1; CTAL-TA v4.0 — TM 1.3.2–1.3.3; CTFL 5.2.2–5.2.3; TA 2.1 | 146 | Giấy/bảng tính | Kế hoạch |
| TEST-148 | Risk-based Test Prioritization | L5 | CTFL v4.0.1; CTAL-TM v3.0; CTAL-TA v4.0 — CTFL 5.1.5; TM 1.3.4; TA 3.5.1 | 147 | Lab (đọc SPEC) | Kế hoạch |
| TEST-149 | Test Estimation | L5 | CTFL v4.0.1; CTAL-TM v3.0 — CTFL 5.1.4; TM 2.2 | 145 | Giấy/bảng tính | Kế hoạch |
| TEST-150 | Test Monitoring & Control | L5 | CTAL-TM v3.0; CTFL v4.0.1 — TM 1.1.2–1.1.3; 2.1.2; CTFL 5.3 | 149 | Giấy/bảng tính | Kế hoạch |
| TEST-151 | Test Metrics & KPIs | L5 | CTAL-TM v3.0; CTFL v4.0.1 — TM 2.1; CTFL 5.3.1–5.3.2 | 150 | Thiết kế/phân tích | Kế hoạch |
| TEST-152 | Defect Management | L5 | CTFL v4.0.1; CTAL-TM v3.0 — CTFL 5.5; TM 2.3 | 004, 010 | Lab (đọc SPEC) | Kế hoạch |
| TEST-153 | Defect Root Cause Analysis | L5 | CTAL-TA v4.0; CTAL-TM v3.0 — TA 5.3; TM 2.3.5–2.3.6 | 152, 054 | Lab (đọc SPEC) | Kế hoạch |
| TEST-154 | Test Coverage Management | L5 | CTFL v4.0.1; CTAL-TM v3.0 — CTFL 1.4.4; TM 2.1 | 010, 062, 147 | Lab (code) | Kế hoạch |
| TEST-155 | Test Environment Management | L5 | CTAL-TA v4.0; CTAL-TM v3.0 — TA 1.3; TM 1.1 | 145 | Thiết kế/phân tích | Kế hoạch |
| TEST-156 | Test Data Management | L5 | CTAL-TA v4.0 — 1.3 | 155, 060 | Lab (code) | Kế hoạch |
| TEST-157 | Release Readiness Assessment | L5 | CTFL v4.0.1; CTAL-TM v3.0 — CTFL 5.1.3; 5.3.2; TM 2.1.3 | 151, 148 | Giấy/bảng tính | Kế hoạch |
| TEST-158 | Quality Gates trong CI/CD | L5 | Mở rộng ngoài ISTQB (liên quan CTAL-TAE v2.0 5.1) — TAE 5.1 (bối cảnh) | 122, 157 | Lab (code) | Kế hoạch |
| TEST-159 | Test Process Improvement | L5 | CTAL-TM v3.0; CTFL v4.0.1; CTEL-TM — TM 1.5; CTFL 2.1.6 | 153, 151 | Giấy/bảng tính | Kế hoạch |
| TEST-160 | QA Organization & Team Management | L5 | CTAL-TM v3.0; CTEL-TM — TM 3.1 | 009, 145 | Thiết kế/phân tích | Kế hoạch |
| TEST-161 | Test Architecture | L5 | Mở rộng ngoài ISTQB (liên quan CTAL-TAE v2.0 ch.2–3) — TAE 2–3 (bối cảnh) | 115, 128, 158 | Thiết kế/phân tích | Kế hoạch |
| TEST-162 | Quality Engineering và văn hoá chất lượng | L5 | CTAL-TM v3.0; mở rộng — TM 3.2 | 159, 161 | Giấy/bảng tính | Kế hoạch |

## 5. Chuỗi bài cần đọc trước (prerequisite chains)

Đồ thị tiền đề gồm 262 cạnh trên 163 bài; đã kiểm bằng thuật toán Kahn: **không có chu trình**, mọi mã tham chiếu đều tồn tại. Chuỗi dài nhất có 23 bài. Phần lớn tham chiếu trỏ về bài đứng trước; 3 tham chiếu trỏ tới bài đứng sau trong cùng series (lý do ở §7).

Cách đọc: danh sách «Toàn bộ bài nên đọc trước» là bao đóng bắc cầu (mọi tổ tiên), xếp theo một thứ tự đọc hợp lệ; «Chuỗi dài nhất» là đường đi dài nhất dẫn tới bài đích.

### TEST-137 · Chaos Engineering — đúng gợi ý của tài liệu định hướng (Performance, Reliability, Integration, Distributed Systems)

- Tiền đề trực tiếp: TEST-094 Performance Testing Fundamentals, TEST-105 Reliability Testing, TEST-065 Integration Testing Fundamentals, TEST-127 Distributed System Testing, TEST-138 Fault Injection Testing, TEST-143 Observability-driven Testing.
- Chuỗi dài nhất (20 bài): TEST-001 Software Testing là gì? Testing khác Debugging như thế nào? → TEST-002 Vì sao cần kiểm thử? (Why is Testing Necessary?) → TEST-006 Mục tiêu kiểm thử và Quality Assurance → TEST-007 Quy trình kiểm thử (Test Process): từ STLC đến CTFL v4 → TEST-008 Kiểm thử trong SDLC: Waterfall, Agile và DevOps → TEST-011 Tổng quan 5 test level của ISTQB → TEST-012 Component Testing (Unit Testing): góc nhìn tổng quan → TEST-013 Component Integration Testing → TEST-015 System Integration Testing → TEST-065 Integration Testing Fundamentals → TEST-066 Top-down Integration Testing → TEST-069 Incremental Integration Testing → TEST-079 System Testing Strategy → TEST-080 End-to-end Testing → TEST-084 Multi-user Scenario Testing → TEST-085 Concurrent Testing: hai tài xế cùng nhận một chuyến → TEST-127 Distributed System Testing → TEST-128 Microservices Testing Strategy → TEST-143 Observability-driven Testing → TEST-137 Chaos Engineering.
- Toàn bộ bài nên đọc trước (53 bài): TEST-001, TEST-002, TEST-004, TEST-006, TEST-007, TEST-008, TEST-010, TEST-011, TEST-012, TEST-013, TEST-014, TEST-015, TEST-017, TEST-018, TEST-019, TEST-031, TEST-043, TEST-044, TEST-050, TEST-055, TEST-057, TEST-059, TEST-060, TEST-065, TEST-066, TEST-067, TEST-068, TEST-069, TEST-070, TEST-075, TEST-077, TEST-078, TEST-079, TEST-080, TEST-084, TEST-085, TEST-093, TEST-094, TEST-095, TEST-098, TEST-105, TEST-112, TEST-113, TEST-114, TEST-115, TEST-116, TEST-119, TEST-122, TEST-123, TEST-127, TEST-128, TEST-138, TEST-143.

### TEST-136 · Saga (Distributed Transaction Testing)

- Tiền đề trực tiếp: TEST-134 Idempotency Testing, TEST-135 Eventual Consistency Testing.
- Chuỗi dài nhất (21 bài): TEST-001 Software Testing là gì? Testing khác Debugging như thế nào? → TEST-002 Vì sao cần kiểm thử? (Why is Testing Necessary?) → TEST-006 Mục tiêu kiểm thử và Quality Assurance → TEST-007 Quy trình kiểm thử (Test Process): từ STLC đến CTFL v4 → TEST-008 Kiểm thử trong SDLC: Waterfall, Agile và DevOps → TEST-011 Tổng quan 5 test level của ISTQB → TEST-012 Component Testing (Unit Testing): góc nhìn tổng quan → TEST-013 Component Integration Testing → TEST-015 System Integration Testing → TEST-065 Integration Testing Fundamentals → TEST-066 Top-down Integration Testing → TEST-069 Incremental Integration Testing → TEST-079 System Testing Strategy → TEST-080 End-to-end Testing → TEST-084 Multi-user Scenario Testing → TEST-085 Concurrent Testing: hai tài xế cùng nhận một chuyến → TEST-127 Distributed System Testing → TEST-131 Event-driven Testing → TEST-133 Duplicate Message Testing → TEST-134 Idempotency Testing → TEST-136 Distributed Transaction Testing: Saga.
- Toàn bộ bài nên đọc trước (39 bài): TEST-001, TEST-002, TEST-006, TEST-007, TEST-008, TEST-010, TEST-011, TEST-012, TEST-013, TEST-014, TEST-015, TEST-017, TEST-018, TEST-031, TEST-043, TEST-044, TEST-050, TEST-055, TEST-057, TEST-065, TEST-066, TEST-067, TEST-068, TEST-069, TEST-070, TEST-072, TEST-075, TEST-076, TEST-077, TEST-078, TEST-079, TEST-080, TEST-084, TEST-085, TEST-127, TEST-131, TEST-133, TEST-134, TEST-135.

### TEST-063 · Mutation Testing

- Tiền đề trực tiếp: TEST-062 Code Coverage trong thực hành.
- Chuỗi dài nhất (12 bài): TEST-001 Software Testing là gì? Testing khác Debugging như thế nào? → TEST-002 Vì sao cần kiểm thử? (Why is Testing Necessary?) → TEST-006 Mục tiêu kiểm thử và Quality Assurance → TEST-007 Quy trình kiểm thử (Test Process): từ STLC đến CTFL v4 → TEST-008 Kiểm thử trong SDLC: Waterfall, Agile và DevOps → TEST-011 Tổng quan 5 test level của ISTQB → TEST-017 Functional Testing → TEST-031 Equivalence Partitioning → TEST-043 Statement Coverage → TEST-044 Branch Coverage / Decision Coverage → TEST-062 Code Coverage trong thực hành → TEST-063 Mutation Testing.
- Toàn bộ bài nên đọc trước (17 bài): TEST-001, TEST-002, TEST-006, TEST-007, TEST-008, TEST-010, TEST-011, TEST-012, TEST-017, TEST-031, TEST-032, TEST-033, TEST-043, TEST-044, TEST-055, TEST-056, TEST-062.

### TEST-161 · Test Architecture

- Tiền đề trực tiếp: TEST-115 Test Automation Architecture, TEST-128 Microservices Testing Strategy, TEST-158 Quality Gates trong CI/CD.
- Chuỗi dài nhất (20 bài): TEST-001 Software Testing là gì? Testing khác Debugging như thế nào? → TEST-002 Vì sao cần kiểm thử? (Why is Testing Necessary?) → TEST-006 Mục tiêu kiểm thử và Quality Assurance → TEST-007 Quy trình kiểm thử (Test Process): từ STLC đến CTFL v4 → TEST-008 Kiểm thử trong SDLC: Waterfall, Agile và DevOps → TEST-011 Tổng quan 5 test level của ISTQB → TEST-012 Component Testing (Unit Testing): góc nhìn tổng quan → TEST-013 Component Integration Testing → TEST-015 System Integration Testing → TEST-065 Integration Testing Fundamentals → TEST-066 Top-down Integration Testing → TEST-069 Incremental Integration Testing → TEST-079 System Testing Strategy → TEST-145 Test Strategy và Test Plan → TEST-149 Test Estimation → TEST-150 Test Monitoring & Control → TEST-151 Test Metrics & KPIs → TEST-157 Release Readiness Assessment → TEST-158 Quality Gates trong CI/CD → TEST-161 Test Architecture.
- Toàn bộ bài nên đọc trước (54 bài): TEST-001, TEST-002, TEST-004, TEST-006, TEST-007, TEST-008, TEST-010, TEST-011, TEST-012, TEST-013, TEST-014, TEST-015, TEST-017, TEST-018, TEST-019, TEST-031, TEST-043, TEST-044, TEST-050, TEST-055, TEST-057, TEST-059, TEST-060, TEST-065, TEST-066, TEST-067, TEST-068, TEST-069, TEST-070, TEST-075, TEST-077, TEST-078, TEST-079, TEST-080, TEST-084, TEST-085, TEST-112, TEST-113, TEST-114, TEST-115, TEST-116, TEST-119, TEST-122, TEST-127, TEST-128, TEST-145, TEST-146, TEST-147, TEST-148, TEST-149, TEST-150, TEST-151, TEST-157, TEST-158.

## 6. Chồng lấn và cách tách

Nguyên tắc: bài đứng trước là **tổng quan / góc nhìn khái niệm**, bài đứng sau là **đi sâu / góc nhìn kỹ thuật–thực thi**; bài sau luôn lấy bài trước làm tiền đề và Description của cả hai nói rõ ranh giới.

| Chủ đề | Bài trước (góc nhìn) | Bài sau (góc nhìn) | Cách tách |
|---|---|---|---|
| Component / Unit testing | TEST-012 (test level: mục tiêu, test object, test basis) | TEST-055, TEST-056 (kỹ thuật viết unit test, thiết kế bộ test Pricing Engine) | TEST-012 không viết code; S06 là thực hành developer/SDET |
| Component integration | TEST-013 (test level) | TEST-065…069 (chiến lược tích hợp) | S02 nêu khái niệm và defect giao tiếp; S07 chọn chiến lược và viết test |
| System integration / bên thứ ba | TEST-015 (SIT tổng quan) | TEST-077 (sandbox PSP, webhook có chữ ký, quota Map) | S02 vẽ ngữ cảnh; S07 kiểm hành vi tích hợp cụ thể |
| System testing | TEST-014 (test level) | TEST-079 (chiến lược system test) · TEST-145 (strategy cấp tổ chức) | Ba tầng: khái niệm → chiến lược cho một test level → strategy/plan quản lý |
| Acceptance testing | TEST-016 (phân loại UAT/OAT/hợp đồng/alpha-beta) | TEST-092 (acceptance criteria, ATDD, tổ chức UAT, ký nghiệm thu) | Phân loại vs thực thi |
| Non-functional | TEST-018 (test type, ISO 25010) | TEST-093 (NFR đo được, operational profile) và cả S09 | Tổng quan vs đặc tả và lập kế hoạch |
| State transition | TEST-034 (state table, valid transitions coverage) | TEST-082 (guard, actor, 1-switch, nhiều state machine liên kết, event trễ) | CTFL K3 cơ bản vs CTAL-TA nâng cao |
| Checklist | TEST-028 (checklist trong review tĩnh) | TEST-053 (checklist khi thực thi test) | Static vs dynamic |
| API testing | TEST-050 (từ code: nhánh trong handler, white-box) | TEST-070, TEST-071 (hộp đen theo HTTP, EP/BVA) | White-box vs black-box; TEST-050 là tiền đề của TEST-070 |
| Auth/Authz | TEST-073 (kiểm quyền ở mức API: 401/403/IDOR) | TEST-102 (account takeover, OTP abuse, leo quyền, maker-checker Admin Portal) | Chức năng phân quyền vs kiểm thử bảo mật |
| Code coverage | TEST-043…047 (kỹ thuật white-box) | TEST-062 (công cụ, ngưỡng, coverage ≠ chất lượng) | Lý thuyết vs vận hành công cụ |
| Data-driven | TEST-060 (cấp unit, dữ liệu bảng giá/voucher) | TEST-116 (cấp framework, chạy song song) | Unit vs automation framework |
| Data-based technique | TEST-040 (nhóm kỹ thuật data-based của CTAL-TA: domain testing) | TEST-060, TEST-116 (data-driven) | Đặt tên rõ để không nhầm «data-based» với «data-driven» (xem §7) |
| Contract testing | TEST-074 (tổng quan trong integration, breaking change gRPC) | TEST-129 (hợp đồng có phiên bản, schema event, chặn deploy) → TEST-130 (CDC/Pact) | Khái niệm vs quy trình trong hệ phân tán |
| Message queue / event | TEST-076 (một producer/consumer, outbox relay) | TEST-131…133 (khẳng định bất đồng bộ, thứ tự, trùng lặp) | Tích hợp với broker vs hành vi hệ event-driven |
| Timeout / retry / idempotency | TEST-078 (timeout + retry cùng khoá trên lab) | TEST-134 (ma trận idempotency, fingerprint) · TEST-138, TEST-139 (fault injection, resilience) | Một lời gọi cụ thể vs cơ chế tổng quát |
| Concurrency / race | TEST-085, TEST-086 (cấp hệ thống, lab) | TEST-140 (tầng database: lock, deadlock, isolation) | Hành vi nghiệp vụ vs cơ chế DB |
| Cross-platform / compatibility | TEST-083 (nhất quán chức năng giữa nền tảng) · TEST-089 (ma trận thiết bị/OS) | TEST-109 (co-existence, interoperability theo ISO 25010) | Ba góc: chức năng, ma trận môi trường, đặc tính chất lượng |
| Recovery | TEST-088 (thành phần chết giữa luồng nghiệp vụ) | TEST-107 (RTO/RPO, backup/restore) → TEST-108 (failover) → TEST-144 (DR) | Hệ thống → đặc tính → hạ tầng → vùng |
| Usability / accessibility | TEST-090, TEST-091 (thực thi phiên test, WCAG) | TEST-111 (đo lường SUS, heuristic, báo cáo tuân thủ) | Thực thi vs đo lường đặc tính chất lượng |
| Test strategy | TEST-079 (system test) | TEST-113 (automation) · TEST-128 (microservices) · TEST-145 (tổ chức/dự án) | Mỗi bài một phạm vi strategy |
| Root cause | TEST-004 (khái niệm) | TEST-054 (defect-based testing) · TEST-153 (RCA, 5 Whys, Ishikawa) | Khái niệm → thiết kế test → quản lý cải tiến |
| Test pyramid | TEST-114 (pyramid + quadrants của CTFL) | TEST-128 (honeycomb cho microservices) | Mô hình chung vs biến thể phân tán |

## 7. Điểm cần chủ blog quyết

1. **«Data-based Testing» ở S04 #10** được tài liệu định hướng mô tả là «kiểm thử với tập dữ liệu lớn». Trong CTAL-TA v4.0, «data-based test techniques» là tên **nhóm** kỹ thuật (domain testing, combinatorial, random). Bài TEST-040 đã đổi thành «Data-based test techniques: domain testing nhiều biến» (chạy trên bảng dữ liệu lớn chỉ là phần ứng dụng). Nếu muốn giữ chủ đề «dữ liệu lớn», nên đổi tên bài để không lệch thuật ngữ ISTQB.
2. **Metamorphic, CRUD, random, scenario-based testing** có LO chính thức trong CTAL-TA v4.0 (TA-3.3.2, TA-3.2.1, TA-3.1.3, TA-3.2.3) — không phải nội dung mở rộng.
3. **Use Case Testing (S04 #5)** không còn trong CTFL v4; CTAL-TA v4.0 gọi là scenario-based testing. TEST-035 dùng tên ghép và ánh xạ TA-3.2.3.
4. **Condition coverage và path testing (S05 #3, #6)** không còn LO riêng trong CTAL-TTA v4.0 (TTA-2.6.1 basis path đã bỏ). **Loop testing (S05 #7)** không có trong syllabus hiện hành — đã ghi «(mở rộng)» / «Mở rộng ngoài ISTQB».
5. **Thứ tự Series 03**: «Review Process và các vai trò» (#7) đứng sau các loại review (#3–#6) nhưng Inspection (#6) cần kiến thức quy trình review ⇒ TEST-026 có tiền đề trỏ tới TEST-027. Đề xuất đưa #7 lên ngay sau #2.
6. **Thứ tự Series 11**: Chaos Engineering (#11) đứng trước Fault Injection (#12) và Observability (#17) — cả hai là tiền đề hợp lý của chaos ⇒ TEST-137 trỏ tới TEST-138 và TEST-143. Đề xuất chuyển Chaos xuống cuối nhóm resilience.
7. **Ba mô hình trạng thái khác nhau**: tài liệu định hướng §7 dùng `CREATED → CONFIRMED → DRIVER_ASSIGNED → … → COMPLETED`; hồ sơ dự án và lab dùng 8 trạng thái `requested … completed/cancelled`; kiến trúc §9.1 tách state theo domain (Booking/Dispatch/Trip/Payment). Curriculum dùng bản của lab cho bài thực hành và bản §9.1 cho bài nâng cao (TEST-082, S11). Nên sửa ví dụ §7 của tài liệu định hướng cho thống nhất.
8. **Biên quãng đường**: ví dụ BVA ở §4 tài liệu định hướng dùng 1–100 km, SPEC lab dùng [0,5; 100] km có làm tròn half-up 0,1 km. Bài TEST-032 theo SPEC; ví dụ trong tài liệu định hướng nên đổi theo.
9. **Số liệu tải**: S09 nêu «10.000 khách đặt xe đồng thời» và «10.000–50.000 request/giây», trong khi hồ sơ dự án cho 70 chuyến tạo mới/giây và ≈ 30.000 lượt đọc/giây lúc cao điểm. Bài TEST-095 suy load profile từ hồ sơ dự án.
10. **SLO tạo chuyến** — *đã xử lý:* SPEC v1.1 của lab lấy đúng SLO khởi tạo §14.1 (R-NFR-01…04: 99,9%/tháng; p95 Booking và Quote < 500 ms; lan truyền trạng thái p95 < 2–3 giây); 70 chuyến/giây và 30.000 lượt đọc/giây chỉ còn là mức tải tham chiếu (R-NFR-07).
11. **Phạm vi dịch vụ**: tài liệu định hướng nói hai dịch vụ (đặt xe, giao hàng); hồ sơ dự án và lab có ba (`ride`, `food`, `parcel`). Curriculum dùng ba.
12. **Bài thực hành giữa các series**: S05 (voucher đạt statement/branch/MC/DC), S06 (calculateFare), S07 (Payment timeout), S08 (hai tài xế cùng nhận), S09 (performance cho hệ đặt xe) có bài thực hành mô tả riêng nhưng không có trong danh sách chủ đề. Curriculum gộp chúng vào TEST-047, TEST-056, TEST-078, TEST-085, TEST-095 thay vì thêm dòng (chỉ Series 01 có dòng bài thực hành riêng). Nếu muốn mỗi series có bài «Thực hành Series NN» như S01, thêm mã `TEST-Snn-LAB`.
13. **Chữ ký hàm calculateFare(distance, vehicleType, surgeMultiplier, discount)** ở §6 tài liệu định hướng khác lab (hệ số cao điểm suy từ giờ đặt, voucher theo mã). TEST-056 giữ tinh thần bài tập nhưng chạy trên API của lab.
14. **CTEL-TM** (Expert Level Test Management) là syllabus cũ; chưa kiểm phiên bản hiện hành. Ở S12 chủ yếu ánh xạ CTAL-TM v3.0 (đã kiểm), CTEL-TM chỉ ghi kèm ở TEST-159, TEST-160.
15. **Mã LO CTAL-TTA chương 5** in trong syllabus là «TTA 5.1.1», «TTA 5.2.1» (thiếu gạch nối); bảng chuẩn hoá thành `TTA-5.x.x`.

Tham chiếu tới bài đứng sau (forward reference) hiện có: TEST-026 → TEST-027, TEST-137 → TEST-138, TEST-137 → TEST-143.

