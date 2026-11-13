# SYS-11 — Compatibility Test: cùng chức năng trên những cấu hình nào?

Trạng thái: bản nháp hoàn chỉnh để review; có ma trận thực thi và lab. Bài trước: [Security Test – Pen test](10-security-pentest.md). [Lộ trình](../system-testing-roadmap.md).

## 1. Tình huống và mục tiêu

Trang đặt xe chạy đúng trên laptop của developer nhưng nút đặt bị khuất trên màn hình hẹp, hoặc định dạng tiền đổi khi browser dùng locale khác. Backend vẫn đúng mà khách không hoàn thành thao tác. Compatibility testing cần xác định cấu hình được hỗ trợ và kiểm hành vi quan trọng trên từng cấu hình đại diện.

Bài dành cho người đã biết System Test và Playwright cơ bản. Sau bài, bạn có thể lập support matrix theo rủi ro, phân biệt browser/engine/device/emulation, chạy cùng case trên nhiều projects, ghi phiên bản thực tế và hiểu giới hạn của phép kiểm layout tự động.

## 2. Khái niệm và phạm vi

Kiểm thử tương thích đánh giá khả năng hoạt động trong hoặc với những môi trường, nền tảng và phiên bản được chọn. Trong thực tiễn web thường xét browser, OS, viewport, input, locale, timezone; với hệ thống khác còn có API version, schema, thiết bị hoặc phần mềm cùng tồn tại. Cần nêu cách dùng thuật ngữ và phạm vi, không gộp mọi thuộc tính chất lượng thành một checklist browser.

| Trục | Ví dụ | Chưa chứng minh chỉ bằng trục đó |
|---|---|---|
| Browser/engine | Chrome, Edge, Firefox, WebKit | Hai browser Chromium không thay engine khác |
| OS/device | Windows, macOS, Android, thiết bị cụ thể | User agent không biến máy Windows thành điện thoại |
| Viewport/input | 1280×800; 390×844, touch | Màn hình hẹp không mô phỏng bàn phím ảo/CPU/GPU thật |
| Locale/timezone | vi-VN, en-US; VN, New York | Đổi locale không có nghĩa UI đã được dịch |
| Version contract | API cũ/mới, schema migration | Browser test không chứng minh backward compatibility API |

Responsive testing tập trung bố cục theo kích thước, còn cross-browser xét hành vi giữa browser; hai việc liên quan nhưng không thay nhau. Accessibility/usability cũng khác: nút click được chưa chắc có tên truy cập đúng hay người dùng hiểu cách dùng.

## 3. Chọn support matrix thay vì nhân mọi tổ hợp

Trong dự án thật, dùng analytics, khách hàng mục tiêu, hợp đồng hỗ trợ, mức rủi ro và chi phí để chọn. Ghi rõ cấu hình bắt buộc, best-effort và chưa hỗ trợ. Không hứa “mọi thiết bị” khi chỉ có vài test. Khi browser auto-update, lưu version từng lượt; tên Chrome đơn lẻ chưa đủ tái hiện.

Ma trận bài tập tận dụng hai browser đã cài trên máy và một cấu hình mô phỏng hẹp:

| Project | Browser | Viewport | Locale / timezone | Input |
|---|---|---|---|---|
| chrome-desktop-vi | Chrome | 1280×800 | vi-VN / Asia/Ho_Chi_Minh | click |
| edge-desktop-en | Edge | 1280×800 | en-US / America/New_York | click |
| chrome-narrow-touch | Chrome | 390×844 | vi-VN / Asia/Ho_Chi_Minh | tap ở các nút; mobile/touch emulation |

Đây là matrix thử nghiệm, không thay support policy của blog hay sản phẩm. Chrome và Edge đều Chromium; không có Firefox, Safari/WebKit, Android hoặc iPhone thật. Project Edge đổi cùng lúc browser, locale và timezone để mở rộng cấu hình đại diện, nhưng nếu fail thì cần tách từng yếu tố để xác định nguyên nhân.

## 4. Oracle nào phải giữ và cái gì được phép khác?

SPEC R-UI-04 yêu cầu số tiền theo định dạng Việt Nam kèm “ đ”. Vì vậy browser en-US vẫn phải hiện `35.000 đ`; UI này chưa có yêu cầu dịch theo locale browser. Cước nghiệp vụ lấy giờ server VN, không được thay bởi timezone client. Test kiểm `quote-peak=Không` khi server 09:00 VN, nhưng chưa kiểm mọi biên giờ hay cách hiển thị giờ hết hạn.

R-UI-01 yêu cầu báo giá/đặt/huỷ và vô hiệu quote khi thay input. R-UI-02 liên quan trang tài xế nhưng chưa được đưa vào matrix bài này. Tiền món mặc định 60.000 + phí giao 35.000 = 95.000 cho food không voucher. Các expected này đến từ SPEC, không copy text do browser đầu tiên trả về rồi coi đó là chuẩn.

Khác biệt font native, viền control hoặc render select giữa browser có thể chấp nhận nếu vẫn đáp ứng yêu cầu. Không yêu cầu pixel giống tuyệt đối khi chưa có design baseline. Ngược lại horizontal overflow hoặc nút không thao tác được có thể là lỗi dù screenshot nhìn gần giống.

## 5. Thiết kế cases và dữ liệu

Tiền điều kiện: seed C001, SG bike, giờ 09:00 VN 04/11/2026, không mưa, PGlite. Mỗi execution reset DB và giờ; một worker để tránh reset chéo. Context được Playwright tạo/đóng riêng, server riêng cổng 3116 không reuse process khác.

| Case | Hành động | Expected | Rủi ro |
|---|---|---|---|
| CP-01 | Nhập 5 km → báo giá → đặt → huỷ searching | 35.000 đ, không cao điểm, tìm tài xế rồi đã huỷ, phí 0, nút huỷ disabled | Input/control, tiền, lifecycle UI |
| CP-02 | Chọn food → báo giá 5 km → quay về ride → nhập 0,1 km → báo giá | Field món hiện/ẩn đúng, vehicle disabled/enabled, 95.000 đ, quote cũ vô hiệu, lỗi quãng đường | Dynamic layout, select/change event, negative path |

Cả hai case kiểm document scrollWidth không vượt clientWidth quá 1 CSS pixel ở các trạng thái được chọn. Đây là assertion không tràn ngang, không phải visual review toàn trang; nó không phát hiện mọi overlap, font clipping hoặc contrast. Không khẳng định responsive hoàn chỉnh chỉ từ một con số.

## 6. Thực hành Playwright projects

Config: `examples/toi-noi-testing/compatibility.config.ts`. Spec: `articles/system-testing-compatibility-test/e2e/compatibility.spec.ts`. Từ thư mục lab, cần Chrome/Edge cài sẵn và Node >=22.12:

```sh
npm ci
npx playwright test --config compatibility.config.ts
npm run typecheck
```

Expected sáu executions: hai case nhân ba projects. Không dùng số lượng project để gọi là ba browser: chỉ có hai browser và một biến thể Chrome. Chạy riêng một cấu hình để điều tra:

```sh
npx playwright test --config compatibility.config.ts --project=edge-desktop-en
```

Các nút trong narrow-touch dùng `tap()` thật trong context touch-enabled; desktop dùng `click()`. Điền input bằng fill và chọn select bằng selectOption vẫn là automation, chưa mô phỏng bàn phím ảo hoặc native picker trên điện thoại. Mobile flag/viewport không đổi OS và không chứng minh app hoạt động trên iOS.

Script attach project, browser version và use config vào test result, đồng thời in metadata. Trace/screenshot chỉ lưu khi fail dưới reports/compatibility. Không có snapshot baseline hoặc kiểm ảnh thủ công trong lượt này. Config mặc định của lab cũng có thể tìm thấy spec nhưng không tạo matrix này; luôn ghi đúng lệnh để báo cáo coverage.

## 7. Kết quả và tiêu chí kết thúc

Xem [báo cáo thực đo](11-test-report.md) cho browser version, số executions và thời gian. Khi đánh giá, phân biệt case không chạy do thiếu browser với case fail nghiệp vụ. “Skipped Safari” không được tính là Safari pass.

Entry: matrix xác định, browser có sẵn, server độc lập và dữ liệu chuẩn. Exit bài tập: sáu executions pass, typecheck đạt và khoảng trống ghi rõ. Exit release cần theo support policy thực tế, có thể đòi engine/OS/thiết bị thật ngoài matrix hiện tại.

| Failure | Điều tra tiếp |
|---|---|
| Chỉ Edge/en-US fail | Chạy Chrome/en-US và Edge/vi-VN để tách browser/locale |
| Chỉ narrow-touch không click được | Viewport, scroll, overlay, touch event và target size |
| Giá đổi theo timezone client | Server clock, request và cách tính/render thời gian |
| Overflow xuất hiện sau đặt | ID dài, min-width, flex wrapping, fieldset sizing |
| Browser không launch | Môi trường/installation, không báo là chức năng hỏng |

Defect cần project, browser version, OS, viewport CSS pixels, device scale nếu có, locale/timezone, input method, bước tái hiện và evidence. Với bug chỉ xuất hiện trên thiết bị thật, giữ tên model/OS và điều kiện bàn phím/orientation; không thay bằng ảnh Chrome hẹp rồi coi đã tái hiện.

## 8. Mở rộng nâng cao

### 8.1. Nhiều engine và thiết bị thật

Bổ sung Firefox và WebKit theo support matrix, sau đó kiểm Safari thực trên môi trường Apple nếu sản phẩm hỗ trợ. WebKit automation không đại diện mọi bản Safari/iOS. Kiểm điện thoại thật cho bàn phím ảo, viewport khi bàn phím bật, safe-area, touch scrolling, permission prompts và hiệu năng phần cứng. Không cần mọi test trên mọi máy; chọn hành trình rủi ro cao và giữ hồ sơ coverage.

### 8.2. Pairwise và tổ hợp rủi ro

Browser×OS×viewport×locale×input có thể rất lớn. Pairwise giúp giảm tổ hợp nhưng không bảo đảm bắt tương tác nhiều yếu tố. Giữ ca chủ đích cho rủi ro như mobile Safari + keyboard + fixed footer. Matrix ba dòng của bài được chọn thủ công, chưa có chứng minh pairwise coverage; không gắn nhãn pairwise chỉ vì có nhiều cấu hình.

### 8.3. Backward compatibility API và dữ liệu

Client cũ có thể gửi payload cũ hoặc đọc response cũ khi server nâng cấp. Kiểm required/optional fields, enum mới, default, version negotiation và migration dữ liệu. Một UI mới chạy trên Chrome/Edge không chứng minh client mobile phiên bản cũ còn dùng được. Với rolling deployment, kiểm app cũ/mới cùng schema theo cửa sổ tương thích.

### 8.4. Locale, timezone và định dạng

Input số có dấu phẩy/chấm, Unicode, font fallback, text dài và ngày ở DST có thể làm lỗi xuất hiện khác nhau. Phân biệt locale hiển thị và quy tắc nghiệp vụ; ví dụ server VN dùng UTC+7 còn người dùng ở New York có thể thấy giờ theo locale client nếu UI thiết kế như vậy. Cần expected được thống nhất, không tự coi mọi khác biệt ngày giờ là bug.

### 8.5. Visual regression có baseline

Nếu bổ sung screenshot comparisons, tạo baseline theo project/font/OS và review khác biệt có chủ đích. Mask vùng biến động chỉ khi không che rủi ro cần kiểm. Không update mọi snapshot để làm xanh. Visual test hỗ trợ nhưng không thay assertion hành vi/API; ảnh đẹp vẫn có thể gửi sai request.

### 8.6. Duy trì support policy

Theo dõi analytics và chi phí hỗ trợ, review matrix khi browser/OS/dependency thay đổi. Phiên bản “latest” dịch chuyển; pin hoặc ghi version để tái hiện. Khi bỏ hỗ trợ một môi trường, cần quyết định sản phẩm và thông báo phù hợp, không chỉ xóa project test vì nó fail.

## 9. Anti-pattern và giới hạn

Chỉ resize desktop rồi báo “Android/iOS pass”; coi Chrome/Edge là hai engine độc lập; kiểm đúng viewport nhưng dùng cùng storage bẩn; đổi locale mà kỳ vọng UI tự dịch; chỉ assert không lỗi console — đều thiếu bằng chứng cho tương thích chức năng.

Lab chưa kiểm Firefox/Safari, mobile thật, orientation, keyboard ảo, zoom, dark mode, high contrast, screen reader, mọi viewport hoặc API version cũ. Không đánh giá usability hay accessibility đầy đủ. Các phép kiểm horizontal overflow chỉ ở trạng thái đã chọn, không thay review thiết kế tổng thể.

## 10. Bài tập và đáp án định hướng

1. **Cơ bản:** sáu executions trên ba projects có phải sáu browser đã test không? Không: hai case trên ba cấu hình, chỉ hai browser cùng họ Chromium.
2. **Trung bình:** Edge/en-US fail định dạng tiền. Đáp án cần đối chiếu R-UI-04 và tách trục locale/browser trước khi kết luận engine lỗi; số tiền nghiệp vụ vẫn phải đúng.
3. **Trung bình:** thêm viewport ngang 844×390. Kiểm hành vi và overflow nhưng không gọi là xoay điện thoại thật vì chưa kiểm sensor/keyboard/native UI.
4. **Nâng cao:** lập matrix cho app hỗ trợ iOS/Android và hai API versions. Đáp án cần trục device/OS/engine/client version, ưu tiên theo usage/risk, phân biệt emulated với real, ghi phần chưa kiểm.
5. **Tự kiểm:** scrollWidth đúng có chứng minh text không chồng nhau không? Không. Screenshot giống có chứng minh nút đặt dùng được không? Không, cần hành động và oracle kết quả.

## 11. Nguồn tham khảo

- [Playwright Projects](https://playwright.dev/docs/test-projects): nhóm cấu hình, browser channels và chọn project khi chạy.
- [Playwright Emulation](https://playwright.dev/docs/emulation): viewport, input, locale/timezone và giới hạn emulation theo cấu hình.
- SPEC R-UI-01/04, R-FARE và R-FOOD: test basis chức năng/định dạng tiền; config/spec bài là nguồn chính xác của matrix đã thực thi.

Nguồn đối chiếu ngày 13/11/2026. Bài kế tiếp: Usability Test — đánh giá khả năng hoàn thành nhiệm vụ bằng quan sát người dùng, phân biệt với automation và accessibility checks.
