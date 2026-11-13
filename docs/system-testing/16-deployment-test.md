# SYS-16 — Deployment Test: bản triển khai có đúng cấu hình, sẵn sàng và quay lui được?

Code đã qua mọi suite vẫn có thể hỏng khi triển khai: biến môi trường sai, API kiểm thử bị bật, migration làm bản cũ không chạy được khi cần rollback. Deployment Test kiểm chính hành động triển khai và bản đang chạy sau đó, không kiểm lại logic nghiệp vụ.

Bài trước: [UAT](15-uat.md). Đây là bài cuối của series. [Mã thực hành](../../examples/toi-noi-testing/articles/system-testing-deployment-test/README.md) · [Báo cáo thực chạy](16-test-report.md).

## 1. Mục tiêu và chuẩn bị

- Phân biệt build, release, deploy, readiness và post-deploy smoke; biết mỗi phép kiểm chứng minh điều gì.
- Khởi động ứng dụng như một bản triển khai (tiến trình riêng, cấu hình qua môi trường) và chờ readiness có hạn chót.
- Viết phép kiểm cấu hình an toàn và chứng minh nó **có thể đỏ** bằng một bản cấu hình sai có chủ đích.
- Kiểm tương thích schema theo expand/contract để rollback mã ứng dụng không làm hỏng dữ liệu.
- Lập quyết định giữ, roll-back hay roll-forward dựa trên bằng chứng.

Cần đã đọc SYS-01 (smoke), SYS-13 (constraint, SQL) và SYS-14 (rollback dữ liệu khác rollback mã). Lab cần Node.js ≥22.12 và dependency của `examples/toi-noi-testing`; không cần DB, Docker hay quyền mạng ngoài.

## 2. Khái niệm và những chỗ dễ nhầm

| Khái niệm | Câu hỏi trả lời | Ví dụ Tới Nơi |
| --- | --- | --- |
| Build | Artifact có tạo được, tái lập được không? | Gói mã + lockfile cố định |
| Deploy | Artifact có chạy được trên môi trường đích với cấu hình đích không? | `NODE_ENV=production`, `TOINOI_DB=pglite` |
| Liveness/health | Tiến trình có phản hồi không? | `GET /api/health` (R-API-15) |
| Readiness | Instance đã đủ điều kiện nhận traffic chưa? | Lab dùng health làm tín hiệu; hệ thật cần kiểm dependency |
| Post-deploy smoke | Đường nghiệp vụ chính có chạy trên bản vừa triển khai không? | Báo giá → đặt → hoàn tất → đọc lại qua HTTP thật |
| Rollback | Quay về mã/artifact trước | Cần schema còn tương thích với mã cũ |
| Roll-forward | Sửa bằng bản mới thay vì quay lui | Khi migration không đảo ngược được |

Deployment test không phải test level riêng; nó thường nằm cuối system testing và trong vận hành (operational acceptance). Smoke của SYS-01 kiểm mã đã lắp ráp trong tiến trình test; post-deploy smoke kiểm **bản đang chạy**, qua cổng thật, với cấu hình thật. Rollback mã khác restore dữ liệu (SYS-14): rollback mã không đưa schema về cũ.

## 3. Test basis, rủi ro và oracle

Basis: `src/server.ts` (danh sách biến môi trường), R-API-15 (health), R-TEST-01 (API kiểm thử chỉ bật khi `NODE_ENV=test` hoặc `TOINOI_TEST_API=1`), R-NFR-06 (API kiểm thử tắt mặc định; 500 không lộ chi tiết), R-FARE-01/08 (oracle 35.000), `schema.sql` và `pg-trip-repository.ts`.

| Rủi ro | Hậu quả | Phép kiểm |
| --- | --- | --- |
| Cấu hình kiểm thử lọt vào môi trường thật | Ai cũng reset dữ liệu được | DP-02, DP-05 |
| Sai biến môi trường (kho dữ liệu, giờ) | Chạy nhầm kho memory, dữ liệu mất khi restart | DP-03 |
| Tiến trình mở cổng nhưng nghiệp vụ hỏng | Health xanh, khách không đặt được | DP-04 |
| Migration không tương thích mã cũ | Rollback mã gây 500 hoặc đọc sai | MG-01…03 |

Oracle độc lập: 35.000 tính tay theo bảng cước (12.000 + 4.500 × 5 = 34.500, làm tròn lên 35.000 theo R-FARE-08); 404 `NOT_FOUND` theo R-TEST-01; thông điệp 500 chung theo R-NFR-06.

Entry: dependency cài theo lockfile, cổng do hệ điều hành cấp, không dùng dữ liệu thật. Exit bài tập: 8/8 pass, typecheck pass, không còn tiến trình máy chủ sau suite. Đây là điều kiện đạt bài tập, không phải phê duyệt release production.

## 4. Thiết kế ca kiểm

| ID | Basis/rủi ro | Tiền điều kiện và hành động | Expected | Cleanup |
| --- | --- | --- | --- | --- |
| DP-01 | R-API-15 | Khởi động bản prod-like; hỏi health mỗi 100 ms, hạn chót 15 s | 200 `{status:'ok'}` trước hạn chót | Dừng tiến trình |
| DP-02 | R-TEST-01, R-NFR-06 | Bản prod-like; POST `/api/test/reset` | 404 `NOT_FOUND` | Dừng tiến trình |
| DP-03 | Cấu hình đang chạy | Đọc log khởi động | `kho: pglite`, `API test: tắt`, giờ 2026-11-04T02:00Z | Dừng tiến trình |
| DP-04 | R-FARE-01/08, R-STATE-02, R-PAY-03 | Báo giá 5 km xe máy SG, đặt VíXanh, D101 nhận và hoàn tất, đọc lại | 35.000; completed/paid; `pricingVersion` `fare-2026.1` | Dừng tiến trình |
| DP-05 | Phép kiểm biết đỏ | Bản cấu hình sai `TOINOI_TEST_API=1` | `/api/test/reset` khác 404; log `API test: BẬT` | Dừng tiến trình |
| MG-01 | Expand an toàn | Thêm cột nullable; bản mới ghi cột; bản cũ nhận chuyến | Bản cũ chạy; giá trị cột mới còn; dòng bản cũ tạo có NULL | DB riêng, đóng |
| MG-02 | Expand sai | Cột NOT NULL không default | Bản cũ tạo chuyến → 500 chung; không có dòng | DB riêng, đóng |
| MG-03 | Contract quá sớm | Đổi tên `cancel_reason` | Ghi → 500; đọc → 200 nhưng mất `cancelReason` | DB riêng, đóng |

"Bản cũ" trong MG là mã hiện tại của `src/`: lab có một phiên bản mã, nên migration được áp trước và mã hiện tại đóng vai bản sẽ quay về. Đây là mô phỏng rollback mã, không phải triển khai hai phiên bản.

## 5. Thực hành

Từ `examples/toi-noi-testing`:

```powershell
npm ci
npm test -- articles/system-testing-deployment-test
npm run typecheck
```

`deployment.test.ts` khởi động `src/server.ts` bằng `node --import tsx` trên cổng trống. Chi tiết quan trọng: Vitest tự đặt `NODE_ENV=test`, nên test **xoá** biến này trước khi tạo tiến trình con. Không xoá, DP-02 sẽ chạy trên bản có API kiểm thử bật và phép kiểm vô nghĩa. Readiness dùng vòng hỏi có hạn chót thay cho sleep cố định. `afterEach` dừng mọi tiến trình con; báo cáo ghi lại việc kiểm không còn tiến trình mồ côi.

DP-05 khởi động bản sai có chủ đích. Nó không phải "ca muốn pass" mà là bằng chứng phép kiểm của DP-02 phân biệt được hai cấu hình; một phép kiểm chưa từng đỏ trên bản sai thì chưa chứng minh được gì.

`migration-compat.test.ts` tạo PGlite riêng mỗi test, áp ALTER TABLE rồi gọi API bằng mã hiện tại. Expected và observed ở [báo cáo](16-test-report.md): 8/8 pass, `readyAfterMs` 1.710–2.275 ms qua bốn lượt — số quan sát trên máy lab, không phải SLO khởi động.

## 6. Đọc thất bại và quyết định

| Dấu hiệu | Giả thuyết | Bằng chứng tiếp theo |
| --- | --- | --- |
| "Máy chủ thoát sớm" | Thiếu dependency, cổng, lỗi khởi tạo PGlite | Log tiến trình con có trong thông điệp lỗi |
| "Không sẵn sàng sau 15000 ms" | Khởi động chậm, nghe sai cổng | Log khởi động, tài nguyên máy |
| DP-02 trả 200 | API kiểm thử bị bật (NODE_ENV/TOINOI_TEST_API) | Môi trường tiến trình, log `API test:` |
| DP-04 lệch 35.000 | Giờ cấu hình, mưa, bảng giá | `TOINOI_START_AT`, `pricingVersion`, SPEC |
| MG đỏ | Migration không tương thích mã đang chạy | Câu ALTER, danh sách cột mã cũ dùng |

**MG-03 là phát hiện cần nhấn mạnh:** sau khi đổi tên cột, ghi trả 500 nhưng **đọc vẫn 200**, chỉ thiếu trường `cancelReason`. Nguyên nhân: repository đọc bằng `SELECT *` và ánh xạ theo tên cột, cột cũ không còn thì trường thành `undefined` và bị bỏ khỏi JSON. Monitor chỉ đếm 5xx sẽ thấy lỗi ghi nhưng bỏ sót đọc sai. Post-deploy smoke cần assert nội dung, không chỉ status.

Quyết định mẫu: DP pass, MG-01 pass ⇒ có thể triển khai bản mới cùng migration expand, giữ cửa sổ rollback. Nếu migration đã contract, rollback mã không còn an toàn ⇒ roll-forward bằng bản sửa, hoặc chạy migration bù theo kế hoạch đã diễn tập.

## 7. Mở rộng nâng cao (chưa chạy trong lab)

**Artifact bất biến và version.** Hệ thật cần build một lần, triển khai cùng artifact qua các môi trường, gắn build ID/commit vào endpoint version và log. Post-deploy smoke assert version trước khi kiểm nghiệp vụ. Lab không có endpoint version; DP-03 dùng log khởi động thay thế.

**Secret và cấu hình.** Secret lấy từ secret manager, không nằm trong repo hay log. Kiểm: thiếu secret thì tiến trình phải từ chối khởi động (fail fast) thay vì chạy với giá trị rỗng; log không in giá trị secret. Lab không có secret.

**Readiness thật.** Readiness nên phản ánh dependency thiết yếu (DB, cổng thanh toán) và trạng thái khởi động (migration xong). Health của lab không chạm DB, như SYS-01 đã chỉ ra.

**Canary và rolling.** Đưa một phần traffic vào bản mới, so sánh tỷ lệ lỗi, độ trễ và **chỉ số nghiệp vụ** (tỷ lệ đặt thành công, số tiền thu) với bản cũ trong cửa sổ quan sát; điều kiện dừng và tự rollback phải chốt trước. Trong rolling, hai phiên bản chạy cùng lúc trên cùng schema — đó là lý do expand/contract bắt buộc.

**Migration trên PostgreSQL thật.** Đo thời gian khoá bảng, chạy trên dữ liệu cỡ production, dùng `lock_timeout`, tạo index đồng thời khi cần. PGlite một kết nối không kiểm được.

**Rollback hai phiên bản thật.** Triển khai v2, ghi dữ liệu, quay về v1 trên staging, chạy smoke và đối soát (SYS-13). Đây là diễn tập nên làm trước release có migration.

## 8. Anti-pattern và giới hạn

Chỉ kiểm health sau triển khai. Sleep cố định thay cho chờ có hạn chót. Chạy smoke với API kiểm thử bật vì "tiện reset". Migration đổi tên/xoá cột cùng release với mã mới. Coi rollback mã là rollback dữ liệu. Smoke chỉ assert status code. Để tiến trình test mồ côi chiếm cổng.

Giới hạn bằng chứng: một tiến trình một máy, chạy từ mã nguồn qua tsx, không build artifact, container, load balancer, TLS, nhiều instance, staging hay canary. Không có hai phiên bản mã thật. Không kiểm graceful shutdown (Windows `kill()` dừng cưỡng bức). Migration chạy trên PGlite, không đo khoá hay thời gian.

## 9. Bài tập, câu hỏi tự kiểm và tài liệu

1. **Cơ bản:** bỏ dòng `delete env.NODE_ENV`. Dự đoán DP-02 và DP-03 ra sao? Gợi ý: API kiểm thử bật, `/api/test/reset` không còn 404; DP-03 đỏ ở `API test: tắt`.
2. **Trung bình:** viết MG-04 thêm cột `channel text NOT NULL DEFAULT 'app'`. Expected: bản cũ tạo chuyến được (201), dòng mới có `'app'`. So sánh với MG-02.
3. **Trung bình:** sửa smoke để bắt được MG-03 trên đường đọc. Gợi ý: assert tập trường hợp đồng của `trip`, gồm cả trường có thể null.
4. **Nâng cao:** thiết kế canary cho thay đổi bảng cước `fare-2026.2`. Đáp án cần: so sánh theo `pricingVersion`, chỉ số tiền thu, điều kiện dừng, cửa sổ quan sát, cách rollback khi báo giá cũ còn hiệu lực.
5. **Tự kiểm:** DP-01 pass có chứng minh bản đã sẵn sàng phục vụ khách? Không — health của lab không chạm DB; DP-04 mới đi qua đường tiền. Vì sao DP-05 cần thiết? Vì nó cho thấy phép kiểm đỏ khi cấu hình sai.

Tài liệu tra cứu ngày 13/11/2026: [ISTQB CTFL v4.0.1](https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf) (test levels/types, maintenance testing); [PostgreSQL 18 — ALTER TABLE](https://www.postgresql.org/docs/18/sql-altertable.html) (thêm cột, NOT NULL, đổi tên, khoá). Nguồn nội bộ: [SPEC](../../examples/toi-noi-testing/SPEC.md), [`src/server.ts`](../../examples/toi-noi-testing/src/server.ts), [repository chuyến](../../examples/toi-noi-testing/src/infra/pglite/pg-trip-repository.ts).
