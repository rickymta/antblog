# DA-P02 · SQL và báo cáo vận hành Tới Nơi

P02-LAB v1, dữ liệu tổng hợp độc lập với baseline P01/BA. Dùng cho DA-011–DA-020 trong [khung chương trình](../../../docs/da-master-curriculum.md). Không phải migration hoặc dữ liệu production.

## Hợp đồng dữ liệu

- Snapshot cố định: 2026-10-13 00:00:00+07; nguồn lab được coi đầy đủ đến thời điểm đó. T05 còn chưa có kết quả nghiệp vụ, không phải thiếu dòng ingest đã biết.
- Cohort: ride/city_id=1, nhận lần đầu từ 2026-10-05 00:00:00+07 tới trước 2026-10-12 00:00:00+07. Timezone báo cáo Asia/Ho_Chi_Minh.
- city_id=1 không được ánh xạ ngầm thành khu vực A của P01. Tuần và số lượng hoàn toàn riêng.
- `da_trips`: một lượt, trip_id duy nhất. Các mốc pickup/cancel và history_complete là trường bổ sung của fixture phân tích, không sửa schema chung.
- `da_events`: một mốc accepted/picked_up/completed/cancelled. Fixture chủ động sinh lịch sử các mốc này; không phải lịch sử mọi event vận hành.
- `da_payments`: một attempt. Chỉ succeeded + settled_at là tiền thu xác nhận; failed/pending chưa cộng.
- `da_refunds`: một yêu cầu hoàn gắn payment. Chỉ succeeded được trừ tiền ròng; pending theo dõi riêng.
- `da_tickets`: một ticket gắn đúng một lượt theo quy ước lab; nguồn thật có thể có mapping nhiều–nhiều.
- Tiền là số nguyên VND. Khoản T02 thu 20.000 rồi hoàn là giả lập để đối soát, không định nghĩa phí huỷ hay thu cọc của Tới Nơi. Net cash chưa phải doanh thu.
- Tất cả bảng/view đều TEMP, chỉ tồn tại trong một kết nối. Không có DROP/UPDATE bảng ứng dụng; verify.sql thay dữ liệu TEMP trong transaction rồi ROLLBACK.

## Chạy tự động trên PostgreSQL riêng

Cần Python 3 và bộ binary PostgreSQL (đã kiểm với PostgreSQL 18). Không cần server có sẵn, không đọc mật khẩu hoặc `.env`. Trên Windows:

```powershell
python verify.py --pg-bin 'C:/Program Files/PostgreSQL/18/bin'
```

Trên máy có initdb trong PATH:

```sh
python verify.py
```

Chạy từ thư mục này hoặc dùng đường dẫn đầy đủ tới verify.py. Runner tạo thư mục tạm có tên `toinoi-da-p02-*`, khởi tạo cluster chỉ lắng nghe localhost trên cổng tạm, chạy setup/queries/assertions và mọi khối SQL trong mười bài, rồi dừng và dọn cluster. Auth trust chỉ dùng cho cluster lab cục bộ ngắn hạn; không dùng cấu hình này cho server chia sẻ. Không chạy initdb bằng root trên Unix.

Runner trả exit code khác 0 khi lỗi; kết quả PASS chỉ xuất hiện sau khi chạy thành công. Nếu muốn làm bài từng câu, dùng một phiên psql theo cách dưới.

## Chạy trong một kết nối psql của database lab

Từ thư mục này, thay tên database bằng database lab đã có:

```sh
psql -X -d your_lab_database -v ON_ERROR_STOP=1 -f setup.sql -f queries.sql -f verify.sql
```

Hoặc trong psql tương tác chạy `\i setup.sql` rồi dán SQL từng bài. Không chạy setup và query bằng hai kết nối riêng vì bảng TEMP đã biến mất. Để nạp lại fixture, mở kết nối mới. `queries.sql` có một truy vấn JOIN cố ý sai grain để quan sát lỗi; nó chỉ đọc, không sửa dữ liệu.

## Tệp và lộ trình

| Tệp | Vai trò |
|---|---|
| setup.sql | 14 lượt, lịch sử mốc, attempt, refund, ticket và view |
| queries.sql | Q11–Q20: truy vấn tham chiếu và ví dụ join sai |
| verify.sql | Assertions về cohort, grain, outcome, tiền, timezone và biến thể |
| verify.py | Cụm PostgreSQL tạm; chạy cả SQL trong bài để phát hiện ví dụ không thực thi được |

`da_cohort`/`da_classified`/`da_cash` là view tạm hỗ trợ; đọc định nghĩa trong setup.sql, không coi chúng là bảng có sẵn trong ứng dụng.

## Kết quả chuẩn

| Kiểm tra | Kỳ vọng |
|---|---|
| Toàn fixture | 14 trips, 32 events, 10 payment attempts, 3 refund requests, 5 tickets |
| Cohort | T01–T08, 8 dòng / 8 trip_id |
| Theo thứ tự nhận | T01,T02,T03,T04,T05,T06,T08,T07 |
| Outcome | 3 cancelled_before_pickup, 4 picked_up, 1 unknown |
| Lý do huỷ | Có 2 trong 3; T04 thiếu lý do nhưng vẫn đếm huỷ |
| Tỷ lệ xác nhận / giới hạn | 37,5%; 37,5–50% nếu T05 là kết quả duy nhất chưa biết |
| Giá mọi lượt / giá lượt hoàn thành | 730.000 / 430.000 VND |
| Thu / hoàn / ròng của cohort đến snapshot | 450.000 / 50.000 / 400.000 VND |
| Ròng cohort settled trong tuần | 320.000 VND |
| Ròng tất cả ride/city1 settled trong tuần | 360.000 VND, thêm khoản T10 từ cohort trước |
| JOIN thô T01 | 2 attempts × 2 tickets = 4 dòng; sum amount sai 400.000 |
| Có ticket | T01/T02/T04; 3 lượt, không phải 5 |
| Chưa có attempt | T05; chưa có succeeded còn gồm T04/T08 |
| Tổng giá trung bình completed | 107.500 VND |
| Giá trung bình cả cohort | 91.250 VND |

Số nhận theo ngày 05–11/10: 2,1,1,1,1,1,1. Luỹ kế: 2,3,4,5,6,7,8. Baseline không có ngày rỗng; date spine bảo vệ biến thể khi một ngày không phát sinh. LAG ngày đầu là NULL, không phải 0.

Tiền ròng theo **ngày nhận của cohort**, đến snapshot: 100.000; 90.000; 0; 0; 100.000; 0; 110.000. Đây không phải dòng tiền settled mỗi ngày.

## Ca biên để review

- T07 nhận 11/10 23:59:59, hoàn thành/thu sang 12/10: vẫn trong cohort.
- T08 nhận 10/10, huỷ 12/10: tử số của cohort nhận, không thuộc huỷ xảy ra trong tuần.
- T09 nhận đúng đầu 12/10: loại; T10 nhận ngay trước đầu 05/10: loại.
- T11 food, T12 city2, T13 chưa nhận, T14 huỷ trước nhận: loại đúng lý do.
- Huỷ xảy ra trong tuần là T02/T04/T14; huỷ của cohort là T02/T04/T08. Cùng tổng 3 không chứng minh cùng tập.
- T05 accepted ở snapshot lâu hơn các lượt khác; giữ unknown và ghi ngoại lệ cần xác minh, không ép thành success.
- R07 pending 10.000 chưa trừ; R03 hoàn 30.000 ở kỳ sau. Không dùng current state tương lai để tái tạo snapshot cũ nếu chưa có lịch sử.

## Bài nộp DA-020

Tạo file trả lời riêng: danh sách cohort; báo cáo bảy ngày; tổng tuần; cầu nối giá–thu–hoàn; danh sách ngoại lệ; nhận định tối đa 150 từ; phiên bản nguồn và hướng dẫn chạy. Đối chiếu trước khi xem truy vấn tham chiếu.

Rubric: 20 điểm mỗi mục phạm vi/thời gian, grain/join, NULL/unknown, tiền/biên kỳ, tái lập/bàn giao. Từ 80/100 và không có lỗi trọng yếu mới coi hoàn thành. Không đổi mẫu số, tính failed/pending thành tiền hoặc gọi net cash là doanh thu.
