# Business Analysis qua dự án Tới Nơi

Khung biên tập v1 · 06/11/2026 · **8 phần × 8 bài = 64 bài**. Mỗi phần có 3 bài cơ bản, 3 bài trung bình và 2 bài nâng cao; bài thực hành cuối phần đã nằm trong giới hạn 8 bài. BA trong chương trình là Business Analysis / Business Analyst.

## Concept và đối tượng

Giữ concept antblog: tiếng Việt dễ hiểu, thuật ngữ Anh được giải thích khi xuất hiện, bắt đầu từ một vấn đề của hệ thống đặt xe và giao hàng hư cấu **Tới Nơi**, phân tích lựa chọn rồi tạo ra sản phẩm thực hành. Dành cho người mới làm BA, developer/tester muốn hiểu nghiệp vụ và BA muốn nâng kỹ năng phân tích. Không yêu cầu biết lập trình ở ba phần đầu.

Trục xuyên suốt: **vấn đề → bằng chứng → nhu cầu → phương án → yêu cầu → nghiệm thu → giá trị thực tế**. Câu chuyện đầu tiên là khách và tài xế khó tìm nhau ở điểm đón; các phần sau mở rộng sang giao hàng, phí huỷ, thanh toán và vận hành. BA không mặc định giải quyết mọi vấn đề bằng tính năng mới.

Tham khảo khái niệm của IIBA; đây là chương trình thực hành riêng của antblog, không phải syllabus luyện chứng chỉ hay bản chuyển thể BABOK. Các kỹ thuật và ví dụ được diễn giải độc lập. Không gán mã learning objective hoặc tuyên bố chuẩn hoá khi chưa kiểm nguồn.

## Cách học và tiêu chí mức độ

- **CB / beginner:** giải thích được khái niệm và hoàn thành mẫu với dữ kiện rõ ràng.
- **TB / intermediate:** tự phân tích tình huống nhiều bên, mô hình hoá ngoại lệ và liên kết đầu ra.
- **NC / advanced:** xử lý thiếu dữ liệu, xung đột, đánh đổi và đánh giá tác động xuyên hệ thống.

Mức độ là độ khó của bài trong chủ đề, không phải chức danh nghề nghiệp. Trong mỗi phần đọc lần lượt 1–8; phần sau kế thừa sản phẩm phần trước. Người có kinh nghiệm có thể bắt đầu từ bài 7, nhưng cần đọc hồ sơ tình huống và các đầu ra được nhắc tới. Đường học đầy đủ: P01 → P02 → P03 → P04 → P05 → P06 → P07 → P08.

Các bảng dưới đây là kế hoạch toàn bộ 64 bài. **BA-001 đến BA-003 đã viết trong source; 61 bài còn lại là kế hoạch, chưa xuất bản.** Tệp seed chỉ liệt kê bài đã có nội dung; không tạo bài rỗng hoặc liên kết đọc tới bài tương lai.

## P01 · Tư duy BA: hiểu vấn đề trước khi viết yêu cầu

Đầu vào: hồ sơ dự án Tới Nơi. Đầu ra phần: problem brief và quyết định discovery cho vấn đề điểm đón.

| ID | Bài | Mức | Làm được sau bài / sản phẩm thực hành |
|---|---|---|---|
| BA-001 | BA là gì? Từ lời đề nghị tính năng đến bài toán nghiệp vụ | CB | Phân biệt nhu cầu với giải pháp; lập problem brief một trang |
| BA-002 | BA, PO, PM, UX, Developer và Tester phối hợp thế nào? | CB | Xác định người quyết định; lập bảng trách nhiệm và decision log |
| BA-003 | Business requirement, stakeholder requirement và solution requirement | CB | Phân loại yêu cầu; nối mục tiêu tới yêu cầu và bằng chứng kiểm tra |
| BA-004 | BACCM: nhìn bài toán qua sáu khái niệm cốt lõi | TB | Phân tích change, need, solution, value, stakeholder, context trên cùng ca |
| BA-005 | Problem statement, mục tiêu và chỉ số thành công | TB | Định nghĩa baseline, mẫu số, thời gian đo và chỉ số bảo vệ |
| BA-006 | Scope, assumption, constraint và dependency | TB | Lập scope in/out và sổ giả định có người xác minh |
| BA-007 | Khi stakeholder đòi tính năng nhưng bằng chứng chưa đủ | NC | So sánh giả thuyết, chi phí trì hoãn và thử nghiệm có thể đảo ngược |
| BA-008 | Thực hành: bảo vệ đề xuất cải thiện điểm đón | NC | Trình bày problem brief, phương án không làm và tiêu chí dừng |

## P02 · Discovery và khai thác thông tin

Đầu vào: problem brief P01. Đầu ra phần: hồ sơ discovery có nguồn bằng chứng và câu hỏi chưa chốt.

| ID | Bài | Mức | Làm được sau bài / sản phẩm thực hành |
|---|---|---|---|
| BA-009 | Stakeholder map: ai chịu tác động, ai có quyền quyết định? | CB | Vẽ bản đồ khách, tài xế, CSKH, vận hành và sponsor |
| BA-010 | Phỏng vấn nghiệp vụ: hỏi để hiểu thay vì dẫn dắt | CB | Soạn guide và ghi chép phân biệt lời kể với diễn giải |
| BA-011 | Quan sát quy trình và đọc tài liệu hiện có | CB | Ghi observation log cho một ca đón khách và nguồn tài liệu |
| BA-012 | Workshop: chuẩn bị, điều phối và xác nhận kết quả | TB | Thiết kế agenda, đầu ra, người tham gia và biên bản xác nhận |
| BA-013 | Khảo sát và lấy mẫu: tránh kết luận từ nhóm dễ tiếp cận | TB | Chọn nhóm mẫu và nhận diện thiên lệch của phản hồi |
| BA-014 | Customer journey và service blueprint | TB | Liên kết trải nghiệm điểm đón với hoạt động hậu trường |
| BA-015 | Giải quyết yêu cầu xung đột giữa khách, tài xế và vận hành | NC | Ghi trade-off, quyền quyết định và ý kiến chưa đồng thuận |
| BA-016 | Thực hành: tổng hợp discovery từ dữ liệu mâu thuẫn | NC | Lập evidence matrix, mức tin cậy và kế hoạch xác minh tiếp |

## P03 · Mô hình hoá quy trình và quy tắc nghiệp vụ

Đầu vào: discovery P02. Đầu ra phần: bộ mô hình as-is/to-be của đặt xe và huỷ chuyến; thống nhất thuật ngữ với hồ sơ Tới Nơi trước khi đặt trạng thái mới.

| ID | Bài | Mức | Làm được sau bài / sản phẩm thực hành |
|---|---|---|---|
| BA-017 | As-is và to-be: mô tả hiện tại trước khi cải tiến | CB | Vẽ flow và nêu vấn đề ở từng bước |
| BA-018 | BPMN cơ bản: task, event, gateway và lane | CB | Mô hình hoá quy trình đón khách có trách nhiệm rõ ràng |
| BA-019 | Business rule: tách chính sách khỏi màn hình | CB | Viết rule có ID, nguồn, người sở hữu và ngày hiệu lực |
| BA-020 | Decision table: đặc tả phí huỷ không bỏ sót tổ hợp | TB | Kiểm tra chồng lấn, thiếu trường hợp và thứ tự ưu tiên |
| BA-021 | State model: trạng thái chuyến và chuyển trạng thái hợp lệ | TB | Lập bảng transition với guard, actor và ngoại lệ |
| BA-022 | Handoff, timer và luồng ngoại lệ trong BPMN | TB | Mô tả quá hạn, mất liên lạc và bàn giao CSKH |
| BA-023 | Kiểm tra nhất quán giữa quy trình, trạng thái và business rule | NC | Tìm mâu thuẫn bằng walkthrough liên mô hình |
| BA-024 | Thực hành: thay đổi chính sách huỷ trên nhiều luồng | NC | Lập impact map, mô hình mới và các quyết định còn mở |

## P04 · Đặc tả yêu cầu để cả nhóm hiểu và kiểm chứng được

Đầu vào: mô hình P03. Đầu ra phần: gói yêu cầu cho một lát cắt chức năng đủ để phát triển và nghiệm thu.

| ID | Bài | Mức | Làm được sau bài / sản phẩm thực hành |
|---|---|---|---|
| BA-025 | User story: vai trò, nhu cầu và giá trị | CB | Viết story, tránh chỉ đổi task kỹ thuật sang câu văn |
| BA-026 | Acceptance criteria và ví dụ Given–When–Then | CB | Viết tiêu chí quan sát được cho tình huống thường và lỗi |
| BA-027 | Use case: actor, precondition, main flow và alternative flow | CB | Đặc tả use case xác nhận điểm đón |
| BA-028 | Story mapping và chia lát cắt có giá trị | TB | Chia MVP theo hành trình thay vì chia tầng kỹ thuật |
| BA-029 | Wireframe và prototype để kiểm tra hiểu biết | TB | Ghi giả thuyết cần thử và kết quả quan sát prototype |
| BA-030 | Yêu cầu phi chức năng: từ “nhanh, an toàn” đến tiêu chí đo | TB | Đặt điều kiện tải, percentile, phạm vi và cách đo |
| BA-031 | BRD, SRS hay tài liệu gọn: chọn theo rủi ro quyết định | NC | Chọn mức tài liệu, version và cơ chế phê duyệt phù hợp |
| BA-032 | Thực hành: review một gói yêu cầu còn mơ hồ | NC | Sửa ambiguity và nối story, rule, model, acceptance criteria |

## P05 · Dữ liệu, API và tích hợp cho BA

Đầu vào: gói yêu cầu P04; hiểu bảng dữ liệu và JSON ở mức đọc. Đầu ra phần: hợp đồng nghiệp vụ tích hợp; bài không yêu cầu tự xây backend.

| ID | Bài | Mức | Làm được sau bài / sản phẩm thực hành |
|---|---|---|---|
| BA-033 | Domain glossary và mô hình dữ liệu khái niệm | CB | Phân biệt booking, trip, payment và chủ sở hữu dữ liệu |
| BA-034 | Data dictionary: định nghĩa trường, đơn vị và giá trị rỗng | CB | Đặc tả tiền, thời gian, toạ độ và nguồn dữ liệu |
| BA-035 | Đọc API request, response và mã lỗi bằng ngôn ngữ nghiệp vụ | CB | Đọc hợp đồng báo giá, chỉ ra câu hỏi cần làm rõ |
| BA-036 | SQL cho BA: kiểm chứng giả thuyết bằng truy vấn | TB | Tính đúng tỷ lệ trên dữ liệu tổng hợp, tránh đếm trùng join |
| BA-037 | Mapping dữ liệu và đối soát giữa các hệ thống | TB | Lập bảng mapping, source of truth và exception queue |
| BA-038 | Event và đồng bộ: người dùng thấy gì khi hệ thống xử lý chậm? | TB | Đặc tả trạng thái chờ, thông báo và kỳ vọng nhất quán |
| BA-039 | Idempotency, retry và giao dịch lỗi từ góc nhìn BA | NC | Nêu invariant không thu tiền hai lần và kịch bản phục hồi |
| BA-040 | Thực hành: hợp đồng hoàn tiền với cổng VíXanh giả lập | NC | Đặc tả timeout, callback trùng, đối soát và xử lý thủ công |

## P06 · Quản lý vòng đời yêu cầu và thay đổi

Đầu vào: bộ yêu cầu và phụ thuộc P04–P05. Đầu ra phần: baseline và hồ sơ thay đổi có khả năng truy vết.

| ID | Bài | Mức | Làm được sau bài / sản phẩm thực hành |
|---|---|---|---|
| BA-041 | Backlog và refinement: yêu cầu trưởng thành như thế nào? | CB | Lập backlog, câu hỏi mở và điều kiện sẵn sàng trao đổi |
| BA-042 | Ưu tiên với MoSCoW: Must có thật sự bắt buộc? | CB | Gắn mức ưu tiên với mục tiêu, thời hạn và lý do |
| BA-043 | Traceability: nối mục tiêu tới yêu cầu và test | CB | Lập ma trận truy vết hai chiều cho điểm đón |
| BA-044 | Baseline, version và approval: biết nhóm đã chốt điều gì | TB | Lưu lịch sử quyết định và thay đổi có người phê duyệt |
| BA-045 | Change request và impact analysis | TB | Phân tích ảnh hưởng tới dữ liệu, vận hành, test và rollout |
| BA-046 | Ưu tiên theo giá trị, chi phí và độ tin cậy | TB | So sánh phương án với giả định và phân tích độ nhạy |
| BA-047 | Quản lý phụ thuộc và yêu cầu xuyên nhiều đội | NC | Chốt owner, hợp đồng bàn giao và sequencing |
| BA-048 | Thực hành: đổi chính sách phí sát ngày phát hành | NC | Đề xuất hoãn, chia phạm vi hoặc triển khai có điều kiện |

## P07 · UAT, triển khai và đánh giá giải pháp

Đầu vào: baseline P06. Đầu ra phần: hồ sơ nghiệm thu và kế hoạch đo giá trị sau phát hành; nối với series Testing nhưng không thay thế chiến lược kiểm thử.

| ID | Bài | Mức | Làm được sau bài / sản phẩm thực hành |
|---|---|---|---|
| BA-049 | UAT là gì? BA và người dùng nghiệp vụ tham gia ra sao? | CB | Chọn người nghiệm thu và mục tiêu UAT |
| BA-050 | Thiết kế UAT scenario từ hành trình nghiệp vụ | CB | Chuẩn bị case, dữ liệu và expected outcome |
| BA-051 | Defect, change request hay câu hỏi chưa được chốt? | CB | Phân loại phản hồi dựa vào baseline và bằng chứng |
| BA-052 | Nghiệm thu có điều kiện và quyết định go/no-go | TB | Ghi rủi ro còn lại, owner và điều kiện phát hành |
| BA-053 | Transition requirement: migration, đào tạo và vận hành | TB | Lập checklist chuyển đổi, đối soát và hỗ trợ |
| BA-054 | Đo kết quả sau phát hành: dashboard có trả lời đúng câu hỏi? | TB | Định nghĩa metric và kiểm tra chất lượng dữ liệu đo |
| BA-055 | Pilot và A/B test: phân biệt tương quan với tác động | NC | Chọn đơn vị thử nghiệm, guardrail và xử lý nhiễu |
| BA-056 | Thực hành: tính năng đạt UAT nhưng chưa tạo giá trị | NC | Chẩn đoán adoption, giới hạn giải pháp và đề xuất bước tiếp |

## P08 · Phân tích chiến lược và năng lực BA nâng cao

Đầu vào: kết quả đo P07. Đầu ra phần: portfolio BA liên kết toàn bộ quyết định của một sáng kiến Tới Nơi.

| ID | Bài | Mức | Làm được sau bài / sản phẩm thực hành |
|---|---|---|---|
| BA-057 | Business case: vì sao nên đầu tư vào thay đổi này? | CB | Viết đề xuất gồm lợi ích, chi phí, rủi ro và phương án giữ nguyên |
| BA-058 | Capability map: tổ chức cần làm tốt việc gì? | CB | Phân biệt năng lực, quy trình và hệ thống |
| BA-059 | Risk và assumption log cho sáng kiến nghiệp vụ | CB | Ghi xác suất/tác động định tính, tín hiệu và owner |
| BA-060 | Gap analysis và roadmap thay đổi | TB | Nối hiện trạng, đích đến, khoảng trống và đợt triển khai |
| BA-061 | Facilitation và kể câu chuyện bằng bằng chứng | TB | Trình bày khuyến nghị cho các nhóm stakeholder khác nhau |
| BA-062 | AI hỗ trợ BA: kiểm chứng, nguồn và dữ liệu nhạy cảm | TB | Review bản tóm tắt/yêu cầu sinh tự động bằng dữ liệu giả |
| BA-063 | Phân tích trade-off khi mở rộng sang giao hàng nhiều điểm | NC | So sánh giá trị, năng lực vận hành và giới hạn tích hợp |
| BA-064 | Capstone: bảo vệ hồ sơ BA từ discovery đến đánh giá giá trị | NC | Bảo vệ quyết định, bằng chứng, traceability và bài học sau pilot |

## Quy chuẩn mỗi bài

1. Tình huống mở đầu trong Tới Nơi; nói rõ đâu là dữ liệu giả định.
2. Thông tin bài: ID, phần, mức, bài cần đọc trước, đầu ra và mục tiêu.
3. Khái niệm vừa đủ và thuật ngữ Anh tương ứng.
4. Phân tích ví dụ có sản phẩm mẫu; chỉ rõ lựa chọn và điều chưa biết.
5. Sai lầm thường gặp, cách phát hiện và sửa.
6. Bài tập độc lập kèm tiêu chí tự đánh giá hoặc gợi ý lời giải.
7. Câu hỏi ôn tập có giải thích; checklist áp dụng.
8. Nguồn tham khảo trực tiếp và điều hướng bài đã tồn tại.

Mỗi bài nhắm khoảng 10–18 phút đọc, bài thực hành 18–25 phút; đo lại theo nội dung thực tế, không kéo dài để đủ số. Ưu tiên bảng, mẫu tài liệu và sơ đồ khi giúp làm rõ quyết định. Không bắt buộc code trong bài về giao tiếp hoặc discovery.

## Hồ sơ thực hành và tính nhất quán

- Nền chung: `api/seed/content/du-an-mau-toi-noi.md`. Không tự sửa trạng thái chuyến, cước, kiến trúc hoặc rule hiện có để vừa ví dụ.
- Hồ sơ riêng series: `examples/toi-noi-ba/README.md`; số liệu discovery và mục tiêu đề xuất chỉ thuộc bài tập BA, không phải dữ liệu production hay yêu cầu đã duyệt của lab Testing.
- Phân biệt bốn trạng thái thông tin: **dữ kiện giả lập**, **giả thuyết**, **đề xuất**, **quyết định đã chốt trong bài tập**. Mọi chỉ số có tử số, mẫu số và cửa sổ thời gian.
- Dùng ID `BA-xxx` cho bài; `BR-`, `SHR-`, `FR-`, `NFR-`, `TR-` cho yêu cầu trong hồ sơ BA. Không đồng nhất với các mã `R-…` trong lab Testing.
- Bài mới thêm vào `api/seed/content` với category `business-analysis`, level hợp lệ và slug ASCII riêng. Bài kế hoạch ở tài liệu này chưa có URL công khai.

## Tiến độ và bước viết tiếp

| ID | Slug | Trạng thái source |
|---|---|---|
| BA-001 | `ba-la-gi-tu-tinh-nang-den-bai-toan` | Đã viết |
| BA-002 | `ba-po-pm-ux-dev-tester-phoi-hop` | Đã viết |
| BA-003 | `ba-phan-loai-yeu-cau-nghiep-vu` | Đã viết |

Tiếp theo viết BA-004–BA-008 để hoàn thành P01; sau đó thực hiện từng phần theo thứ tự. Trước khi coi là hoàn tất một phần, kiểm tra đủ 8 bài, phân bổ 3 CB / 3 TB / 2 NC và sản phẩm thực hành cuối phần truy vết được về các bài trước.

Lộ trình seed P01 chứa ba bài đã viết; P02–P08 chưa tạo lộ trình rỗng. Lệnh seed hiện đặt bài ở trạng thái published khi chạy, vì vậy việc có file source không đồng nghĩa đã lên website. Phiên làm nội dung này không chạy seed hay deploy.

## Nguồn định hướng

- [IIBA — Defining Business Analysis](https://www.iiba.org/knowledgehub/the-business-analysis-standard/2-understanding-business-analysis/2-1-defining-business-analysis/): định nghĩa BA.
- [IIBA — BACCM](https://www.iiba.org/knowledgehub/the-business-analysis-standard/2-understanding-business-analysis/2-2-a-model-for-effective-analysis-baccm/): sáu khái niệm cốt lõi.
- [IIBA — Needs and Solutions, Requirements and Designs](https://www.iiba.org/professional-development/knowledge-centre/articles/needs-and-solutions/): quan hệ nhu cầu, giải pháp, yêu cầu và thiết kế.

Nguồn chuyên sâu về BPMN, Scrum, yêu cầu chất lượng và thử nghiệm cần đối chiếu tài liệu gốc khi viết từng bài; danh sách chủ đề không ngụ ý đã kiểm mọi phiên bản tiêu chuẩn.
