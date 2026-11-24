---
slug: system-testing-deployment-test
title: "Deployment Test: bản vừa triển khai có đúng cấu hình, sẵn sàng và còn quay lui được?"
summary: "SYS-16: khởi động máy chủ Tới Nơi như một bản triển khai thật, chờ readiness có hạn chót, chứng minh phép kiểm cấu hình biết đỏ, chạy post-deploy smoke qua HTTP thật và kiểm migration expand/contract để rollback mã an toàn — kèm một lệnh đọc trả 200 nhưng mất trường mà không báo lỗi."
category: testing
level: advanced
tags: [testing, sys-16, deployment-testing, post-deploy-smoke, readiness, configuration, schema-migration, rollback, expand-contract, vitest]
language: vi
---

Thứ Sáu, 17:40. Bản mới của Tới Nơi vừa lên. Pipeline xanh từ unit, integration, system tới regression; `/api/health` trả 200. Mười phút sau, một kỹ sư phát hiện ai đó gọi được `POST /api/test/reset` trên môi trường thật: biến `TOINOI_TEST_API=1` của staging bị chép nhầm sang. Cùng tối, đội rollback vì một lỗi giao diện, và mọi lệnh tạo chuyến trả 500: migration của bản mới đã đổi tên một cột bản cũ còn dùng. Không dòng code nghiệp vụ nào sai; thứ sai là **hành động triển khai**. Bài này trả lời: **deployment test kiểm gì mà các suite trước không kiểm, chờ bản triển khai sẵn sàng thế nào mà không sleep đoán mò, làm sao biết phép kiểm cấu hình thật sự có thể đỏ, và thiết kế migration thế nào để rollback mã còn an toàn**.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · SYS-16</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>Kiểm thử hệ thống Tới Nơi · Phần 2</td></tr>
<tr><th scope="row">Cấp độ</th><td>Advanced — QA/SDET, kỹ sư release/vận hành đã nắm smoke, SQL và khái niệm rollback</td></tr>
<tr><th scope="row">Test level / hướng kiểm</th><td>Cuối system testing, giáp operational acceptance: kiểm hành động triển khai, cấu hình đang chạy và khả năng rollback; không kiểm lại logic nghiệp vụ</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/system-testing-smoke-test">SYS-01 · Smoke Test</a> · <a href="/articles/system-testing-recovery-test">SYS-14 · Recovery Test</a> · <a href="/articles/system-testing-uat">SYS-15 · UAT</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): 5 test case DP-01…05 khởi động <code>src/server.ts</code> thành tiến trình riêng và kiểm qua HTTP thật (R-API-15, R-TEST-01, R-NFR-06, R-FARE-01/08); 3 test case MG-01…03 kiểm migration expand/contract với mã đang chạy — <code>examples/toi-noi-testing/articles/system-testing-deployment-test/</code> · Minh hoạ: CurlStudio với collection <code>examples/toi-noi-testing/curlstudio/</code></td></tr>
<tr><th scope="row">Nguồn chính</th><td>ISTQB CTFL v4.0.1 (test level/type, maintenance testing); PostgreSQL 18 · ALTER TABLE; <code>SPEC.md</code>, <code>src/server.ts</code>, <code>src/infra/pglite/pg-trip-repository.ts</code> của lab</td></tr>
</tbody>
</table>
</div>

## 1. Bài toán và mục tiêu

### 1.1. Tình huống

Mười lăm bài trước đều kiểm **mã**: hàm tính cước, chỗ nối repository, hành trình khách–tài xế, độ trễ, quyền truy cập, dữ liệu, phục hồi, nghiệm thu. Nhưng giữa «mã đã đúng» và «khách đặt được xe» còn một bước: đem mã lên một môi trường, với một bộ cấu hình, trên một schema có sẵn dữ liệu, rồi mở cho traffic thật. Bước đó hỏng theo những cách chưa suite nào chạm tới:

- **Cấu hình sai.** Cùng một mã, `NODE_ENV=test` thì có `/api/test/reset`, `NODE_ENV=production` thì không (R-TEST-01). Một biến môi trường chép nhầm đủ biến bản triển khai thành cửa hậu.
- **Sẵn sàng giả.** Tiến trình đã mở cổng nhưng chưa khởi tạo xong kho dữ liệu, hoặc health xanh trong khi đường tiền hỏng — SYS-01 đã chỉ ra health của lab không chạm DB.
- **Không quay lui được.** Migration xoá hoặc đổi tên thứ mã cũ cần thì nút rollback thành nút gây sự cố.

### 1.2. Đối tượng và điều kiện tiên quyết

Bài dành cho người đã viết smoke test (SYS-01), đọc được SQL và constraint (SYS-13), và hiểu rollback mã khác restore dữ liệu (SYS-14). Lab cần Node.js ≥22.12 và dependency của `examples/toi-noi-testing` cài theo lockfile; không cần PostgreSQL ngoài, Docker hay mạng vì máy chủ lab chạy `TOINOI_DB=pglite` — PostgreSQL dạng WASM trong chính tiến trình.

### 1.3. Mục tiêu đo được

Sau bài này, bạn có thể:

1. Phân biệt build, deploy, liveness, readiness, post-deploy smoke, rollback, roll-forward và nói mỗi phép kiểm chứng minh điều gì.
2. Khởi động ứng dụng thành tiến trình riêng với cấu hình kiểu triển khai, chờ readiness bằng vòng hỏi có hạn chót.
3. Viết phép kiểm cấu hình an toàn và chứng minh nó **đỏ** trên một bản cấu hình sai có chủ đích.
4. Kiểm migration theo expand/contract: bản cũ còn đọc/ghi được sau expand; contract quá sớm phá rollback.
5. Ra quyết định giữ, rollback hay roll-forward dựa trên bằng chứng.

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

SYS-01 chạy smoke trên ứng dụng **lắp ráp trong tiến trình test** bằng supertest — không cổng mạng, không biến môi trường, không bước khởi động — để trả lời «build này có đủ tốt để kiểm thử chuyên sâu không». Deployment test chạy trên **bản đang chạy**: tiến trình riêng, đọc cấu hình từ môi trường, nghe một cổng, kiểm qua HTTP thật — để trả lời «chính bản triển khai này có đúng cấu hình, sẵn sàng và dùng được không».

Deployment test không phải test level của ISTQB; nó thường nằm cuối system testing và chồng lấn operational acceptance testing (cài đặt, cấu hình, backup/restore). Ghi rõ phạm vi trong test plan quan trọng hơn tranh luận tên gọi.

### 2.3. Rollback mã không phải restore dữ liệu

SYS-14 restore **dữ liệu** từ bản sao. Rollback ở đây đưa **mã** về phiên bản trước, còn schema và dữ liệu giữ trạng thái mới nhất. Câu hỏi then chốt: *mã cũ có chạy được trên schema mới không?* Nếu không, rollback biến lỗi nhỏ thành sự cố toàn hệ thống, như tình huống mở bài.

## 3. Test basis, rủi ro và oracle

### 3.1. Test basis

- `src/server.ts`: biến môi trường chú thích đầu tệp (`PORT`, `TOINOI_DB`, `TOINOI_START_AT`, `TOINOI_RAIN`, `TRIP_RATE_LIMIT_*`, `NODE_ENV`/`TOINOI_TEST_API`) và dòng log khởi động.
- **R-API-15** — `GET /api/health` ⇒ 200 `{ "status": "ok" }`, dùng cho giám sát và để công cụ kiểm thử chờ máy chủ sẵn sàng.
- **R-TEST-01** — `/api/test/*` chỉ tồn tại khi `NODE_ENV=test` hoặc `TOINOI_TEST_API=1`; mặc định (kể cả `npm start`) trả 404 `NOT_FOUND`.
- **R-NFR-06** — API kiểm thử tắt mặc định; lỗi 500 không lộ stack trace.
- **R-FARE-01…08** — oracle tiền cho post-deploy smoke.
- `schema.sql` và `pg-trip-repository.ts` — cách mã hiện tại đọc/ghi bảng `trips`, nền cho test case migration.

### 3.2. Rủi ro và nơi kiểm

| Rủi ro | Hậu quả | Phép kiểm |
|---|---|---|
| Cấu hình kiểm thử lọt vào môi trường thật | Ai cũng reset được dữ liệu, đổi giờ, đổi chế độ ví | DP-02, DP-05 |
| Sai kho dữ liệu hoặc giờ hệ thống | Chạy nhầm kho memory, mất dữ liệu khi restart; giá sai giờ cao điểm | DP-03 |
| Mở cổng nhưng nghiệp vụ hỏng | Health xanh, khách không đặt/không trả được tiền | DP-04 |
| Khởi động treo hoặc chậm bất thường | Traffic vào instance chưa sẵn sàng | DP-01 |
| Migration không tương thích mã cũ | Rollback gây 500 hoặc đọc sai âm thầm | MG-01…03 |

### 3.3. Oracle độc lập

- **Tiền:** xe máy 5 km tại SG, 09:00 giờ Việt Nam (không phải cao điểm theo R-FARE-04: [11:00; 13:00) và [17:00; 19:00)), không mưa, không voucher: 12.000 + 4.500 × 5 = 34.500, làm tròn **lên** tới bội 1.000 theo R-FARE-08 ⇒ **35.000**, tính tay từ SPEC, không gọi lại hàm tính cước.
- **Cấu hình:** 404 `NOT_FOUND` theo R-TEST-01; log khởi động in `API test: tắt`.
- **Lỗi:** thông điệp 500 chung `Lỗi hệ thống, vui lòng thử lại sau.` theo R-NFR-06.
- **Phiên bản giá:** `pricingVersion` `fare-2026.1` — hằng số `PRICING_VERSION` của mã; oracle về **bản đang chạy**, không về nghiệp vụ.

### 3.4. Entry và exit criteria

Entry criteria: dependency cài theo lockfile; cổng do hệ điều hành cấp; không có dữ liệu thật. Exit criteria của bài tập: 8/8 test case pass, typecheck pass, không còn tiến trình máy chủ nào sau suite. Đây là điều kiện **đạt bài tập**, không phải phê duyệt release production.

## 4. Thiết kế test case

### 4.1. Bảng test case

| ID | Basis / rủi ro | Tiền điều kiện và hành động | Mong đợi | Cleanup |
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

Lab chỉ có **một** phiên bản mã. Các test case MG áp migration trước rồi dùng mã hiện tại trong `src/` đóng vai **bản cũ** — bản đội sẽ quay về. Cách này mô phỏng trung thực câu hỏi «mã cũ có chạy trên schema mới không», nhưng không phải triển khai hai phiên bản thật; mục Mở rộng nâng cao nói cách làm phần đó trên staging.

### 4.3. Vì sao DP-05 là một test case, không phải một lỗi

DP-05 cố ý khởi động bản sai để chứng minh phép kiểm của DP-02 **phân biệt được** cấu hình đúng và sai. Phép kiểm chưa từng đỏ trên đầu vào sai thì chưa chứng minh được nó kiểm gì — cùng tinh thần thí nghiệm inject lỗi SYS-01 đề xuất.

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
<figcaption><b>Hình 1 · Hai nửa của lab SYS-16.</b> Nửa trái kiểm bản triển khai qua cổng thật; nửa phải kiểm «mã cũ có chạy trên schema mới không». Không nửa nào chạm <code>src/</code> dùng chung.</figcaption>
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

Vitest tự đặt `NODE_ENV=test`. Nếu tiến trình máy chủ thừa kế nguyên môi trường, API kiểm thử bật theo R-TEST-01 và DP-02 kiểm một cấu hình **không phải** cấu hình triển khai — phép kiểm vẫn chạy, chỉ là kiểm sai thứ.

```ts
async function deploy(extraEnv: Record<string, string>): Promise<Deployment> {
  const port = await freePort();
  const env: NodeJS.ProcessEnv = { ...process.env };
  delete env.NODE_ENV; // Vitest tự đặt NODE_ENV=test — phải xoá, nếu không API kiểm thử sẽ bật.
  delete env.TOINOI_TEST_API;
  Object.assign(env, { PORT: String(port), ...extraEnv });

  const child = spawn(process.execPath, ['--import', 'tsx', 'src/server.ts'], { cwd: LAB_ROOT, env });
```

`freePort()` xin hệ điều hành một cổng trống thay vì cố định 3100, để không đụng máy chủ lab bạn đang chạy và để các test case không tranh cổng. `node --import tsx` chạy TypeScript trong **một** tiến trình node; lệnh `tsx` trực tiếp có thể sinh tiến trình con, khi bị dừng trên Windows dễ để lại tiến trình mồ côi giữ cổng.

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

- **Thoát sớm khi chắc chắn hỏng.** Tiến trình đã chết thì chờ thêm vô ích; thông điệp lỗi mang theo log.
- **Có hạn chót.** 15 giây là giả định thiết kế, đủ rộng cho máy chậm, không phải SLO khởi động; hệ thật cần hạn chót có căn cứ đo đạc.
- **Đo được.** `readyAfterMs` được ghi để theo dõi xu hướng nhưng không assert ngưỡng chặt, vì thời gian khởi động trên máy dev dao động.

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

DP-03 đọc bằng chứng cấu hình từ log khởi động. Lab không có endpoint version, nên dòng log của `src/server.ts` là nơi duy nhất bản đang chạy «tự khai» kho, giờ và trạng thái API test:

```ts
    expect(d.log()).toContain('kho: pglite');
    expect(d.log()).toContain('API test: tắt');
    expect(d.log()).toMatch(/giờ hệ thống: 2026-11-04T02:00:\d{2}/);
```

09:00 giờ Việt Nam là 02:00 UTC. Đồng hồ của máy chủ lab là `AdjustableClock`, dời tới mốc rồi chạy tiếp, nên giây có thể lệch vài đơn vị — regex chỉ cố định tới phút.

DP-04 là post-deploy smoke: đường tiền như SYS-01 nhưng qua `fetch` tới cổng thật, có header `Idempotency-Key` như client thật, và đọc lại cả `pricingVersion`:

```ts
    const read = await call('GET', `/api/trips/${id}`, 'customer:C001');
    expect(read.status).toBe(200);
    expect(read.body.trip).toMatchObject({ id, status: 'completed', fare: 35_000, paymentStatus: 'paid', pricingVersion: 'fare-2026.1' });
```

`pricingVersion` trả lời điều health không trả lời: bản đang chạy dùng bảng cước nào. Release đổi bảng cước mà smoke vẫn thấy `fare-2026.1` nghĩa là bản mới chưa thực sự lên.

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

Cùng `NODE_ENV=production`, chỉ thêm một cờ là cửa hậu mở — đúng kịch bản mở bài. DP-05 pass nghĩa là DP-02 chạy trên bản này sẽ đỏ.

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

Test chờ sự kiện `exit` trước khi sang test case sau, không chỉ gửi tín hiệu rồi đi tiếp; báo cáo kiểm lại bằng `Get-CimInstance Win32_Process`. Trên Windows `kill()` dừng cưỡng bức, nên lab **không** kiểm graceful shutdown dù `src/server.ts` có xử lý `SIGTERM`.

### 6.7. Minh hoạ với CurlStudio

Post-deploy smoke cũng chạy được bằng một API client có giao diện, dùng lại collection «Tới Nơi lab · Kiểm thử hệ thống» đã giới thiệu ở [SYS-01](/articles/system-testing-smoke-test). Điểm khác duy nhất so với các bài trước là **environment**:

1. Khởi động bản prod-like theo README của collection: cổng 3101, `NODE_ENV=production`, `TOINOI_DB=pglite`, giờ 09:00 ngày 04/11/2026 — cùng cấu hình `PROD_LIKE` của DP-02, API kiểm thử tắt.
2. Chuột phải folder «SYS-16 · Post-deploy smoke» → **Chạy thư mục**; trong runner chọn environment «Tới Nơi bản triển khai (prod-like)» (`baseUrl = http://localhost:3101`).
3. Chạy, rồi đọc thanh tổng trước, từng dòng sau.

#### Hai environment, hai bản

Collection không chứa địa chỉ máy chủ; mọi request viết `{{baseUrl}}`. Bản nào nhận request do environment quyết định, nên trước khi bấm chạy hãy mở panel **Môi trường** và đọc giá trị thật của biến.

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/system-testing/sys-16/02-moi-truong-prod-like.webp" alt="Panel Môi trường của CurlStudio đang chọn «Tới Nơi bản triển khai (prod-like)». Editor bên phải có biến baseUrl với giá trị http://localhost:3101, nút «Dùng môi trường này» chưa được bấm và nút Xuất. Danh sách bên trái còn Globals và «Tới Nơi lab (local, API test bật)»." width="2000" height="1125" loading="lazy"></div>
<figcaption><b>Hình 3 · Environment prod-like.</b> Chỉ một biến khác nhau giữa hai environment, nhưng biến đó quyết định request đi tới bản nào.</figcaption>
</figure>

Environment prod-like chỉ đổi `baseUrl` sang `http://localhost:3101`, không đổi request hay test script nào. Nút «Dùng môi trường này» vẫn chưa bấm: xem một environment chưa có nghĩa là đang dùng nó, nên ô góc phải trên mới là thứ cần nhìn khi chạy.

#### Smoke trên bản prod-like

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/system-testing/sys-16/01-post-deploy-smoke-prod-like.webp" alt="Collection Runner của CurlStudio chạy folder «SYS-16 · Post-deploy smoke» với environment «Tới Nơi bản triển khai (prod-like)»; ô environment ở góc phải trên và thanh trạng thái cũng hiện tên này. Thanh tổng ghi Requests 3, Tests 4 / 4 (100%), Thất bại 0, Non-2xx 1. Ba dòng kết quả: DP · Health trả 200; DP · API kiểm thử phải đóng gọi POST localhost:3101/api/test/reset nhận 404 với test R-TEST-01 pass; DP · Báo giá 5 km trả 200 với test cước 35.000 đ pass." width="2000" height="1125" loading="lazy"></div>
<figcaption><b>Hình 4 · Post-deploy smoke trên bản prod-like.</b> Non-2xx duy nhất là 404 của <code>/api/test/reset</code> — kết quả mong đợi, có dấu ✓ «R-TEST-01: /api/test/* trả 404 trên bản triển khai» đi kèm. Mọi URL đều trỏ <code>localhost:3101</code>, không phải cổng 3100 của lab.</figcaption>
</figure>

Ba chi tiết đáng đọc trong ảnh:

- **Non-2xx 1 không phải lỗi.** Đó là phép kiểm của DP-02 làm bằng GUI: bản triển khai phải đóng cửa hậu, nên 404 là đạt. Với post-deploy smoke, con số Non-2xx chỉ có nghĩa khi đặt cạnh danh sách mã lỗi mong đợi.
- **Cùng collection, khác bản.** Đổi environment là chạy cùng các request trên bản khác — đúng tinh thần «build một lần, chỉ cấu hình khác nhau» ở mục 10.1. URL trong từng dòng là bằng chứng runner đã gọi đúng cổng 3101.
- **Environment đổi cho cả ứng dụng.** Chọn environment trong runner cũng đổi environment chung của CurlStudio (ô góc phải trên). Chạy xong phải đổi lại «Tới Nơi lab (local, API test bật)»; quên bước này thì folder kế tiếp gọi `/api/test/*` vào bản prod-like và nhận 404 ngay ở bước reset.

#### Một request, đọc tới tận cổng

Thanh tổng của runner chỉ nói «đạt». Muốn biết 404 đến từ đâu, đổi environment chung sang prod-like rồi gửi tay request «DP · API kiểm thử phải đóng».

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/system-testing/sys-16/05-api-test-404-prod-like.webp" alt="Request «DP · API kiểm thử phải đóng» gửi tay trong CurlStudio, ô environment góc phải trên ghi «Tới Nơi bản triển khai (prod-like)». Response 404 Not Found với body JSON có error.code NOT_FOUND và message «Không có đường dẫn này.», kết quả test 1/1." width="2000" height="1125" loading="lazy"></div>
<figcaption><b>Hình 5 · API kiểm thử đóng trên bản prod-like.</b> Đây là phép kiểm của DP-02 làm bằng tay: mã <code>NOT_FOUND</code> giống hệt mọi đường dẫn không tồn tại.</figcaption>
</figure>

Body ghi `NOT_FOUND` «Không có đường dẫn này.», đúng điều DP-02 assert: bản triển khai không để lộ rằng cửa hậu từng có. Tab **Timeline** của cùng request cho biết request đã thực sự đi tới đâu.

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/system-testing/sys-16/06-timeline-3101.webp" alt="Tab Timeline của request trên trong CurlStudio. Dòng địa chỉ ghi POST http://localhost:3101/api/test/reset (127.0.0.1:3101); phần RESPONSE ghi HTTP/1.1 404 Not Found và content-type application/json; charset=utf-8." width="2000" height="1125" loading="lazy"></div>
<figcaption><b>Hình 6 · Timeline: request đi đúng cổng 3101.</b> Địa chỉ kèm IP và cổng thật là bằng chứng request không rơi vào máy chủ lab ở cổng 3100.</figcaption>
</figure>

Dòng `POST http://localhost:3101/api/test/reset (127.0.0.1:3101)` gắn kết quả 404 với tiến trình nghe cổng 3101. Đó là cổng, chưa phải build: Timeline vẫn không nói tiến trình ấy chạy mã nào.

#### Phép kiểm biết đỏ

DP-05 dạy rằng phép kiểm chưa từng đỏ thì chưa chứng minh được nó kiểm gì. Bản GUI có một cách tái hiện rẻ hơn: chạy folder SYS-16 với environment **local**, tức máy chủ lab ở cổng 3100 có API kiểm thử bật. Đây chính là lỗi vận hành mà gạch đầu dòng thứ ba ở trên cảnh báo, chỉ là theo chiều ngược lại.

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/system-testing/sys-16/03-smoke-tren-lab-do.webp" alt="Collection Runner của CurlStudio chạy folder SYS-16 với environment Tới Nơi lab (local, API test bật). Thanh tổng ghi Requests 3, Tests 3 / 4 (75%), Thất bại 1. Dòng «DP · API kiểm thử phải đóng» gọi localhost:3100/api/test/reset nhận 200 và có dấu ✗ với thông điệp R-TEST-01: /api/test/* trả 404 trên bản triển khai — expected response to have status code 404 but got 200." width="2000" height="1125" loading="lazy"></div>
<figcaption><b>Hình 7 · Cùng folder, chạy nhầm trên lab: đỏ.</b> Health và báo giá vẫn đạt; chỉ phép kiểm cấu hình phát hiện bản này có cửa hậu mở.</figcaption>
</figure>

Tests 3/4: health và báo giá 35.000 đ trên lab cũng đúng, nên nếu chỉ có hai phép kiểm đó, smoke sẽ xanh trên một bản đang mở `/api/test/*`. Dòng đỏ ghi rõ mong đợi 404 nhưng nhận 200 từ `localhost:3100`. Với folder dài hơn, bộ lọc **Thất bại** giúp đi thẳng tới dòng đó.

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/system-testing/sys-16/04-loc-that-bai.webp" alt="Cùng lượt chạy trong Collection Runner, bộ lọc «Thất bại (1)» đang bật nên danh sách chỉ còn dòng «DP · API kiểm thử phải đóng» với lỗi expected response to have status code 404 but got 200." width="2000" height="1125" loading="lazy"></div>
<figcaption><b>Hình 8 · Lọc thất bại.</b> Một dòng duy nhất, thông điệp của nó đủ để biết lượt chạy đã trỏ nhầm bản.</figcaption>
</figure>

Thông điệp mang mã yêu cầu R-TEST-01 nên người đọc báo cáo không phải mở test script để hiểu vì sao đỏ. Lượt đỏ này cũng nằm lại trong nhật ký hoạt động của CurlStudio; [SYS-17](/articles/system-testing-release-dossier) dùng chính nhật ký đó để bàn chuyện danh tính của từng lượt chạy.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Các ảnh này chứng minh gì và không chứng minh gì</span>
<p><b>Có:</b> trên tiến trình nghe cổng 3101 (máy chủ chạy <code>NODE_ENV=production</code>), health trả 200, API kiểm thử trả 404, báo giá xe máy 5 km đúng 35.000 đ theo oracle tính tay ở mục 3.3. Phép kiểm R-TEST-01 của folder đỏ khi gặp một bản có API kiểm thử bật.</p>
<p><b>Không:</b> lab không có endpoint version nên runner <b>không xác nhận được build ID</b> — nó không biết bản nào đang nghe cổng 3101. Folder không đọc log khởi động như DP-03, không đi hết đường tiền như DP-04 (chỉ báo giá, không đặt, hoàn tất hay đọc lại). Lượt đỏ ở Hình 7 chạy trên máy chủ lab, không phải bản <code>NODE_ENV=production</code> thêm cờ <code>TOINOI_TEST_API=1</code> như DP-05, nên nó chỉ cùng tinh thần với DP-05 chứ không thay DP-05. Tiến trình do người khởi động, không có vòng chờ readiness. Staging, canary và rollback thật không được thực hiện. Đây là lượt minh hoạ ngày 23/11/2026 (Hình 4) và 24/11/2026 (các hình còn lại) bằng CurlStudio 0.2.1 (<code>pnpm tauri dev</code>) trên Windows 11, máy chủ lab chạy Node v22.18.0; bằng chứng của bài vẫn là DP-01…05 và MG-01…03 trong Vitest.</p>
</div>

## 7. Thực hành B — migration và khả năng rollback

### 7.1. Đọc mã trước khi thiết kế test case

Hai chi tiết trong `pg-trip-repository.ts` quyết định cách mã phản ứng với schema mới. Ghi dùng danh sách cột tường minh:

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

Từ đó dự đoán được: thêm cột mới không hỏng ghi (cột không nằm trong danh sách) và không hỏng đọc (cột thừa bị bỏ qua); bỏ hoặc đổi tên một cột trong danh sách sẽ hỏng ghi; còn đọc thì phải chạy mới biết — lý do MG-03 tồn tại.

### 7.2. MG-01 — expand an toàn

```ts
    await sql('ALTER TABLE trips ADD COLUMN pickup_note text');
    // Bản mới (giả lập bằng SQL) ghi giá trị cho cột mới.
    await system.db!.query('UPDATE trips SET pickup_note = $1 WHERE id = $2', ['Cổng số 2', oldTripId]);
```

Sau đó bản cũ nhận chính chuyến đó — chạy `UPDATE` mọi cột nó biết trên dòng đã có dữ liệu của bản mới — rồi tạo thêm một chuyến:

```ts
    expect(rows).toEqual([
      { id: oldTripId, pickup_note: 'Cổng số 2' }, // bản cũ UPDATE cả dòng nhưng không chạm cột nó không biết
      { id, pickup_note: null },                    // dòng do bản cũ tạo: bản mới phải chịu được NULL
    ]);
```

Hai assertion là hai nửa của hợp đồng expand: bản cũ không xoá dữ liệu của bản mới, và bản mới chịu được dữ liệu bản cũ để lại (NULL) suốt cửa sổ rollback.

### 7.3. MG-02 — expand sai

```ts
    await sql(`ALTER TABLE trips ADD COLUMN channel text;
               UPDATE trips SET channel = 'app';
               ALTER TABLE trips ALTER COLUMN channel SET NOT NULL;`);
    const res = await book();
    expect(res.status).toBe(500);
    expect(res.body).toEqual({ error: { code: 'INTERNAL_ERROR', message: 'Lỗi hệ thống, vui lòng thử lại sau.' } });
```

Migration «chỉ thêm cột» vẫn có thể phá bản cũ: cột NOT NULL không default khiến mọi `INSERT` không biết cột đó vi phạm ràng buộc. Test case còn xác nhận R-NFR-06 trên đường lỗi: 500 với thông điệp chung, không lộ tên cột hay câu SQL. Bảng `trips` trống sau đó, không có dòng dở dang.

### 7.4. MG-03 — contract quá sớm và phát hiện quan trọng nhất của bài

```ts
    await sql('ALTER TABLE trips RENAME COLUMN cancel_reason TO cancellation_reason');

    const write = await book();
    expect(write.status).toBe(500);
    expect(write.body.error.code).toBe('INTERNAL_ERROR');

    const afterRead = await request(app).get(`/api/trips/${id}`).set('x-user', 'customer:C001').expect(200);
    expect(afterRead.body.trip).not.toHaveProperty('cancelReason');
```

Ghi hỏng, đúng dự đoán. Nhưng **đọc trả 200**: `SELECT *` không còn cột `cancel_reason`, `r.cancel_reason` thành `undefined`, và JSON bỏ trường `undefined` — trước migration, cùng chuyến đó có `cancelReason: null`. Khách không thấy lỗi, chỉ thiếu một trường.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Đọc sai mà không báo lỗi, qua mặt monitor chỉ đếm 5xx</span>
<p>Sau rollback trong tình huống MG-03, dashboard tỷ lệ 5xx đỏ ở đường ghi, nhưng đường đọc vẫn «khoẻ» khi trả dữ liệu thiếu; smoke chỉ kiểm status code cũng bỏ sót. Phép kiểm sau triển khai phải assert <b>nội dung</b> theo hợp đồng (tập trường mong đợi, kể cả trường có thể null), và migration contract chỉ được chạy khi không còn phiên bản nào dùng cột cũ. Đây là hành vi quan sát được của mã lab, ghi lại làm ví dụ; bài không sửa <code>src/</code>.</p>
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
<figcaption><b>Hình 2 · Expand → migrate → contract và cửa sổ rollback.</b> Mỗi bước là một release riêng có phép kiểm riêng. Lab chỉ kiểm bước expand và hai cách làm sai; các bước giữa là quy trình đề xuất.</figcaption>
</figure>

Nguyên tắc: tách một thay đổi phá vỡ thành các release mà **mỗi release đều tương thích với release ngay trước nó**. Bài [Schema migration tương thích](/articles/postgresql-schema-migration-tuong-thich) của series PostgreSQL đi sâu kỹ thuật SQL; bài này tập trung vào phép kiểm chứng minh tương thích.

## 8. Kết quả: mong đợi và thực tế

**Mong đợi:** 8/8 test case pass như bảng mục 4.1; typecheck exit 0; không còn tiến trình máy chủ sau suite. `readyAfterMs` không có ngưỡng ngoài hạn chót 15 giây.

**Thực tế — báo cáo gốc (13/11/2026):** Windows 11, Node.js 22.18.0, Vitest 3.2.7, tsx 4.23.15, PGlite 0.3.16:

```text
Tests  8 passed (8)
Duration  9.32s (… tests 12.10s …)
```

- `readyAfterMs` của DP-01 qua bốn lượt chạy: **1.710, 2.275, 1.789, 1.926 ms** — quan sát trên máy lab, không phải SLO.
- Typecheck pass sau khi sửa ba lỗi kiểu `unknown` của chính mã test (không phải của sản phẩm) ở lần chạy đầu.
- Sau các lượt chạy không còn tiến trình `node … src/server.ts` nào.
- MG-03 đúng như mục 7.4: ghi 500, đọc 200 thiếu `cancelReason`.

**Thực tế — chạy lại khi biên tập (13/11/2026):** cùng máy và phiên bản, 8/8 pass, duration 9,66 s, `readyAfterMs` 1.846 ms — trong khoảng của bốn lượt trước.

**Ghi nhận ngoài phạm vi:** chạy toàn bộ `npm test` của lab, 85/86 file pass; 3 test của bài TEST-040 (domain testing) đỏ cả khi chạy riêng, cùng một kiểu mỗi lần. Bộ test đó và `src/` không đổi trong đợt viết bài, nên đây là lỗi sẵn có cần điều tra riêng. Nguyên tắc của deployment gate: ghi rõ phần đỏ không thuộc thay đổi đang xét, thay vì im lặng bỏ qua hoặc chặn toàn bộ mà không phân loại.

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
- **Rollback:** lỗi trong mã mới, migration là expand ⇒ quay về an toàn; chạy lại post-deploy smoke trên bản cũ.
- **Roll-forward:** migration đã contract (tình huống MG-03) ⇒ rollback mã gây 500 và đọc sai; sửa bằng bản mới hoặc chạy migration bù đã diễn tập.

## 10. Mở rộng nâng cao — chưa chạy trong lab

Thiết kế đề xuất cho Tới Nơi thật, **không** thực hiện trong bài.

### 10.1. Artifact bất biến và endpoint version

Build một lần, triển khai cùng artifact qua dev → staging → production; chỉ cấu hình khác nhau. Gắn commit/build ID vào endpoint version và log; post-deploy smoke assert version **trước** khi kiểm nghiệp vụ, để kết quả không thuộc nhầm bản cũ còn chạy. Lab không có endpoint version: DP-03 dùng log khởi động thay thế, còn `pricingVersion` chỉ phản ánh bảng cước, không phải cả build.

### 10.2. Secret và fail fast

Secret lấy từ secret manager, không nằm trong repo, image hay log. Phép kiểm nên có: thiếu secret bắt buộc ⇒ tiến trình từ chối khởi động với thông điệp rõ (fail fast) thay vì chạy với chuỗi rỗng rồi lỗi ở request đầu; log khởi động không in giá trị secret. Áp tư duy DP-05: thử khởi động thiếu secret và mong đợi thất bại.

### 10.3. Readiness thật

Readiness nên phản ánh dependency thiết yếu (DB trả lời, migration đã xong, cổng thanh toán đạt được) và tách khỏi liveness: instance chưa sẵn sàng không bị khởi động lại, chỉ không nhận traffic. Health của lab không chạm DB, nên DP-04 mới là bằng chứng nghiệp vụ.

### 10.4. Rolling, blue-green và canary

Trong rolling, hai phiên bản chạy cùng lúc trên cùng schema — vì vậy expand/contract là bắt buộc. Canary cho thay đổi bảng cước: đưa một phần nhỏ traffic vào bản mới, so tỷ lệ lỗi, p95 (SYS-07) và **chỉ số nghiệp vụ** theo `pricingVersion` (tỷ lệ đặt thành công, tổng tiền thu, tỷ lệ huỷ) với bản cũ trong cửa sổ quan sát chốt trước; vượt điều kiện dừng thì tự rollback. Báo giá đã phát dưới bảng cước cũ vẫn còn hiệu lực (R-QUOTE) khi chuyển bản.

### 10.5. Migration trên PostgreSQL thật

PGlite một kết nối không đo được thời gian giữ khoá, ảnh hưởng tới traffic đồng thời hay thời gian chạy trên bảng cỡ production. Trên staging với dữ liệu cỡ thật: đặt `lock_timeout`, tạo index đồng thời khi cần, đo thời gian migration và kiểm truy vấn đang chạy lúc đó. Xem thêm [Lock hàng và bảng](/articles/postgresql-lock-row-table).

### 10.6. Rollback giữa hai phiên bản thật

Diễn tập trên staging trước release có migration: triển khai v2, tạo dữ liệu qua v2, quay về v1, chạy post-deploy smoke **có assert nội dung**, rồi đối soát như SYS-13. Đây là phiên bản đầy đủ của MG-01, nơi bắt được MG-03 trước khi khách thấy.

## 11. Anti-pattern và giới hạn bằng chứng

- Chỉ kiểm health sau triển khai và gọi đó là «đã verify».
- `sleep(10000)` thay cho chờ readiness có hạn chót.
- Chạy smoke trên bản bật API kiểm thử «cho tiện reset» — kiểm một cấu hình khác cấu hình thật.
- Phép kiểm cấu hình chưa từng chạy trên một bản sai.
- Đổi tên/xoá cột trong cùng release với mã mới.
- Coi rollback mã là rollback dữ liệu.
- Smoke chỉ assert status code.
- Để tiến trình test mồ côi giữ cổng, làm lượt sau đỏ khó hiểu.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Lab này chứng minh gì và không chứng minh gì</span>
<p><b>Có:</b> máy chủ khởi động được từ mã nguồn với cấu hình kiểu triển khai; phép kiểm cấu hình phân biệt được bản bật/tắt API kiểm thử; post-deploy smoke đi hết đường tiền qua HTTP thật; mã hiện tại chịu được migration expand nhưng không chịu được contract sớm.</p>
<p><b>Không:</b> một tiến trình trên một máy, chạy từ mã nguồn qua tsx — không build artifact, container, load balancer, TLS, nhiều instance, staging, canary hay secret manager. Không có hai phiên bản mã thật, không kiểm graceful shutdown. Migration chạy trên PGlite một kết nối, không đo khoá hay thời gian. Kết quả đạt bài tập SYS-16, không suy ra quy trình triển khai production đã được kiểm.</p>
</div>

## 12. Bài tập và câu hỏi tự kiểm

### Bài tập

**Bài 1 (cơ bản).** Xoá dòng `delete env.NODE_ENV;` trong `deploy()` rồi chạy lại. Dự đoán DP-02 và DP-03 ra sao trước khi chạy.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Tiến trình con thừa kế <code>NODE_ENV=test</code> từ Vitest ⇒ API kiểm thử bật theo R-TEST-01. DP-02 đỏ vì <code>/api/test/reset</code> không còn 404; DP-03 đỏ ở <code>API test: tắt</code> vì log in <code>BẬT</code>. Bài học: phép kiểm triển khai phải kiểm soát môi trường của tiến trình được kiểm, không thừa kế môi trường công cụ test.</p>
</details>

**Bài 2 (trung bình).** Viết MG-04: thêm cột `channel text NOT NULL DEFAULT 'app'`. Mong đợi gì khi bản cũ tạo chuyến? So sánh với MG-02.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Bản cũ tạo được (201) vì PostgreSQL điền default cho cột không có trong <code>INSERT</code>; dòng mới có <code>channel = 'app'</code>. Khác MG-02 ở chỗ có default. Thêm assert đọc thẳng SQL rằng giá trị là <code>'app'</code>. Lưu ý: default tĩnh có thể sai nghĩa nghiệp vụ cho chuyến tạo từ kênh khác — tương thích kỹ thuật chưa chắc đúng dữ liệu.</p>
</details>

**Bài 3 (trung bình).** Sửa post-deploy smoke để bắt được MG-03 trên đường đọc.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Assert tập trường hợp đồng của <code>trip</code>, ví dụ <code>expect(Object.keys(trip).sort()).toEqual([...])</code> hoặc <code>toHaveProperty('cancelReason')</code> cho cả trường có thể null. Sau migration rename, smoke đó phải đỏ ở bước đọc, không chỉ bước ghi. Assert tập trường cứng làm test nhạy với trường mới hợp lệ, nên chọn «phải có» thay vì «chỉ có».</p>
</details>

**Bài 4 (nâng cao).** Thiết kế canary cho thay đổi bảng cước `fare-2026.2` của Tới Nơi.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Cần: tỷ lệ traffic ban đầu và các nấc tăng; chỉ số so sánh theo <code>pricingVersion</code> (tỷ lệ báo giá → đặt, tỷ lệ huỷ, tổng tiền thu, lỗi 5xx, p95); cửa sổ quan sát và điều kiện dừng chốt trước; cách xử lý báo giá đã phát dưới bảng cũ khi rollback; post-deploy smoke với oracle tính tay từ bảng mới. Ghi rõ đây là thiết kế, chưa chạy.</p>
</details>

**Bài 5 (nâng cao).** Thêm DP-06: khởi động với `TOINOI_DB=postgres` (giá trị không hỗ trợ). Quan sát hành vi và đề xuất thay đổi.

<details>
<summary><b>Lời giải gợi ý</b></summary>
<p>Đọc <code>src/server.ts</code>: mọi giá trị khác <code>pglite</code> rơi về kho memory, và log in <code>kho: memory</code>. Bản triển khai sai này <b>khởi động thành công</b> và sẽ mất dữ liệu khi restart. DP-03 bắt được nhờ assert <code>kho: pglite</code>. Đề xuất cho sản phẩm (không sửa trong bài): giá trị lạ ⇒ từ chối khởi động — phép kiểm cấu hình tìm ra rủi ro thiết kế.</p>
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
<p>DP-02 pass chỉ cho thấy bản đúng không lộ API kiểm thử; DP-05 cho thấy cùng phép kiểm phân biệt được bản sai. Thiếu DP-05, một DP-02 luôn xanh vì lý do khác (ví dụ gọi sai đường dẫn) vẫn trông như bằng chứng.</p>
</details>

**Câu 3.** Rollback mã sau migration contract khác rollback sau migration expand thế nào?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>Sau expand, schema mới là tập lớn hơn của schema cũ nên mã cũ vẫn đọc/ghi được (MG-01). Sau contract, schema thiếu thứ mã cũ cần: ghi lỗi và có thể đọc sai mà không báo lỗi (MG-03). Vì vậy contract chỉ chạy khi cửa sổ rollback đã đóng; sau đó chỉ còn roll-forward.</p>
</details>

**Câu 4.** Một dashboard chỉ theo dõi tỷ lệ 5xx có phát hiện được tình huống MG-03 không?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>Một phần. Đường ghi trả 500 nên tỷ lệ 5xx tăng, nhưng đường đọc trả 200 với dữ liệu thiếu <code>cancelReason</code> và dashboard coi là khoẻ. Cần phép kiểm assert nội dung hoặc chỉ số nghiệp vụ (ví dụ tỷ lệ chuyến huỷ có lý do hiển thị).</p>
</details>

**Câu 5.** Vì sao test chờ health bằng vòng hỏi có hạn chót thay vì `sleep` 5 giây?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><code>sleep</code> vừa chậm khi máy nhanh, vừa đỏ ngẫu nhiên khi máy chậm, và không phân biệt «chưa sẵn sàng» với «đã chết». Vòng hỏi trả ngay khi sẵn sàng, thoát sớm kèm log khi tiến trình chết, chỉ thất bại khi vượt hạn chót.</p>
</details>

## 13. Tài liệu tham khảo

Tra cứu và đối chiếu ngày 13/11/2026.

- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* (2024) — test levels và test types, operational acceptance testing, maintenance testing: https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- PostgreSQL 18 — *ALTER TABLE* (ADD COLUMN, SET NOT NULL, RENAME COLUMN, mức khoá): https://www.postgresql.org/docs/18/sql-altertable.html
- Lab Tới Nơi: `SPEC.md` (R-API-15, R-TEST-01, R-NFR-06, R-FARE-01/04/08, R-STATE-02, R-PAY-03), `src/server.ts`, `src/infra/pglite/pg-trip-repository.ts`, `articles/system-testing-deployment-test/` (`deployment.test.ts`, `migration-compat.test.ts`) — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing
- CurlStudio 0.2.1 — API client và load test (mã nguồn, tra cứu 23/11/2026): https://github.com/rickymta/curl-studio · collection minh hoạ: `examples/toi-noi-testing/curlstudio/`
- Bài liên quan: [SYS-01 Smoke Test](/articles/system-testing-smoke-test) · [SYS-13 Data Integrity Test](/articles/system-testing-data-integrity-test) · [SYS-14 Recovery Test](/articles/system-testing-recovery-test) · [SYS-15 UAT](/articles/system-testing-uat) · [Schema migration tương thích](/articles/postgresql-schema-migration-tuong-thich) · [Lock hàng và bảng](/articles/postgresql-lock-row-table)

Bài tiếp theo: [SYS-17 · Release Dossier](/articles/system-testing-release-dossier) — gom bằng chứng của cả mười sáu bài thành một đề xuất release có người chịu trách nhiệm.
