---
slug: system-testing-deployment-test
title: "Deployment Test: bản vừa triển khai có đúng cấu hình, sẵn sàng và còn quay lui được?"
summary: "SYS-16, bài cuối của series kiểm thử hệ thống: khởi động máy chủ Tới Nơi như một bản triển khai thật, chờ readiness có hạn chót, kiểm cấu hình an toàn và chứng minh phép kiểm biết đỏ, chạy post-deploy smoke qua HTTP thật, rồi kiểm migration expand/contract để rollback mã không làm hỏng dữ liệu — kèm phát hiện một lệnh đọc trả 200 nhưng mất trường mà không báo lỗi."
category: testing
level: advanced
tags: [testing, sys-16, deployment-testing, post-deploy-smoke, readiness, configuration, schema-migration, rollback, expand-contract, vitest]
language: vi
---

Thứ Sáu, 17:40. Bản mới của Tới Nơi vừa lên. Pipeline xanh từ đầu tới cuối: unit, integration, system, regression. `/api/health` trả 200. Mười phút sau, một kỹ sư phát hiện ai đó gọi được `POST /api/test/reset` trên môi trường thật — biến `TOINOI_TEST_API=1` từ cấu hình staging được chép nhầm sang. Cùng tối đó, đội quyết định rollback vì một lỗi giao diện, và mọi lệnh tạo chuyến trả 500: migration của bản mới đã đổi tên một cột mà bản cũ còn dùng. Không có dòng code nghiệp vụ nào sai. Thứ sai là **hành động triển khai**. Bài này trả lời: **deployment test kiểm cái gì mà các suite trước không kiểm, làm sao chờ một bản triển khai sẵn sàng mà không sleep đoán mò, làm sao biết phép kiểm cấu hình của mình thật sự có thể đỏ, và thiết kế migration thế nào để rollback mã còn an toàn**.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · SYS-16</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>Kiểm thử hệ thống Tới Nơi · Phần 2</td></tr>
<tr><th scope="row">Cấp độ</th><td>Advanced — QA/SDET, kỹ sư release/vận hành đã nắm smoke, SQL và khái niệm rollback</td></tr>
<tr><th scope="row">Test level / hướng kiểm</th><td>Cuối system testing, giáp operational acceptance: kiểm hành động triển khai, cấu hình đang chạy và khả năng rollback; không kiểm lại logic nghiệp vụ</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/system-testing-smoke-test">SYS-01 · Smoke Test</a> · <a href="/articles/system-testing-recovery-test">SYS-14 · Recovery Test</a> · <a href="/articles/system-testing-uat">SYS-15 · UAT</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): 5 test case DP-01…05 khởi động <code>src/server.ts</code> thành tiến trình riêng và kiểm qua HTTP thật (R-API-15, R-TEST-01, R-NFR-06, R-FARE-01/08); 3 test case MG-01…03 kiểm migration expand/contract với mã đang chạy — <code>examples/toi-noi-testing/articles/system-testing-deployment-test/</code></td></tr>
<tr><th scope="row">Nguồn chính</th><td>ISTQB CTFL v4.0.1 (test level/type, maintenance testing); PostgreSQL 18 · ALTER TABLE; <code>SPEC.md</code>, <code>src/server.ts</code>, <code>src/infra/pglite/pg-trip-repository.ts</code> của lab</td></tr>
</tbody>
</table>
</div>

## 1. Bài toán và mục tiêu

### 1.1. Tình huống

Mười lăm bài trước đều kiểm **mã**: hàm tính cước, chỗ nối repository, hành trình khách–tài xế, độ trễ, quyền truy cập, dữ liệu, phục hồi, nghiệm thu. Nhưng mã không tự chạy. Giữa "mã đã đúng" và "khách đặt được xe" còn một bước: đem mã lên một môi trường, với một bộ cấu hình, trên một schema database có sẵn dữ liệu, và mở cho traffic thật. Bước đó hỏng theo những cách không suite nào trước đó chạm tới:

- **Cấu hình sai.** Cùng một mã, `NODE_ENV=test` thì có `/api/test/reset`, `NODE_ENV=production` thì không (R-TEST-01). Một biến môi trường chép nhầm là đủ biến bản triển khai thành cửa hậu.
- **Sẵn sàng giả.** Tiến trình đã mở cổng nhưng chưa khởi tạo xong kho dữ liệu; hoặc health xanh trong khi đường tiền hỏng — SYS-01 đã chỉ ra health của lab không chạm DB.
- **Không quay lui được.** Migration chạy trước hoặc cùng lúc mã mới. Nếu nó xoá hoặc đổi tên thứ mã cũ cần, nút rollback trở thành nút gây sự cố.

### 1.2. Đối tượng và điều kiện tiên quyết

Bài dành cho người đã viết smoke test (SYS-01), đọc được SQL và constraint (SYS-13), và hiểu rollback mã khác restore dữ liệu (SYS-14). Lab cần Node.js ≥22.12 và dependency của `examples/toi-noi-testing` cài theo lockfile. Không cần PostgreSQL ngoài, Docker hay quyền mạng ra ngoài: máy chủ lab chạy với `TOINOI_DB=pglite`, PostgreSQL dạng WASM trong chính tiến trình.

### 1.3. Mục tiêu đo được

Sau bài này, bạn có thể:

1. Phân biệt build, deploy, liveness, readiness, post-deploy smoke, rollback và roll-forward, và nói mỗi phép kiểm chứng minh điều gì.
2. Khởi động ứng dụng thành tiến trình riêng với cấu hình kiểu triển khai và chờ readiness bằng vòng hỏi có hạn chót.
3. Viết phép kiểm cấu hình an toàn, và chứng minh nó **đỏ** trên một bản cấu hình sai có chủ đích.
4. Kiểm một migration theo nguyên tắc expand/contract: bản cũ còn đọc/ghi được sau expand, và nhận ra contract quá sớm phá rollback.
5. Đọc kết quả và ra quyết định giữ, rollback hay roll-forward có bằng chứng.

## 2. Khái niệm và những chỗ dễ nhầm

### 2.1. Mỗi phép kiểm trả lời một câu hỏi

| Khái niệm | Câu hỏi trả lời | Ví dụ Tới Nơi |
|---|---|---|
| Build | Artifact có tạo được và tái lập được không? | Mã + lockfile cố định |
| Deploy | Artifact có chạy trên môi trường đích với cấu hình đích không? | `NODE_ENV=production`, `TOINOI_DB=pglite` |
| Liveness / health | Tiến trình có phản hồi không? | `GET /api/health` ⇒ 200 `{ "status": "ok" }` (R-API-15) |
| Readiness | Instance đủ điều kiện nhận traffic chưa? | Lab dùng health làm tín hiệu; hệ thật cần kiểm dependency |
| Post-deploy smoke | Đường nghiệp vụ chính có chạy trên **bản đang chạy** không? | Báo giá → đặt → hoàn tất → đọc lại qua cổng thật |
| Rollback | Quay về artifact trước | Chỉ an toàn nếu schema còn tương thích mã cũ |
| Roll-forward | Sửa bằng bản mới thay vì quay lui | Khi migration không đảo ngược được |

### 2.2. Deployment test khác smoke của SYS-01 ở đâu

SYS-01 chạy smoke trên ứng dụng **lắp ráp trong tiến trình test** bằng supertest: không có cổng mạng, không có biến môi trường, không có bước khởi động. Smoke test trả lời "mã của build này có đủ tốt để kiểm thử chuyên sâu không". Deployment test chạy trên **bản đang chạy**: một tiến trình riêng, đọc cấu hình từ môi trường như khi triển khai, nghe một cổng, và được kiểm qua HTTP thật. Deployment test trả lời "chính bản triển khai này có đúng cấu hình, sẵn sàng, và dùng được không".

Deployment test không phải một test level của ISTQB. Nó thường nằm cuối system testing và chồng lấn với operational acceptance testing (mục kiểm tra khả năng cài đặt, cấu hình, backup/restore trong acceptance). Ghi rõ phạm vi trong test plan quan trọng hơn tranh luận tên gọi.

### 2.3. Rollback mã không phải restore dữ liệu

SYS-14 restore **dữ liệu** từ bản sao. Rollback trong bài này đưa **mã** về phiên bản trước, còn schema và dữ liệu giữ nguyên trạng thái mới nhất. Vì vậy câu hỏi then chốt của rollback là: *mã cũ có chạy được trên schema mới không?* Nếu không, rollback biến một lỗi nhỏ thành sự cố toàn hệ thống — đúng như tình huống mở bài.

## 3. Test basis, rủi ro và oracle

### 3.1. Test basis

- `src/server.ts`: danh sách biến môi trường được chú thích ngay đầu tệp (`PORT`, `TOINOI_DB`, `TOINOI_START_AT`, `TOINOI_RAIN`, `TRIP_RATE_LIMIT_*`, `NODE_ENV`/`TOINOI_TEST_API`) và dòng log khởi động.
- **R-API-15** — `GET /api/health` ⇒ 200 `{ "status": "ok" }`, dùng cho giám sát và để công cụ kiểm thử chờ máy chủ sẵn sàng.
- **R-TEST-01** — các endpoint `/api/test/*` chỉ tồn tại khi `NODE_ENV=test` hoặc `TOINOI_TEST_API=1`; mặc định (kể cả `npm start`) trả 404 `NOT_FOUND`.
- **R-NFR-06** — API kiểm thử tắt mặc định; lỗi 500 không lộ stack trace.
- **R-FARE-01…08** — oracle tiền cho post-deploy smoke.
- `schema.sql` và `pg-trip-repository.ts` — cách mã hiện tại đọc/ghi bảng `trips`, nền cho các test case migration.

### 3.2. Rủi ro và nơi kiểm

| Rủi ro | Hậu quả | Phép kiểm |
|---|---|---|
| Cấu hình kiểm thử lọt vào môi trường thật | Ai cũng reset được dữ liệu, đổi giờ, đổi chế độ ví | DP-02, DP-05 |
| Sai kho dữ liệu hoặc giờ hệ thống | Chạy nhầm kho memory, mất dữ liệu khi restart; giá sai giờ cao điểm | DP-03 |
| Mở cổng nhưng nghiệp vụ hỏng | Health xanh, khách không đặt/không trả được tiền | DP-04 |
| Khởi động treo hoặc chậm bất thường | Traffic vào instance chưa sẵn sàng | DP-01 |
| Migration không tương thích mã cũ | Rollback gây 500 hoặc đọc sai âm thầm | MG-01…03 |

### 3.3. Oracle độc lập

- **Tiền:** xe máy 5 km tại SG, 09:00 giờ Việt Nam (không phải cao điểm theo R-FARE-04: [11:00; 13:00) và [17:00; 19:00)), không mưa, không voucher: 12.000 + 4.500 × 5 = 34.500, làm tròn **lên** tới bội 1.000 theo R-FARE-08 ⇒ **35.000**. Số này tính tay từ SPEC, không gọi lại hàm tính cước.
- **Cấu hình:** 404 `NOT_FOUND` theo R-TEST-01; log khởi động in `API test: tắt`.
- **Lỗi:** thông điệp 500 chung `Lỗi hệ thống, vui lòng thử lại sau.` theo R-NFR-06.
- **Phiên bản giá:** `pricingVersion` `fare-2026.1` — hằng số `PRICING_VERSION` của mã; đây là oracle về **bản đang chạy**, không phải về nghiệp vụ.

### 3.4. Tiêu chí bắt đầu và kết thúc

Entry: dependency cài theo lockfile; cổng do hệ điều hành cấp; không có dữ liệu thật. Exit của bài tập: 8/8 test case pass, typecheck pass, và không còn tiến trình máy chủ nào sau suite. Đây là điều kiện **đạt bài tập**, không phải phê duyệt release production.

## 4. Thiết kế test case

### 4.1. Bảng test case

| ID | Basis / rủi ro | Tiền điều kiện và hành động | Expected | Cleanup |
|---|---|---|---|---|
| DP-01 | R-API-15 | Khởi động bản prod-like; hỏi health mỗi 100 ms, hạn chót 15 s | 200 `{status:'ok'}` trước hạn chót | Dừng tiến trình |
| DP-02 | R-TEST-01, R-NFR-06 | Bản prod-like; `POST /api/test/reset` | 404 `NOT_FOUND` | Dừng tiến trình |
| DP-03 | Cấu hình đang chạy | Đọc log khởi động | `kho: pglite`, `API test: tắt`, giờ `2026-11-04T02:00:xxZ` | Dừng tiến trình |
| DP-04 | R-FARE-01/08, R-STATE-02, R-PAY-03 | Báo giá 5 km xe máy SG; đặt VíXanh; D101 nhận, tới, đón, chạy, hoàn tất; khách đọc lại | 35.000; completed/paid; `pricingVersion` `fare-2026.1` | Dừng tiến trình |
| DP-05 | Phép kiểm biết đỏ | Bản **cấu hình sai** `TOINOI_TEST_API=1` | `/api/test/reset` khác 404; log `API test: BẬT` | Dừng tiến trình |
| MG-01 | Expand an toàn | Thêm cột nullable; bản mới ghi cột; bản cũ tạo, nhận chuyến, đọc lại | Bản cũ chạy; giá trị cột mới còn nguyên; dòng bản cũ tạo có NULL | DB riêng, đóng |
| MG-02 | Expand sai | Thêm cột NOT NULL không default | Bản cũ tạo chuyến ⇒ 500 chung; bảng trips trống | DB riêng, đóng |
| MG-03 | Contract quá sớm | Đổi tên `cancel_reason` | Ghi ⇒ 500; đọc chuyến cũ ⇒ 200 nhưng mất `cancelReason` | DB riêng, đóng |

### 4.2. Một mã, hai vai

Lab chỉ có **một** phiên bản mã. Để kiểm rollback, các test case MG áp migration trước, rồi dùng mã hiện tại trong `src/` đóng vai **bản cũ** — bản đội sẽ quay về. Đây là mô phỏng trung thực của câu hỏi "mã cũ có chạy trên schema mới không", nhưng không phải triển khai hai phiên bản thật; mục 9 nói cách làm phần đó trên staging.

### 4.3. Vì sao DP-05 là một test case, không phải một lỗi

DP-05 không "muốn" hệ thống tốt. Test case này cố ý khởi động một bản sai để chứng minh rằng phép kiểm của DP-02 **phân biệt được** cấu hình đúng và cấu hình sai. Một phép kiểm chưa từng đỏ trên một đầu vào sai thì chưa chứng minh được nó có kiểm gì. Đây cùng tinh thần với thí nghiệm inject lỗi mà SYS-01 đề xuất.

## 5. Kiến trúc lab

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Hai nửa của lab SYS-16. Bên trái: tiến trình Vitest xoá NODE_ENV=test, khởi động tiến trình con node --import tsx src/server.ts với cấu hình NODE_ENV=production, TOINOI_DB=pglite, TOINOI_START_AT, trên cổng trống; test hỏi /api/health tới khi 200 hoặc hết hạn 15 giây, rồi gọi HTTP thật và đọc log khởi động. Bên phải: mỗi ca MG tạo PGlite riêng, áp ALTER TABLE, rồi gọi API bằng mã hiện tại đóng vai bản cũ.">
<defs>
<marker id="sys16-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<text x="230" y="22" text-anchor="middle" class="t-b">deployment.test.ts · DP-01…05</text>
<rect x="20" y="40" width="170" height="70" rx="8" class="f-phys-soft s-phys"/><text x="105" y="66" text-anchor="middle" class="t-b">Tiến trình Vitest</text><text x="105" y="86" text-anchor="middle" class="t-small">xoá NODE_ENV=test</text><text x="105" y="101" text-anchor="middle" class="t-small">xoá TOINOI_TEST_API</text>
<rect x="250" y="40" width="200" height="110" rx="8" class="f-pk-soft s-pk"/><text x="350" y="64" text-anchor="middle" class="t-b">Tiến trình máy chủ</text><text x="350" y="84" text-anchor="middle" class="t-small">node --import tsx src/server.ts</text><text x="350" y="102" text-anchor="middle" class="t-small">NODE_ENV=production</text><text x="350" y="118" text-anchor="middle" class="t-small">TOINOI_DB=pglite · cổng trống</text><text x="350" y="136" text-anchor="middle" class="t-small">TOINOI_START_AT 09:00 VN</text>
<line x1="190" y1="62" x2="246" y2="62" class="s-phys" stroke-width="1.6" marker-end="url(#sys16-arrow)"/><text x="218" y="56" text-anchor="middle" class="t-small">spawn</text>
<line x1="250" y1="130" x2="194" y2="98" class="s-pk dash" stroke-width="1.4" marker-end="url(#sys16-arrow)"/>
<rect x="20" y="170" width="430" height="44" rx="8" class="f-bg s-pk dash"/><text x="235" y="190" text-anchor="middle">hỏi /api/health mỗi 100 ms ⇒ 200 hoặc hết hạn 15 s</text><text x="235" y="206" text-anchor="middle" class="t-small">không sleep cố định; tiến trình thoát sớm ⇒ lỗi kèm log</text>
<rect x="20" y="230" width="430" height="44" rx="8" class="f-bg s-pk"/><text x="235" y="250" text-anchor="middle">HTTP thật: test API, smoke đường tiền · đọc log khởi động</text><text x="235" y="266" text-anchor="middle" class="t-small">afterEach: dừng mọi tiến trình con</text>
<line x1="470" y1="20" x2="470" y2="290" class="s-pk dash" stroke-width="1"/>
<text x="695" y="22" text-anchor="middle" class="t-b">migration-compat.test.ts · MG-01…03</text>
<rect x="500" y="40" width="190" height="60" rx="8" class="f-warn-soft s-warn"/><text x="595" y="64" text-anchor="middle" class="t-b">PGlite riêng</text><text x="595" y="84" text-anchor="middle" class="t-small">mỗi test một database</text>
<rect x="720" y="40" width="180" height="60" rx="8" class="f-warn-soft s-warn"/><text x="810" y="64" text-anchor="middle" class="t-b">ALTER TABLE trips</text><text x="810" y="84" text-anchor="middle" class="t-small">expand / sai / contract</text>
<line x1="690" y1="70" x2="716" y2="70" class="s-warn" stroke-width="1.6" marker-end="url(#sys16-arrow)"/>
<rect x="560" y="140" width="290" height="60" rx="8" class="f-pk-soft s-pk"/><text x="705" y="164" text-anchor="middle" class="t-b">API với mã hiện tại</text><text x="705" y="184" text-anchor="middle" class="t-small">đóng vai "bản cũ" sau rollback</text>
<line x1="810" y1="100" x2="760" y2="136" class="s-pk" stroke-width="1.6" marker-end="url(#sys16-arrow)"/>
<text x="705" y="236" text-anchor="middle" class="t-small">oracle: status HTTP + nội dung JSON + SQL đọc thẳng bảng trips</text>
<text x="705" y="258" text-anchor="middle" class="t-small t-warn">không có hai phiên bản mã thật · không đo khoá bảng</text>
</svg>
</div>
<figcaption><b>Hình 1 · Hai nửa của lab SYS-16.</b> Nửa trái kiểm bản triển khai qua cổng thật; nửa phải kiểm câu hỏi "mã cũ có chạy trên schema mới không". Không nửa nào chạm <code>src/</code> dùng chung.</figcaption>
</figure>

## 6. Thực hành A — khởi động như một bản triển khai

### 6.1. Chạy lab

Từ `examples/toi-noi-testing`:

```powershell
npm ci
npm test -- articles/system-testing-deployment-test
npm run typecheck
```

### 6.2. Bước 1 — Xoá cấu hình mà công cụ test tự thêm

Vitest tự đặt `NODE_ENV=test` cho tiến trình của nó. Nếu tiến trình máy chủ thừa kế nguyên môi trường, nó sẽ bật API kiểm thử theo đúng R-TEST-01 — và DP-02 sẽ kiểm một cấu hình **không phải** cấu hình triển khai. Đây là lỗi thiết kế test rất dễ mắc: phép kiểm vẫn chạy, chỉ là kiểm sai thứ.

```ts
async function deploy(extraEnv: Record<string, string>): Promise<Deployment> {
  const port = await freePort();
  const env: NodeJS.ProcessEnv = { ...process.env };
  delete env.NODE_ENV; // Vitest tự đặt NODE_ENV=test — phải xoá, nếu không API kiểm thử sẽ bật.
  delete env.TOINOI_TEST_API;
  Object.assign(env, { PORT: String(port), ...extraEnv });

  const child = spawn(process.execPath, ['--import', 'tsx', 'src/server.ts'], { cwd: LAB_ROOT, env });
```

Hai lựa chọn khác cũng có lý do. `freePort()` xin hệ điều hành một cổng trống thay vì cố định 3100, để không đụng máy chủ lab bạn đang chạy và để hai test case không tranh cổng. `node --import tsx` chạy mã TypeScript trong **một** tiến trình node; dùng lệnh `tsx` trực tiếp có thể sinh tiến trình con, khi bị dừng trên Windows dễ để lại tiến trình mồ côi giữ cổng.

### 6.3. Bước 2 — Chờ readiness có hạn chót

```ts
  const baseUrl = `http://127.0.0.1:${port}`;
  const started = Date.now();
  while (Date.now() - started < READY_DEADLINE_MS) {
    if (child.exitCode !== null) throw new Error(`Máy chủ thoát sớm (exit ${child.exitCode}):\n${output}`);
    try {
      const res = await fetch(`${baseUrl}/api/health`);
      if (res.status === 200) return { baseUrl, process: child, log: () => output, readyAfterMs: Date.now() - started };
    } catch {
      // Chưa nghe cổng — thử lại sau 100 ms.
    }
    await new Promise((r) => setTimeout(r, 100));
  }
  throw new Error(`Không sẵn sàng sau ${READY_DEADLINE_MS} ms:\n${output}`);
}
```

Ba điểm phân biệt vòng chờ này với `sleep(5000)`:

- **Thoát sớm khi chắc chắn hỏng.** Tiến trình đã chết thì chờ thêm vô ích; thông điệp lỗi mang theo log để điều tra ngay.
- **Có hạn chót.** 15 giây là giả định thiết kế của bài, đủ rộng cho máy chậm. Con số này không phải SLO khởi động; một hệ thật cần hạn chót có căn cứ từ đo đạc.
- **Đo được.** `readyAfterMs` được ghi lại để theo dõi xu hướng, nhưng không assert ngưỡng chặt — thời gian khởi động trên máy dev dao động.

### 6.4. Bước 3 — Bốn test case trên bản prod-like

Cấu hình triển khai của lab:

```ts
const PROD_LIKE = {
  NODE_ENV: 'production',
  TOINOI_DB: 'pglite',
  TOINOI_START_AT: '2026-11-04T09:00:00+07:00',
};
```

DP-02 kiểm điều R-TEST-01 hứa:

```ts
  it('DP-02 · R-TEST-01/R-NFR-06: cấu hình triển khai không mở API kiểm thử', async () => {
    const d = await deploy(PROD_LIKE);
    const res = await fetch(`${d.baseUrl}/api/test/reset`, { method: 'POST' });
    expect(res.status).toBe(404);
    expect(await res.json()).toMatchObject({ error: { code: 'NOT_FOUND' } });
  }, 30_000);
```

DP-03 đọc bằng chứng cấu hình từ log khởi động. Lab không có endpoint version, nên dòng log do `src/server.ts` in ra là nơi duy nhất bản đang chạy "tự khai" nó dùng kho nào, giờ nào, API test bật hay tắt:

```ts
    expect(d.log()).toContain('kho: pglite');
    expect(d.log()).toContain('API test: tắt');
    expect(d.log()).toMatch(/giờ hệ thống: 2026-11-04T02:00:\d{2}/);
```

09:00 giờ Việt Nam là 02:00 UTC. Đồng hồ của máy chủ lab là `AdjustableClock`: dời tới mốc rồi chạy tiếp, nên giây có thể lệch vài đơn vị — regex chỉ cố định tới phút.

DP-04 là post-deploy smoke: cùng đường tiền như SYS-01 nhưng đi qua `fetch` tới cổng thật, có header `Idempotency-Key` như client thật, và đọc lại cả `pricingVersion`:

```ts
    const read = await call('GET', `/api/trips/${id}`, 'customer:C001');
    expect(read.status).toBe(200);
    expect(read.body.trip).toMatchObject({ id, status: 'completed', fare: 35_000, paymentStatus: 'paid', pricingVersion: 'fare-2026.1' });
```

`pricingVersion` trả lời câu hỏi mà health không trả lời: bản đang chạy dùng bảng cước nào. Nếu một release đổi bảng cước mà smoke vẫn thấy `fare-2026.1`, bản mới chưa thực sự lên.

### 6.5. Bước 4 — DP-05: phép kiểm phải đỏ trên bản sai

```ts
  it('DP-05 · phép kiểm cấu hình bắt được bản triển khai sai: bật TOINOI_TEST_API=1 thì API kiểm thử lộ ra', async () => {
    const d = await deploy({ ...PROD_LIKE, TOINOI_TEST_API: '1' });
    const res = await fetch(`${d.baseUrl}/api/test/reset`, { method: 'POST' });
    // Đây là bản triển khai SAI có chủ đích: phép kiểm DP-02 chạy trên nó phải đỏ.
    expect(res.status).not.toBe(404);
    expect(d.log()).toContain('API test: BẬT');
  }, 30_000);
```

Cùng `NODE_ENV=production`, chỉ thêm một cờ, và cửa hậu mở. Đó đúng là kịch bản mở bài. DP-05 pass có nghĩa: nếu DP-02 được chạy trên bản này, nó sẽ đỏ. Ta có bằng chứng rằng phép kiểm của mình bắt được lỗi thật.

### 6.6. Bước 5 — Dọn dẹp

```ts
afterEach(async () => {
  for (const child of running.splice(0)) {
    if (child.exitCode === null) {
      const exited = new Promise((r) => child.once('exit', r));
      child.kill();
      await exited;
    }
  }
});
```

Chờ sự kiện `exit` trước khi sang test case sau, không chỉ gửi tín hiệu rồi đi tiếp. Báo cáo ghi lại kết quả kiểm bằng `Get-CimInstance Win32_Process`: không còn tiến trình `node … src/server.ts` nào sau các lượt chạy. Trên Windows `kill()` dừng cưỡng bức, nên lab **không** kiểm graceful shutdown dù `src/server.ts` có xử lý `SIGTERM`.

## 7. Thực hành B — migration và khả năng rollback

### 7.1. Đọc mã trước khi thiết kế test case

Hai chi tiết trong `pg-trip-repository.ts` quyết định cách mã hiện tại phản ứng với schema mới. Phần ghi dùng danh sách cột liệt kê rõ ràng:

```ts
const INSERT_SQL = `INSERT INTO trips (${COLUMNS.join(', ')}) VALUES (${COLUMNS.map((_, i) => `$${i + 1}`).join(', ')})`;
```

Đọc dùng `SELECT *` rồi ánh xạ theo tên cột:

```ts
    const { rows } = await this.db.query<TripRow>('SELECT * FROM trips WHERE id = $1', [id]);
```

```ts
    cancelReason: r.cancel_reason,
```

Từ đó có thể dự đoán: thêm cột mới không làm hỏng ghi (cột không có trong danh sách) và không làm hỏng đọc (cột thừa bị bỏ qua); bỏ hoặc đổi tên một cột trong danh sách sẽ làm hỏng ghi; còn đọc thì… cần chạy mới biết. Đó là lý do MG-03 tồn tại.

### 7.2. MG-01 — expand an toàn

```ts
    await sql('ALTER TABLE trips ADD COLUMN pickup_note text');
    // Bản mới (giả lập bằng SQL) ghi giá trị cho cột mới.
    await system.db!.query('UPDATE trips SET pickup_note = $1 WHERE id = $2', ['Cổng số 2', oldTripId]);
```

Sau đó bản cũ nhận chính chuyến đó — tức chạy `UPDATE` toàn bộ các cột nó biết trên dòng đã có dữ liệu của bản mới — rồi tạo thêm một chuyến:

```ts
    expect(rows).toEqual([
      { id: oldTripId, pickup_note: 'Cổng số 2' }, // bản cũ UPDATE cả dòng nhưng không chạm cột nó không biết
      { id, pickup_note: null },                    // dòng do bản cũ tạo: bản mới phải chịu được NULL
    ]);
```

Hai assertion này là hai nửa của hợp đồng expand: bản cũ không xoá dữ liệu của bản mới, và bản mới phải chịu được dữ liệu bản cũ để lại (NULL) trong suốt cửa sổ rollback.

### 7.3. MG-02 — expand sai

```ts
    await sql(`ALTER TABLE trips ADD COLUMN channel text;
               UPDATE trips SET channel = 'app';
               ALTER TABLE trips ALTER COLUMN channel SET NOT NULL;`);
    const res = await book();
    expect(res.status).toBe(500);
    expect(res.body).toEqual({ error: { code: 'INTERNAL_ERROR', message: 'Lỗi hệ thống, vui lòng thử lại sau.' } });
```

Migration "chỉ thêm cột" vẫn có thể phá bản cũ: cột NOT NULL không default khiến mọi `INSERT` không biết cột đó vi phạm ràng buộc. Test case này còn xác nhận R-NFR-06 trên đường lỗi: phản hồi 500 là thông điệp chung, không lộ tên cột hay câu SQL. Bảng `trips` trống sau đó — không có dòng dở dang.

### 7.4. MG-03 — contract quá sớm và phát hiện quan trọng nhất của bài

```ts
    await sql('ALTER TABLE trips RENAME COLUMN cancel_reason TO cancellation_reason');

    const write = await book();
    expect(write.status).toBe(500);
    expect(write.body.error.code).toBe('INTERNAL_ERROR');

    const afterRead = await request(app).get(`/api/trips/${id}`).set('x-user', 'customer:C001').expect(200);
    expect(afterRead.body.trip).not.toHaveProperty('cancelReason');
```

Phần ghi hỏng, đúng như dự đoán. Nhưng **đọc trả 200**. `SELECT *` không còn cột `cancel_reason`, `r.cancel_reason` thành `undefined`, và JSON bỏ trường `undefined` — trước migration, cùng chuyến đó có `cancelReason: null`. Khách không thấy lỗi; ứng dụng khách chỉ thấy thiếu một trường.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Đọc sai mà không báo lỗi, qua mặt monitor chỉ đếm 5xx</span>
<p>Sau rollback trong tình huống MG-03, dashboard tỷ lệ 5xx sẽ đỏ ở đường ghi — nhưng đường đọc vẫn "khoẻ" trong khi trả dữ liệu thiếu. Một post-deploy smoke chỉ kiểm status code cũng bỏ sót. Phép kiểm sau triển khai phải assert <b>nội dung</b> theo hợp đồng (tập trường kỳ vọng, kể cả trường có thể null), và migration contract chỉ được chạy khi không còn phiên bản nào dùng cột cũ. Đây là hành vi quan sát được của mã lab, ghi lại làm ví dụ; bài không sửa <code>src/</code>.</p>
</div>

### 7.5. Expand → migrate → contract

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 230" role="img" aria-label="Dòng thời gian của một thay đổi schema theo expand và contract. Release 1 chạy migration expand thêm cột nullable, mã v1 vẫn chạy. Release 2 triển khai mã v2 ghi cả hai dạng; cửa sổ rollback về v1 còn mở. Sau khi backfill và không còn v1, release 3 chạy contract xoá cột cũ; từ đây rollback về v1 không còn an toàn, chỉ roll-forward. MG-03 mô phỏng việc contract chạy khi v1 còn có thể quay lại.">
<line x1="30" y1="110" x2="895" y2="110" class="s-pk" stroke-width="2" marker-end="url(#sys16-arrow)"/>
<rect x="40" y="40" width="190" height="50" rx="8" class="f-phys-soft s-phys"/><text x="135" y="62" text-anchor="middle" class="t-b">Expand</text><text x="135" y="80" text-anchor="middle" class="t-small">thêm cột nullable (MG-01)</text>
<rect x="260" y="40" width="190" height="50" rx="8" class="f-pk-soft s-pk"/><text x="355" y="62" text-anchor="middle" class="t-b">Triển khai v2</text><text x="355" y="80" text-anchor="middle" class="t-small">ghi cả dạng cũ và mới</text>
<rect x="480" y="40" width="170" height="50" rx="8" class="f-pk-soft s-pk"/><text x="565" y="62" text-anchor="middle" class="t-b">Backfill + quan sát</text><text x="565" y="80" text-anchor="middle" class="t-small">không còn v1 chạy</text>
<rect x="680" y="40" width="200" height="50" rx="8" class="f-warn-soft s-warn"/><text x="780" y="62" text-anchor="middle" class="t-b">Contract</text><text x="780" y="80" text-anchor="middle" class="t-small">xoá/đổi tên cột cũ</text>
<rect x="40" y="128" width="610" height="34" rx="6" class="f-bg s-phys dash"/><text x="345" y="150" text-anchor="middle">cửa sổ rollback về v1: schema vẫn tương thích mã cũ</text>
<rect x="680" y="128" width="200" height="34" rx="6" class="f-warn-soft s-warn"/><text x="780" y="150" text-anchor="middle">chỉ roll-forward</text>
<text x="460" y="194" text-anchor="middle" class="t-small t-warn">MG-03 = contract chạy khi v1 còn có thể quay lại ⇒ ghi 500, đọc 200 nhưng thiếu trường</text>
<text x="460" y="216" text-anchor="middle" class="t-small">MG-02 = "expand" nhưng NOT NULL không default ⇒ v1 không ghi được ngay từ bước đầu</text>
</svg>
</div>
<figcaption><b>Hình 2 · Expand → migrate → contract và cửa sổ rollback.</b> Mỗi bước là một release riêng có phép kiểm riêng. Lab chỉ kiểm điểm đầu (expand) và hai cách làm sai; các bước giữa là quy trình đề xuất.</figcaption>
</figure>

Nguyên tắc rút ra: tách một thay đổi phá vỡ thành các release mà **mỗi release đều tương thích với release ngay trước nó**. Bài [Schema migration tương thích](/articles/postgresql-schema-migration-tuong-thich) trong series PostgreSQL đi sâu kỹ thuật SQL; bài này tập trung vào phép kiểm chứng minh tính tương thích.

## 8. Kết quả: mong đợi và thực tế

### 8.1. Mong đợi

8/8 test case pass như bảng mục 4.1; typecheck exit 0; không còn tiến trình máy chủ sau suite. `readyAfterMs` không có ngưỡng ngoài hạn chót 15 giây.

### 8.2. Thực tế — báo cáo ngày 13/11/2026

Windows 11, Node.js 22.18.0, Vitest 3.2.7, tsx 4.23.15, PGlite 0.3.16:

```text
Tests  8 passed (8)
Duration  9.32s (… tests 12.10s …)
```

- `readyAfterMs` của DP-01 qua bốn lượt chạy: **1.710, 2.275, 1.789, 1.926 ms**. Đây là quan sát trên máy lab, không phải SLO khởi động.
- Typecheck pass sau khi sửa ba lỗi kiểu `unknown` ở lần chạy đầu — các lỗi của chính mã test, không phải của sản phẩm.
- Sau các lượt chạy không còn tiến trình `node … src/server.ts` nào.
- MG-03 quan sát đúng như mục 7.4: ghi 500, đọc 200 thiếu `cancelReason`.

### 8.3. Thực tế — lượt chạy lại khi biên tập bài, cùng ngày

Cùng máy và phiên bản: 8/8 pass, duration 9,66 s, `readyAfterMs` 1.846 ms — nằm trong khoảng của bốn lượt trước.

### 8.4. Một ghi nhận ngoài phạm vi

Khi chạy toàn bộ `npm test` của lab, 85/86 file pass; 3 test của bài TEST-040 (domain testing) đỏ cả khi chạy riêng, theo cùng một kiểu mỗi lần. Bộ test đó và `src/` không bị thay đổi trong đợt viết bài này, nên đây là lỗi sẵn có cần điều tra riêng. Trường hợp này minh hoạ một nguyên tắc của deployment gate: ghi rõ phần đỏ không thuộc thay đổi đang xét, thay vì bỏ qua mà không nói gì, hoặc chặn toàn bộ mà không phân loại.

## 9. Đọc thất bại và ra quyết định

| Dấu hiệu | Giả thuyết | Bằng chứng tiếp theo |
|---|---|---|
| `Máy chủ thoát sớm (exit …)` | Thiếu dependency, lỗi khởi tạo PGlite, cổng bị chiếm | Log tiến trình con nằm ngay trong thông điệp lỗi |
| `Không sẵn sàng sau 15000 ms` | Khởi động chậm bất thường, nghe sai cổng | Log khởi động, tài nguyên máy, `PORT` |
| DP-02 nhận 200 | API kiểm thử bật do `NODE_ENV` hoặc `TOINOI_TEST_API` | Môi trường tiến trình, dòng log `API test:` |
| DP-04 lệch 35.000 | Giờ cấu hình, thành phố mưa, bảng cước | `TOINOI_START_AT`, `TOINOI_RAIN`, `pricingVersion` |
| MG-01 đỏ | Migration được gọi là expand nhưng không tương thích | Câu ALTER, danh sách cột mã cũ ghi |
| Chạy riêng pass, chạy chung fail | Tranh cổng, tiến trình mồ côi | Danh sách tiến trình node, `freePort()` |

Quyết định mẫu:

- **Giữ bản mới:** DP-01…04 pass trên bản đang chạy, DP-05 đã chứng minh phép kiểm cấu hình bắt được lỗi thật, migration của release chỉ là expand đã qua MG-01.
- **Rollback:** lỗi trong mã mới, migration của release là expand ⇒ quay về an toàn; chạy lại post-deploy smoke trên bản cũ.
- **Roll-forward:** migration đã contract (tình huống MG-03) ⇒ rollback mã sẽ gây 500 và đọc sai; sửa bằng bản mới hoặc chạy migration bù đã diễn tập.

## 10. Mở rộng nâng cao — chưa chạy trong lab

Các phần dưới đây là thiết kế đề xuất cho Tới Nơi thật, **không** được thực hiện trong bài.

### 10.1. Artifact bất biến và endpoint version

Build một lần, triển khai cùng artifact qua dev → staging → production; khác nhau chỉ ở cấu hình. Gắn commit/build ID vào endpoint version và log. Post-deploy smoke assert version **trước** khi kiểm nghiệp vụ, để kết quả không thuộc nhầm bản cũ còn đang chạy. Lab không có endpoint version; DP-03 dùng log khởi động thay thế và `pricingVersion` chỉ phản ánh bảng cước, không phải toàn bộ build.

### 10.2. Secret và fail fast

Secret lấy từ secret manager, không nằm trong repo, image hay log. Phép kiểm nên có: thiếu secret bắt buộc ⇒ tiến trình từ chối khởi động với thông điệp rõ (fail fast) thay vì chạy với chuỗi rỗng rồi lỗi ở request đầu tiên; log khởi động không in giá trị secret. Đây cũng là chỗ áp tư duy DP-05: thử khởi động thiếu secret và kỳ vọng thất bại.

### 10.3. Readiness thật

Readiness nên phản ánh dependency thiết yếu (DB trả lời, migration đã xong, cổng thanh toán đạt được) và tách khỏi liveness — instance chưa sẵn sàng không nên bị khởi động lại, chỉ không nhận traffic. Health của lab chỉ trả `{status:'ok'}` mà không chạm DB, như SYS-01 đã chỉ ra; vì vậy DP-04 mới là bằng chứng nghiệp vụ.

### 10.4. Rolling, blue-green và canary

Trong rolling, hai phiên bản chạy cùng lúc trên cùng schema — đó là lý do expand/contract không phải tuỳ chọn. Với canary cho thay đổi bảng cước của Tới Nơi: đưa một phần nhỏ traffic vào bản mới, so sánh tỷ lệ lỗi, p95 (SYS-07) và **chỉ số nghiệp vụ** theo `pricingVersion` — tỷ lệ đặt thành công, tổng tiền thu, tỷ lệ huỷ — với bản cũ trong cửa sổ quan sát chốt trước; vượt điều kiện dừng thì tự rollback. Lưu ý báo giá đã phát ra dưới bảng cước cũ vẫn còn hiệu lực (R-QUOTE) khi chuyển bản.

### 10.5. Migration trên PostgreSQL thật

PGlite một kết nối không đo được thời gian giữ khoá, ảnh hưởng tới traffic đồng thời hay thời gian chạy trên bảng cỡ production. Trên staging với dữ liệu cỡ thật: đặt `lock_timeout`, tạo index đồng thời khi cần, đo thời gian migration và kiểm truy vấn đang chạy trong lúc đó. Xem thêm [Lock hàng và bảng](/articles/postgresql-lock-row-table).

### 10.6. Rollback giữa hai phiên bản thật

Diễn tập trên staging trước release có migration: triển khai v2, tạo dữ liệu qua v2, quay về v1, chạy post-deploy smoke **có assert nội dung**, rồi đối soát như SYS-13. Đây là phiên bản đầy đủ của MG-01 và là chỗ bắt được MG-03 trước khi khách thấy.

## 11. Anti-pattern và giới hạn bằng chứng

- Chỉ kiểm health sau triển khai và gọi đó là "đã verify".
- `sleep(10000)` thay cho chờ readiness có hạn chót.
- Chạy smoke trên bản bật API kiểm thử "cho tiện reset" — kiểm một cấu hình khác cấu hình thật.
- Phép kiểm cấu hình chưa từng được chạy trên một bản sai.
- Migration đổi tên/xoá cột trong cùng release với mã mới.
- Coi rollback mã là rollback dữ liệu.
- Smoke chỉ assert status code.
- Để tiến trình test mồ côi giữ cổng, làm lượt sau đỏ khó hiểu.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Lab này chứng minh gì và không chứng minh gì</span>
<p><b>Có:</b> máy chủ khởi động được từ mã nguồn với cấu hình kiểu triển khai; phép kiểm cấu hình phân biệt được bản bật/tắt API kiểm thử; post-deploy smoke đi hết đường tiền qua HTTP thật; mã hiện tại chịu được migration expand nhưng không chịu được contract sớm.</p>
<p><b>Không:</b> một tiến trình trên một máy, chạy từ mã nguồn qua tsx — không build artifact, container, load balancer, TLS, nhiều instance, staging, canary hay secret manager. Không có hai phiên bản mã thật. Không kiểm graceful shutdown. Migration chạy trên PGlite một kết nối, không đo khoá hay thời gian. Kết quả đạt bài tập SYS-16; không suy ra quy trình triển khai production đã được kiểm.</p>
</div>

## 12. Bài tập và câu hỏi tự kiểm

### Bài tập

**Bài 1 (cơ bản).** Xoá dòng `delete env.NODE_ENV;` trong `deploy()` rồi chạy lại. Dự đoán DP-02 và DP-03 ra sao trước khi chạy.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Tiến trình con thừa kế <code>NODE_ENV=test</code> từ Vitest ⇒ API kiểm thử bật theo R-TEST-01. DP-02 đỏ vì <code>/api/test/reset</code> không còn 404; DP-03 đỏ ở <code>API test: tắt</code> vì log in <code>BẬT</code>. Bài học: một phép kiểm triển khai phải kiểm soát môi trường của tiến trình được kiểm, không thừa kế môi trường của công cụ test.</p>
</details>

**Bài 2 (trung bình).** Viết MG-04: thêm cột `channel text NOT NULL DEFAULT 'app'`. Kỳ vọng gì khi bản cũ tạo chuyến? So sánh với MG-02.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Bản cũ tạo được (201) vì PostgreSQL điền default cho cột không có trong <code>INSERT</code>; dòng mới có <code>channel = 'app'</code>. Khác MG-02 ở chỗ có default. Cần thêm assert đọc thẳng SQL rằng giá trị là <code>'app'</code>, và lưu ý: default tĩnh có thể không đúng nghĩa nghiệp vụ cho chuyến tạo từ kênh khác — tương thích kỹ thuật chưa chắc đúng dữ liệu.</p>
</details>

**Bài 3 (trung bình).** Sửa post-deploy smoke để bắt được MG-03 trên đường đọc.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Assert tập trường hợp đồng của <code>trip</code>, ví dụ <code>expect(Object.keys(trip).sort()).toEqual([...])</code> hoặc <code>toHaveProperty('cancelReason')</code> cho cả trường có thể null. Chạy smoke đó sau migration rename: nó phải đỏ ở bước đọc, không chỉ bước ghi. Cân nhắc: assert tập trường cứng làm test nhạy với trường mới thêm hợp lệ — chọn assert "phải có" thay vì "chỉ có".</p>
</details>

**Bài 4 (nâng cao).** Thiết kế canary cho thay đổi bảng cước `fare-2026.2` của Tới Nơi.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Cần: tỷ lệ traffic ban đầu và các nấc tăng; chỉ số so sánh theo <code>pricingVersion</code> (tỷ lệ báo giá → đặt, tỷ lệ huỷ, tổng tiền thu, lỗi 5xx, p95); cửa sổ quan sát và điều kiện dừng chốt trước; cách xử lý báo giá đã phát dưới bảng cũ khi rollback; post-deploy smoke với oracle tính tay từ bảng mới. Trình bày rõ đây là thiết kế, chưa chạy.</p>
</details>

**Bài 5 (nâng cao).** Thêm DP-06: khởi động với `TOINOI_DB=postgres` (giá trị không hỗ trợ). Quan sát hành vi và đề xuất thay đổi.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Đọc <code>src/server.ts</code>: mọi giá trị khác <code>pglite</code> rơi về kho memory, và log in <code>kho: memory</code>. Bản triển khai sai này <b>khởi động thành công</b> — dữ liệu sẽ mất khi restart. DP-03 bắt được nhờ assert <code>kho: pglite</code>. Đề xuất cho sản phẩm (không sửa trong bài): giá trị lạ ⇒ từ chối khởi động. Đây là ví dụ phép kiểm cấu hình tìm ra rủi ro thiết kế.</p>
</details>

### Câu hỏi tự kiểm

**Câu 1.** DP-01 pass có chứng minh bản triển khai sẵn sàng phục vụ khách?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>Không. Health của lab chỉ trả <code>{status:'ok'}</code>, không chạm DB hay ví. DP-01 chứng minh tiến trình phản hồi trong hạn chót; DP-04 mới đi qua đường tiền. Với hệ thật, readiness cần kiểm dependency thiết yếu.</p>
</details>

**Câu 2.** Vì sao DP-05 cần thiết khi DP-02 đã pass?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>DP-02 pass chỉ cho thấy bản đúng không lộ API kiểm thử. DP-05 cho thấy cùng phép kiểm phân biệt được bản sai. Không có DP-05, một DP-02 luôn xanh vì lý do khác (ví dụ gọi sai đường dẫn) vẫn trông như bằng chứng.</p>
</details>

**Câu 3.** Rollback mã sau migration contract khác rollback sau migration expand thế nào?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>Sau expand, schema mới là tập lớn hơn của schema cũ — mã cũ vẫn đọc/ghi được (MG-01). Sau contract, schema thiếu thứ mã cũ cần — ghi lỗi và có thể đọc sai mà không báo lỗi (MG-03). Vì vậy contract chỉ chạy khi cửa sổ rollback đã đóng; sau đó lựa chọn là roll-forward.</p>
</details>

**Câu 4.** Một dashboard chỉ theo dõi tỷ lệ 5xx có phát hiện được tình huống MG-03 không?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>Một phần. Đường ghi trả 500 nên tỷ lệ 5xx tăng; nhưng đường đọc trả 200 với dữ liệu thiếu <code>cancelReason</code> — dashboard coi là khoẻ. Cần phép kiểm assert nội dung hoặc chỉ số nghiệp vụ (ví dụ tỷ lệ chuyến huỷ có lý do hiển thị).</p>
</details>

**Câu 5.** Vì sao test chờ health bằng vòng hỏi có hạn chót thay vì `sleep` 5 giây?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><code>sleep</code> vừa chậm khi máy nhanh, vừa đỏ ngẫu nhiên khi máy chậm, và không phân biệt "chưa sẵn sàng" với "đã chết". Vòng hỏi trả ngay khi sẵn sàng, thoát sớm kèm log khi tiến trình chết, và chỉ thất bại khi vượt hạn chót đã chốt.</p>
</details>

## 13. Tài liệu tham khảo

Tra cứu và đối chiếu ngày 13/11/2026.

- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024) — test levels và test types, operational acceptance testing, maintenance testing: https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- PostgreSQL 18 — *ALTER TABLE* (ADD COLUMN, SET NOT NULL, RENAME COLUMN, mức khoá): https://www.postgresql.org/docs/18/sql-altertable.html
- Lab Tới Nơi: `SPEC.md` (R-API-15, R-TEST-01, R-NFR-06, R-FARE-01/04/08, R-STATE-02, R-PAY-03), `src/server.ts`, `src/infra/pglite/pg-trip-repository.ts`, `articles/system-testing-deployment-test/` (`deployment.test.ts`, `migration-compat.test.ts`) — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- Bài liên quan: [SYS-01 Smoke Test](/articles/system-testing-smoke-test) · [SYS-13 Data Integrity Test](/articles/system-testing-data-integrity-test) · [SYS-14 Recovery Test](/articles/system-testing-recovery-test) · [SYS-15 UAT](/articles/system-testing-uat) · [Schema migration tương thích](/articles/postgresql-schema-migration-tuong-thich) · [Lock hàng và bảng](/articles/postgresql-lock-row-table)

Bài tiếp theo: [SYS-17 · Release Dossier](/articles/system-testing-release-dossier) — gom bằng chứng của cả mười sáu bài thành một đề xuất release có người chịu trách nhiệm.
