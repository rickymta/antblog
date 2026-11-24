---
slug: system-testing-uat
title: "UAT: nghiệm thu theo nhu cầu nghiệp vụ, không theo màu của pipeline"
summary: "UAT trên lab Tới Nơi: biến nhu cầu thành acceptance criteria đo được, tổ chức phiên khách – tài xế – vận hành, phân loại defect, yêu cầu mới, giới hạn môi trường và lập biên bản quyết định có người chịu trách nhiệm. Kèm acceptance kit và 4 supporting check tự động — không thay sign-off."
category: testing
level: intermediate
tags: [testing, sys-15, uat, acceptance-testing, acceptance-criteria, given-when-then, sign-off, risk-based-testing, vitest, toi-noi]
language: vi
---

Chiều thứ Sáu, pipeline của Tới Nơi xanh từ smoke tới recovery. Product owner hỏi: «Vậy mình chấp nhận bản này được chưa?». Câu trả lời trung thực là *chưa biết*: chưa ai đại diện khách, tài xế hay vận hành thực sự dùng bản đó để làm công việc của họ, và chưa ai có thẩm quyền xem bằng chứng rồi ký. Bài này trả lời: **UAT tạo ra bằng chứng gì, ai quyết định, quyết định được viết thế nào** — và vì sao 4/4 test tự động pass không bao giờ được đọc thành «100% UAT đạt».

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · SYS-15</span>
<table>
<tbody>
<tr><th scope="row">Series</th><td>Kiểm thử hệ thống Tới Nơi · Phần 2</td></tr>
<tr><th scope="row">Cấp độ</th><td>Intermediate — tester/QA lead, BA, product owner chuẩn bị nghiệm thu một phạm vi bàn giao</td></tr>
<tr><th scope="row">Test level / hướng kiểm</th><td>Acceptance (user acceptance); hướng kiểm chức năng theo nhu cầu nghiệp vụ, kèm quan sát khả năng tự hoàn thành công việc</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/system-testing-recovery-test">SYS-14 · Recovery Test</a> · <a href="/articles/acceptance-testing-tong-quan">Acceptance Testing tổng quan</a> · <a href="/articles/system-testing-functional-test">SYS-03 · Functional Test</a> · <a href="/articles/system-testing-system-test">SYS-05 · System Test</a></td></tr>
<tr><th scope="row">Thực hành</th><td>(1) Acceptance kit: 6 scenario UA-01…06, worksheet, risk register, biên bản quyết định trống. (2) Lab (code): 4 supporting check API + PGlite cho UA-01…03 (R-QUOTE-01/03/04, R-IDEM-02, R-STATE-06, R-CANCEL-01/02/03/06/07, R-PAY-01) — <code>examples/toi-noi-testing/articles/system-testing-uat/</code> · Minh hoạ: CurlStudio với collection <code>examples/toi-noi-testing/curlstudio/</code></td></tr>
<tr><th scope="row">Nguồn chính</th><td>ISTQB Glossary (acceptance testing); ISTQB CT-AcT syllabus v1.0; <code>SPEC.md</code> của lab; báo cáo SYS-12/13/14</td></tr>
</tbody>
</table>
</div>

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Trạng thái thật của bài này</span>
<p>Chưa có phiên UAT nào với người dùng thật, chưa có biên bản được ký, chưa có quyết định nghiệm thu hay release. Năm scenario người dùng đang <b>NOT RUN</b>, UA-04 <b>BLOCKED</b>. Bốn test tự động pass là bằng chứng kỹ thuật <i>hỗ trợ</i> ba tiêu chí, không phải kết quả UAT. Mọi tên người, chữ ký và quyết định trong mẫu đều để trống có chủ đích.</p>
</div>

## 1. Bài toán và mục tiêu

### 1.1. Tình huống

Tới Nơi chuẩn bị bàn giao phạm vi **đặt xe máy tại SG, thanh toán tiền mặt**. Qua mười bốn bài trước, nhóm đã có nhiều bằng chứng: smoke, functional, integration, system journey, regression, hiệu năng, bảo mật, tương thích, toàn vẹn dữ liệu, phục hồi. Tất cả đều trả lời «*hệ thống làm đúng điều đặc tả nói không?*». UAT hỏi khác: «*người dùng có làm được công việc của họ với phạm vi bàn giao này không, và người có thẩm quyền có chấp nhận rủi ro còn lại không?*».

Hai câu hỏi có thể có câu trả lời khác nhau. Một chuyến đi qua bảy trạng thái đúng thứ tự vẫn có thể khiến khách tưởng mình bị thu hai lần vì thông báo mơ hồ. Một phí huỷ 10.000 đ đúng R-CANCEL-03 vẫn có thể bị business từ chối vì họ muốn khách thấy phí *trước* khi bấm huỷ — yêu cầu chưa từng nằm trong SPEC.

### 1.2. Đối tượng và điều kiện tiên quyết

- Đã đọc SPEC của lab, SYS-03 (oracle tiền), SYS-05 (hành trình khách – tài xế), và giới hạn của SYS-12 (chưa có user study) và SYS-14 (khoản thu chưa xác nhận).
- Lab API cần Node.js ≥ 22.12 và dependency của `examples/toi-noi-testing`. Phiên UAT qua UI cần đại diện khách, tài xế và nghiệp vụ vận hành — điều kiện mà lần thực hiện bài này **chưa có**.

### 1.3. Mục tiêu đo được

Sau bài này, bạn có thể:

1. Viết một acceptance criterion có ID, nguồn, vai trò xác nhận, dữ liệu và điều kiện đạt — kiểm được bằng quan sát.
2. Tổ chức kịch bản liên vai trò (khách + tài xế + vận hành) và ghi worksheet có mong đợi, thực tế, bằng chứng; phân biệt PASS, FAIL, BLOCKED, NOT RUN.
3. Phân loại một sai lệch thành defect, yêu cầu thay đổi hoặc giới hạn môi trường, và nói được bước tiếp theo.
4. Lập biên bản quyết định có phạm vi, ngoại lệ, owner, điều kiện vô hiệu — và từ chối quy đổi supporting checks thành tỷ lệ UAT.

## 2. Khái niệm và các ranh giới

Khái niệm acceptance testing theo ISTQB đã có ở bài [Acceptance Testing tổng quan](/articles/acceptance-testing-tong-quan); ở đây chỉ nhắc điều cần cho thực hành. Acceptance testing xem xét nhu cầu, yêu cầu và quy trình nghiệp vụ để **hỗ trợ quyết định chấp nhận**. UAT tập trung vào việc phạm vi bàn giao có đáp ứng nhu cầu sử dụng không; người có thẩm quyền nghiệp vụ quyết định dựa trên bằng chứng. BA/product owner và tester phải phối hợp từ lúc làm rõ tiêu chí, không phải tuần cuối trước release.

| Hoạt động | Câu hỏi chính | Đầu ra |
|---|---|---|
| System test | Hệ thống thực hiện yêu cầu như thế nào? | Kết quả kiểm thử kỹ thuật/nghiệp vụ |
| UAT | Phạm vi bàn giao có đáp ứng công việc đã thống nhất? | Bằng chứng và quyết định của đại diện được giao thẩm quyền |
| Usability study | Người dùng làm việc có thuận lợi, dễ hiểu? | Quan sát, vấn đề tương tác, insight |
| Operational acceptance | Có thể vận hành, giám sát, hỗ trợ, phục hồi? | Runbook, diễn tập, tiêu chí vận hành |
| Deployment test | Bản phát hành được triển khai và hoạt động đúng? | Bằng chứng cấu hình, migration, smoke, rollback |

Ba điều hay bị nhầm:

- **Automation trong UAT.** UAT có thể dùng automation để thu bằng chứng, nhưng suite xanh không thay người ra quyết định. Automation trả lời «dữ liệu có đúng tiêu chí không», không trả lời «khách có hiểu», «vận hành có nhận trách nhiệm».
- **Demo khác UAT.** Demo do người làm sản phẩm điều khiển. UAT cần tiêu chí thống nhất trước, dữ liệu xác định, kết quả thực tế do người dùng tạo ra, bằng chứng và cách xử lý sai lệch.
- **Không phải mọi thứ gần ngày release đều là UAT.** Alpha/beta, nghiệm thu hợp đồng, nghiệm thu theo quy định có mục đích và phạm vi riêng; gọi tất cả là «UAT» làm mờ câu hỏi mỗi hoạt động đang trả lời.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 920 300" role="img" aria-label="Luồng UAT của bài: nhu cầu nghiệp vụ được chuyển thành acceptance criteria. Từ đó có hai nguồn evidence: supporting checks tự động, 4 trên 4 pass, và phiên người dùng với 6 scenario, trong đó 5 NOT RUN và 1 BLOCKED. Hai nguồn đi vào triage gồm defect, yêu cầu mới, giới hạn môi trường. Cuối cùng là quyết định của người có thẩm quyền, hiện trạng: chưa có quyết định.">
<defs>
<marker id="sys15-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" class="f-pk"/></marker>
</defs>
<rect x="16" y="110" width="150" height="70" rx="8" class="f-phys-soft s-phys"/><text x="91" y="140" text-anchor="middle" class="t-b">Nhu cầu</text><text x="91" y="160" text-anchor="middle" class="t-small">khách · tài xế · vận hành</text>
<rect x="200" y="110" width="160" height="70" rx="8" class="f-pk-soft s-pk"/><text x="280" y="140" text-anchor="middle" class="t-b">Acceptance criteria</text><text x="280" y="160" text-anchor="middle" class="t-small">ID · nguồn R-… · vai trò</text>
<rect x="395" y="30" width="230" height="80" rx="8" class="f-pk-soft s-pk"/><text x="510" y="58" text-anchor="middle" class="t-b">Supporting checks</text><text x="510" y="78" text-anchor="middle" class="t-small">API + PGlite, UA-01…03</text><text x="510" y="96" text-anchor="middle" class="t-small">4/4 pass (13/11/2026)</text>
<rect x="395" y="180" width="230" height="80" rx="8" class="f-warn-soft s-warn"/><text x="510" y="208" text-anchor="middle" class="t-b">Phiên người dùng</text><text x="510" y="228" text-anchor="middle" class="t-small">UA-01…06 qua / và /driver.html</text><text x="510" y="246" text-anchor="middle" class="t-small t-warn">5 NOT RUN · 1 BLOCKED</text>
<rect x="660" y="110" width="110" height="70" rx="8" class="f-pk-soft s-pk"/><text x="715" y="140" text-anchor="middle" class="t-b">Triage</text><text x="715" y="160" text-anchor="middle" class="t-small">defect / mới / môi trường</text>
<rect x="790" y="100" width="120" height="90" rx="8" class="f-warn-soft s-warn"/><text x="850" y="130" text-anchor="middle" class="t-b">Quyết định</text><text x="850" y="150" text-anchor="middle" class="t-small">người có thẩm quyền</text><text x="850" y="172" text-anchor="middle" class="t-small t-warn">CHƯA CÓ</text>
<line x1="166" y1="145" x2="196" y2="145" class="s-pk" stroke-width="1.6" marker-end="url(#sys15-arrow)"/>
<line x1="360" y1="135" x2="391" y2="80" class="s-pk" stroke-width="1.6" marker-end="url(#sys15-arrow)"/>
<line x1="360" y1="155" x2="391" y2="210" class="s-pk" stroke-width="1.6" marker-end="url(#sys15-arrow)"/>
<line x1="625" y1="80" x2="656" y2="130" class="s-pk" stroke-width="1.6" marker-end="url(#sys15-arrow)"/>
<line x1="625" y1="215" x2="656" y2="165" class="s-pk" stroke-width="1.6" marker-end="url(#sys15-arrow)"/>
<line x1="770" y1="145" x2="786" y2="145" class="s-pk" stroke-width="1.6" marker-end="url(#sys15-arrow)"/>
<text x="460" y="290" text-anchor="middle" class="t-small">supporting checks chỉ đi vào triage như evidence kỹ thuật — không có mũi tên nào từ "4/4 pass" thẳng tới quyết định</text>
</svg>
</div>
<figcaption><b>Hình 1 · Bằng chứng và quyết định trong UAT của lab.</b> Hai nguồn bằng chứng khác bản chất được triage chung, nhưng chỉ người có thẩm quyền mới tạo ra quyết định. Hiện trạng ngày 13/11/2026: nhánh dưới chưa chạy, ô quyết định trống.</figcaption>
</figure>

## 3. Từ nhu cầu đến acceptance criteria

### 3.1. Viết tiêu chí kiểm được

Nhu cầu: *«Là khách, tôi muốn chuyến giữ đúng giá đã đồng ý để không bị tính lại ngoài dự kiến.»* Tiêu chí dựa trên **R-QUOTE-04** (không đổi giá hồi tố): chuyến dùng ảnh chụp giá của báo giá hợp lệ; kết quả khách xem sau khi hoàn thành vẫn có cùng `fare`. Không viết tiêu chí mơ hồ như «giá phải tốt» hay «thanh toán chạy ổn» — không ai quan sát được chúng.

Ví dụ Given–When–Then cho UA-01:

```gherkin
Given khách C001 có báo giá xe máy SG 5 km lúc 09:00, không mưa, tổng 35.000 đ
And chọn tiền mặt
When khách đặt chuyến và gửi lại cùng yêu cầu
And tài xế D101 nhận, đến đón, đón khách, bắt đầu và hoàn thành
Then khách có đúng một chuyến completed với giá 35.000 đ
And hệ thống ghi thanh toán cash, không gọi ví điện tử
And tài xế trở lại online
```

Ba lưu ý khi đọc ví dụ này:

- **35.000 đ được tính độc lập**, không lấy từ hệ thống đang kiểm: 12.000 + 4.500 × 5 = 34.500, làm tròn lên nghìn (R-FARE-08) thành 35.000 — cùng oracle với [SYS-01](/articles/system-testing-smoke-test).
- **`cash` là phương thức/trạng thái trong lab** (R-PAY-01), không chứng minh tiền mặt đã trao tay ngoài đời; tiêu chí UAT thật về tiền mặt cần quy trình đối chiếu của vận hành.
- Given–When–Then giúp làm rõ điều kiện và kết quả; **không bắt buộc** dùng Cucumber, và đừng biến từng click thành một tiêu chí nghiệp vụ. Cách viết chi tiết: [Acceptance criteria Given–When–Then](/articles/ba-acceptance-criteria-given-when-then).

### 3.2. Năm thuộc tính bắt buộc của một tiêu chí

| Thuộc tính | Ví dụ UA-02 | Thiếu thì sao |
|---|---|---|
| ID | UA-02 | Không truy được bằng chứng, defect, retest |
| Nguồn yêu cầu | R-CANCEL-01/02/03/06/07 | Tranh cãi «đây là bug hay ý muốn mới» |
| Vai trò xác nhận | Khách + tài xế; business owner chốt | Không ai đủ thẩm quyền nói «đạt» |
| Phạm vi dữ liệu | C001, D101, bike, SG, cash; hai chuyến độc lập | Kết quả không tái hiện được |
| Điều kiện đạt | 1 phút: phí 0, `grace_period`; đúng 2 phút: phí 10.000, `charged` | PASS theo cảm giác |

Nếu business muốn **báo trước phí huỷ** trước khi khách bấm xác nhận, phải ghi thành yêu cầu riêng. SPEC hiện chỉ mô tả phí và lý do trả về *sau* thao tác huỷ (R-CANCEL-06); tester không tự suy ra màn hình preview phí rồi đánh FAIL vì thiếu nó.

## 4. Phạm vi nghiệm thu và test basis

Phạm vi mẫu là **lab đặt xe máy, SG, khách C001, tài xế D101**, báo giá và tiền mặt, kèm review tình huống ví lỗi. Đây **không** phải nghiệm thu toàn sản phẩm Tới Nơi: giao đồ ăn, bưu kiện, các thành phố/loại xe khác, bản đồ thật, token thật, ledger/webhook và support console nằm ngoài.

| Scenario | Nguồn | Kết quả nghiệp vụ cần xem | Trạng thái 13/11/2026 |
|---|---|---|---|
| UA-01 | R-QUOTE-04, R-IDEM-02, R-STATE-02/06, R-PAY-01 | Đặt – hoàn thành, giữ giá, không tạo trùng | Phiên: NOT RUN · API hỗ trợ: pass |
| UA-02 | R-CANCEL-01/02/03/06/07 | Huỷ trước 2 phút miễn; đúng 2 phút thu 10.000 đ; biết lý do | Phiên: NOT RUN · API hỗ trợ: pass (2 mốc) |
| UA-03 | R-QUOTE-01/03 | Báo giá đúng 5 phút hết hạn, không tạo chuyến | Phiên: NOT RUN · API hỗ trợ: pass |
| UA-04 | R-PAY-04/06, SYS-14 RC-03 | Nhận biết khoản thu chưa xác nhận và cách chuyển xử lý | BLOCKED (thiếu công cụ đối soát) |
| UA-05 | R-UI-01/02/03/04 | Người dùng hiểu giá, trạng thái, lỗi và bước tiếp theo | NOT RUN |
| UA-06 | Đề xuất mở rộng vận hành | Có người chịu trách nhiệm incident/tồn đọng | NOT RUN |

UA-06 là **đề xuất** tiêu chí bàn giao vận hành, chưa phải chức năng có trong SPEC. UA-04 không được coi là support scenario end-to-end khi chưa có công cụ đối soát.

**Entry criteria (mẫu, chưa ai phê duyệt):** build cố định, smoke đạt, môi trường sẵn sàng, tiêu chí đã được review. **Exit criteria dự kiến:** đủ bằng chứng cho scenario bắt buộc, lỗi chặn đã xử lý, mọi ngoại lệ có owner, người quyết định xác nhận build và phạm vi.

## 5. Vai trò và cách tổ chức phiên

| Vai trò | Làm gì | Không được làm gì |
|---|---|---|
| Business owner | Chốt tiêu chí, phạm vi, ra quyết định | Uỷ quyền ngầm cho pipeline |
| Đại diện khách/tài xế | Thực hiện công việc, mô tả điều thấy và hiểu | Đọc kịch bản từng click do QA soạn |
| QA | Chuẩn bị dữ liệu, clock, ghi bằng chứng, hỗ trợ tái hiện | Sửa dữ liệu ngầm để luồng «đẹp» |
| Developer | Điều tra lỗi | Tự quyết sai lệch là «không đáng kể» |
| Vận hành | Xác nhận quy trình hỗ trợ, tồn đọng thuộc trách nhiệm | Nhận tồn đọng mà không có runbook |

Trong nhóm nhỏ, một người có thể giữ nhiều vai trò, nhưng biên bản phải ghi rõ người ký **với trách nhiệm nào**. Không bao giờ tự điền tên hoặc chữ ký cho người chưa tham gia — kể cả «để điền cho đủ mẫu».

**Trước phiên:** đóng băng build và bộ tiêu chí; chọn dữ liệu đại diện; xác nhận môi trường và giới hạn; chạy [smoke](/articles/system-testing-smoke-test) để thời gian của người dùng không bị phí vào lỗi khởi động. **Trong phiên:** đưa mục tiêu công việc trước («hãy đặt một chuyến xe máy 5 km và trả tiền mặt»), để người dùng tự thao tác; QA phải hướng dẫn thì ghi rõ mức hỗ trợ. Ghi mong đợi/thực tế theo scenario, không chỉ «OK». Gặp lỗi cản trở thì ghi **BLOCKED** cho phần chưa làm được, tách khỏi **FAIL** của phần đã kiểm và không đạt. **Sau phiên:** triage sai lệch, chốt retest, cập nhật quyết định trên đúng build.

## 6. Thực hành 1 — supporting checks tự động

### 6.1. Chạy lab

Mã đầy đủ: `examples/toi-noi-testing/articles/system-testing-uat/acceptance-support.test.ts`. Từ thư mục lab:

```sh
npm ci
npm test -- articles/system-testing-uat
npm run typecheck
```

Suite có bốn test case: UA-01, UA-02 với hai mốc thời gian, UA-03. API Express nối PGlite; `FixedClock` cố định 09:00 giờ VN ngày 04/11/2026, seed reset trước từng test case, DB đóng cuối suite; không cần browser hay server ngoài.

### 6.2. Đọc assertion

Hai helper dùng chung ghim dữ liệu của tiêu chí: báo giá phải ra đúng 35.000, đặt chuyến luôn kèm cùng `Idempotency-Key` và `paymentMethod: 'cash'`.

```ts
async function quote() {
  const r = await request(app).post('/api/quotes').set('x-user', 'customer:C001')
    .send({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 }).expect(200);
  expect(r.body.quote.total).toBe(35_000);
  return r.body.quote.quoteId as string;
}
const book = (quoteId: string) => request(app).post('/api/trips').set('x-user', 'customer:C001')
  .set('Idempotency-Key', 'uat-book-001').send({ quoteId, paymentMethod: 'cash' });
```

**UA-01** kiểm «không tạo trùng» bằng hai lớp: replay trả về đúng thân response cũ (R-IDEM-02), và đếm thẳng trong bảng `trips`. Sau đó kiểm chuỗi trạng thái trong nhật ký (R-STATE-06), dữ liệu khách đọc lại, ví không bị gọi và tài xế được giải phóng:

```ts
const first = await book(qid).expect(201);
const replay = await book(qid).expect(201);
expect(replay.body).toEqual(first.body);
// ... accept, arrive, pickup, start, complete
expect(view.body.trip).toMatchObject({ id, fare: 35_000, status: 'completed', paymentStatus: 'cash' });
expect((await system.db!.query('SELECT count(*)::int AS n FROM trips')).rows).toEqual([{ n: 1 }]);
expect((await system.deps.trips.listEvents(id)).map(e => e.status))
  .toEqual(['requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit', 'completed']);
expect(system.payments.calls).toHaveLength(0);
expect((await system.deps.drivers.findById('D101'))?.status).toBe('online');
```

**UA-02** dùng `it.each` cho hai mốc quanh biên R-CANCEL-02 — 1 phút (miễn, `grace_period`) và **đúng** 2 phút (phí 10.000, `charged`). Đúng 2 phút 0 giây tính là «từ 2 phút trở lên» — giá trị biên dễ sai nhất nên phải có mặt:

```ts
it.each([
  { minutes: 1, fee: 0, reason: 'grace_period' },
  { minutes: 2, fee: 10_000, reason: 'charged' },
])('UA-02 · cancel at $minutes minutes costs $fee', async ({ minutes, fee, reason }) => {
  // ... book, accept
  clock.advanceMinutes(minutes);
  // ... cancel với Idempotency-Key 'uat-cancel-001'
  expect(cancelled.body).toMatchObject({ cancellationFee: fee, feeReason: reason,
    trip: { status: 'cancelled', cancellationFee: fee } });
  expect((await system.deps.trips.findById(id))?.cancellationFee).toBe(fee);
  expect((await system.deps.drivers.findById('D101'))?.status).toBe('online');
  expect(system.payments.calls).toHaveLength(0);
});
```

Với chuyến `cash`, phí huỷ được ghi nhận vào chuyến chứ không thu qua ví (R-PAY-05, R-CANCEL-07), nên `payments.calls` bằng 0 là đúng tiêu chí, không phải dấu hiệu «quên thu tiền».

**UA-03** đặt đúng biên hết hạn 5 phút (R-QUOTE-01) và kiểm cả ba hậu quả phải *không* xảy ra: không có chuyến, báo giá chưa bị đánh dấu dùng, không gọi ví:

```ts
clock.advanceMinutes(5);
const r = await book(qid).expect(422);
expect(r.body.error.code).toBe('QUOTE_EXPIRED');
expect((await system.db!.query('SELECT count(*)::int AS n FROM trips')).rows).toEqual([{ n: 0 }]);
expect((await system.deps.quotes.findById(qid))?.usedByTripId).toBeNull();
expect(system.payments.calls).toHaveLength(0);
```

### 6.3. Kết quả: mong đợi và thực tế

**Mong đợi:** 1 file, 4 test pass, typecheck exit 0. Thời gian thay đổi theo máy.

**Thực tế — báo cáo gốc (13/11/2026):** Windows, Node.js 24.15.0, Vitest 3.2.7, PGlite 0.3.16 — 4/4 pass, duration 2,67 giây (test 1,56 giây); typecheck pass, exit 0.

**Thực tế — chạy lại khi biên tập (13/11/2026):** Windows, Node v22.18.0, Vitest 3.2.7, lệnh `npx vitest run articles/system-testing-uat`:

```text
✓ articles/system-testing-uat/acceptance-support.test.ts (4 tests) 3023ms
  ✓ SYS-15 · acceptance evidence support, not business sign-off > UA-01 · agreed fare, replay and complete cash journey 369ms

Test Files  1 passed (1)
     Tests  4 passed (4)
  Duration  4.89s
```

Lượt chạy lại không chạy typecheck; kết quả typecheck lấy từ báo cáo gốc. Thời gian giữa hai lượt chênh do máy, phiên bản Node và tải nền — không phải phép đo hiệu năng.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">4/4 pass chứng minh gì — và không chứng minh gì</span>
<p>Chứng minh: trong lab API/PGlite với clock cố định, dữ liệu lưu đúng tiêu chí UA-01…03. Không chứng minh: người dùng hiểu màn hình (không có browser trong suite), xác thực thật (header <code>x-user</code> là mô phỏng danh tính), ví/provider thật, hiệu năng, khả năng vận hành production, phạm vi food/parcel. Không được báo «UAT đạt 4/4» hay «100%» khi sáu scenario người dùng chưa chạy.</p>
</div>

### 6.4. Minh hoạ với CurlStudio

Supporting check UA-02 cũng chạy được bằng Collection Runner của CurlStudio, với collection đã giới thiệu ở [SYS-01](/articles/system-testing-smoke-test). Chuột phải folder «SYS-15 · Supporting check UA-02 (huỷ chuyến)» → **Chạy thư mục**, rồi trong runner bấm **Chọn file** và chọn `huy-chuyen.json` — 2 dòng, mỗi dòng một iteration: huỷ sau 1 phút và huỷ sau 3 phút. Mỗi iteration reset dữ liệu, đặt đồng hồ 09:00 ngày 04/11/2026, báo giá xe máy 5 km, đặt chuyến tiền mặt, cho tài xế D101 nhận, dời đồng hồ lab qua `/api/test/clock` theo số phút trong tệp dữ liệu, rồi khách huỷ.

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/system-testing/sys-15/01-runner-ua02-huy-chuyen.webp" alt="Collection Runner của CurlStudio chạy folder «SYS-15 · Supporting check UA-02 (huỷ chuyến)» với tệp dữ liệu huy-chuyen.json, 2 dòng, mỗi dòng một iteration: thanh tổng Requests 14, Tests 16/16 (100%), Thất bại 0. Danh sách kết quả cho thấy các request «UA-02 · Dời đồng hồ theo dữ liệu» và «UA-02 · Khách huỷ chuyến» ở cả hai iteration; iteration #1 có test «Huỷ sau 1 phút → phí 0 đ (grace_period)», iteration #2 có test «Huỷ sau 3 phút → phí 10000 đ (charged)»." width="2000" height="1125" loading="lazy"></div>
<figcaption><b>Hình 2 · Supporting check UA-02 chạy bằng tệp dữ liệu hai dòng.</b> Iteration #1 huỷ sau 1 phút: phí 0 đ, <code>grace_period</code>; iteration #2 huỷ sau 3 phút: phí 10000 đ, <code>charged</code>. Mốc đúng 2 phút không có ở đây — nó thuộc test Vitest UA-02.</figcaption>
</figure>

Thanh tổng ghi **Requests 14 · Tests 16 / 16 (100%) · Thất bại 0**. Hai test đáng đọc nằm ở request «UA-02 · Khách huỷ chuyến» của mỗi iteration: «Huỷ sau 1 phút → phí 0 đ (grace_period)» và «Huỷ sau 3 phút → phí 10000 đ (charged)» — một mốc mỗi bên của biên R-CANCEL-02.

#### Tiêu chí nằm trong tệp dữ liệu, không nằm trong request

Cùng một folder 7 request chạy được hai kịch bản là vì các con số của tiêu chí — số phút, phí, lý do — được đưa ra ngoài request. Request dời đồng hồ chỉ giữ một chỗ trống:

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/system-testing/sys-15/02-body-dong-ho-theo-du-lieu.webp" alt="CurlStudio mở request «UA-02 · Dời đồng hồ theo dữ liệu» trong folder SYS-15, tab Body chứa JSON {&quot;now&quot;: &quot;{{now}}&quot;}, biến {{now}} tô đỏ vì chưa có giá trị ngoài runner." width="2000" height="1125" loading="lazy"></div>
<figcaption><b>Hình 3 · Thời điểm huỷ lấy từ tệp dữ liệu.</b> Thân chỉ có <code>{"now": "{{now}}"}</code>; biến tô đỏ vì ngoài runner chưa có dòng dữ liệu nào để thế vào.</figcaption>
</figure>

Màu đỏ là lời nhắc thực tế: gửi tay request này sẽ không dời đồng hồ đến đâu cả. Folder chỉ có nghĩa khi chạy kèm `huy-chuyen.json`, giống như tiêu chí UA-02 chỉ kiểm được khi có đủ hai mốc thời gian.

Request huỷ thì cố định, đúng thao tác người khách sẽ làm trong phiên:

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/system-testing/sys-15/03-body-huy-chuyen.webp" alt="CurlStudio mở request «UA-02 · Khách huỷ chuyến»: POST {{baseUrl}}/api/trips/{{tripId}}/cancel, tab Body chứa JSON với reason là «Thay đổi kế hoạch»." width="2000" height="1125" loading="lazy"></div>
<figcaption><b>Hình 4 · Thao tác huỷ của khách.</b> POST tới <code>/api/trips/{{tripId}}/cancel</code> với lý do «Thay đổi kế hoạch»; <code>tripId</code> do bước đặt chuyến của cùng iteration lưu lại.</figcaption>
</figure>

Phép kiểm ở tab **Tests** đọc kết quả mong đợi từ cùng dòng dữ liệu:

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/system-testing/sys-15/04-tests-phi-huy.webp" alt="Tab Tests của request «UA-02 · Khách huỷ chuyến» trong CurlStudio: test HTTP 200, const b = pm.response.json(), rồi một test có tên ghép «Huỷ sau … → phí … đ (…)» từ pm.iterationData, kiểm b.trip.status bằng cancelled, b.cancellationFee bằng fee và b.feeReason bằng reason của dòng dữ liệu." width="2000" height="1125" loading="lazy"></div>
<figcaption><b>Hình 5 · Oracle đọc từ dòng dữ liệu.</b> Tên test và ba giá trị mong đợi — <code>cancelled</code>, <code>fee</code>, <code>reason</code> — đều lấy qua <code>pm.iterationData</code>.</figcaption>
</figure>

So với UA-02 trong Vitest ở 6.2, script này kiểm ba trường của response nhưng không kiểm tài xế D101 trở lại `online`, không đọc lại phí đã lưu từ repository và không đếm lần gọi ví. Thêm nữa, kết quả mong đợi giờ nằm trong tệp JSON: ai sửa tệp đó là sửa tiêu chí. Tệp dữ liệu vì vậy cần được business owner xác nhận giống như bảng tiêu chí ở mục 3, chứ không phải chỉ QA tự điền.

#### Console: thứ tự từng bước trong mỗi iteration

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/system-testing/sys-15/05-console-ua02.webp" alt="Collection Runner của CurlStudio cho folder SYS-15 với tệp huy-chuyen.json, Requests 14, Tests 16/16 (100%); panel Console 14 dòng bên dưới liệt kê theo thứ tự: /api/test/reset, /api/test/clock, /api/quotes, /api/trips → 201 Created, /accept, /api/test/clock lần hai, rồi /cancel → 200 OK, lặp lại cho iteration thứ hai." width="2000" height="1125" loading="lazy"></div>
<figcaption><b>Hình 6 · Console của lượt UA-02.</b> Mỗi iteration đi đúng chuỗi reset → clock 09:00 → quotes → trips 201 → accept → clock (dời) → cancel 200.</figcaption>
</figure>

Hai lệnh `/api/test/clock` trong mỗi iteration là điểm đáng đọc: lệnh đầu đặt 09:00, lệnh sau dời theo dữ liệu, và request huỷ đến ngay sau đó. Giờ ở đầu dòng là giờ máy thật, không phải đồng hồ lab.

Vì sao 3 phút chứ không đúng 2? `FixedClock` trong suite Vitest đứng yên giữa các bước, nên `clock.advanceMinutes(2)` chạm đúng 2 phút 0 giây. Đồng hồ lab trên máy chủ thì vẫn trôi sau khi được dời: giữa request dời đồng hồ và request huỷ luôn có thêm một khoảng thời gian, nên runner không đặt được đúng mốc biên. Folder vì vậy dùng 3 phút; giá trị biên dễ sai nhất vẫn do test Vitest UA-02 ở 6.2 kiểm.

#### Ghi giới hạn ngay cạnh bằng chứng

Giới hạn ấy không chỉ nằm trong bài viết. Mở **Cài đặt thư mục** của folder SYS-15, tab **Tổng quan**:

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/system-testing/sys-15/06-mo-ta-thu-muc.webp" alt="Cài đặt thư mục của folder SYS-15 trong CurlStudio, tab Tổng quan, ô Mô tả dạng Markdown ghi: chạy bằng Collection Runner với data/huy-chuyen.json; huỷ sau 1 phút được miễn phí, sau 3 phút thu 10.000 đ (R-CANCEL-01/02/03/06); đây là supporting check, không thay phiên UAT và chữ ký nghiệp vụ." width="2000" height="1125" loading="lazy"></div>
<figcaption><b>Hình 7 · Mô tả folder nói rõ đây là gì và không phải là gì.</b> Câu cuối: «Đây là supporting check, không thay phiên UAT và chữ ký nghiệp vụ.»</figcaption>
</figure>

Mô tả nằm ngay trong collection, nên người mở folder sau này đọc được cách chạy (tệp dữ liệu nào), tiêu chí nào được chạm tới và giới hạn của kết quả — trước khi kịp báo «UA-02 xanh». Nó hỗ trợ đúng điều mục 9.3 yêu cầu: supporting checks được báo tách riêng, không trộn vào số scenario PASS của phiên người dùng.

Các hình 2–7 là một supporting check bằng công cụ GUI, cùng loại bằng chứng với 4/4 Vitest ở 6.3: dữ liệu API đúng tiêu chí ở hai mốc. Chúng **không** phải phiên UAT — không có đại diện khách hay tài xế thao tác trên `/` và `/driver.html`, không ai được hỏi có hiểu phí và lý do không, không có người ký; bảng ở mục 4 vẫn ghi UA-02 «Phiên: NOT RUN». Chỗ dùng hợp lý của nó là **vật liệu chuẩn bị phiên**: QA chạy folder trước buổi UAT để chắc môi trường và móc đồng hồ hoạt động, và cho business owner xem trước dữ liệu mong đợi của hai lượt huỷ. Hình 2 là lượt chạy minh hoạ ngày 23/11/2026, các hình 3–7 chụp ngày 24/11/2026 (CurlStudio 0.2.1 qua `pnpm tauri dev`, Windows 11, máy chủ lab Node v22.18.0); không cộng vào «4/4» và không quy đổi thành tỷ lệ UAT.

## 7. Thực hành 2 — acceptance kit cho phiên người dùng

Đây là bộ mẫu dùng được ngay. Khởi động lab theo README của `examples/toi-noi-testing`, dùng trang khách `/` và trang tài xế `/driver.html`. QA chuẩn bị clock cho UA-02/03 bằng móc test có sẵn của lab; không bắt người dùng thao tác SQL hay canh biên 2 phút bằng đồng hồ tay. **Không bao giờ bật móc test trên production.**

### 7.1. Phiếu chuẩn bị

| Trường | Giá trị cần điền trước phiên |
|---|---|
| Build/commit, môi trường/backend | Chưa điền |
| Ngày, browser/version, URL | Chưa điền |
| Người quyết định và phạm vi thẩm quyền | Chưa chỉ định |
| Đại diện khách/tài xế/vận hành, QA | Chưa chỉ định |
| Phạm vi | Mẫu: ride, SG, bike, tiền mặt; review rủi ro ví |
| Ngoài phạm vi | Food/parcel, provider thật, auth thật, toàn bộ thiết bị/thành phố |
| Dữ liệu | C001, D101, 5 km, không mưa, 09:00 VN; reset giữa scenario |
| Entry | Build cố định, smoke đạt, môi trường sẵn, tiêu chí đã review |
| Exit dự kiến | Evidence đủ, lỗi chặn xử lý, ngoại lệ có owner, người quyết định xác nhận |

### 7.2. Sáu kịch bản

**UA-01 — Đặt và hoàn thành chuyến tiền mặt.** Vai trò: khách + tài xế. Basis: R-QUOTE-04, R-IDEM-02, R-STATE-02/06, R-PAY-01.

1. Khách lấy báo giá bike/SG/5 km, xác nhận thấy 35.000 đ và chọn tiền mặt.
2. Đặt chuyến, bấm lại theo hành vi tự nhiên của UI; lưu trip ID. Tài xế mở danh sách, nhận với ETA 5 phút.
3. Tài xế lần lượt tới đón, đón khách, bắt đầu, hoàn thành. Khách quan sát kết quả.
4. Đối chiếu: một chuyến, giá giữ nguyên, completed; QA bổ sung bằng chứng không gọi ví và tài xế online.

Cuối scenario, hỏi khách giải thích trạng thái cuối và số tiền. Kết quả phiên: **NOT RUN**.

**UA-02 — Huỷ trước và đúng mốc tính phí.** Vai trò: khách + tài xế. Basis: R-CANCEL-01/02/03/06/07. Hai chuyến độc lập, reset giữa lượt; QA đặt clock 1 phút hoặc đúng 2 phút sau khi tài xế nhận. Khách huỷ, quan sát phí/lý do: lượt 1 miễn phí, lượt 2 phí 10.000 đ. Kiểm trạng thái cancelled, tài xế tiếp tục nhận việc được. Ghi khách có hiểu phí và lý do không; UI dùng chữ khác mã API (`grace_period`, `charged`) là bình thường nếu ý nghĩa đúng. Không đòi màn preview chưa có trong SPEC. Kết quả: **NOT RUN**.

**UA-03 — Báo giá hết hạn.** Vai trò: khách. Basis: R-QUOTE-01/03. Lấy báo giá, QA đưa clock tới đúng hạn 5 phút, khách đặt. Mong đợi: hệ thống trả lỗi hết hạn, không tạo chuyến; khách tự lấy báo giá mới và đặt lại. Phần tự động chỉ kiểm việc từ chối báo giá cũ; bước khách *tự hiểu và báo giá lại* phải quan sát riêng. Kết quả: **NOT RUN**.

**UA-04 — Khoản thu chưa xác nhận.** Vai trò: nghiệp vụ thanh toán/vận hành. Basis: R-PAY-04/06 và [SYS-14](/articles/system-testing-recovery-test) RC-03 (ví mất phản hồi cả ba lần: ví đã trừ một lần nhưng DB ghi `failed`). Xem bằng chứng mô phỏng, mô tả cách tra cứu, đối soát, tránh thu trùng và cập nhật có kiểm toán. Kết quả: **BLOCKED cho xử lý end-to-end** — lab chưa có console, ledger thật hay worker reconciliation. Có thể review bằng chứng kỹ thuật, nhưng không đánh dấu workflow xử lý tiền là đạt. Owner: **chưa chỉ định**.

**UA-05 — Hiểu trạng thái và tự tiếp tục công việc.** Vai trò: khách + tài xế. Basis: R-UI-01/02/03/04. Quan sát song song UA-01/02/03; yêu cầu người dùng giải thích giá, trạng thái, lỗi và bước tiếp theo; ghi chỗ cần hỗ trợ. Đây là bằng chứng UAT về tác vụ, không thay nghiên cứu usability của [SYS-12](/articles/system-testing-usability-test). Kết quả: **NOT RUN**.

**UA-06 — Bàn giao xử lý tồn đọng.** Vai trò: vận hành + business owner. Tiêu chí đề xuất: có owner, kênh tiếp nhận, mức ưu tiên, bằng chứng và cách đóng incident. Review một trường hợp `failed`, đối chiếu giới hạn recovery. Lab không có support console nên thực hiện bằng tabletop nếu được thống nhất và ghi rõ hình thức. Kết quả: **NOT RUN**, chưa có owner/runbook được duyệt.

### 7.3. Worksheet cho mỗi lượt

Sao chép worksheet này cho mỗi lượt chạy. Khách và tài xế làm cùng một chuyến nên ghi trip ID để nối bằng chứng hai phía. Chụp ảnh bằng dữ liệu giả, không đưa thông tin cá nhân vào kho; lưu phiên bản trình duyệt và các bước QA đã trợ giúp.

```text
Run ID / Scenario ID / build:
Người thực hiện / vai trò / thời điểm:
Dữ liệu và trip/quote ID:
Expected theo tiêu chí:
Actual quan sát được:
Evidence (ảnh/log/test report; dữ liệu giả):
Mức trợ giúp từ QA:
Kết quả: PASS | FAIL | BLOCKED | NOT RUN
Defect hoặc yêu cầu thay đổi:
Người xử lý / hạn / retest build:
```

**PASS** chỉ khi đã kiểm đủ tiêu chí của scenario. **BLOCKED** nêu nguyên nhân không thể kiểm. **NOT RUN** là chưa thực hiện. Ghi kết quả thực tế cụ thể («khách thấy *Phí huỷ 10.000 đ*, nói rằng "vì tài xế đã nhận lâu rồi"»), không ghi «như mong đợi».

Sau mỗi scenario, hỏi người thực hiện mô tả kết quả và bước tiếp theo. Nếu API đúng nhưng thông báo khiến khách nghĩ mình bị thu trùng, ghi thành vấn đề UI/diễn giải riêng. Khi tiêu chí chưa nói gì về điều này, business phải làm rõ trước khi biến nó thành tiêu chí bắt buộc mới, và giữ lịch sử thay đổi tiêu chí.

### 7.4. Risk register cần review

| ID | Bằng chứng hiện có | Cần bổ sung | Owner/quyết định |
|---|---|---|---|
| UAR-01 | SYS-14: ví đã thu nhưng DB `failed` | Quy trình/worker đối soát, kiểm tránh thu trùng | Chưa chỉ định |
| UAR-02 | SYS-12 chưa có user study | Quan sát người dùng thật theo phạm vi | Chưa chỉ định |
| UAR-03 | SYS-14 restore probe, chưa có RTO/RPO ứng dụng | Diễn tập dữ liệu/hành trình thật trên staging | Chưa chỉ định |
| UAR-04 | Lab dùng `x-user`, ví giả | Kiểm môi trường tích hợp/auth/provider phù hợp trước production | Chưa chỉ định |

## 8. Triage: defect, thay đổi hay giới hạn môi trường?

| Tình huống | Phân loại ban đầu | Việc tiếp theo |
|---|---|---|
| Đúng 2 phút nhưng miễn phí | Sai tiêu chí hiện hành (R-CANCEL-02) | Bug, tái hiện, sửa, retest |
| Khách muốn màn hình preview phí | Yêu cầu mới nếu chưa có trong basis | Đánh giá phạm vi, cập nhật tiêu chí |
| Không gọi được ví thật vì lab dùng fake | Giới hạn môi trường | Chuyển scenario sang môi trường phù hợp; không ghi PASS |
| Mất phản hồi, ví thu nhưng DB `failed` | Rủi ro đã biết (SYS-14) | Yêu cầu quy trình đối soát/owner; xem xét phạm vi chấp nhận |
| QA phải chỉ nút mới tiếp tục được | Bằng chứng usability | Ghi mức hỗ trợ và tác động nghiệp vụ, đối chiếu SYS-12 |

Hai nguyên tắc đi kèm bảng:

- **Severity khác priority.** Severity đo tác động; priority quyết định thứ tự xử lý. Lỗi thu sai phí ít người gặp vẫn có severity cao.
- **Workaround không đóng bug.** Ghi chi phí workaround, ai thực hiện, điều kiện áp dụng, rủi ro còn lại. Retest bản sửa và regression luồng liên quan **trên build mới**; bằng chứng của build cũ chỉ còn giá trị lịch sử — xem cách chọn phạm vi ở [SYS-06 · Regression Test](/articles/system-testing-regression-test).

## 9. Quyết định và chỉ số

### 9.1. Ba lựa chọn

**Chấp nhận**, **chấp nhận có điều kiện**, **chưa chấp nhận**. Chấp nhận có điều kiện phải ghi phạm vi, ngoại lệ, owner, hạn xử lý và **điều kiện làm quyết định hết hiệu lực** (ví dụ: «hết hiệu lực nếu chưa có quy trình đối soát UAR-01 trước khi bật VíXanh»). Đây không phải cách đổi lỗi chặn thành PASS. Gate cụ thể do dự án thống nhất; đừng đặt một tỷ lệ pass để thay quyết định.

### 9.2. Biên bản — để trống có chủ đích

```text
Build và phạm vi nghiệm thu:
Các tiêu chí bắt buộc / evidence đã review:
Số scenario PASS / FAIL / BLOCKED / NOT RUN:
Quyết định: CHƯA CÓ QUYẾT ĐỊNH
Các lựa chọn khi review: chấp nhận | chấp nhận có điều kiện | chưa chấp nhận
Ngoại lệ, ảnh hưởng, workaround:
Owner / hạn xử lý / điều kiện cần retest:
Điều kiện làm quyết định hết hiệu lực:
Người có thẩm quyền / vai trò / ngày xác nhận: CHƯA ĐIỀN
```

### 9.3. Chỉ số nên báo cáo

- Tiêu chí đã có bằng chứng / tổng tiêu chí trong phạm vi.
- Scenario PASS / FAIL / BLOCKED / NOT RUN — tách riêng supporting checks.
- Mức bao phủ từng vai trò (khách, tài xế, vận hành).
- Số rủi ro chưa có owner; số retest còn thiếu.

Hiện trạng theo các chỉ số này: 6 scenario, 0 PASS từ phiên người dùng, 5 NOT RUN, 1 BLOCKED; 4 supporting checks pass, báo như bằng chứng riêng; 4/4 rủi ro trong register chưa có owner. **Gate người dùng chưa đạt và chưa có sign-off.** Đề xuất gate cho bài tập: đủ bằng chứng cho các scenario bắt buộc; không còn lỗi chặn theo phân loại đã thống nhất; mọi ngoại lệ có owner và quyết định; người được giao thẩm quyền xác nhận build/phạm vi.

## 10. Mở rộng nâng cao

**ATDD (Acceptance Test-Driven Development).** Thống nhất ví dụ trước khi triển khai, dùng chính ví dụ đó làm acceptance check. Cái bẫy: fixture kỹ thuật (tên bảng, mã lỗi) dần thay ngôn ngữ nghiệp vụ. UA-02 nên viết «khách huỷ đúng 2 phút sau khi tài xế nhận thì trả 10.000 đ»; `feeReason: 'charged'` là chi tiết của supporting check.

**Risk-based UAT.** Ưu tiên tiền, huỷ, quyền truy cập, xử lý gián đoạn và vai trò ít được kiểm (ở đây là vận hành). Chọn biến thể dữ liệu bằng bảng quyết định như [SYS-03](/articles/system-testing-functional-test); không nhân mọi tổ hợp thành phố × xe × phương thức nếu không có lý do rủi ro, và ghi rõ nhóm bị loại khỏi phạm vi.

**Nghiệm thu nhiều bên có mục tiêu xung đột.** Hoàn thành chuyến dù ví lỗi (R-PAY-06) giúp tài xế làm tiếp nhưng tạo khoản cần đối soát cho vận hành. Biên bản phải ghi ai sở hữu quyết định từng tiêu chí; không lấy lợi ích của bên này che rủi ro của bên kia.

**Continuous delivery.** Pipeline có thể tự thu bằng chứng và áp dụng cơ chế phê duyệt đã được tổ chức uỷ quyền (ví dụ: owner phê duyệt trên một change request gắn build ID), miễn truy được build → tiêu chí → bằng chứng → quyết định. Pipeline xanh không tự sinh ra thẩm quyền nghiệm thu.

**Release dossier.** Nối bằng chứng [security](/articles/system-testing-security-pentest), [performance](/articles/system-testing-performance-test), [compatibility](/articles/system-testing-compatibility-test), [usability](/articles/system-testing-usability-test), [integrity](/articles/system-testing-data-integrity-test) và [recovery](/articles/system-testing-recovery-test) với phạm vi UAT. Không gom mọi kết quả lịch sử thành một «chứng nhận hiện tại»: mỗi dòng ghi build, ngày, môi trường, và thay đổi nào buộc phải chạy lại. [SYS-16](/articles/system-testing-deployment-test) kiểm chính bản triển khai đó.

## 11. Anti-pattern và giới hạn bằng chứng

- **«Pipeline xanh = UAT đạt.»** Tự động hoá được đổi tên thành nghiệm thu, người có thẩm quyền biến mất khỏi quy trình.
- **UAT là demo.** Người làm sản phẩm bấm, người dùng gật đầu; không có kết quả thực tế do người dùng tạo ra.
- **Kịch bản từng click.** Người dùng đọc theo kịch bản, nên phiên không phát hiện chỗ họ không hiểu.
- **Sửa dữ liệu giữa phiên** để luồng chạy tiếp, rồi ghi PASS.
- **Điền chữ ký hộ** «cho đủ hồ sơ», hoặc ký trên build khác build đã kiểm.
- **Chấp nhận có điều kiện không có owner và hạn** — thực chất là chấp nhận vô điều kiện kèm một ghi chú.

Đánh đổi chi phí: phiên UAT liên vai trò tốn thời gian của những người đắt nhất và khó lên lịch nhất. Vì vậy cần supporting checks và smoke để phiên không phí vào lỗi kỹ thuật, và risk-based selection để không bắt business duyệt từng tổ hợp. Nhưng bỏ hẳn phiên người dùng thì không còn là UAT.

Giới hạn của bài đã nêu ở hai callout và mục 4; thêm: suite chỉ chạy API/PGlite một kết nối, và bằng chứng SYS-12/14 được tham chiếu như kết quả lịch sử có giới hạn, không chạy lại trong bài này.

## 12. Bài tập và câu hỏi tự kiểm

### Bài tập

**Bài 1 (cơ bản).** Viết tiêu chí cho yêu cầu «khách nhìn thấy lý do phí huỷ».

<details>
<summary><b>Gợi ý</b></summary>
<p>Tách hai tiêu chí: (a) số tiền và lý do đúng R-CANCEL-03/06 — supporting check làm được; (b) khách hiểu được — chỉ quan sát trong phiên. Chỉ rõ màn hình (trang <code>/</code> sau thao tác huỷ), thời điểm (ngay sau khi huỷ, không phải trước), bằng chứng (ảnh + câu khách tự giải thích). Nếu business muốn hiển thị <i>trước</i> khi huỷ, đó là yêu cầu mới, phải bổ sung vào basis.</p>
</details>

**Bài 2 (trung bình).** Mở rộng UA-01 sang VíXanh.

<details>
<summary><b>Gợi ý</b></summary>
<p>Đổi <code>paymentMethod</code> thành <code>vixanh</code>; supporting check kiểm số tiền thực thu tại ví giả (một lần, đúng 35.000, khoá idempotency gắn trip). Ghi rõ chưa có provider thật. Thêm tình huống mất phản hồi như SYS-14 RC-03 và yêu cầu owner đối soát — nếu chưa có owner, scenario BLOCKED chứ không PASS.</p>
</details>

**Bài 3 (trung bình).** Điền biên bản «chưa chấp nhận» cho phạm vi ví điện tử.

<details>
<summary><b>Gợi ý</b></summary>
<p>Phạm vi: VíXanh trên ride SG. Viện dẫn SYS-14 RC-03 và UAR-01; nêu khoản đã thu nhưng hệ thống ghi failed chưa có quy trình xử lý. Ghi bằng chứng cần có để đổi quyết định (quy trình đối soát, retest trên build mới). Để trống ô người có thẩm quyền — bài tập không được giả chữ ký.</p>
</details>

**Bài 4 (nâng cao).** Build mới chỉ sửa giao diện hiển thị phí huỷ. Cần retest gì?

<details>
<summary><b>Gợi ý</b></summary>
<p>UA-02 và UA-05 trên UI với người dùng; supporting check UA-02 xác nhận số tiền không đổi; kiểm accessibility liên quan (R-UI-03: thông báo lỗi <code>role="alert"</code>, vùng <code>aria-live</code>) và định dạng tiền (R-UI-04). Làm impact analysis trước khi chọn regression; bằng chứng UA-02 của build cũ không còn dùng cho quyết định.</p>
</details>

**Bài 5 (nâng cao).** Tạo traceability để mở rộng UAT sang đồ ăn/bưu kiện.

<details>
<summary><b>Gợi ý</b></summary>
<p>Lấy đúng các mã R-FOOD và R-PARCEL có trong SPEC; thêm vai trò mới (nhà hàng, người nhận nếu có trong basis), dữ liệu và oracle tiền tính độc lập (ví dụ phụ phí khối lượng R-PARCEL-03). Bốn test hiện tại không chứng minh gì cho phạm vi mới; phiếu chuẩn bị phải chuyển food/parcel từ "ngoài phạm vi" vào "phạm vi".</p>
</details>

### Câu hỏi tự kiểm

**Câu 1.** Supporting checks 4/4 pass, sáu scenario người dùng chưa chạy. Báo cáo nào đúng?

- a) UAT đạt 100%
- b) UAT đạt 4/10
- c) 4/4 supporting checks pass; phiên người dùng 0 PASS, 5 NOT RUN, 1 BLOCKED; chưa có quyết định
- d) UAT không cần thiết vì supporting checks đã pass

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>c)</b>. Hai nguồn bằng chứng khác bản chất phải báo riêng. a) và b) quy đổi automation thành tỷ lệ UAT; d) bỏ hẳn câu hỏi của UAT.</p>
</details>

**Câu 2.** Trong phiên, khách huỷ đúng 2 phút sau khi tài xế nhận và không bị tính phí. Phân loại?

- a) Yêu cầu thay đổi
- b) Defect so với tiêu chí hiện hành
- c) Giới hạn môi trường
- d) Vấn đề usability

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>b)</b>. R-CANCEL-02 nói đúng 2 phút 0 giây là «từ 2 phút trở lên» ⇒ có phí. Đây là sai lệch so với basis đã thống nhất: tái hiện, sửa, retest trên build mới.</p>
</details>

**Câu 3.** UA-04 nên ghi kết quả gì khi lab không có công cụ đối soát?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p><b>BLOCKED</b> cho xử lý end-to-end, kèm nguyên nhân. Không phải FAIL (chưa kiểm được hành vi), không phải PASS (review bằng chứng kỹ thuật không chứng minh quy trình xử lý tiền chạy được), cũng không phải NOT RUN: điều kiện tiên quyết được biết là thiếu ngay khi chuẩn bị, nên ghi BLOCKED kèm nguyên nhân để không ai xếp lịch chạy nó như một scenario bình thường.</p>
</details>

**Câu 4.** Vì sao UA-02 với chuyến tiền mặt assert <code>payments.calls</code> bằng 0 lại đúng?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>Theo R-PAY-05 và R-CANCEL-07, phí huỷ của chuyến <code>cash</code> được ghi nhận vào chuyến (<code>cancellationFee</code>), chỉ chuyến <code>vixanh</code> mới thu qua ví. Assertion còn đi kèm kiểm <code>cancellationFee</code> đã lưu trong repository, nên không thể pass nếu phí bị bỏ quên.</p>
</details>

**Câu 5.** «Chấp nhận có điều kiện» khác «chấp nhận kèm ghi chú» ở điểm nào?

<details>
<summary><b>Đáp án và giải thích</b></summary>
<p>Chấp nhận có điều kiện phải có phạm vi, ngoại lệ, owner, hạn xử lý và điều kiện làm quyết định hết hiệu lực, do người có thẩm quyền ký trên đúng build. Thiếu owner/hạn/điều kiện vô hiệu thì thực chất là chấp nhận vô điều kiện.</p>
</details>

## 13. Tài liệu tham khảo

Tra cứu ngày 13/11/2026.

- ISTQB Glossary — *acceptance testing*: https://glossary.istqb.org/en_US/term/acceptance-testing/1
- ISTQB, *Certified Tester Acceptance Testing (CT-AcT)*: https://istqb.org/certifications/certified-tester-acceptance-testing/ — syllabus v1.0 (2019): https://www.istqb.org/wp-content/uploads/2024/11/ISTQB-CT-AcT_Syllabus_v1.0_2019.pdf
- Lab và bộ test của bài: https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing/articles/system-testing-uat — `SPEC.md` (R-QUOTE, R-IDEM, R-STATE, R-CANCEL, R-PAY, R-UI) là test basis trực tiếp.
- Nguồn nội bộ: báo cáo SYS-12 (usability), SYS-13 (data integrity), SYS-14 (recovery).
- Các mẫu worksheet, risk register, gate và biên bản trong bài là thiết kế thực hành cho lab, không phải biểu mẫu bắt buộc của ISTQB.
- CurlStudio 0.2.1 — API client và load test (mã nguồn, tra cứu 23/11/2026): https://github.com/rickymta/curl-studio · collection minh hoạ: `examples/toi-noi-testing/curlstudio/`
- Bài liên quan: [Acceptance Testing tổng quan](/articles/acceptance-testing-tong-quan) · [Acceptance criteria Given–When–Then](/articles/ba-acceptance-criteria-given-when-then) · [Thiết kế UAT scenario](/articles/ba-thiet-ke-uat-scenario) · [UAT: vai trò người dùng nghiệp vụ](/articles/ba-uat-vai-tro-nguoi-dung-nghiep-vu) · [SYS-14 · Recovery Test](/articles/system-testing-recovery-test)

Bài tiếp theo: [SYS-16 · Deployment Test](/articles/system-testing-deployment-test).
