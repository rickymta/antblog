# Hồ sơ thực hành DA · Tới Nơi

Đọc cùng [khung chương trình Data Analysis](../../docs/da-master-curriculum.md) và [hồ sơ nghiệp vụ chung](../../api/seed/content/du-an-mau-toi-noi.md).

**Trạng thái: DA-001–DA-003 đã có trong source cùng [hồ sơ P01](p01-nen-tang.md); chưa có dataset từng chuyến hoặc script thực thi.** Dự án xuyên suốt là Tới Nơi với đặt xe, giao đồ ăn và giao bưu kiện. Mọi dữ liệu bổ sung sẽ là giả lập và phải ghi nguồn, phiên bản, grain, thời gian cùng kết quả đối soát.

## Tình huống xuyên suốt

Vận hành đặt câu hỏi: “Báo cáo cho thấy tỷ lệ huỷ tăng; mức tăng có thật không, tập trung ở đâu và nên kiểm chứng phương án nào?” Đây là yêu cầu phân tích giả lập, chưa khẳng định nguyên nhân hoặc hiệu quả của bất kỳ tính năng nào.

Các hướng cần kiểm tra gồm cách định nghĩa chỉ số, thay đổi thành phần dịch vụ/thành phố, thời gian đón, cung tài xế, thời tiết, khuyến mãi, thay đổi chính sách và lỗi ghi nhận dữ liệu. Chưa có số liệu thì giữ trạng thái giả thuyết.

## Gói đầu ra dự kiến

| Phần | Hồ sơ cần tạo khi triển khai |
|---|---|
| P01 | Analysis brief, source inventory, metric contract, assumption log |
| P02 | Dataset SQL nhỏ, truy vấn, số đối soát và hướng dẫn chạy |
| P03 | Data dictionary, quality report, cleaning log và notebook EDA |
| P04 | Workload, ma trận tiêu chí, benchmark plan và decision record |
| P05 | Schema mart, mapping nguồn, lineage và kiểm tra pipeline |
| P06 | Dashboard, định nghĩa filter, bản khuyến nghị một trang |
| P07 | Kế hoạch đánh giá tác động, giả định, kết quả và bất định |
| P08 | Gói tái lập, runbook, incident exercise và hồ sơ capstone |

## Mẫu hợp đồng chỉ số

| Trường | Cần xác định |
|---|---|
| Tên / phiên bản / owner | Tên nghiệp vụ, người xác nhận và lịch sử thay đổi |
| Quyết định hỗ trợ | Ai dùng số liệu để quyết định điều gì? |
| Grain và đơn vị | Một dòng, một đối tượng và một đơn vị đo là gì? |
| Tử số / mẫu số | Trạng thái và điều kiện đưa vào từng tập |
| Cohort / thời gian | Sự kiện dùng để xếp cohort, timezone và biên thời gian |
| Ngoại lệ | Dữ liệu test, bản ghi trùng, thiếu và chưa có kết quả cuối |
| Độ mới | Hạn chốt kỳ, xử lý dữ liệu trễ và sửa lại số liệu |
| Nguồn / kiểm tra | Mapping nguồn, truy vấn, đối soát và giới hạn sử dụng |

Không mặc định tỷ lệ huỷ trên mọi yêu cầu giống tỷ lệ huỷ trước đón trên lượt đã được nhận. Không cộng số lượt liên hệ CSKH thành số chuyến. Không lấy baseline hay mục tiêu giả lập của BA làm kết quả DA đã kiểm chứng.

## Tiêu chí hoàn thành capstone

- Câu hỏi, người ra quyết định và chỉ số được xác định rõ.
- Có thể truy nguồn và chạy lại kết quả; phát hiện được lỗi join, thiếu/trùng và dữ liệu trễ.
- Lựa chọn kho dữ liệu có workload, tiêu chí và bằng chứng thử nghiệm.
- Kết luận tách dữ kiện, giả thuyết và suy luận; nêu bất định và yếu tố gây nhiễu.
- Dashboard thống nhất với truy vấn đã đối soát và chỉ rõ dữ liệu cập nhật tới đâu.
- Khuyến nghị có người sở hữu, cách đo lại, chỉ số bảo vệ và điều kiện xem xét lại.
