# SYS-15 — User Acceptance Test: nghiệm thu theo nhu cầu nghiệp vụ

Một hành trình chạy hết không tự trả lời được câu hỏi “người dùng có thể dùng hệ thống này để làm công việc của họ chưa?”. UAT tạo bằng chứng để đại diện nghiệp vụ quyết định chấp nhận phạm vi bàn giao dựa trên tiêu chí đã thống nhất và rủi ro còn lại.

Bài trước: [Recovery Test](14-recovery-test.md). Bài tiếp theo: Deployment Test. [Hồ sơ thực hành UAT](15-acceptance-kit.md) · [Lab](../../examples/toi-noi-testing/articles/system-testing-uat/README.md) · [Báo cáo](15-test-report.md).

## 1. Mục tiêu và kiến thức nền

Sau bài này, bạn có thể chuyển một nhu cầu thành acceptance criteria đo được, tổ chức kịch bản liên vai trò, phân biệt lỗi với yêu cầu thay đổi, lập hồ sơ bằng chứng và đề xuất quyết định nghiệm thu có phạm vi rõ ràng.

Cần đọc SPEC, SYS-03, SYS-05 và giới hạn của SYS-12/SYS-14. Lab API cần Node.js và dependency của `examples/toi-noi-testing`. Buổi UAT qua UI cần đại diện khách, tài xế và nghiệp vụ vận hành; chưa có người thật tham gia trong lần thực hiện bài này.

## 2. Khái niệm và các ranh giới

Acceptance testing xem xét nhu cầu, yêu cầu và quy trình nghiệp vụ để hỗ trợ quyết định chấp nhận. UAT tập trung vào khả năng đáp ứng nhu cầu sử dụng; người có thẩm quyền nghiệp vụ ra quyết định dựa trên evidence. Việc phối hợp business analyst/product owner với tester cần bắt đầu từ lúc làm rõ tiêu chí. [ISTQB Glossary](https://glossary.istqb.org/en_US/term/acceptance-testing/1), [ISTQB CT-AcT](https://istqb.org/certifications/certified-tester-acceptance-testing/).

| Hoạt động | Câu hỏi chính | Đầu ra |
| --- | --- | --- |
| System test | Hệ thống thực hiện yêu cầu như thế nào? | Kết quả kiểm thử kỹ thuật/nghiệp vụ |
| UAT | Phạm vi bàn giao có đáp ứng công việc đã thống nhất? | Evidence và quyết định của đại diện được giao thẩm quyền |
| Usability study | Người dùng thực hiện công việc có thuận lợi, dễ hiểu? | Quan sát, vấn đề tương tác, insight |
| Operational acceptance | Có thể vận hành, giám sát, hỗ trợ và phục hồi? | Runbook, diễn tập và tiêu chí vận hành |
| Deployment test | Bản phát hành được triển khai và hoạt động đúng? | Bằng chứng cấu hình, migration, smoke và rollback |

UAT có thể sử dụng automation, nhưng một suite xanh không thay người ra quyết định. Demo là trình diễn; UAT cần tiêu chí, dữ liệu, actual result, evidence và cách xử lý sai lệch. Alpha/beta, nghiệm thu hợp đồng và quy định có mục đích/phạm vi riêng; không gọi tất cả là UAT chỉ vì xảy ra gần ngày release.

## 3. Từ nhu cầu đến acceptance criteria

Nhu cầu: “Là khách, tôi muốn chuyến giữ đúng giá đã đồng ý để không bị tính lại ngoài dự kiến”. Tiêu chí dựa trên R-QUOTE-04: chuyến dùng snapshot báo giá hợp lệ; kết quả khách xem sau hoàn thành vẫn có cùng fare. Không viết tiêu chí mơ hồ như “giá phải tốt” hoặc “thanh toán chạy ổn”.

Ví dụ Given–When–Then cho UA-01:

```gherkin
Given khách C001 có báo giá xe máy SG 5 km lúc 09:00, không mưa, tổng 35.000 đ
And chọn tiền mặt
When khách đặt chuyến và gửi lại cùng yêu cầu
And tài xế D101 nhận, đến đón, đón khách, bắt đầu và hoàn thành
Then khách có đúng một chuyến completed với giá 35.000 đ
And hệ thống ghi thanh toán cash, không gọi ví điện tử
And tài xế trở lại online
```

`cash` là phương thức/trạng thái trong lab, không chứng minh tiền mặt thực tế đã trao tay. Given–When–Then giúp làm rõ điều kiện và kết quả; không bắt buộc dùng Cucumber hay biến từng click thành một tiêu chí nghiệp vụ.

Mỗi tiêu chí cần ID, nguồn yêu cầu, vai trò xác nhận, phạm vi dữ liệu và điều kiện đạt. Nếu business muốn báo trước phí huỷ trước khi bấm xác nhận, phải ghi yêu cầu đó riêng: SPEC hiện mô tả phí/lý do trả về sau thao tác, không tự suy có màn hình preview phí.

## 4. Phạm vi nghiệm thu của bài

Phạm vi mẫu là **lab đặt xe máy, SG, khách C001, tài xế D101**, báo giá và tiền mặt; bổ sung review tình huống ví lỗi. Không phải nghiệm thu toàn sản phẩm Tới Nơi. Giao đồ ăn, bưu kiện, tất cả thành phố/loại xe, bản đồ thật, token thật, ledger/webhook và support console chưa được bộ bài này bao phủ.

| Scenario | Nguồn | Kết quả nghiệp vụ cần xem |
| --- | --- | --- |
| UA-01 | R-QUOTE-04, R-IDEM-02, R-STATE-06, R-PAY-01 | Đặt–hoàn thành, giữ giá, không tạo trùng |
| UA-02 | R-CANCEL-01/02/03/06/07 | Huỷ trước 2 phút miễn; đúng 2 phút thu 10.000 đ; biết lý do |
| UA-03 | R-QUOTE-01/03 | Báo giá đúng 5 phút hết hạn, không tạo chuyến |
| UA-04 | R-PAY-04/06, SYS-14 | Nhận biết khoản thu chưa xác nhận và cách chuyển xử lý |
| UA-05 | R-UI-01/02/03/04 | Người dùng hiểu giá, trạng thái, lỗi và thao tác tiếp theo |
| UA-06 | Phạm vi mở rộng vận hành | Có người chịu trách nhiệm xử lý incident/tồn đọng |

UA-06 là đề xuất tiêu chí bàn giao vận hành, chưa phải chức năng đã có trong SPEC. UA-04 không được coi là end-to-end support scenario khi chưa có công cụ đối soát. Trong [kit](15-acceptance-kit.md), các phiên người dùng đều giữ trạng thái NOT RUN hoặc BLOCKED, không điền PASS từ kết quả API.

## 5. Vai trò và cách tổ chức phiên

Business owner chốt tiêu chí và phạm vi được chấp nhận. Đại diện khách/tài xế thực hiện công việc, mô tả điều họ thấy và hiểu. QA chuẩn bị dữ liệu, ghi bằng chứng và hỗ trợ tái hiện. Developer điều tra lỗi, không tự quyết định thay business rằng sai lệch “không đáng kể”. Người vận hành xác nhận quy trình hỗ trợ và tồn đọng thuộc trách nhiệm của họ.

Một người có thể giữ nhiều vai trò trong nhóm nhỏ, nhưng biên bản phải ghi rõ người đang ký với trách nhiệm nào. Không tự điền tên hoặc chữ ký cho người chưa tham gia.

Trước phiên, đóng băng build và bộ tiêu chí; chọn dữ liệu đại diện; xác nhận môi trường và giới hạn; chạy smoke để tránh dành phiên người dùng cho lỗi khởi động. Đưa mục tiêu công việc trước, để người dùng thao tác; nếu QA phải hướng dẫn, ghi rõ mức hỗ trợ. Không sửa dữ liệu ngầm giữa phiên để làm luồng “đẹp”.

Trong phiên, ghi expected/actual và evidence theo scenario, không chỉ ghi “OK”. Khi gặp lỗi cản trở, đánh dấu BLOCKED cho phần chưa thực hiện; tách khỏi FAIL của một kết quả đã kiểm và không đạt. Sau phiên, triage sai lệch, chốt retest, cập nhật quyết định trên đúng build.

## 6. Thực hành tự động hỗ trợ

Từ thư mục lab:

```powershell
npm ci
npm test -- articles/system-testing-uat
npm run typecheck
```

Suite có bốn case: UA-01, UA-02 với hai mốc, UA-03. API Express kết nối PGlite; đồng hồ cố định, reset seed từng test; đóng DB cuối suite. Giá expected 35.000 được tính độc lập: `12.000 + 4.500 × 5 = 34.500`, làm tròn lên nghìn thành 35.000.

UA-01 kiểm toàn bộ chuỗi trạng thái, dữ liệu khách đọc lại, một chuyến, tài xế online và không gọi ví. UA-02 kiểm phí/lý do, giá trị lưu và giải phóng tài xế; khoản cash không gọi cổng điện tử. UA-03 kiểm lỗi QUOTE_EXPIRED, không tạo trip hoặc đánh dấu dùng quote.

Đây là evidence kỹ thuật cho tiêu chí, không kiểm người dùng có hiểu màn hình. Không chạy trình duyệt trong suite này. Header x-user là mô phỏng danh tính. Không kết luận usability, xác thực, hiệu năng hoặc khả năng vận hành production từ 4 test.

## 7. Thực hành phiên người dùng với kit

Khởi động lab theo [README hiện có](../../examples/toi-noi-testing/README.md), dùng trang khách `/` và tài xế `/driver.html`. Chọn backend, dữ liệu và đồng hồ theo cấu hình lab, ghi lại trước phiên. Người điều phối chuẩn bị clock cho UA-02/03 bằng móc test đã có; không yêu cầu người dùng thao tác SQL hoặc đo biên 2 phút bằng tay.

Sao chép worksheet trong kit cho mỗi lượt chạy. Khách và tài xế thực hiện cùng một chuyến; ghi trip ID để liên kết evidence. Chụp phần kết quả cần thiết bằng dữ liệu giả, không đưa thông tin cá nhân vào kho. Lưu phiên bản trình duyệt và các bước QA đã trợ giúp.

Sau mỗi scenario, hỏi người thực hiện mô tả kết quả và bước tiếp theo. Nếu API đúng nhưng thông báo làm khách nghĩ bị thu trùng, ghi vấn đề UI/diễn giải riêng. Khi tiêu chí chưa mô tả điều này, business cần làm rõ trước khi dùng làm tiêu chí bắt buộc mới; giữ lịch sử thay đổi.

## 8. Triage: defect, thay đổi hay giới hạn môi trường?

| Tình huống | Phân loại ban đầu | Việc tiếp theo |
| --- | --- | --- |
| Đúng 2 phút nhưng miễn phí | Sai tiêu chí hiện hành | Bug, tái hiện, sửa và retest |
| Khách yêu cầu thêm màn hình preview phí | Yêu cầu mới nếu chưa có trong basis | Đánh giá phạm vi và cập nhật tiêu chí |
| Không gọi được ví thật vì lab dùng fake | Giới hạn môi trường | Chuyển scenario sang môi trường phù hợp, không ghi PASS |
| Mất phản hồi, ví thu nhưng DB failed | Rủi ro đã biết ở SYS-14 | Yêu cầu quy trình đối soát/owner, xem xét phạm vi chấp nhận |
| QA phải chỉ nút mới tiếp tục được | Evidence usability | Ghi hỗ trợ và tác động nghiệp vụ, đối chiếu SYS-12 |

Severity đo tác động; priority quyết định thứ tự xử lý. Đừng đóng bug chỉ vì đã có workaround: ghi chi phí, người thực hiện, điều kiện áp dụng và rủi ro còn lại. Retest sửa lỗi và regression luồng liên quan trên build mới; evidence build cũ chỉ còn giá trị lịch sử.

## 9. Quyết định và chỉ số

Ba lựa chọn mẫu: chấp nhận, chấp nhận có điều kiện, chưa chấp nhận. Chấp nhận có điều kiện phải ghi phạm vi, ngoại lệ, owner, hạn xử lý và điều kiện vô hiệu hóa quyết định. Nó không phải cách đổi lỗi chặn thành PASS. Gate cụ thể do dự án thống nhất, không tự đặt một tỷ lệ pass để thay quyết định.

Chỉ số nên có: tiêu chí đã có evidence/tổng tiêu chí trong scope; scenario pass/fail/blocked/not run; mức bao phủ từng vai trò; số rủi ro chưa có owner; retest còn thiếu. Ví dụ 4/4 automated checks pass không thể báo “100% UAT đạt” khi sáu scenario người dùng chưa chạy.

Đề xuất gate cho bài tập: đủ evidence các scenario bắt buộc; không còn lỗi chặn theo phân loại đã thống nhất; mọi ngoại lệ có owner và quyết định; người được giao thẩm quyền xác nhận build/phạm vi. Hiện chưa đủ gate người dùng: báo cáo chỉ xác nhận supporting checks, không có sign-off.

## 10. Mở rộng nâng cao

**ATDD:** thống nhất ví dụ trước khi triển khai, dùng ví dụ làm acceptance checks. Cần giữ ngôn ngữ nghiệp vụ, tránh để fixture kỹ thuật quyết định thay yêu cầu.

**Risk-based UAT:** ưu tiên tiền, huỷ, quyền truy cập, xử lý gián đoạn và vai trò ít được kiểm. Kết hợp biến thể dữ liệu bằng bảng quyết định; không nhân mọi tổ hợp nếu không có lý do rủi ro. Nêu rõ nhóm bị loại khỏi phạm vi.

**Nghiệm thu nhiều bên:** nếu khách, tài xế và vận hành có mục tiêu xung đột, ghi ai sở hữu quyết định từng tiêu chí. Ví dụ hoàn thành chuyến khi ví lỗi giúp tài xế tiếp tục làm việc nhưng tạo khoản cần đối soát; không lấy lợi ích một bên để che rủi ro bên kia.

**Continuous delivery:** có thể tự động thu evidence và áp dụng cơ chế phê duyệt đã được tổ chức ủy quyền. Vẫn phải truy được build, tiêu chí và quyết định; pipeline xanh không tự sinh thẩm quyền nghiệm thu.

**Release dossier:** nối bằng chứng security, performance, compatibility, usability, integrity và recovery với scope UAT. Không gom mọi kết quả lịch sử thành một chứng nhận hiện tại: ghi build, ngày, môi trường và thay đổi nào làm evidence cần chạy lại. SYS-16 sẽ kiểm bản triển khai cụ thể.

## 11. Bài tập và gợi ý

1. Viết tiêu chí cho khách nhìn thấy lý do phí huỷ. Gợi ý: tách tính đúng số tiền khỏi khả năng hiểu; chỉ rõ màn hình, thời điểm và bằng chứng quan sát.
2. Mở rộng UA-01 sang VíXanh. Gợi ý: kiểm tiền thực thu tại fake, ghi rõ chưa có provider thật; thêm tình huống mất phản hồi và owner đối soát.
3. Điền một biên bản “chưa chấp nhận” cho phạm vi ví. Gợi ý: viện dẫn SYS-14, nêu khoản chưa rõ, yêu cầu evidence để đổi quyết định, không giả chữ ký.
4. Build mới sửa giao diện phí huỷ, cần retest gì? Gợi ý: UA-02/05 trên UI, kiểm số tiền không đổi, accessibility liên quan; đánh giá impact trước khi chọn regression.
5. Tạo traceability cho food/parcel. Gợi ý: lấy đúng R-FOOD/R-PARCEL trong SPEC, thêm vai trò và oracle; bốn test hiện tại không đủ chứng minh phạm vi mới.

## 12. Tài liệu tham khảo

Tra cứu ngày 13/11/2026: [ISTQB — Acceptance testing glossary](https://glossary.istqb.org/en_US/term/acceptance-testing/1), [CT-AcT](https://istqb.org/certifications/certified-tester-acceptance-testing/), [Acceptance Testing syllabus v1.0](https://www.istqb.org/wp-content/uploads/2024/11/ISTQB-CT-AcT_Syllabus_v1.0_2019.pdf). Nguồn nội bộ: SPEC và báo cáo SYS-12/13/14. Các mẫu worksheet/gate trong bài là thiết kế thực hành cho lab, không phải biểu mẫu bắt buộc của ISTQB.
