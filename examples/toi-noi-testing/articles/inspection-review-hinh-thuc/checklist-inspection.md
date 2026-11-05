# Checklist inspection đặc tả nghiệp vụ — bản cho sẵn (CL-SPEC v1.0)

Dùng ở bước **individual review** của inspection TEST-026. Mỗi mục trả lời Có / Không / Không áp dụng,
mọi «Không» ghi thành một dòng anomaly (mục, vị trí, mô tả). Checklist không bao trùm mọi thứ: thấy vấn đề
ngoài danh sách vẫn ghi (CTAL-TA v4.0 mục 5.2.2).

| Mã | Câu hỏi | Gợi ý tìm |
|---|---|---|
| CL-1 | Mọi mốc số (thời gian, tiền, số lần) có ghi rõ đầu mút được tính hay không? | «trong vòng», «sau», «quá», «tối đa» |
| CL-2 | Mọi trạng thái / tổ hợp điều kiện đầu vào đều có kết quả được nêu? | thử lập bảng quyết định, tìm ô trống |
| CL-3 | Một khái niệm chỉ có một tên, trong tài liệu và so với SPEC? | mã lý do, tên trường, tên mốc thời gian |
| CL-4 | Không mâu thuẫn với tài liệu liên quan (R-GEN, R-STATE, R-PAY, R-IDEM)? | đọc chéo từng mục tham chiếu |
| CL-5 | Mỗi yêu cầu kiểm chứng được: có số, có điều kiện, có kết quả quan sát được? | «đáng kể», «hợp lý», «nhanh» |
| CL-6 | Mọi tài liệu / bảng được tham chiếu đều tồn tại và đã nêu tên? | «xem bảng…», «theo quy định…» |
| CL-7 | Hành vi khi thành phần phụ thuộc lỗi (ví, đồng hồ, mạng, gửi lại) có được nêu? | thu tiền thất bại, yêu cầu gửi hai lần |
| CL-8 | Dữ liệu dùng để tính tiền lấy từ nguồn tin cậy (không do client tự khai)? | giờ máy khách, số tiền do app gửi |
