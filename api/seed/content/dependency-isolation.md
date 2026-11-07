---
slug: dependency-isolation
title: "Dependency Isolation: kiểm giờ cao điểm và timeout mà không chờ thật"
summary: "Tiêm Clock và WeatherService vào use case Tới Nơi, kiểm biên giờ Việt Nam, hạn báo giá, 5 phút tìm tài xế và retry thanh toán bằng fake timers; phân biệt thời gian nghiệp vụ với scheduler."
category: testing
level: advanced
tags: [testing, test-058, unit-testing, dependency-injection, ports-adapters, fake-timers, clock, vitest]
language: vi
---

Một test chỉ trượt lúc 13:00 hoặc khi chạy trên máy CI ở UTC thường đang phụ thuộc vào một điều kiện ngoài ý muốn. Với Tới Nơi, đồng hồ quyết định giờ cao điểm, hạn báo giá và thời gian tìm tài xế; thời tiết quyết định phụ phí mưa. Muốn tái hiện một tình huống, ta phải điều khiển những đầu vào đó thay vì đợi thế giới thật đi qua đúng thời điểm.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-058</span>
<table><tbody>
<tr><th scope="row">Series</th><td>S06 · Unit Testing &amp; Component Testing</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L3 · Advanced</td></tr>
<tr><th scope="row">ISTQB Mapping</th><td>Mở rộng thực hành kỹ thuật; không gán LO riêng cho dependency injection hoặc fake timers</td></tr>
<tr><th scope="row">Test Level / Technique</th><td>Component và cộng tác trong bộ nhớ / Dependency isolation, clock abstraction</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/test-doubles-dummy-stub-fake-mock-spy">TEST-057 · Test Doubles</a></td></tr>
<tr><th scope="row">Thực hành</th><td><code>examples/toi-noi-testing/articles/dependency-isolation/</code></td></tr>
</tbody></table>
</div>

## 1. Giới thiệu

Dependency isolation là kiểm phần hành vi quan tâm trong khi kiểm soát những cộng tác viên có thể khiến test chậm, không ổn định hoặc khó tái hiện. Không phải mọi phụ thuộc đều cần thay thế: hàm tính cước và chuyển trạng thái thuần vẫn có thể chạy thật; mạng, đồng hồ và kho lưu trữ được chọn theo ranh giới test.

Trong bài trước, ta thay cổng thanh toán. Ở đây ta đi theo luồng báo giá và thời gian: từ một thời điểm UTC xác định đến cờ cao điểm theo giờ Việt Nam, từ một báo giá có hạn đến chuyến đang tìm tài xế, rồi sang timeout của cổng ví.

## 2. Nền tảng: port, adapter và điểm lắp ráp

**Port** mô tả năng lực use case cần; **adapter** cài đặt năng lực đó. **Dependency injection** đưa cài đặt vào từ bên ngoài thay vì để use case tự chọn phụ thuộc. Tách việc lắp ráp khỏi việc sử dụng là ý tưởng cốt lõi; không bắt buộc có DI container.

Trong lab, các port nhỏ có dạng:

```ts
interface Clock {
  now(): Date;
}

interface WeatherService {
  isRaining(cityCode: CityCode, at: Date): Promise<boolean>;
}
```

Use case nhận `AppDeps` qua tham số. `createMemorySystem({ clock })` ở `infra/compose.ts` lắp các repository bộ nhớ, ví giả, thời tiết giả và đồng hồ được truyền vào. Test có thể thay riêng `deps.weather`, giữ phần còn lại chạy thật.

Sơ đồ phụ thuộc của bài:

```text
test → createQuote(deps, actor, input)
          ├─ clock.now()           → FixedClock
          ├─ weather.isRaining()   → stub / FakeWeather
          ├─ pricing domain       → mã thật
          └─ quotes.save()         → repository bộ nhớ
```

**Seam** là điểm cho phép thay một cộng tác viên mà không sửa logic đang kiểm. `deps.clock` và `deps.weather` là hai seam đã có; không cần patch `Date` toàn cục để kiểm báo giá.

### Hai loại thời gian khác nhau

| Câu hỏi | Cơ chế trong lab | Cách kiểm |
|---|---|---|
| Bây giờ là thời điểm nào? | `Clock.now()` | `FixedClock.set/advanceMs` |
| Callback timeout đã đến lượt chạy chưa? | `setTimeout` trong `withTimeout` | Vitest fake timers |

Đẩy `FixedClock` thêm 5 phút không tự chạy `setTimeout` hay bộ quét định kỳ. Ngược lại, đẩy fake timers không thay đổi trạng thái riêng bên trong `FixedClock`. Trộn hai cơ chế mà không nêu rõ sẽ tạo test chạy nhanh nhưng kiểm sai điều kiện.

## 3. Tình huống kiểm thử tại Tới Nơi

| Yêu cầu | Biên hoặc rủi ro |
|---|---|
| R-GEN-02 | Dùng UTC+7 dù host ở múi giờ khác |
| R-FARE-04 | Cao điểm [11:00; 13:00) và [17:00; 19:00) |
| R-GEN-05 / R-FARE-05 | Thời tiết tại thời điểm báo giá quyết định phụ phí |
| R-QUOTE-01 | Báo giá còn hạn khi now < expiresAt; bằng mốc là hết hạn |
| R-STATE-05 | Searching đủ 5 phút bị huỷ; phí 0, lý do no_driver |
| R-STATE-04 | Tài xế huỷ đặt lại searchingSince |
| R-PAY-04 | Mỗi lần chờ ví tối đa 3 giây; retry cùng khoá |

Ta dùng xe máy 2 km: thường không mưa trả 21.000 đ; cao điểm không mưa trả 27.000 đ; cao điểm có mưa trả 35.000 đ. Oracle tiền giữ nguyên từ hai bài trước để tập trung vào việc suy ra đúng điều kiện.

Lab không có GPS hay MapViet thật. Không thêm ví dụ “mock GPS” rồi ngụ ý đã kiểm tích hợp bản đồ; seam đang kiểm ở đây là Clock và WeatherService.

## 4. Thiết kế test

### 4.1. Các mốc UTC có oracle giờ Việt Nam độc lập

| UTC ngày 07/11/2026 | Giờ Việt Nam | Peak | Tổng |
|---|---|---|---:|
| 03:59:59Z | 10:59:59 | Không | 21.000 |
| 04:00:00Z | 11:00:00 | Có | 27.000 |
| 05:59:59Z | 12:59:59 | Có | 27.000 |
| 06:00:00Z | 13:00:00 | Không | 21.000 |
| 09:59:59Z | 16:59:59 | Không | 21.000 |
| 10:00:00Z | 17:00:00 | Có | 27.000 |
| 11:59:59Z | 18:59:59 | Có | 27.000 |
| 12:00:00Z | 19:00:00 | Không | 21.000 |

Dữ liệu test dùng chuỗi ISO có `Z`, không dùng chuỗi ngày giờ thiếu offset. Expected được ghi tường minh; không gọi chính `isPeakTime` để tính expected cho test `createQuote`, vì như vậy lỗi xác định giờ có thể xuất hiện ở cả hai phía.

### 4.2. Hạn báo giá và hạn tìm tài xế là hai đồng hồ nghiệp vụ

Cả hai mặc định là 5 phút nhưng bắt đầu ở mốc khác nhau. Hạn báo giá tính từ `createdAt`; tìm tài xế tính từ `searchingSince`. Khi tài xế huỷ và hệ thống tìm lại, `searchingSince` phải được đặt lại, còn tuổi chuyến vẫn tăng.

Thiết kế ba nhóm:

- Báo giá ở 299.999 ms tạo chuyến được; đúng 300.000 ms bị `QUOTE_EXPIRED`. Mỗi ca tạo fixture riêng để không bị “báo giá đã dùng” che lỗi hết hạn.
- Chuyến searching: quét ở 299.999 ms không huỷ; thêm 1 ms rồi quét phải huỷ; quét lại không huỷ thêm lần nữa.
- Chuyến đã accepted không thuộc tập cần huỷ; chuyến được tìm lại sau khi tài xế huỷ phải tính đủ 5 phút từ mốc mới.

### 4.3. Lỗi phụ thuộc cần một kỳ vọng rõ ràng

Ca thời tiết thành công kiểm tham số port là thành phố `SG` và thời điểm của clock. Ca thời tiết lỗi kiểm code hiện tại truyền lỗi ra và không lưu báo giá. Đây là **characterization test của hành vi hiện tại**, không phải tự đặt một quy tắc fallback mới trong SPEC. Nếu sản phẩm muốn lấy thời tiết cache khi lỗi, cần quyết định yêu cầu rồi cập nhật test tương ứng.

## 5. Thực hành

Từ `examples/toi-noi-testing`, sau `npm ci`:

```bash
npx vitest run articles/dependency-isolation
```

### 5.1. Tiêm đồng hồ và weather stub

```ts
const clock = new FixedClock(new Date('2026-11-07T04:00:00Z'));
const { deps } = createMemorySystem({ clock });
const isRaining = vi.fn<WeatherService['isRaining']>()
  .mockResolvedValue(true);
deps.weather = { isRaining };

const quote = await createQuote(deps, customer, {
  kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 2,
});

expect(isRaining).toHaveBeenCalledWith('SG', clock.now());
expect(quote.details).toMatchObject({ peak: true, raining: true, total: 35_000 });
```

Bản đầy đủ có import, actor và fixture trong `clock-weather.test.ts`. `FixedClock.now()` trả bản sao `Date`, nên code nhận kết quả không vô tình sửa được thời điểm giữ trong clock bằng `setTime`.

### 5.2. Nhảy tới biên thay vì sleep

```ts
clock.advanceMs(299_999);
expect(await expireStaleSearches(deps)).toBe(0);
clock.advanceMs(1);
expect(await expireStaleSearches(deps)).toBe(1);
```

Sau đó đọc lại chuyến, kiểm `cancelled`, `no_driver`, `cancellationFee: 0`, `paymentStatus: none`, và không có lời gọi ví. Số chuyến huỷ là một assertion cần thiết nhưng chưa đủ: nếu hệ thống huỷ đúng số lượng với sai phí, người dùng vẫn bị ảnh hưởng.

Test gọi `expireStaleSearches` tường minh. Nó chứng minh điều kiện xử lý khi bộ quét chạy, không chứng minh server thật quét đúng chu kỳ 15 giây. Vì quét theo chu kỳ, chuyến ngoài đời cũng không nhất thiết đổi trạng thái đúng ngay mili giây thứ 300.000.

### 5.3. Fake timers cho timeout 3 giây

`withTimeout` dùng `setTimeout`, vì vậy file `timers.test.ts` điều khiển scheduler:

```ts
vi.useFakeTimers();
let settled = false;
const outcome = withTimeout(new Promise<never>(() => {}), 3_000).then(
  () => { settled = true; return 'unexpected'; },
  error => { settled = true; return error; },
);

await vi.advanceTimersByTimeAsync(2_999);
expect(settled).toBe(false);
await vi.advanceTimersByTimeAsync(1);
expect(await outcome).toMatchObject({ name: 'PaymentTimeoutError' });
```

Gắn rejection handler trước khi đẩy timer để không tạo rejection chưa được xử lý. Dùng biến thể bất đồng bộ để các promise tiếp nối được xử lý cùng bước tiến timer. `afterEach` dọn timer rồi gọi `vi.useRealTimers()`; không để fake scheduler rò sang test khác.

### 5.4. Đã thu tiền nhưng mất phản hồi

Ca thứ hai cấu hình `fake.failNext('lost_response')`: fake ghi khoản thu nhưng không trả lời lần đầu. Sau khi đẩy 3.000 ms, `chargeWithRetry` thử lại; lần này fake nhận cùng khoá và trả khoản đã có. Test kiểm `attempts: 2`, hai khoá giống nhau, tổng tiền chỉ 21.000 và không còn timer chờ.

Tình huống này khác `timeout`: trong chế độ timeout của fake, chưa thu tiền. Nếu chỉ test timeout trước thu, ta chưa kiểm được nhánh mơ hồ “không có phản hồi nhưng có thể đã thành công”.

Bộ thực hành có **17 ca**: 15 ca clock/weather và 2 ca scheduler. Kiểm chứng ngày 07/11/2026 với Vitest 3.2.7: cả 17 ca đạt khi chạy tiến trình với `TZ=UTC` và với `TZ=America/Los_Angeles`, không cần chờ thật 5 phút hay gọi mạng.

## 6. Điểm nâng cao và giới hạn

**Timeout không huỷ tác vụ gốc.** `Promise.race` chỉ chọn kết quả đến trước. Nó không tự huỷ request tới nhà cung cấp. Trong hệ thật, request đầu có thể vẫn hoàn thành sau khi ứng dụng bắt đầu retry; idempotency phải là hợp đồng với bên thu tiền, không chỉ là một assertion trong test.

**Múi giờ phải được kiểm ở tiến trình mới.** Chạy cùng bộ test với `TZ=UTC` và một múi giờ khác giúp phát hiện việc dùng giờ local ngoài ý muốn. Trên PowerShell, đặt `$env:TZ` trước khi gọi runner và khôi phục trong `finally`; README của bài có lệnh đầy đủ. Thay TZ giữa chừng trong một test dễ để cache hoặc phần khởi tạo trước đó làm kết quả khó giải thích.

**Đồng hồ đứng yên không mô phỏng toàn bộ thời gian.** Nó không kiểm clock skew giữa máy, bước nhảy đồng hồ hệ điều hành hay độ trễ mạng. Bài dùng một tiến trình và một nguồn thời gian nhất quán để kiểm quy tắc. Các rủi ro phân tán cần môi trường khác.

**Repository bộ nhớ không thay bằng chứng giao dịch.** Ca accepted không bị bộ quét huỷ là ca tuần tự; không chứng minh đúng khi tài xế nhận chuyến đồng thời với bộ quét trên nhiều kết nối DB. Cần kiểm nguyên tử và tranh chấp ở tầng lưu trữ thật.

**Giữ quy tắc thuần chạy thật.** Mock luôn `isPeakTime` thành true có thể hữu ích nếu chỉ muốn kiểm chỗ khác, nhưng sẽ vô hiệu mục tiêu kiểm giờ Việt Nam của bài này. Chọn seam bên ngoài quy tắc cần xác minh.

## 7. Bài tập và tự đánh giá

### Bài tập

1. Thêm ca báo giá lúc 10:59:59 và tạo chuyến lúc 11:00:00. Theo R-QUOTE-03, giá của chuyến nên theo báo giá cũ hay tính lại theo cao điểm?
2. Đổi ca `lost_response` thành ba lần `timeout`. Giữ cấu hình 3.000 ms, hai retry; xác định thời gian giả cần đẩy và outcome.
3. Thêm ca kiểm thời tiết lỗi mà chỉ assert “promise rejects”. Nó còn bỏ sót điều gì so với bài?
4. Vẽ ranh giới kiểm thử nếu muốn chứng minh scheduler server gọi bộ quét mỗi 15 giây. Thành phần nào phải được đưa vào SUT?

<details><summary>Gợi ý đáp án</summary><p>1. Giữ giá báo giá còn hiệu lực; không tự tính lại khi tạo chuyến. 2. Tổng ba lần chờ, 9.000 ms giả; failed, attempts = 3, failure = timeout, chưa thu tiền. 3. Có thể đã lưu báo giá sai trước khi reject; cần assertion không save và kiểm đúng lỗi. 4. Phải bao gồm phần lập lịch trong server cùng điểm quan sát lời gọi bộ quét; bài hiện tại chỉ gọi use case trực tiếp.</p></details>

**Câu 1.** Đẩy `FixedClock` thêm 5 phút nhưng chuyến vẫn searching. Giải thích phù hợp nhất?

- A. FixedClock luôn sai.
- B. Thay thời điểm không tự gọi bộ quét; test cần kích hoạt bước xử lý.
- C. Phải sleep thật thêm 5 phút.
- D. Dùng mock toàn bộ repository sẽ tự huỷ chuyến.

<details><summary>Đáp án: B</summary><p>Clock cung cấp dữ liệu thời gian; bộ quét là một hành động riêng. Điều khiển đầu vào và kích hoạt SUT là hai bước khác nhau.</p></details>

**Câu 2.** Ca mất phản hồi dùng fake đạt. Kết luận nào đủ bằng chứng?

- A. Ví production chắc chắn không bao giờ thu trùng.
- B. Trong mô hình lab, use case retry cùng khoá và fake ghi một khoản thu.
- C. Promise.race đã huỷ request đầu tiên.
- D. Mọi race condition DB đã được kiểm.

<details><summary>Đáp án: B</summary><p>Các kết luận còn lại vượt phạm vi: ví thật, huỷ request và tranh chấp DB không được mô phỏng đầy đủ.</p></details>

Câu hỏi do bài viết tự biên soạn, không thuộc đề thi ISTQB chính thức.

## 8. Tài liệu tham khảo

- [Martin Fowler — Dependency Injection](https://martinfowler.com/articles/injection.html): tách cấu hình phụ thuộc khỏi phần sử dụng.
- [Vitest — vi utilities](https://vitest.dev/api/vi.html): fake timers, `advanceTimersByTimeAsync`, khôi phục timer thật.
- [Lab Tới Nơi](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing): SPEC R-GEN-02, R-FARE-04, R-QUOTE-01, R-STATE-04/05, R-PAY-04; `app/quote.ts`, `app/trips.ts`, `app/payment.ts`, `infra/clock.ts`.

Bài trước: [TEST-057 · Test Doubles](/articles/test-doubles-dummy-stub-fake-mock-spy). Bài tiếp theo trong kế hoạch: TEST-059 · Parameterized Testing.
