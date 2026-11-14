---
slug: ba-state-model-trang-thai-chuyen
title: "State model: trạng thái chuyến và chuyển trạng thái hợp lệ"
summary: "Mô hình hoá vòng đời chuyến bằng state, event, actor, guard và effect. Phân biệt khách huỷ với tài xế rời chuyến và tách trạng thái chuyến khỏi trạng thái thanh toán."
category: business-analysis
level: intermediate
tags: [business-analysis, ba-021, state-model, transitions, toi-noi]
language: vi
---

Tài xế bấm huỷ nhưng chuyến quay về tìm tài xế; khách bấm huỷ thì chuyến kết thúc. Nếu chỉ có một mũi tên “huỷ → cancelled”, tài liệu sẽ làm mất khác biệt quan trọng này. State model giúp nhóm xác định sự kiện nào được phép thay đổi đối tượng và với điều kiện gì.

<div class="tkh-callout"><span class="tkh-callout-title">BA-021 · Phần 3/8 · Trung bình</span><p>Đọc trước: <a href="/articles/ba-decision-table-phi-huy">Bảng quyết định phí huỷ</a>. Đầu ra: transition table có actor, guard và effect. Nguồn baseline là R-STATE trong SPEC lab; không đổi tên hoặc thêm trạng thái cho vừa sơ đồ.</p></div>

## 1. State, event, guard và effect

State mô tả tình trạng đối tượng; event là việc xảy ra hoặc lệnh được gửi; guard là điều kiện cho phép chuyển; effect là thay đổi đi kèm. “accepted” khác “accept”: một cái là trạng thái, cái kia là sự kiện/lệnh nhận.

Chỉ actor đúng vẫn chưa đủ: khách chủ chuyến không thể customer_cancel sau picked_up theo baseline. Chỉ state đúng cũng chưa đủ: tài xế khác không được thao tác trên chuyến đã gán cho người khác.

## 2. Giữ nguyên từ vựng của hệ thống mẫu

Tới Nơi có requested, searching, accepted, arriving, picked_up, in_transit, completed, cancelled. Trong hồ sơ chung, arriving là đang tới đón; không tự dịch thành “đã đến nơi và bắt đầu tính thời gian chờ”. Nếu muốn có trạng thái arrived riêng, đó là đề xuất thay đổi cần phân tích.

<figure class="tkh-fig"><div class="tkh-fig-box"><svg viewBox="0 0 900 240" role="img" aria-label="Trích đoạn trạng thái: searching sang accepted, arriving, picked_up. Khách huỷ từ searching, accepted hoặc arriving sang cancelled. Tài xế rời accepted hoặc arriving quay về searching."><defs><marker id="ba21-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0L10 5L0 10z" class="f-pk"/></marker></defs><g class="f-pk-soft s-pk"><rect x="30" y="70" width="160" height="45" rx="8"/><rect x="250" y="70" width="160" height="45" rx="8"/><rect x="470" y="70" width="160" height="45" rx="8"/><rect x="690" y="70" width="180" height="45" rx="8"/></g><g class="t-mono" text-anchor="middle"><text x="110" y="98">searching</text><text x="330" y="98">accepted</text><text x="550" y="98">arriving</text><text x="780" y="98">picked_up</text></g><g class="f-none s-pk" stroke-width="2" marker-end="url(#ba21-a)"><path d="M190 92H248"/><path d="M410 92H468"/><path d="M630 92H688"/><path d="M330 70V35H110V68"/><path d="M550 70V35H330"/><path d="M110 115V150H330V178"/><path d="M330 115V178"/><path d="M550 115V150H330"/></g><rect x="240" y="180" width="180" height="42" rx="8" class="f-warn-soft s-warn"/><text x="330" y="207" text-anchor="middle" class="t-mono">cancelled</text><text x="140" y="25" class="t-small">driver_cancel: quay về searching</text><text x="580" y="155" class="t-small">customer_cancel</text><text x="690" y="190" class="t-small">Sau đón: không có</text><text x="690" y="213" class="t-small">nhánh khách huỷ</text></svg></div><figcaption>Trích đoạn trước đón. Sơ đồ bỏ qua requested và phần sau picked_up; bảng dưới bổ sung vòng đời đầy đủ.</figcaption></figure>

## 3. Transition table là hợp đồng chi tiết

| Từ | Event | Actor / guard chính | Tới |
|---|---|---|---|
| requested | search | Hệ thống ngay khi tạo | searching |
| searching | accept | Tài xế đủ điều kiện R-ACCEPT | accepted |
| accepted | arrive | Tài xế được gán | arriving |
| arriving | pickup | Tài xế được gán | picked_up |
| picked_up | start | Tài xế được gán | in_transit |
| in_transit | complete | Tài xế được gán | completed |
| searching/accepted/arriving | customer_cancel | Khách chủ chuyến | cancelled |
| accepted/arriving | driver_cancel | Tài xế được gán | searching |
| searching | system_cancel | Chờ tìm từ 5 phút theo searchingSince | cancelled |

Bảng tóm tắt không thay thế toàn bộ R-ACCEPT hoặc xác thực API. Những cặp state/event không có trong R-STATE-02 bị từ chối theo R-STATE-03, không tự thêm self-transition cho mọi lệnh lặp.

## 4. Effect quan trọng không nằm trong tên trạng thái

Khi driver_cancel, baseline còn bỏ gán tài xế, xoá acceptedAt và ETA, đưa tài xế online và đặt lại searchingSince. Nếu chỉ đổi accepted thành searching mà giữ acceptedAt cũ, lần nhận tiếp theo có thể tính sai khoảng miễn.

Khi hoàn thành, ghi completedAt; khi khách/hệ thống huỷ, ghi cancelledAt; mỗi lần chuyển ghi trip_events. Cần cùng nhóm kỹ thuật review các effect về transaction và tính nhất quán dữ liệu, không chỉ xuất hiện như ghi chú dễ bỏ qua.

Hệ thống kiểm tra các chuyến tìm tài xế quá hạn theo chu kỳ 15 giây trong lab. Điều kiện từ 5 phút không có nghĩa mọi chuyến sẽ đổi state đúng tại mili giây thứ 300.000. Cần phân biệt điều kiện đủ để xử lý với lịch thực thi.

## 5. Tách các vòng đời liên quan

Chuyến cancelled có thể đi cùng paymentStatus failed nếu thu phí ví thất bại. Không thêm cancelled_payment_failed vào vòng đời chuyến chỉ để chứa tổ hợp này. Model chuyến và model thanh toán có liên hệ nhưng trả lời câu hỏi khác nhau.

Tương tự, kết thúc một quy trình BPMN không đồng nghĩa với completed của chuyến. Quy trình “xử lý yêu cầu huỷ bị từ chối” có thể kết thúc trong khi chuyến vẫn picked_up và tiếp tục vận hành.

## 6. Xem xét sự kiện đến đồng thời và replay

Khách huỷ và tài xế pickup gần đồng thời là tình huống cần thống nhất kết quả theo state được chấp nhận tại lúc xử lý. Nếu pickup đã được ghi nhận trước, customer_cancel không còn hợp lệ; nếu huỷ đã thắng, pickup không thể tiếp tục từ cancelled.

BA nêu invariant: không có cả hai kết quả nghiệp vụ mâu thuẫn cho cùng một lần chuyển trạng thái; nhóm kỹ thuật chọn cơ chế thực thi. Không mô tả “event đến màn hình trước thắng” nếu không có hợp đồng đó.

Replay cùng khoá/cùng nội dung đã xử lý là vấn đề idempotency: trả kết quả cũ, không phải chuyển cancelled sang cancelled một lần nữa. Giữ sự phân biệt này để không thu phí lặp.

## 7. Bài tập và câu hỏi ôn tập

Lập bảng effect cho driver_cancel và customer_cancel. Đi qua ba tình huống: tài xế rời rồi người khác nhận; khách huỷ sau pickup; ví thất bại sau khách huỷ có phí. Với mỗi ca ghi state, paymentStatus nếu liên quan, dữ liệu phải thay đổi và nguồn SPEC.

**Câu hỏi:** Có thể suy ra khách được huỷ từ mọi trạng thái vì app có nút huỷ không?

<details><summary>Gợi ý trả lời</summary><p>Không. Nút không quyết định transition hợp lệ. Cần kiểm state, actor và rule tại nơi thực thi; giao diện chỉ phản ánh quyền/hành vi đã xác định.</p></details>

## 8. Tham khảo và bài tiếp theo

- SPEC: `examples/toi-noi-testing/SPEC.md`, R-STATE-01–07, R-PAY-06, R-IDEM.
- [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi).

Bài tiếp theo: [Handoff, timer và luồng ngoại lệ trong BPMN](/articles/ba-bpmn-handoff-timer-ngoai-le).
