# DA-P01 · Hồ sơ nền tảng phân tích huỷ chuyến

Đây là dữ liệu giả lập riêng của DA, không kế thừa số liệu baseline BA và không phải dữ liệu production. Các bảng tổng hợp dưới đây đủ cho DA-001; chưa có dữ liệu từng chuyến hay bằng chứng nguyên nhân.

## Phạm vi và quy ước

- Dịch vụ đặt xe, khu vực giả lập A; chưa đại diện toàn bộ thành phố hoặc dịch vụ Tới Nơi.
- W1 và W2 là hai tuần liên tiếp theo Asia/Ho_Chi_Minh. Ngày cụ thể chưa đặt; không dùng để tra thời tiết hoặc ngày lễ thật.
- Đơn vị đếm là một lượt đặt xe đã được nhận, định danh duy nhất. Xếp cohort theo thời điểm được tài xế nhận lần đầu; mỗi lượt thuộc đúng một tuần và một nhóm giờ.
- Tử số: lượt thuộc cohort bị huỷ trước đón. Mẫu số: tất cả lượt thuộc cohort. Chưa gồm yêu cầu không được nhận.
- Trong bảng luyện tính, giả định đã chờ đủ kết quả, không thiếu/trùng; người học được phép dùng điều kiện này. Khi có dataset chi tiết, phải kiểm chứng lại, không coi giả định là kết quả kiểm toán.
- Hai nhóm “cao điểm” và “còn lại” loại trừ nhau, bao phủ cohort và dùng cùng lịch ở hai tuần. Lịch giờ cụ thể chưa chốt; cần xác nhận trước khi tạo dataset timestamp.

| Kỳ | Nhóm | Được nhận | Huỷ trước đón | Tỷ lệ |
|---|---|---:|---:|---:|
| W1 | Cao điểm | 500 | 80 | 16% |
| W1 | Còn lại | 500 | 40 | 8% |
| W2 | Cao điểm | 750 | 135 | 18% |
| W2 | Còn lại | 750 | 45 | 6% |

Đối soát W1: 1.000 lượt, 120 huỷ, 12%. W2: 1.500 lượt, 180 huỷ, 12%. Số huỷ tăng 50%; chênh lệch tỷ lệ chung 0 điểm phần trăm. Cao điểm tăng 2 điểm phần trăm, còn lại giảm 2 điểm phần trăm. Không suy ra nguyên nhân hoặc ý nghĩa thống kê từ phép tính này.

## Câu hỏi mở

| ID | Cần xác minh | Người phối hợp dự kiến | Trạng thái |
|---|---|---|---|
| Q-01 | Khung giờ cao điểm và phạm vi khu vực A | Vận hành | Chưa chốt |
| Q-02 | Thời điểm nhận/đón/huỷ được ghi nhận thế nào? | Backend, chủ nguồn | Chưa kiểm tra hệ thống |
| Q-03 | Có thay đổi tracking hoặc pipeline giữa W1 và W2? | DE, backend | Chưa biết |
| Q-04 | Có thay đổi cung tài xế, thời tiết, giá hoặc chiến dịch? | Vận hành, sản phẩm | Giả thuyết cần dữ liệu |
| Q-05 | Thời gian chờ kết quả và cách chốt cohort trong dataset thật? | DA, vận hành, chủ nguồn | Chưa chốt |

## Sản phẩm DA-001

Viết bản nhận định tối đa một trang: phạm vi, số liệu và công thức, phát hiện theo nhóm, điều chưa biết, khuyến nghị xác minh và người phối hợp. Tiêu chí: đúng mẫu số; đủ số đếm/tỷ lệ; tổng nhóm khớp; không quy kết nhân quả; có bước tiếp.

Bài tập cohort 2.000/220 là biến thể độc lập: 11%, giảm 1 điểm phần trăm so với W2. Không thêm vào bảng baseline.
