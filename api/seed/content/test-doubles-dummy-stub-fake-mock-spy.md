---
slug: test-doubles-dummy-stub-fake-mock-spy
title: "Test Doubles: chọn Dummy, Stub, Fake, Mock và Spy cho cổng thanh toán"
summary: "Thay cổng VíXanh bằng test double để kiểm hoàn thành chuyến, từ chối thanh toán, retry và idempotency; phân biệt kiểm trạng thái với kiểm tương tác, tránh mock chi tiết nội bộ."
category: testing
level: advanced
tags: [testing, test-057, unit-testing, test-doubles, mock, stub, fake, spy, vitest]
language: vi
---

Tới Nơi cần kiểm rằng chuyến hoàn thành sẽ thu đúng tiền qua VíXanh. Gọi ví thật cho mỗi lượt test vừa chậm vừa khiến kết quả phụ thuộc dịch vụ bên ngoài. Nhưng thay mọi thứ bằng hàm luôn trả thành công cũng không chứng minh được retry giữ nguyên khoá hay chuyến tiền mặt không gọi ví. Ta cần chọn một bản thay thế đúng với câu hỏi của từng test.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-057</span>
<table><tbody>
<tr><th scope="row">Series</th><td>S06 · Unit Testing &amp; Component Testing</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L3 · Advanced</td></tr>
<tr><th scope="row">ISTQB Mapping</th><td>Mở rộng thực hành kỹ thuật; không gán LO ISTQB riêng cho phân loại năm test double</td></tr>
<tr><th scope="row">Test Level / Technique</th><td>Component và cộng tác các thành phần trong bộ nhớ / Test doubles</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/unit-testing-fundamentals">TEST-055 · Unit Testing Fundamentals</a></td></tr>
<tr><th scope="row">Thực hành</th><td><code>examples/toi-noi-testing/articles/test-doubles-dummy-stub-fake-mock-spy/</code></td></tr>
</tbody></table>
</div>

## 1. Giới thiệu

Test double là tên chung cho đối tượng thay thế một cộng tác viên của phần mềm trong lúc kiểm thử. Ta giữ `driverStep` và logic thanh toán của lab chạy thật, thay cổng `PaymentGateway` ở ranh giới với ví.

Bài này trả lời ba câu hỏi: cần chuẩn bị phản hồi nào để đi vào nhánh đang quan tâm; cần quan sát trạng thái hay tương tác; bản thay thế có thể chứng minh đến đâu? Mục tiêu không phải dùng đủ mọi API mocking, mà là giữ test dễ hiểu khi mã được refactor.

## 2. Nền tảng và thuật ngữ

Phân loại dưới đây theo cách dùng phổ biến trong tài liệu *Mocks Aren't Stubs* của Martin Fowler; đây là nội dung thực hành mở rộng, không phải một bộ LO mới của ISTQB.

| Loại | Vai trò chính | Ví dụ tại ranh giới ví |
|---|---|---|
| Dummy | Lấp chỗ tham số, không dùng trong kịch bản | Cổng ví ở chuyến tiền mặt |
| Stub | Cung cấp phản hồi đã định | Luôn trả `declined` |
| Fake | Có cài đặt đơn giản và trạng thái hoạt động | Ví lưu khoản thu trong `Map`, khử trùng lặp theo khoá |
| Mock | Hỗ trợ kiểm kỳ vọng tương tác | Đòi gửi đúng khách, số tiền, khoá |
| Spy | Ghi nhận các lần gọi để kiểm tra | Quan sát `charge` trên fake đang hoạt động |

Các tên nói về **vai trò trong test**, không nhất thiết là năm lớp khác nhau. `vi.fn()` có thể cung cấp phản hồi như stub, rồi được dùng để kiểm tương tác. Trong Vitest, ta thường assert sau khi chạy; bài gọi cách đó là “mock-style verification”, không đồng nhất tuyệt đối với mock có cơ chế khai báo kỳ vọng trước của mọi framework.

**State verification** kiểm kết quả hoặc trạng thái: chuyến `completed`, thanh toán `failed`, số tiền fake đã thu. **Behavior verification** kiểm cộng tác: cổng được gọi với số tiền nào, khoá nào, bao nhiêu lần. Hai kiểu có thể cùng cần trong một test case.

## 3. Tình huống kiểm thử tại Tới Nơi

Một chuyến xe máy 2 km, 09:00 giờ Việt Nam, không mưa và không voucher có cước 21.000 đ. Khách chọn tiền mặt hoặc ví. Tài xế đi qua các trạng thái hợp lệ rồi hoàn thành chuyến.

| Yêu cầu | Quan sát cần có |
|---|---|
| R-PAY-02 | Chuyến tiền mặt không gọi ví |
| R-PAY-03 | Hoàn thành chuyến ví thu đúng `fare`, khoá `trip:<id>:fare` |
| R-PAY-04 | Lỗi tạm thời/hết giờ được thử lại tối đa hai lần; giữ khoá; declined không retry |
| R-PAY-06 | Từ chối thu tiền không ngăn chuyến hoàn thành |

Port nằm ở `src/app/ports.ts`:

```ts
interface PaymentGateway {
  charge(request: ChargeRequest): Promise<ChargeResult>;
  refund(request: RefundRequest): Promise<RefundResult>;
}
```

`driverStep` tạo yêu cầu thanh toán; `chargeWithRetry` thực hiện retry và timeout. Để kiểm khoá do use case tạo, phải gọi `driverStep`; chỉ truyền một khoá tự dựng vào `chargeWithRetry` không chứng minh được use case tạo khoá đúng.

## 4. Thiết kế test theo bằng chứng cần thu

| Ca | Đối tượng được kiểm | Double | Assertion chính |
|---|---|---|---|
| TD-01 | Hoàn thành chuyến cash | Dummy có guard | `completed`, `cash`, tài xế trở lại online |
| TD-02 | Hoàn thành chuyến ví bị từ chối | Stub declined | Trạng thái trả về và lưu đều `completed`/`failed` |
| TD-03 | Gửi lại cùng khoản | Fake có trạng thái | Hai lần gọi nhưng chỉ thu 21.000 đ |
| TD-04 | Lập yêu cầu thu cước | `vi.fn`, kiểm tương tác | Đúng khách, 21.000 đ, `trip:<id>:fare` |
| TD-05 | Quan sát cổng đang hoạt động | `vi.spyOn` trên fake | Có gọi cổng và trạng thái thu tiền thay đổi |
| TD-06 | Retry lỗi tạm thời | Stub theo thứ tự + bộ ghi lần gọi | Hai lần cùng khoá và số tiền |
| TD-07 | Dừng khi declined | Stub + kiểm tương tác | Chỉ một lần gọi, outcome thất bại |

TD-02 kiểm kết quả nghiệp vụ khi bị từ chối, TD-07 kiểm không retry. Hai test case trả lời hai câu hỏi khác nhau; không suy ra số lần gọi chỉ từ `paymentStatus` cuối.

Helper `ongoingTrip` tạo hệ thống bộ nhớ mới cho từng test, tạo báo giá, tạo chuyến, nhận chuyến rồi chạy `arrive`, `pickup`, `start`. Các use case và repository trong bộ nhớ đều chạy thật. Đây là test cộng tác trong tiến trình, không phải test HTTP hay unit cô lập mỗi hàm.

## 5. Thực hành

Từ `examples/toi-noi-testing`, sau khi cài dependency bằng `npm ci`:

```bash
npx vitest run articles/test-doubles-dummy-stub-fake-mock-spy
```

### 5.1. Dummy cho phần không được dùng

Trong ca tiền mặt, `PaymentGateway` vẫn là một trường bắt buộc của `AppDeps`, nhưng không nên được gọi:

```ts
const dummy: PaymentGateway = {
  charge: async () => { throw new Error('Chuyến cash không được gọi ví'); },
  refund: async () => { throw new Error('Không dùng refund'); },
};
```

Đối tượng này là dummy về vai trò, có thêm guard để lời gọi ngoài dự kiến làm test thất bại. Đừng dùng bản trả thành công mà không báo gì: nếu use case gọi nhầm ví rồi vẫn trả `cash`, test chỉ kiểm trạng thái có thể bỏ lọt tác dụng phụ.

### 5.2. Stub đưa hệ thống vào tình huống khó

```ts
const stub: PaymentGateway = {
  charge: async () => ({ status: 'declined', chargeId: null }),
  refund: async () => { throw new Error('Không dùng refund'); },
};
```

Test hoàn thành chuyến bằng stub này kiểm trạng thái trả về lẫn trạng thái đọc lại từ kho. Assertion đó bảo vệ R-PAY-06; nó không chứng minh kho dữ liệu thật ghi bền vững, vì repository đang là bản bộ nhớ.

### 5.3. Fake giúp quan sát trạng thái qua nhiều lần gọi

`FakePaymentGateway` có sổ khoản thu theo `idempotencyKey`, danh sách `calls` và phương thức `totalCharged()`. Gọi hai lần cùng request qua `chargeWithRetry`, rồi kiểm:

```ts
expect(replay.chargeId).toBe(first.chargeId);
expect(fake.calls).toHaveLength(2);
expect(fake.totalCharged()).toBe(21_000);
```

Hai lần gọi không đồng nghĩa hai lần trừ tiền. Fake cho ta cách quan sát sự khác biệt này trong mô hình lab. Một stub luôn trả cùng `chargeId` không tự có sổ khoản thu để chứng minh điều đó.

### 5.4. Mock-style verification ở hợp đồng công khai

```ts
const charge = vi.fn<PaymentGateway['charge']>()
  .mockResolvedValue({ status: 'succeeded', chargeId: 'charge-demo' });
```

Sau khi hoàn thành chuyến, kiểm các trường thuộc R-PAY-03:

```ts
expect(charge).toHaveBeenCalledTimes(1);
expect(charge).toHaveBeenCalledWith(expect.objectContaining({
  customerId: 'C001',
  amount: 21_000,
  idempotencyKey: `trip:${trip.id}:fare`,
}));
```

Không khoá cứng ID do bộ sinh ID tạo, nhưng vẫn kiểm đúng mẫu khoá gắn với chuyến đang xét. Không kiểm nguyên văn `description` nếu yêu cầu đang kiểm không quy định câu đó. Điều này tránh failure do sửa câu mô tả trong khi hợp đồng thanh toán không đổi.

### 5.5. Spy giữ cài đặt thật của đối tượng đang quan sát

```ts
const fake = new FakePaymentGateway();
const charge = vi.spyOn(fake, 'charge');
```

Ở môi trường Node của lab, nếu chưa thay implementation, lời gọi vẫn đi vào `fake.charge`. Bởi vậy test kiểm được cả số lần gọi lẫn `totalCharged`. “Gọi thật” ở đây là gọi cài đặt của **fake**, không phải gọi VíXanh bên ngoài. Sau mỗi test, `vi.restoreAllMocks()` khôi phục spy; các fake và hệ thống bộ nhớ được tạo mới nên không dùng chung sổ thu tiền.

### 5.6. Một stub theo thứ tự cho retry

```ts
const charge = vi.fn<PaymentGateway['charge']>()
  .mockRejectedValueOnce(new PaymentUnavailableError())
  .mockResolvedValue({ status: 'succeeded', chargeId: 'retry-ok' });
```

Test case này cho thất bại ngay ở lần đầu và thành công ở lần sau, nên không chờ timeout. Test case này kiểm phản ứng với lỗi tạm thời; bài kế tiếp dùng fake timers để kiểm đúng thời hạn 3 giây và tình huống đã thu tiền nhưng mất phản hồi.

Kết quả kiểm chứng ngày 07/11/2026 với Vitest 3.2.7: **7 test đạt**. Không có HTTP, DB hay giao dịch ví thật trong lượt chạy.

## 6. Giới hạn, over-mocking và độ bền khi refactor

**Đừng kiểm tên helper private.** Một test cố truy cập `collectPayment` bằng ép kiểu hoặc sửa export chỉ để gắn spy sẽ phụ thuộc cấu trúc `trips.ts`. Đổi tên helper hoặc gộp nó vào `driverStep` sẽ làm kiểu test đó hỏng dù hành vi không đổi. Các test case trong bài chỉ gọi use case công khai và kiểm port; đổi tên helper nội bộ không làm thay đổi những bằng chứng đó.

**Số lần gọi có thể là hợp đồng.** “Không retry declined” cần assertion số lần gọi. Ngược lại, kiểm mọi lần repository đọc chuyến chỉ để tái hiện thứ tự mã hiện tại dễ trói test vào cách cài đặt. Hãy chỉ kiểm tương tác khi nó diễn đạt rủi ro hoặc yêu cầu cụ thể.

**Fake có thể sai cùng giả định với ứng dụng.** Ví giả chỉ khử trùng lặp theo khoá; nó không kiểm đầy đủ trường hợp cùng khoá nhưng thay amount/customer, và cũng không có xác thực, lưu trữ bền vững hay mạng thật. Test với fake không chứng minh ví bên ngoài có cùng ngữ nghĩa. Cần contract/integration test cho adapter thật khi có môi trường phù hợp.

**Reset lịch sử khác khôi phục cài đặt.** Xoá các lần gọi không nhất thiết đưa method đã thay implementation về trạng thái ban đầu. Trong bài, chọn tạo fake mới cho mỗi test case và khôi phục spy sau mỗi test case; đừng dùng một singleton ví giữa nhiều test rồi trông cậy vào thứ tự chạy.

**Không thay SUT bằng stub.** Nếu stub luôn trả `completed` ngay tại `driverStep`, bạn đã thay chính hành vi cần kiểm. Mục tiêu là điều khiển cộng tác viên ở port, giữ logic lập yêu cầu và xử lý kết quả chạy thật.

## 7. Bài tập và tự đánh giá

### Bài tập

1. Thêm ca cổng trả lỗi tạm thời ở cả ba lần. Kiểm outcome `failed`, `attempts: 3`, `failure: unavailable` và cùng khoá ở cả ba request.
2. Đổi stub thành công thành stub luôn trả `chargeId: same`. Nó có đủ bằng chứng cho “chỉ trừ tiền một lần” không?
3. Trên nhánh thử nghiệm, đổi tên helper private `collectPayment` nhưng giữ hành vi. Chạy lại bộ test; liệt kê kiểu assertion nào sẽ làm test giòn nếu thêm vào.

<details><summary>Gợi ý đáp án</summary><p>1. Dùng mockRejectedValue với PaymentUnavailableError, rồi kiểm cả outcome lẫn danh sách request. 2. Không: phản hồi không phải sổ khoản thu; cần quan sát trạng thái hoặc bằng chứng từ hệ thống ví. 3. Các test hiện tại nên tiếp tục đạt; assertion nhắm tên hoặc số lần gọi helper private sẽ buộc sửa test vì refactor.</p></details>

**Câu 1.** Cần kiểm “declined không retry”. Assertion nào trực tiếp nhất?

- A. Tổng tiền khách phải trả là số nguyên.
- B. `charge` được gọi đúng một lần và outcome là declined.
- C. File test có từ “mock”.
- D. Fake chứa một `Map`.

<details><summary>Đáp án: B</summary><p>Yêu cầu mô tả số lần cộng tác với cổng trong tình huống cụ thể. Chỉ kiểm trạng thái cuối có thể bỏ lọt retry thừa.</p></details>

**Câu 2.** `vi.spyOn(fake, 'charge')` chưa có `mockImplementation` chứng minh điều gì khi test đạt?

- A. VíXanh thật đã thu tiền.
- B. Fake đã được quan sát trong kịch bản test; cài đặt của fake vẫn chạy.
- C. Adapter HTTP đúng hợp đồng production.
- D. Mọi hành vi idempotency đều đúng.

<details><summary>Đáp án: B</summary><p>Phạm vi quan sát dừng ở fake trong tiến trình. Các kết luận còn lại cần bằng chứng bên ngoài bộ test này.</p></details>

Câu hỏi do bài viết biên soạn, không phải câu hỏi thi ISTQB chính thức.

## 8. Tài liệu tham khảo

- [Martin Fowler — Mocks Aren't Stubs](https://martinfowler.com/articles/mocksArentStubs.html): thuật ngữ test double và hai cách kiểm trạng thái/tương tác.
- [Vitest — vi utilities](https://vitest.dev/api/vi.html): `vi.fn`, `vi.spyOn`, khôi phục spy. Ví dụ được chạy trên phiên bản khoá trong lab.
- [Lab Tới Nơi](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing): SPEC R-PAY; `app/ports.ts`, `app/payment.ts`, `app/trips.ts`, `infra/fake-payment-gateway.ts`.

Bài trước: [TEST-056 · Unit Test Design cho Pricing Engine](/articles/unit-test-design-pricing-engine). Bài tiếp theo: [TEST-058 · Dependency Isolation](/articles/dependency-isolation).
