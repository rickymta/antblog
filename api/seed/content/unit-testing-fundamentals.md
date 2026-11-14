---
slug: unit-testing-fundamentals
title: "Unit Testing Fundamentals: năm test case đầu tiên cho chức năng tính cước của Tới Nơi"
summary: "Viết unit test theo Arrange–Act–Assert, đánh giá chất lượng bằng FIRST và hiểu vòng red–green–refactor qua năm quy tắc tính cước có oracle từ SPEC."
category: testing
level: advanced
tags: [testing, istqb, ctfl, test-055, unit-testing, vitest, aaa, first]
language: vi
---

Một lần sửa hệ số cao điểm có thể khiến phụ phí trời mưa cũng bị nhân lên. Mở ứng dụng, tạo chuyến rồi nhìn số tiền là một cách phát hiện lỗi; nhưng khi đang sửa hàm tính cước, ta cần một phản hồi nhỏ và nhanh hơn. Bài này bắt đầu Series 06 bằng năm test gọi thẳng mã nghiệp vụ của Tới Nơi, mỗi test nói rõ một quy tắc và số tiền mong đợi.

<div class="tkh-callout">
<span class="tkh-callout-title">Thông tin bài · TEST-055</span>
<table><tbody>
<tr><th scope="row">Series</th><td>S06 · Unit Testing &amp; Component Testing</td></tr>
<tr><th scope="row">ISTQB Level</th><td>Foundation · CTFL v4.0.1</td></tr>
<tr><th scope="row">Knowledge Level</th><td>L3 · Advanced — thực hành developer/SDET</td></tr>
<tr><th scope="row">Test Level / Type</th><td>Component / Functional</td></tr>
<tr><th scope="row">Test Technique</th><td>Unit testing; thiết kế từ quy tắc nghiệp vụ</td></tr>
<tr><th scope="row">Bài cần đọc trước</th><td><a href="/articles/component-testing-unit-testing-tong-quan">TEST-012 · Component Testing</a> · <a href="/articles/equivalence-partitioning">TEST-031 · Equivalence Partitioning</a></td></tr>
<tr><th scope="row">Tham chiếu</th><td>CTFL 2.1.3 · FL-2.1.3 (K1); 2.2.1 · FL-2.2.1 (K2). AAA, FIRST và cách dùng Vitest là thực hành kỹ thuật mở rộng.</td></tr>
<tr><th scope="row">Lab</th><td><code>examples/toi-noi-testing/articles/unit-testing-fundamentals/</code></td></tr>
</tbody></table>
</div>

## 1. Giới thiệu

[TEST-012](/articles/component-testing-unit-testing-tong-quan) giải thích component testing nằm ở đâu trong các test level. Ở đây, câu hỏi là: khi mở editor, ta viết gì để kiểm một quy tắc mà đồng nghiệp có thể hiểu và chạy lại?

Đối tượng là `calculateFare(input)` trong `src/domain/fare.ts`. Hàm nhận loại xe, quãng đường, cờ cao điểm, cờ mưa và voucher tuỳ chọn; trả về một bảng chi tiết cước. Hàm này không mở kết nối mạng, không đọc DB, không lấy giờ hiện tại. Vì thế ta có thể kiểm các phép tính với dữ liệu cố định.

Sau bài này, bạn có thể viết test theo AAA, nhận ra test phụ thuộc trạng thái và dùng kết quả thất bại để tìm quy tắc bị phá vỡ. Năm test case đầu tiên là điểm khởi đầu; thiết kế có hệ thống cho biên và tổ hợp nằm ở TEST-056.

## 2. Lý thuyết và nền tảng ISTQB

CTFL 2.2.1 đặt component testing ở phạm vi các thành phần được kiểm riêng. CTFL 2.1.3 giới thiệu các cách tiếp cận phát triển hướng kiểm thử, trong đó có TDD. Hai mục này là cơ sở về test level và cách tiếp cận; không phải quy định bắt buộc về framework hay cấu trúc file.

### 2.1. Arrange–Act–Assert

AAA chia một test thành ba phần:

| Phần | Câu hỏi cần trả lời | Ví dụ |
|---|---|---|
| Arrange — chuẩn bị | Điều kiện ban đầu là gì? | Xe máy, 2 km, không mưa, giờ thường |
| Act — thực thi | Hành vi nào được kiểm? | Gọi `calculateFare` |
| Assert — xác nhận | Quan sát nào chứng minh quy tắc? | Tổng bằng 21.000 đ |

Một hành vi có thể cần nhiều assertion liên quan. Với cước tối thiểu, kiểm `base`, `minimumFare` và `total` trong cùng một test giúp giải thích phép nâng lên cước tối thiểu. Không cần tách thành ba test chỉ để đạt khẩu hiệu “một assert”.

### 2.2. FIRST như một bảng tự review

Trong bài này, FIRST được dùng theo nghĩa Fast, Independent, Repeatable, Self-validating, Timely. Đây là quy ước thực hành ngoài các mục tiêu học tập (LO) nêu trên.

| Thuộc tính | Cách áp dụng vào bài cước |
|---|---|
| Fast — nhanh | Gọi domain trực tiếp, không khởi động server |
| Independent — độc lập | Mỗi test tự tạo input; không lấy dữ liệu test trước |
| Repeatable — lặp lại được | Không dùng thời tiết thật, giờ thật hay số ngẫu nhiên |
| Self-validating — tự xác định đạt/trượt | Dùng `expect`, không yêu cầu đọc log để kết luận |
| Timely — đúng lúc | Viết sát lúc phân tích và sửa quy tắc; với TDD, viết trước mã đáp ứng quy tắc |

Độ nhanh chỉ là yêu cầu tương đối, so với vòng phản hồi của nhóm. Không suy ra một ngưỡng mili giây phổ quát từ bảng này. Độc lập cũng không đồng nghĩa với mock mọi hàm: `calculateFare` có thể gọi helper tiền tệ thật mà bộ test vẫn tất định.

### 2.3. Thuật ngữ

**SUT** là phần mềm đang được kiểm, ở đây là hàm tính cước cùng các helper được gọi. **Fixture** là dữ liệu và trạng thái chuẩn bị cho test. **Oracle** là cơ sở xác định kết quả đúng: bảng giá và quy tắc trong SPEC. **Test runner** tìm, thực thi và tổng hợp kết quả test; runner không tự biết số tiền đúng nếu ta chưa viết assertion.

## 3. Tình huống kiểm thử tại Tới Nơi

Một khách chọn xe máy cho chuyến 2 km. Nhóm sắp thay đổi cách cấu hình cao điểm và muốn giữ nguyên hành vi tính cước.

| Yêu cầu | Quy tắc dùng trong bài |
|---|---|
| R-FARE-01 | Xe máy: mở cửa 12.000 đ, 4.500 đ/km, tối thiểu 15.000 đ; ô tô: 25.000 đ, 11.000 đ/km, tối thiểu 30.000 đ |
| R-FARE-04 | Khi cờ cao điểm đúng, nhân cước cơ bản với 1,25 |
| R-FARE-05 | Cộng 8.000 đ tiền mưa sau bước cao điểm |
| R-FARE-07 | Sau voucher, nâng lên cước tối thiểu nếu cần |
| R-FARE-08 | Cuối cùng làm tròn lên bội số 1.000 đ |

Rủi ro cần bắt ngay: chọn nhầm bảng giá, bỏ cước tối thiểu, nhân nhầm phụ phí trời mưa hoặc làm tròn xuống. Cờ `peak` đã là đầu vào; xác định 11:00 có thuộc cao điểm hay không là trách nhiệm của phần xử lý thời gian, năm test này chưa kiểm chứng.

## 4. Phân tích kỹ thuật và thiết kế test

Trước khi viết `expect`, tính tay theo SPEC:

| Test case | Dữ liệu | Phép tính độc lập | Kết quả |
|---|---|---|---:|
| UT-01 | Bike, 2 km, thường, khô | 12.000 + 4.500 × 2 | 21.000 |
| UT-02 | Car, 2 km, thường, khô | 25.000 + 11.000 × 2 | 47.000 |
| UT-03 | Bike, 0,5 km, thường, khô | 14.250 thấp hơn cước tối thiểu 15.000 | 15.000 |
| UT-04 | Bike, 2 km, cao điểm, khô | 21.000 × 1,25 = 26.250; làm tròn lên | 27.000 |
| UT-05 | Bike, 2 km, cao điểm, mưa | 26.250 + 8.000 = 34.250; làm tròn lên | 35.000 |

Nếu UT-04 đạt nhưng UT-05 trượt, phần xử lý kết hợp phụ phí trời mưa với cao điểm nên được kiểm tra trước. Nếu người viết nhân cả tiền mưa, tổng trở thành 37.000 đ: `(21.000 + 8.000) × 1,25 = 36.250`, rồi làm tròn lên. Bộ test có dữ liệu phân biệt được hai cách cài đặt.

Không viết `expect(actual.total).toBe(calculateFare(input).total)`. Đó là so hàm với chính nó; cả hai phía có thể cùng sai. Cũng tránh import bảng giá của sản phẩm để tự dựng mọi expected: khi bảng giá bị sửa nhầm, test có thể đổi theo.

## 5. Thực hành với Vitest

Chạy từ `examples/toi-noi-testing`. Nếu chưa cài dependency, dùng `npm ci`; lab có lockfile riêng.

```bash
npx vitest run articles/unit-testing-fundamentals
```

File `fare.test.ts` có năm test. Test case đầu tiên thể hiện đầy đủ AAA:

```ts
import { expect, it } from 'vitest';
import { calculateFare } from '../../src/domain/fare';

it('R-FARE-01/08: xe máy 2 km giờ thường trả 21.000 đ', () => {
  // Arrange
  const input = {
    vehicle: 'bike' as const,
    distanceKm: 2,
    peak: false,
    raining: false,
  };
  // Act
  const fare = calculateFare(input);
  // Assert
  expect(fare.total).toBe(21_000);
});
```

Đường dẫn import tính từ thư mục bài. `as const` giữ kiểu literal `bike` khi lưu input vào biến; nó không xác thực dữ liệu từ người dùng. Mỗi test tạo dữ liệu riêng nên không cần `beforeEach` chỉ để khởi tạo vài trường.

Ở test case có mưa, assertion kiểm cả bước trung gian:

```ts
expect(fare).toMatchObject({
  afterPeak: 26_250,
  rainSurcharge: 8_000,
  total: 35_000,
});
```

Các trường này là kết quả nghiệp vụ công khai của R-FARE-10, không phải tên biến nội bộ. `toMatchObject` cho phép thêm trường mới vào kết quả mà test không trượt vì thay đổi không liên quan.

Để chạy lại khi lưu file:

```bash
npx vitest articles/unit-testing-fundamentals
```

Thoát watch bằng `q`. Dùng lệnh có `run` trong kiểm tra tự động để tiến trình kết thúc sau một lượt. Kết quả mong đợi là một file, năm test đạt; với mỗi failure, hãy đọc tên quy tắc, expected và actual trước khi sửa assertion. Kiểm chứng ngày 07/11/2026 với Vitest 3.2.7: cả 5 test đạt trên mã lab hiện tại.

### Thử vòng red–green–refactor

Các test trong repo đang chạy trên mã đã có, vì vậy lần chạy xanh **không phải bằng chứng nhóm đã làm TDD**. Để luyện, tạo một hàm riêng trong thư mục bài, bắt đầu với ca xe máy 2 km: viết test đòi 21.000, quan sát thất bại vì hành vi chưa có, cài đặt tối thiểu rồi chạy lại. Thêm test case ô tô và cước tối thiểu, sau đó gom cấu hình trùng khi test vẫn xanh.

Không sửa `src/domain/fare.ts` để cố tình tạo lỗi trong bài luyện. Giữ bản luyện tập ngay cạnh bài để các lab khác vẫn dùng quy tắc chuẩn.

## 6. Những điểm dễ bỏ sót

**Unit không nhất thiết là một hàm tách khỏi mọi helper.** Ranh giới bài này gồm tính cước và các phép toán thuần. Mock `roundUpToThousandDong` sẽ làm mất cơ hội phát hiện lỗi làm tròn trong hành vi cước. Khi có phụ thuộc bên ngoài như cổng thanh toán, việc thay thế phụ thuộc cần quyết định khác; đó là nội dung TEST-057.

**Test xanh chưa chứng minh đủ quy tắc.** Năm test case chưa kiểm voucher, quãng đường không hợp lệ, phụ phí bưu kiện, giá đồ ăn, HTTP status hay đồng thời. Ghi giới hạn giúp người đọc không nhầm bộ khởi đầu với bộ hồi quy đầy đủ.

**Tên test cần chỉ ra hành vi.** `case 1` không cho người bảo trì biết tại sao 35.000 đúng. Tên chứa R-FARE-05 và “mưa cộng sau cao điểm” giải thích lý do tồn tại của test.

**Refactor không được làm mất oracle.** Có thể trích helper tạo input khi nhiều test cùng cần; nhưng mặc định ngầm về cao điểm hoặc voucher sẽ làm người đọc phải mở nhiều file. Giữ biến có ý nghĩa với từng test case ngay trong test case đó.

**Thất bại có thể nằm ở test.** Khi SPEC thay đổi hợp lệ, kiểm lại yêu cầu trước khi cập nhật expected. Khi chỉ sửa cài đặt, đừng đổi 35.000 thành 37.000 để làm runner xanh.

## 7. Bài tập và tự đánh giá

### Bài tập

1. Thêm test case xe máy 2 km, giờ thường, đang mưa. Ghi phép tính vào phần giải thích trước khi viết assertion.
2. Review một test dùng `console.log(fare.total)` nhưng không có `expect`. Thuộc tính FIRST nào chưa đạt?
3. Chọn một thay đổi cấu trúc code không đổi nghiệp vụ. Giải thích vì sao năm test trên nên tiếp tục đạt.

<details><summary>Gợi ý đáp án</summary>
<p>1. 21.000 + 8.000 = 29.000 đ. 2. Self-validating: runner không tự biết log đó đúng hay sai. 3. Đổi tên biến nội bộ hoặc trích bảng cấu hình giữ nguyên giá không đổi kết quả công khai; test không nên phụ thuộc các chi tiết đó.</p>
</details>

### Câu hỏi tự luyện

**Câu 1.** Một test gọi `calculateFare` hai lần với cùng input rồi so hai tổng. Nhược điểm lớn nhất là gì?

- A. Có hai lần gọi nên chắc chắn chậm.
- B. Không có oracle độc lập để phát hiện cùng một lỗi ở cả hai phía.
- C. Không được phép gọi hàm thuần trong test.
- D. Phải đổi sang snapshot.

<details><summary>Đáp án: B</summary><p>Hai kết quả bằng nhau chỉ chứng minh tính nhất quán trong test case đó. Snapshot cũng cần một cơ sở xác nhận giá trị ban đầu đúng.</p></details>

**Câu 2.** Bộ test này truyền `peak: true`. Nó chứng minh điều gì?

- A. Múi giờ máy chủ luôn đúng.
- B. API tự xác định đúng mọi giờ cao điểm.
- C. Phép tính áp dụng đúng khi đã nhận cờ cao điểm trong các test case đã chạy.
- D. Hệ thống báo giá chịu được tải cao.

<details><summary>Đáp án: C</summary><p>Đầu vào đã bỏ qua bước suy ra cờ cao điểm từ thời gian. Muốn kết luận về bước đó phải kiểm thêm thành phần thời gian và cách các thành phần ghép nối với nhau.</p></details>

Các câu hỏi do bài viết tự biên soạn, không phải đề thi ISTQB chính thức.

## 8. Tài liệu tham khảo

- [ISTQB CTFL Syllabus v4.0.1](https://www.istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf): mục 2.1.3 và 2.2.1. Quyền tác giả syllabus thuộc ISTQB và các tác giả được nêu trong tài liệu.
- [Vitest — Test API](https://vitest.dev/api/test): định nghĩa và chạy test; lab dùng phiên bản khoá trong `package-lock.json`.
- [Lab Tới Nơi](https://github.com/rickymta/antblog/tree/main/examples/toi-noi-testing): `SPEC.md`, `src/domain/fare.ts` và thư mục bài TEST-055.

Bài trước: [TEST-054 · Defect-based Testing](/articles/defect-based-testing). Bài tiếp theo: [TEST-056 · Unit Test Design cho Pricing Engine](/articles/unit-test-design-pricing-engine).
