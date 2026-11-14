---
slug: ba-acceptance-criteria-given-when-then
title: "Acceptance criteria và ví dụ Given–When–Then"
summary: "Viết tiêu chí chấp nhận quan sát được cho xác nhận điểm đón, bao gồm revision cũ, trạng thái thay đổi, sai quyền và kết quả chưa biết khi timeout."
category: business-analysis
level: beginner
tags: [business-analysis, ba-026, acceptance-criteria, gherkin, toi-noi]
language: vi
---

“Nhấn xác nhận thì lưu thành công” bỏ qua câu hỏi quan trọng: khách xác nhận bản nào, lượt còn cho sửa không và hệ thống có thực sự lưu hay chỉ app đổi nhãn? Tiêu chí chấp nhận giúp cả nhóm cùng biết điều gì phải đúng trong từng tình huống.

<div class="tkh-callout"><span class="tkh-callout-title">BA-026 · Phần 4/8 · Cơ bản</span><p>Đọc trước: <a href="/articles/ba-user-story-vai-tro-nhu-cau-gia-tri">User story</a>. Đầu ra: AC và ví dụ cho US-P04-01. Mọi hành vi điểm đón trong bài là đề xuất thiết kế, chưa phải test đã chạy trên lab.</p></div>

## 1. Tiêu chí chấp nhận mô tả kết quả, không chỉ thao tác

AC-P04-01: với khách chủ lượt, trạng thái searching và revision được gửi khớp nguồn chuẩn, hệ thống ghi nhận xác nhận đúng revision và phản hồi kết quả cho khách. Đây vẫn cần review nghiệp vụ, nhưng đã rõ hơn “lưu tốt”.

Acceptance criteria áp dụng cho phạm vi yêu cầu cụ thể. Checklist chất lượng dùng chung như review code hoặc kiểm tra triển khai có vai trò khác; hoàn thành checklist đó không chứng minh từng AC nghiệp vụ đạt.

## 2. Given, When, Then giúp tách ba phần

[Tài liệu Gherkin của Cucumber](https://cucumber.io/docs/gherkin/reference/) giải thích Given thiết lập bối cảnh, When mô tả hành động và Then mô tả kết quả kỳ vọng. Ví dụ dưới dùng cú pháp Gherkin để trao đổi; chưa có step definition hay hệ thống thực thi.

```gherkin
Feature: Xác nhận thông tin điểm đón
  Scenario: Chủ lượt xác nhận đúng bản đang xem
    Given khách K sở hữu lượt B đang searching
    And nguồn chuẩn và màn hình đều có pickupRevision 4
    And mô tả điểm đón là "Cổng số 2"
    When K xác nhận revision 4
    Then xác nhận được ghi cho revision 4 của B
    And phản hồi cho biết đã xác nhận đúng nội dung đó
    And trạng thái chuyến vẫn là searching
```

Then không nên chỉ viết “thành công”. Mệnh đề đó phải chỉ ra thành công của việc gì, trên dữ liệu nào và điều gì không bị biến thành một hành vi khác.

## 3. Viết scenario làm lộ mâu thuẫn

```gherkin
  Scenario: Khách đang xem một bản cũ
    Given khách K sở hữu lượt B đang searching
    And màn hình của K đang hiển thị revision 4
    And nguồn chuẩn của B đã là revision 5
    When K xác nhận revision 4
    Then revision 4 không được ghi nhận như xác nhận hiện hành
    And khách được yêu cầu xem lại thông tin mới
```

Không tự chọn mã HTTP khi hợp đồng API chưa chốt. Ở mức BA, cần thống nhất kết quả nghiệp vụ trước; mã, schema và cơ chế retry được bổ sung ở hợp đồng kỹ thuật mà vẫn truy vết về cùng AC.

## 4. Bao phủ các nhóm rủi ro

| AC | Tình huống | Kỳ vọng đề xuất |
|---|---|---|
| 01 | Đúng quyền/state/revision | Xác nhận đúng revision |
| 02 | Revision đã đổi | Không xác nhận dữ liệu cũ; xem lại |
| 03 | State đã accepted trước xử lý ghi | Từ chối ghi; đọc lại trạng thái |
| 04 | Không sở hữu lượt | Không ghi, không lộ dữ liệu ngoài quyền |
| 05 | Mất phản hồi | Hiển thị chưa biết kết quả; kiểm tra lại bằng định danh ổn định |

Ghi chú rỗng là câu hỏi mở trong hồ sơ P04. Không tự thêm scenario “rỗng luôn được phép” hoặc “rỗng luôn lỗi” chỉ để làm bảng đủ hàng.

## 5. Phân biệt timeout và thất bại chắc chắn

Request có thể đã được lưu trước khi response mất. Nếu app báo thất bại chắc chắn rồi tạo yêu cầu mới, người dùng có thể làm lặp thao tác. AC cần nói rõ trạng thái chưa biết, cách kiểm lại và chỉ báo thành công khi có kết quả chuẩn.

Đây là yêu cầu thiết kế cần hợp đồng idempotency/đối soát; không tự mượn endpoint huỷ của lab làm API xác nhận điểm đón. Cùng nguyên tắc chống lặp không có nghĩa cùng một API hoặc cùng retention.

## 6. AC không phải toàn bộ test suite

AC nêu kỳ vọng cần chấp nhận; test design có thể thêm nhiều dữ liệu, biên, phân quyền và tải. Một scenario luồng thuận không đủ cho bảo mật hoặc hiệu năng. Ngược lại, hàng trăm scenario lặp chi tiết click có thể che mất rule chính.

Review cùng BA/nghiệp vụ, developer và tester: mỗi ví dụ đang kiểm rule nào, dữ kiện có hợp lệ không, kết quả có quan sát được không? Nếu hai scenario đòi kết quả khác nhau cho cùng dữ kiện, tìm quyết định còn thiếu trước khi tự động hoá.

## 7. Bài tập và câu hỏi ôn tập

Viết scenario cho state đã accepted và scenario request đã lưu nhưng response mất. Ghi rõ phần nào cần hợp đồng thêm. Sau đó đổi Then “lưu vào database thành công” thành kết quả nghiệp vụ quan sát được từ kênh phù hợp.

**Câu hỏi:** Một file Gherkin parse được có nghĩa yêu cầu đã đúng không?

<details><summary>Gợi ý trả lời</summary><p>Không. Đúng cú pháp chỉ giúp công cụ đọc cấu trúc. Nội dung vẫn có thể mơ hồ, sai nghiệp vụ hoặc thiếu ngoại lệ; và chưa có step definition thì file đó chưa phải kiểm thử thực thi.</p></details>

## 8. Tham khảo và bài tiếp theo

- [Cucumber — Gherkin Reference](https://cucumber.io/docs/gherkin/reference/).
- Hồ sơ thiết kế: `examples/toi-noi-ba/p04-goi-yeu-cau.md`.

Bài tiếp theo: [Use case: actor, precondition và các luồng](/articles/ba-use-case-xac-nhan-diem-don).
