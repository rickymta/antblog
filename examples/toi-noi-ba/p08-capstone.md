# P08 · Chiến lược và capstone

Hồ sơ bài tập, không có approval hoặc số liệu production. Trục chính capstone là điểm đón P01–P04 và đánh giá giả lập P07. Refund P05, CR phí P06 và multi-stop P08 là nhánh thực hành riêng.

## Business case minh họa BA-057

Giả định độc lập: giảm 200 cuộc gọi/tháng × 5 phút × 60.000 đồng/giờ = 1 triệu đồng/tháng năng lực quy đổi. Ban đầu 12 triệu, duy trì 0,2 triệu/tháng → ròng 0,8 triệu, hoàn vốn đơn giản 15 tháng nếu lợi ích hiện thực hóa đều. Giảm 100 cuộc gọi → lợi ích 0,5 triệu, ròng 0,3 triệu, hoàn vốn 40 tháng. Không suy từ 80 cuộc gọi P01; không coi năng lực giải phóng chắc chắn là tiền mặt tiết kiệm. Bỏ qua chiết khấu và rủi ro để học phép tính.

## Artifact chiến lược

- Business case: quyết định cần xin, giữ nguyên/cải thiện/triển khai rộng, giả định nhạy, owner lợi ích.
- Capability map: phục vụ chuyến, giao dịch, hỗ trợ khách, cải tiến dịch vụ; phân biệt người/quy trình/công cụ.
- Risk log: A-01 tài xế tìm được thông tin; R-01 telemetry thiếu; I-01 guardrail chưa có baseline; R-02 vượt công suất CS.
- Roadmap: A kiểm dữ liệu/rào cản → B cải thiện giới hạn → C quyết định mở rộng/chỉnh/dừng; không hứa rollout trước bằng chứng.
- Decision narrative: cùng số liệu và giới hạn cho sponsor, CS, nhóm thực hiện.
- AI review: claim ledger, nguồn và version, dữ liệu giả, người review; không tự nâng giả thuyết thành approval.

## Multi-stop đề xuất

So chuyến riêng/tối đa ba điểm cố định/nhiều điểm động. Phân biệt trip, stop, shipment; chưa chốt cardinality và state machine mới. Case A giao thành công, B không liên lạc, C còn chờ. Phải quyết định tiếp tục, thử lại/hoàn về, quyền thay đổi, báo giá, phí điều chỉnh và đối soát. Không kế thừa máy móc rule huỷ một chuyến của lab.

## Checklist portfolio

1. README nói rõ dự án hư cấu, vai trò, phạm vi và nguồn số liệu.
2. Một trang khuyến nghị có phương án khác và điều kiện đổi quyết định.
3. Artifact có ID/version và liên kết truy vết; tách nháp, đề xuất, giả định và quyết định thật sự được chốt trong bài tập.
4. UAT scenario không bị trình bày thành execution report thật; giả định pass ở BA-056 được ghi rõ.
5. Metric đúng cohort/tử số/mẫu số/timezone, không suy nhân quả từ tự chọn.
6. Các bài tập refund/CR/multi-stop tách phạm vi với điểm đón.
7. Có reflection mô tả bằng chứng khiến thay đổi ý kiến và artifact đã sửa.

Rubric sáu tiêu chí × 0–2 = 12: nguồn, nhất quán mô hình, lựa chọn, yêu cầu kiểm chứng được, thay đổi, đánh giá giá trị. Không phải chuẩn chứng chỉ. Sai nguồn, approval hoặc nhân quả phải sửa trước hoàn tất.

## Điều hướng hồ sơ

[P01](p01-ho-so-de-xuat.md) → [P02](p02-discovery-ho-so.md) → [P03](p03-mo-hinh-quy-trinh.md) → [P04](p04-goi-yeu-cau.md) → [P05](p05-tich-hop.md) → [P06](p06-quan-ly-thay-doi.md) → [P07](p07-nghiem-thu-gia-tri.md) → P08.
