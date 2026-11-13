---
slug: system-testing-compatibility-test
title: "Compatibility Test: cùng chức năng Tới Nơi trên những cấu hình nào?"
summary: "Bài thực hành compatibility testing: tách các trục browser, engine, OS, viewport, input, locale và timezone; lập support matrix theo rủi ro thay vì nhân mọi tổ hợp; giữ oracle nghiệp vụ cố định giữa các cấu hình; chạy hai hành trình trên ba Playwright project và ghi đúng phiên bản browser thực chạy, kèm giới hạn của emulation."
category: testing
level: intermediate
tags: [testing, sys-11, compatibility-testing, cross-browser, responsive, playwright, support-matrix, locale, emulation]
language: vi
---

Trên laptop của developer, trang đặt xe của Tới Nơi chạy trơn tru. Trên màn hình 390 px, nút «Đặt chuyến» bị đẩy ra ngoài khung và trang có thanh cuộn ngang. Trên một máy cài tiếng Anh, có người lo số tiền sẽ hiện thành `35,000 đ`. Backend vẫn tính đúng 35.000 đ, mọi bài API vẫn xanh, nhưng khách không hoàn thành được thao tác. Bài này trả lời: **làm sao chọn những cấu hình đáng kiểm, giữ chung một oracle cho mọi cấu hình, và nói chính xác ma trận đã chạy chứng minh được gì** — mà không hứa «chạy trên mọi thiết bị» chỉ từ vài test.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · SYS-11</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>Kiểm thử hệ thống Tới Nơi · Phần 2</td></tr>
<tr><th scope="row">Cấp độ</th><td>Trung cấp — đã đọc System Test, biết Playwright cơ bản</td></tr>
<tr><th scope="row">Test level / hướng kiểm</th><td>System · Non-functional (compatibility), giữ oracle chức năng làm điều kiện đạt</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/system-testing-security-pentest">SYS-10 Security Test – Pen test</a>; hành trình UI và server riêng ở <a href="/articles/system-testing-system-test">SYS-05 System Test</a>; nhóm đặc tính chất lượng ở <a href="/articles/non-functional-testing-tong-quan">Non-functional testing tổng quan</a></td></tr>
<tr><th scope="row">Thực hành</th><td>Lab (code): 2 case CP-01…02 × 3 Playwright project (Chrome desktop vi-VN, Edge desktop en-US/New York, Chrome 390×844 touch emulation), server PGlite riêng cổng 3116 — <code>examples/toi-noi-testing/articles/system-testing-compatibility-test/</code>, config <code>compatibility.config.ts</code></td></tr>
<tr><th scope="row">Nguồn chính</th><td>Playwright Projects và Emulation; <code>SPEC.md</code> của lab (R-UI-01, R-UI-04, R-FARE, R-FOOD, R-CANCEL-01)</td></tr>
</tbody>
</table>
</div>

## 1. Tình huống và mục tiêu

### 1.1. Vì sao backend xanh vẫn chưa đủ

Ở [SYS-05](/articles/system-testing-system-test), ba hành trình khách–tài xế chạy xanh trên **một** cấu hình: Chrome headless, viewport mặc định, máy của người viết. Kết luận đúng của bài đó là «hệ thống hoạt động trong ranh giới đã vẽ». Nó không nói gì về:

- Edge, Firefox hay Safari có thao tác được form không;
- màn hình hẹp có bị tràn ngang, nút có bấm được bằng chạm không;
- browser đặt `en-US` và múi giờ New York có làm đổi cách hiển thị tiền hay cách tính giờ cao điểm không.

Những lỗi đó không nằm trong use case hay SQL. Chúng nằm ở chỗ **phần mềm gặp môi trường chạy**: engine render, sự kiện input, kích thước khung, định dạng theo locale. Đó là phạm vi của compatibility testing.

### 1.2. Đối tượng và điều kiện tiên quyết

Bài dành cho tester/SDET đã viết được Playwright test, hiểu khái niệm system under test và đã đọc SYS-05. Cần Node ≥ 22.12, Chrome và Edge cài sẵn trên máy (lab dùng `channel`, không tải browser riêng).

### 1.3. Mục tiêu đo được

Sau bài này, bạn có thể:

1. Liệt kê các **trục** tương thích và nói một trục **không** chứng minh được gì.
2. Lập **support matrix** có phân loại bắt buộc / best-effort / chưa hỗ trợ, dựa trên dữ liệu và rủi ro.
3. Viết một case chạy được trên nhiều Playwright project mà **oracle không đổi theo cấu hình**.
4. Ghi metadata đủ tái hiện: project, browser version, viewport, locale, timezone, input.
5. Đọc một failure chỉ xảy ra trên một project và thiết kế thí nghiệm tách yếu tố.

## 2. Khái niệm và những chỗ dễ nhầm

### 2.1. Compatibility testing là gì

Compatibility testing đánh giá khả năng hệ thống **hoạt động trong hoặc cùng với** những môi trường, nền tảng và phiên bản đã chọn. Với web, các trục thường gặp là browser, engine, OS, thiết bị, viewport, phương thức nhập, locale và timezone. Với hệ thống có nhiều client, trục còn gồm phiên bản API, schema dữ liệu, phiên bản ứng dụng mobile và phần mềm cùng tồn tại trên máy.

Điểm then chốt: compatibility không phải một checklist browser. Mỗi trục trả lời một câu hỏi khác, và một trục **không thay được** trục khác.

| Trục | Ví dụ | Chưa chứng minh được nếu chỉ có trục này |
|---|---|---|
| Browser / engine | Chrome, Edge, Firefox, WebKit | Hai browser Chromium không thay một engine khác |
| OS / thiết bị | Windows, macOS, Android, model cụ thể | User agent không biến máy Windows thành điện thoại |
| Viewport / input | 1280×800 chuột; 390×844 chạm | Khung hẹp không mô phỏng bàn phím ảo, CPU, GPU thật |
| Locale / timezone | vi-VN, en-US; Asia/Ho_Chi_Minh, America/New_York | Đổi locale không có nghĩa giao diện đã được dịch |
| Version contract | API cũ/mới, schema sau migration | Browser test không chứng minh backward compatibility của API |

### 2.2. Các cặp khái niệm hay bị gộp

- **Responsive vs cross-browser.** Responsive testing tập trung bố cục theo kích thước khung. Cross-browser testing xét hành vi giữa các browser. Một trang có thể responsive hoàn hảo trên Chrome mà vẫn hỏng sự kiện `change` trên engine khác.
- **Emulation vs thiết bị thật.** Playwright đặt được viewport, `isMobile`, `hasTouch`, locale, timezone. Nó **không** đổi OS, không bật bàn phím ảo, không có native picker hay giới hạn phần cứng của điện thoại.
- **Compatibility vs accessibility/usability.** Nút bấm được trên mọi cấu hình chưa chắc có tên truy cập đúng, càng chưa chắc người dùng hiểu cách dùng. Hai chủ đề đó có bài riêng ([SYS-12](/articles/system-testing-usability-test)).
- **Project vs browser.** Trong Playwright, một *project* là một tổ hợp cấu hình. Ba project không có nghĩa ba browser — ở lab này là hai browser và một biến thể của Chrome.

### 2.3. Khi nào áp dụng, ưu và nhược điểm

Compatibility test có giá trị nhất khi: sản phẩm có support policy công khai; analytics cho thấy tỉ lệ người dùng đáng kể trên một cấu hình; vừa nâng cấp framework UI, CSS hay polyfill; hoặc vừa có báo lỗi «chỉ máy tôi bị».

Ưu điểm: bắt được lỗi môi trường mà mọi test API bỏ sót, chạy lại được cùng case trên nhiều cấu hình với chi phí viết test một lần. Nhược điểm: số tổ hợp tăng theo cấp số nhân, browser tự cập nhật làm kết quả trôi theo thời gian, và test tự động chỉ kiểm được những gì có assertion — phần «nhìn có ổn không» vẫn cần người.

## 3. Test basis, rủi ro và support matrix

### 3.1. Chọn cấu hình bằng dữ liệu, không bằng cảm tính

Trong dự án thật, support matrix được lập từ: analytics phân bố browser/OS/thiết bị, khách hàng mục tiêu (ví dụ tài xế dùng Android tầm trung), hợp đồng hỗ trợ, mức rủi ro của hành trình và chi phí duy trì. Kết quả nên chia ba nhóm:

- **Bắt buộc** — lỗi chặn release.
- **Best-effort** — kiểm định kỳ, lỗi được xếp ưu tiên nhưng không chặn.
- **Chưa hỗ trợ** — ghi rõ, không kiểm, không hứa.

Không hứa «mọi thiết bị» khi chỉ có vài test. Và vì browser tự cập nhật, chỉ ghi «Chrome» là không đủ để tái hiện; phải lưu **version** của từng lượt chạy.

### 3.2. Ma trận thử nghiệm của bài

Lab tận dụng hai browser đã cài trên máy Windows và một cấu hình hẹp mô phỏng:

| Project | Browser | Viewport | Locale / timezone | Input |
|---|---|---|---|---|
| `chrome-desktop-vi` | Chrome | 1280×800 | vi-VN / Asia/Ho_Chi_Minh | click |
| `edge-desktop-en` | Edge | 1280×800 | en-US / America/New_York | click |
| `chrome-narrow-touch` | Chrome | 390×844 | vi-VN / Asia/Ho_Chi_Minh | tap; `isMobile` + `hasTouch` emulation |

Đây là matrix bài tập, **không** phải support policy của Tới Nơi. Ba giới hạn cần nói ngay từ đầu:

1. Chrome và Edge đều thuộc họ Chromium. Không có Firefox (Gecko) hay Safari/WebKit trong matrix.
2. `chrome-narrow-touch` là emulation trên Windows, không phải Android hay iPhone thật.
3. Project Edge đổi **cùng lúc** browser, locale và timezone để phủ nhiều cấu hình hơn với ít project. Đổi lại, nếu project này fail, ta chưa biết yếu tố nào gây lỗi — cần thí nghiệm tách (mục 6.2).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Ma trận tương thích của SYS-11. Hàng là ba Playwright project: chrome-desktop-vi, edge-desktop-en, chrome-narrow-touch. Cột là các trục: engine, viewport, locale và timezone, input. Hai project đầu dùng engine Chromium, project thứ ba cũng Chromium. Viewport 1280×800, 1280×800 và 390×844. Locale vi-VN Việt Nam, en-US New York, vi-VN Việt Nam. Input click, click, tap. Bên phải, một khung oracle chung: 35.000 đ cho ride, 95.000 đ cho food, không tràn ngang quá 1 px. Dưới cùng là các trục chưa phủ: Firefox, WebKit Safari, điện thoại thật, bàn phím ảo, xoay màn hình, API version.">
<defs>
<marker id="sys11-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<text x="20" y="30" class="t-b">Project</text>
<text x="230" y="30" class="t-b">Engine</text>
<text x="330" y="30" class="t-b">Viewport</text>
<text x="440" y="30" class="t-b">Locale / TZ</text>
<text x="590" y="30" class="t-b">Input</text>
<rect x="16" y="44" width="620" height="44" rx="8" class="f-pk-soft s-pk"/>
<text x="28" y="71">chrome-desktop-vi</text><text x="230" y="71">Chromium</text><text x="330" y="71">1280×800</text><text x="440" y="71">vi-VN · VN</text><text x="590" y="71">click</text>
<rect x="16" y="98" width="620" height="44" rx="8" class="f-pk-soft s-pk"/>
<text x="28" y="125">edge-desktop-en</text><text x="230" y="125">Chromium</text><text x="330" y="125">1280×800</text><text x="440" y="125" class="t-warn">en-US · New York</text><text x="590" y="125">click</text>
<rect x="16" y="152" width="620" height="44" rx="8" class="f-pk-soft s-pk"/>
<text x="28" y="179">chrome-narrow-touch</text><text x="230" y="179">Chromium</text><text x="330" y="179" class="t-warn">390×844</text><text x="440" y="179">vi-VN · VN</text><text x="590" y="179" class="t-warn">tap (emul.)</text>
<rect x="690" y="44" width="214" height="152" rx="10" class="f-bg s-phys dash"/>
<text x="797" y="70" text-anchor="middle" class="t-b">Oracle chung</text>
<text x="797" y="98" text-anchor="middle">ride 5 km: 35.000 đ</text>
<text x="797" y="122" text-anchor="middle">food 5 km: 95.000 đ</text>
<text x="797" y="146" text-anchor="middle">huỷ searching: 0 đ</text>
<text x="797" y="170" text-anchor="middle" class="t-small">scrollWidth ≤ clientWidth + 1</text>
<line x1="636" y1="66" x2="686" y2="100" class="s-pk" stroke-width="1.4" marker-end="url(#sys11-arrow)"/>
<line x1="636" y1="120" x2="686" y2="120" class="s-pk" stroke-width="1.4" marker-end="url(#sys11-arrow)"/>
<line x1="636" y1="174" x2="686" y2="140" class="s-pk" stroke-width="1.4" marker-end="url(#sys11-arrow)"/>
<rect x="16" y="222" width="888" height="60" rx="8" class="f-warn-soft s-warn"/>
<text x="460" y="246" text-anchor="middle" class="t-b t-warn">Chưa phủ trong lab</text>
<text x="460" y="268" text-anchor="middle" class="t-small">Firefox · WebKit/Safari · Android/iOS thật · bàn phím ảo · xoay màn hình · zoom · screen reader · API version cũ</text>
</svg>
</div>
<figcaption><b>Hình 1 · Ma trận SYS-11 và oracle chung.</b> Mỗi project chỉ đổi một vài trục (chữ màu cảnh báo là trục khác cấu hình gốc). Cả ba project đều phải đạt cùng một oracle nghiệp vụ; những gì nằm ở dải dưới chưa được bài này kiểm.</figcaption>
</figure>

### 3.3. Rủi ro được chọn

| Rủi ro | Biểu hiện với khách | Case |
|---|---|---|
| Control không thao tác được trên một cấu hình | Bấm «Báo giá»/«Đặt chuyến» không có tác dụng | CP-01 |
| Tiền đổi định dạng theo locale browser | `35,000 đ` hoặc `35.000 ₫` trên máy tiếng Anh | CP-01, CP-02 |
| Giá đổi theo timezone của client | Khách ở New York thấy giá cao điểm sai | CP-01 |
| Trường động hiện/ẩn sai khi đổi dịch vụ | Ô tiền món không biến mất, ô loại xe bị khoá mãi | CP-02 |
| Bố cục tràn ngang ở khung hẹp | Phải cuộn ngang mới thấy nút | CP-01, CP-02 |
| Báo giá cũ còn dùng được sau khi sửa dữ liệu | Đặt chuyến theo giá không còn đúng | CP-02 |

## 4. Oracle: cái gì phải giữ, cái gì được phép khác

Câu hỏi khó nhất của compatibility testing không phải «chạy trên bao nhiêu cấu hình» mà là **«thế nào là đúng trên cấu hình này»**. Nếu lấy kết quả của browser đầu tiên làm chuẩn rồi so các browser sau với nó, ta chỉ kiểm được *tính nhất quán*, không kiểm được *tính đúng*. Oracle phải đến từ đặc tả.

### 4.1. Những gì phải giống nhau trên mọi cấu hình

- **Định dạng tiền.** R-UI-04: «Số tiền hiện theo định dạng Việt Nam kèm « đ»». SPEC không yêu cầu đổi định dạng theo locale browser, nên project `en-US` **vẫn** phải hiện `35.000 đ`. Lab đáp ứng điều này vì formatter được cố định locale ở `src/web/api.js`:

```js
const money = new Intl.NumberFormat('vi-VN', { maximumFractionDigits: 0 });

/** 52000 ⇒ "52.000 đ" (SPEC R-UI-04). */
export function formatDong(value) {
  return `${money.format(value)} đ`;
}
```

  Nếu một ngày ai đó đổi thành `new Intl.NumberFormat(navigator.language)`, chỉ project Edge/en-US sẽ đỏ — đúng loại lỗi matrix này được thiết kế để bắt.

- **Số tiền nghiệp vụ.** Xe máy 5 km lúc 09:00 giờ Việt Nam, không mưa, không voucher: 12.000 + 4.500 × 5 = 34.500 đ, làm tròn **lên** bội số 1.000 theo R-FARE-08 ⇒ **35.000 đ**. Đồ ăn: tiền món mặc định của form là 60.000 đ (`value="60000"` trong `index.html`), phí giao tính theo công thức xe máy 5 km (R-FOOD-04) là 35.000 đ, tổng theo R-FOOD-01 là **95.000 đ**.

- **Giờ nghiệp vụ.** R-FARE-04 xác định cao điểm theo **giờ Việt Nam** tại thời điểm đặt. Server được đặt 09:00+07:00, nên dù browser ở America/New_York (tức 21:00 ngày hôm trước), `quote-peak` vẫn phải là «Không». Timezone client không được tham gia tính giá.

- **Quy tắc trạng thái.** Huỷ khi `searching` có phí 0 đ (R-CANCEL-01); sau khi huỷ, nút huỷ bị vô hiệu. Sửa ô sau khi báo giá ⇒ phải báo giá lại (R-UI-01).

### 4.2. Những gì được phép khác

Font native, viền control, cách render `<select>`, độ dày con trỏ, khoảng cách vài pixel giữa các engine — đều có thể chấp nhận khi chưa có design baseline và vẫn đáp ứng yêu cầu. Bài **không** đòi pixel giống tuyệt đối.

Ngược lại, có những khác biệt **không** được chấp nhận dù screenshot nhìn gần giống: trang có thanh cuộn ngang, nút bị che bởi phần tử khác nên không bấm được, hay ô lỗi không hiện. Oracle vì vậy là **hành vi và một số đo bố cục cụ thể**, không phải ảnh.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của oracle bố cục</span>
<p>Phép kiểm <code>scrollWidth ≤ clientWidth + 1</code> chỉ chứng minh trang không tràn ngang quá 1 CSS pixel <b>tại những thời điểm test gọi nó</b>. Nó không phát hiện phần tử chồng nhau, chữ bị cắt, độ tương phản kém hay vùng chạm quá nhỏ. Đừng gọi đây là «responsive pass» cho toàn trang.</p>
</div>

## 5. Thiết kế case và dữ liệu

### 5.1. Tiền điều kiện chung

- Seed khách C001, thành phố SG, xe máy; PGlite trong server riêng cổng **3116**, `reuseExistingServer: false` để không dính vào process của bài khác.
- Giờ 09:00 giờ Việt Nam ngày 04/11/2026, không mưa (`TOINOI_RAIN: ''`).
- Mỗi execution: `POST /api/test/reset` rồi `POST /api/test/clock` trong `beforeEach`; một worker để các project không reset chéo DB của nhau.
- Playwright tạo và đóng browser context riêng cho từng test — không có cookie hay storage dùng chung.

### 5.2. Bảng case

| ID | Requirement / rủi ro | Precondition | Input | Action | Expected result | Cleanup |
|---|---|---|---|---|---|---|
| CP-01 | R-UI-01, R-UI-04, R-FARE-01…08, R-CANCEL-01; control, tiền, vòng đời UI | Reset, 09:00 VN | ride, bike, 5 km | Nhập 5 → kiểm tràn → Báo giá → Đặt → kiểm tràn → Huỷ | `35.000 đ`; cao điểm «Không»; «Đang tìm tài xế» → «Đã huỷ»; «Phí huỷ: 0 đ»; nút huỷ disabled; không tràn ngang | Context đóng; reset ở test kế |
| CP-02 | R-UI-01, R-UI-04, R-FOOD-01, R-FOOD-04, R-FARE-02; trường động, select, negative path | Reset, 09:00 VN | food 5 km, rồi ride 0,1 km | Chọn food → Báo giá → kiểm tràn → chọn ride → nhập 0.1 → Báo giá → kiểm tràn | Ô tiền món hiện rồi ẩn; loại xe disabled rồi enabled; `95.000 đ`; nút đặt disabled sau khi đổi dịch vụ; lỗi chứa «Quãng đường» | Như trên |

Vì sao chỉ hai case? Các tổ hợp giá đã được khoá nhanh ở [SYS-03](/articles/system-testing-functional-test); vòng đời chuyến nhiều vai trò ở [SYS-05](/articles/system-testing-system-test). Ở đây ta chọn những thao tác **nhạy với môi trường**: click/tap, `select` đổi trường động, hiển thị tiền, lỗi đầu vào. Mỗi case chạy trên ba project ⇒ sáu execution.

### 5.3. Entry và exit criteria

- **Entry:** matrix đã xác định; Chrome và Edge có sẵn; server riêng khởi động được; dữ liệu chuẩn; lệnh chạy đúng config.
- **Exit của bài tập:** sáu execution pass, không retry, typecheck đạt, khoảng trống ghi rõ.
- **Exit của release thật:** theo support policy; có thể đòi engine, OS và thiết bị thật nằm ngoài matrix này.

Một quy tắc quan trọng: case **không chạy** vì thiếu browser khác với case **fail**. «Skipped Safari» không bao giờ được đếm là «Safari pass».

## 6. Thực hành với Playwright projects

### 6.1. Cấu hình matrix

File `examples/toi-noi-testing/compatibility.config.ts` định nghĩa ba project và server riêng:

```ts
projects: [
  { name: 'chrome-desktop-vi', use: { channel: 'chrome', viewport: { width: 1280, height: 800 }, locale: 'vi-VN', timezoneId: 'Asia/Ho_Chi_Minh' } },
  { name: 'edge-desktop-en', use: { channel: 'msedge', viewport: { width: 1280, height: 800 }, locale: 'en-US', timezoneId: 'America/New_York' } },
  { name: 'chrome-narrow-touch', use: { channel: 'chrome', viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true, locale: 'vi-VN', timezoneId: 'Asia/Ho_Chi_Minh' } },
],
webServer: {
  command: 'npx tsx src/server.ts', url: 'http://127.0.0.1:3116/api/health',
  reuseExistingServer: false, timeout: 60_000,
  env: { PORT: '3116', NODE_ENV: 'test', TOINOI_TEST_API: '1', TOINOI_DB: 'pglite',
    TOINOI_RAIN: '', TOINOI_START_AT: '2026-11-04T09:00:00+07:00',
    TRIP_RATE_LIMIT_MAX: '5', TRIP_RATE_LIMIT_WINDOW_MS: '60000' },
},
```

Cùng file đặt `workers: 1, retries: 0` và chỉ lưu trace/screenshot khi lỗi dưới `reports/compatibility`. `retries: 0` là chủ ý: với compatibility, một lần fail rồi pass khi chạy lại thường là dấu hiệu timing phụ thuộc engine — cần thấy nó, không che nó.

### 6.2. Chạy

Từ `examples/toi-noi-testing`:

```sh
npm ci
npx playwright test --config compatibility.config.ts
npm run typecheck
```

Chỉ cần `npm ci` khi chưa cài dependency. Để điều tra một cấu hình:

```sh
npx playwright test --config compatibility.config.ts --project=edge-desktop-en
```

Config mặc định của lab có thể tìm thấy spec này nhưng **không** tạo ma trận ba project. Khi báo cáo coverage, luôn ghi đúng lệnh đã chạy.

### 6.3. Ghi metadata môi trường vào kết quả

Mỗi execution bắt đầu bằng reset, đặt giờ, rồi ghi môi trường thực chạy — gồm `browser.version()` lấy từ chính browser, không phải tên project:

```ts
test.beforeEach(async ({ request, browser }, info) => {
  expect((await request.post('/api/test/reset')).status()).toBe(200);
  expect((await request.post('/api/test/clock', { data: { now: '2026-11-04T09:00:00+07:00' } })).status()).toBe(200);
  const environment = { project: info.project.name, browser: browser.version(), use: info.project.use };
  console.log(JSON.stringify(environment));
  await info.attach('environment', { body: JSON.stringify(environment, null, 2), contentType: 'application/json' });
});
```

Lưu ý `beforeEach` assert mã 200 của reset và clock. Nếu bước chuẩn bị hỏng, test dừng ngay ở đó — failure sẽ chỉ vào môi trường, không bị đọc nhầm thành lỗi giao diện.

### 6.4. Input và oracle bố cục

Hai helper giữ cho phần thân test giống hệt nhau giữa các project:

```ts
async function activate(page: Page, id: string, touch: boolean) {
  if (touch) await page.getByTestId(id).tap();
  else await page.getByTestId(id).click();
}
async function noHorizontalOverflow(page: Page) {
  const size = await page.evaluate('({content: document.documentElement.scrollWidth, viewport: document.documentElement.clientWidth})') as { content: number; viewport: number };
  expect(size.content).toBeLessThanOrEqual(size.viewport + 1);
}
```

Project `chrome-narrow-touch` dùng `tap()` thật trong context có `hasTouch`; desktop dùng `click()`. Tuy nhiên điền ô bằng `fill()` và chọn bằng `selectOption()` vẫn là automation: chưa mô phỏng bàn phím ảo che nửa màn hình hay native picker của Android/iOS. Tolerance 1 px tránh báo lỗi giả do làm tròn sub-pixel.

### 6.5. Phần thân CP-01

```ts
test('CP-01 · báo giá, đặt, huỷ và bố cục ngang trên cấu hình được chọn', async ({ page }, info) => {
  const touch = !!info.project.use.hasTouch;
  await page.goto('/');
  await page.getByTestId('distance').fill('5');
  await noHorizontalOverflow(page);
  await activate(page, 'quote-button', touch);
  await expect(page.getByTestId('quote-total')).toHaveText('35.000 đ');
  await expect(page.getByTestId('quote-peak')).toHaveText('Không');
  await activate(page, 'book-button', touch);
  await expect(page.getByTestId('trip-status')).toHaveText('Đang tìm tài xế');
  await noHorizontalOverflow(page);
  await activate(page, 'cancel-button', touch);
  await expect(page.getByTestId('trip-status')).toHaveText('Đã huỷ');
  await expect(page.getByTestId('cancel-fee')).toHaveText('Phí huỷ: 0 đ');
  await expect(page.getByTestId('cancel-button')).toBeDisabled();
});
```

Đọc các assertion theo rủi ro: `toHaveText('35.000 đ')` khoá cả số tiền lẫn định dạng (R-UI-04) — trên Edge/en-US đây là phép kiểm locale. `quote-peak = Không` trên project New York là phép kiểm «timezone client không tham gia tính giá». Kiểm tràn ngang hai lần — trước báo giá và sau khi đặt — vì sau khi đặt trang hiện thêm khối trạng thái, nơi chuỗi ID dài có thể đẩy bố cục.

### 6.6. Phần thân CP-02

```ts
test('CP-02 · chọn food rồi quay lại ride, trường động và lỗi không mất chức năng', async ({ page }, info) => {
  const touch = !!info.project.use.hasTouch;
  await page.goto('/');
  await page.getByTestId('kind').selectOption('food');
  await expect(page.getByTestId('food-amount')).toBeVisible();
  await expect(page.getByTestId('vehicle')).toBeDisabled();
  await page.getByTestId('distance').fill('5');
  await activate(page, 'quote-button', touch);
  await expect(page.getByTestId('quote-total')).toHaveText('95.000 đ');
  await noHorizontalOverflow(page);
  await page.getByTestId('kind').selectOption('ride');
  await expect(page.getByTestId('food-amount')).toBeHidden();
  await expect(page.getByTestId('vehicle')).toBeEnabled();
  await expect(page.getByTestId('book-button')).toBeDisabled();
  await page.getByTestId('distance').fill('0.1');
  await activate(page, 'quote-button', touch);
  await expect(page.getByTestId('error')).toContainText('Quãng đường');
  await noHorizontalOverflow(page);
});
```

CP-02 nhắm vào sự kiện `change` của `<select>` — loại hành vi các engine từng xử lý khác nhau — và vào khối lỗi, nơi thông báo dài dễ làm tràn khung hẹp. `book-button` disabled ngay sau khi đổi dịch vụ là oracle cho «sửa ô sau khi báo giá ⇒ phải báo giá lại» (R-UI-01). 0,1 km nằm dưới mức 0,5 km của R-FARE-02 nên phải ra lỗi.

## 7. Kết quả thực đo và cách đọc lỗi

### 7.1. Expected

Sáu execution pass (2 case × 3 project), không retry; typecheck exit 0. Mỗi execution in một dòng JSON metadata môi trường.

### 7.2. Observed — báo cáo lab ngày 13/11/2026

Báo cáo gốc (Windows, Node v24.15.0, lệnh `npx playwright test --config compatibility.config.ts`):

| Project | Browser thực đo | Kết quả |
|---|---|---|
| chrome-desktop-vi | Chrome 157.0.8103.51 | CP-01/02 PASS, 984/597 ms |
| edge-desktop-en | Edge 157.0.4341.46 | CP-01/02 PASS, 1,1 s/618 ms |
| chrome-narrow-touch | Chrome 157.0.8103.51 | CP-01/02 PASS, 930/645 ms |

Tổng 6 execution pass trong 10,8 giây, không retry; `npm run typecheck` exit 0. Có cảnh báo terminal về NO_COLOR/FORCE_COLOR, không ảnh hưởng kết quả.

### 7.3. Observed — chạy lại khi biên tập bài, cùng ngày

Chạy lại trên Windows 11, Node v22.18.0, Playwright 1.64.1, cùng lệnh:

```text
Running 6 tests using 1 worker
  ok 1 [chrome-desktop-vi] › … CP-01 · báo giá, đặt, huỷ và bố cục ngang trên cấu hình được chọn (1.4s)
  ok 2 [chrome-desktop-vi] › … CP-02 · chọn food rồi quay lại ride, trường động và lỗi không mất chức năng (932ms)
  ok 3 [edge-desktop-en] › … CP-01 · … (1.5s)
  ok 4 [edge-desktop-en] › … CP-02 · … (813ms)
  ok 5 [chrome-narrow-touch] › … CP-01 · … (1.1s)
  ok 6 [chrome-narrow-touch] › … CP-02 · … (634ms)

  6 passed (18.9s)
```

Metadata in ra cho thấy Chrome đã tự cập nhật lên **157.0.8103.78** (báo cáo gốc: .51); Edge vẫn **157.0.4341.46**. Đây chính là lý do phải ghi version từng lượt: hai lượt «Chrome pass» trong cùng một ngày không chạy trên cùng một binary. Thời gian của lượt chạy lại dài hơn vì máy đang chạy song song các tác vụ khác; số giây ở đây không phải số đo hiệu năng.

### 7.4. Diễn giải

Cả ba cấu hình hiện đúng 35.000 đ cho ride và 95.000 đ cho food; báo giá, đặt, huỷ hoạt động; phí huỷ khi searching là 0 đ; trường động và trạng thái disabled đúng; lỗi quãng đường hiện ra; không có tràn ngang vượt 1 px tại các điểm kiểm. **Không phát hiện defect** trong phạm vi đó. Kết luận đúng: «hai hành trình đạt trên hai browser Chromium và một cấu hình hẹp mô phỏng, với các version đã ghi» — không rộng hơn.

### 7.5. Đọc failure theo trục

| Dấu hiệu | Giả thuyết | Thí nghiệm tách yếu tố |
|---|---|---|
| Chỉ `edge-desktop-en` fail ở tiền | Formatter theo locale browser, hoặc khác biệt Edge | Chạy Chrome + en-US và Edge + vi-VN; yếu tố nào đổi kết quả là nguyên nhân |
| Chỉ narrow-touch không bấm được | Phần tử bị che, ngoài khung, target quá nhỏ, sự kiện touch | Xem trace, đo bounding box, thử `click()` cùng viewport |
| Giá cao điểm đổi theo project | Server dùng giờ client hoặc UI tự tính lại | So request/response; kiểm server clock |
| Tràn ngang chỉ xuất hiện sau khi đặt | ID dài, `min-width`, flex không wrap, fieldset | Screenshot trace; đo `scrollWidth` từng khối |
| Browser không khởi chạy | Thiếu channel, cài đặt hỏng | Báo **blocked** do môi trường, không báo chức năng hỏng |

Defect compatibility cần: tên project, browser + version, OS, viewport CSS pixel, device scale nếu có, locale, timezone, input method, bước tái hiện và trace/screenshot. Với lỗi chỉ thấy trên điện thoại thật, giữ tên model, OS version, hướng màn hình, trạng thái bàn phím — đừng thay bằng ảnh Chrome thu hẹp rồi tuyên bố đã tái hiện.

## 8. Mở rộng nâng cao

### 8.1. Thêm engine và thiết bị thật

Bước kế tiếp hợp lý là thêm project Firefox và WebKit của Playwright theo support matrix, rồi kiểm Safari thật trên môi trường Apple nếu sản phẩm hỗ trợ — WebKit automation không đại diện mọi bản Safari/iOS. Với điện thoại thật, chọn vài hành trình rủi ro cao để kiểm những thứ emulation không có: bàn phím ảo làm co viewport, safe-area, cuộn bằng chạm, hộp xin quyền vị trí, hiệu năng trên phần cứng yếu. Không cần mọi test trên mọi máy; cần hồ sơ coverage nói rõ máy nào chạy gì.

### 8.2. Pairwise và tổ hợp rủi ro

Browser × OS × viewport × locale × input tăng rất nhanh. Pairwise testing chọn tập nhỏ phủ mọi cặp giá trị, nhưng không bảo đảm bắt lỗi do ba yếu tố cùng lúc. Hãy giữ thêm những tổ hợp chủ đích, ví dụ «Safari iOS + bàn phím ảo + footer cố định». Matrix ba dòng của bài được chọn thủ công, **chưa** có chứng minh pairwise coverage — đừng gắn nhãn pairwise chỉ vì có nhiều cấu hình.

### 8.3. Backward compatibility của API và dữ liệu

Ứng dụng tài xế cũ vẫn còn trên máy người dùng khi server đã nâng cấp. Cần kiểm: trường mới có bắt buộc không, enum mới có làm client cũ vỡ không, giá trị mặc định, đàm phán version, migration dữ liệu. Với rolling deployment, phiên bản cũ và mới của server cùng chạy trên một schema trong một khoảng thời gian — đó cũng là compatibility. Chrome/Edge xanh không chứng minh client mobile phiên bản cũ còn dùng được.

### 8.4. Locale, timezone và định dạng nhập

Ô số nhận dấu phẩy hay dấu chấm, chuỗi Unicode, font fallback cho tiếng Việt, tên địa chỉ dài, ngày chuyển giờ mùa hè (DST) đều có thể sinh lỗi khác nhau theo cấu hình. Phân biệt **locale hiển thị** với **quy tắc nghiệp vụ**: giá tính theo giờ Việt Nam, nhưng hạn báo giá hiển thị cho người ở New York có thể theo giờ địa phương nếu sản phẩm thiết kế như vậy. Cần expected được thống nhất trước; đừng tự coi mọi khác biệt ngày giờ là bug.

### 8.5. Visual regression có baseline

Nếu thêm so sánh screenshot, tạo baseline theo từng project/font/OS và review khác biệt có chủ đích. Chỉ mask vùng biến động (giờ, ID) khi việc mask không che rủi ro cần kiểm. Không cập nhật mọi snapshot để làm xanh. Visual test bổ trợ, không thay assertion hành vi: một trang nhìn hoàn hảo vẫn có thể gửi sai request.

### 8.6. Duy trì support matrix theo thời gian

«Latest» là mục tiêu di động: lượt chạy lại của chính bài này đã thấy Chrome đổi build trong cùng ngày. Hãy ghi version mỗi lượt, review matrix khi analytics hoặc dependency thay đổi, và khi bỏ hỗ trợ một môi trường thì đó là quyết định sản phẩm có thông báo — không phải xoá project test vì nó hay fail.

## 9. Anti-pattern, giới hạn và đánh đổi

### 9.1. Anti-pattern

- Thu hẹp cửa sổ desktop rồi báo «Android/iOS pass».
- Đếm Chrome và Edge là hai engine độc lập.
- Kiểm đúng viewport nhưng dùng chung storage/cookie bẩn giữa các cấu hình.
- Đổi locale rồi kỳ vọng giao diện tự dịch khi SPEC không yêu cầu.
- Lấy output của browser đầu tiên làm expected cho các browser sau.
- Chỉ assert «không có lỗi console» và gọi đó là tương thích.
- Bật retry để làm xanh một project hay chập chờn.

### 9.2. Giới hạn bằng chứng của lab

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Những gì SYS-11 chưa chứng minh</span>
<p>Lab chưa kiểm Firefox, WebKit/Safari, Android hay iOS thật, macOS/Linux, xoay màn hình, zoom, bàn phím ảo, dark mode, high contrast, screen reader, mọi viewport hay phiên bản API cũ. Trang tài xế (R-UI-02) chưa nằm trong matrix. Phép kiểm tràn ngang chỉ ở các trạng thái đã chọn. Browser chạy headless; không có visual baseline hay ảnh được người review ở lượt pass. Kết quả không suy rộng thành support policy.</p>
</div>

### 9.3. Đánh đổi chi phí và phạm vi

Mỗi project thêm vào nhân thời gian chạy và chi phí điều tra flaky. Hai case × ba project chạy dưới 20 giây là phù hợp cho pipeline mỗi lần merge; matrix mười browser × thiết bị thật thì hợp với lịch chạy đêm hoặc trước release. Hãy để support policy quyết định cấu hình nào nằm ở gate nào, thay vì nhét mọi tổ hợp vào mọi lần chạy.

## 10. Bài tập và câu hỏi tự kiểm

### Bài tập

**Bài 1 (cơ bản).** Báo cáo ghi «6 passed». Có phải đã test sáu browser không? Viết lại câu kết luận cho chính xác.

<details>
<summary><b>Gợi ý đáp án</b></summary>
<p>Không. Đó là 2 case × 3 project; chỉ có hai browser (Chrome, Edge) cùng họ Chromium, và project thứ ba là Chrome với viewport hẹp + touch emulation. Câu đúng: «CP-01 và CP-02 đạt trên Chrome 157.0.8103.x và Edge 157.0.4341.46 desktop, cùng một cấu hình Chrome 390×844 mô phỏng chạm; chưa kiểm Gecko, WebKit hay thiết bị thật».</p>
</details>

**Bài 2 (trung bình).** Giả sử project `edge-desktop-en` fail: `quote-total` hiện `35,000 đ`. Bạn kết luận Edge lỗi được chưa? Thiết kế thí nghiệm.

<details>
<summary><b>Gợi ý đáp án</b></summary>
<p>Chưa. Project này đổi cùng lúc browser và locale. Thêm hai project tạm: Chrome + en-US và Edge + vi-VN. Nếu Chrome + en-US cũng hiện <code>35,000 đ</code> thì nguyên nhân là locale (ví dụ formatter đọc <code>navigator.language</code>), không phải engine. Dù nguyên nhân nào, đây vẫn là defect vì R-UI-04 yêu cầu định dạng Việt Nam; số tiền nghiệp vụ 35.000 vẫn phải đúng ở API.</p>
</details>

**Bài 3 (trung bình).** Thêm project hướng ngang 844×390. Cần sửa gì và được phép kết luận gì?

<details>
<summary><b>Gợi ý đáp án</b></summary>
<p>Thêm một project với <code>viewport: { width: 844, height: 390 }</code>, giữ <code>isMobile</code>, <code>hasTouch</code>, locale như narrow-touch; không cần sửa spec vì oracle không phụ thuộc kích thước. Chỉ kết luận được «bố cục và hành vi đạt ở khung 844×390 mô phỏng», không gọi là «xoay điện thoại thật» vì chưa có sensor, bàn phím ảo hay UI native.</p>
</details>

**Bài 4 (nâng cao).** Lập support matrix cho Tới Nơi giả định có ứng dụng khách trên iOS/Android và hai phiên bản API (v1, v2) cùng chạy trong ba tháng.

<details>
<summary><b>Gợi ý đáp án</b></summary>
<p>Các trục: OS + version, model đại diện, engine (WebView/Safari/Chrome), phiên bản app, phiên bản API. Ưu tiên theo analytics (ví dụ Android tầm trung chiếm đa số tài xế) và rủi ro (đặt, thanh toán, huỷ). Phân loại bắt buộc / best-effort / chưa hỗ trợ. Ghi rõ dòng nào chạy emulation, dòng nào trên máy thật; kiểm app cũ × API v2 và app mới × API v1 trong cửa sổ tương thích. Liệt kê phần chưa kiểm.</p>
</details>

### Câu hỏi tự kiểm

**Câu 1.** `scrollWidth ≤ clientWidth + 1` đạt có chứng minh chữ không chồng lên nhau không?

<details>
<summary><b>Đáp án</b></summary>
<p>Không. Nó chỉ chứng minh không có tràn ngang tại thời điểm đo. Chồng lấn, chữ bị cắt hay tương phản cần kiểm khác (visual review, accessibility check).</p>
</details>

**Câu 2.** Hai screenshot giống hệt nhau giữa Chrome và Edge có chứng minh nút «Đặt chuyến» dùng được trên Edge không?

<details>
<summary><b>Đáp án</b></summary>
<p>Không. Cần thao tác thật và oracle kết quả (trạng thái «Đang tìm tài xế»). Ảnh giống nhau không nói gì về sự kiện, request hay phản hồi.</p>
</details>

**Câu 3.** Vì sao project New York vẫn phải thấy «Cao điểm: Không» lúc server 09:00 giờ Việt Nam?

<details>
<summary><b>Đáp án</b></summary>
<p>R-FARE-04 xác định cao điểm theo giờ Việt Nam tại thời điểm đặt, do server tính. Timezone của browser không được tham gia tính giá; nếu kết quả đổi theo timezone client thì đó là defect.</p>
</details>

**Câu 4.** Chrome và Edge chạy cùng một ngày đều pass, vậy có cần ghi version không?

<details>
<summary><b>Đáp án</b></summary>
<p>Có. Chính lượt chạy lại của bài thấy Chrome đổi từ 157.0.8103.51 sang .78 trong cùng ngày. Không có version thì không tái hiện được, và không biết bằng chứng áp dụng cho binary nào.</p>
</details>

**Câu 5.** Project Edge không khởi chạy được vì máy không cài Edge. Ghi kết quả thế nào?

<details>
<summary><b>Đáp án</b></summary>
<p>Ghi <b>blocked</b> do môi trường kèm thông báo lỗi khởi chạy. Không ghi fail chức năng, và càng không ghi pass. Coverage của Edge trong lượt đó là «chưa kiểm».</p>
</details>

## 11. Tài liệu tham khảo

Nguồn ngoài tra cứu 13/11/2026.

- Playwright — *Projects*: nhóm cấu hình, browser channel, chọn project khi chạy. https://playwright.dev/docs/test-projects
- Playwright — *Emulation*: viewport, `isMobile`, `hasTouch`, locale, timezone và phạm vi của emulation. https://playwright.dev/docs/emulation
- ISTQB, *Certified Tester Foundation Level Syllabus v4.0.1* — test types và đặc tính chất lượng phi chức năng (dùng để chuẩn hoá thuật ngữ). https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf
- Phòng lab: `SPEC.md` (R-UI-01, R-UI-04, R-FARE-01…08, R-FOOD-01, R-FOOD-04, R-CANCEL-01), `compatibility.config.ts` và spec của bài — https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/system-testing-compatibility-test
- Bài liên quan: [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) · [Non-functional testing tổng quan](/articles/non-functional-testing-tong-quan) · [SYS-05 System Test](/articles/system-testing-system-test) · [SYS-10 Security Test – Pen test](/articles/system-testing-security-pentest)

Bài tiếp theo: [SYS-12 · Usability Test](/articles/system-testing-usability-test).
