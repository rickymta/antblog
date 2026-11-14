---
slug: location-bai-toan-quang-duong-khach-va-tai-xe
title: "Quãng đường của hai người: bài toán thật của service định vị"
summary: "Bài 01: ghim sai phía đường, tài xế «gần nhất» đi vòng hai cây số. Vì sao tối ưu cho một bên thì đều sai, và các chỉ số walk_m, deadhead_m, meet_gap_s đặt khách lẫn tài xế lên cùng một cân."
category: location-routing
level: intermediate
tags: [toi-noi, location, rust, dispatch, pickup, metrics]
language: vi
---

Khách CU-001 ghim điểm đón ở số nhà bên vỉa hè phía bắc của một đại lộ có dải phân cách cứng. Tài xế DR-001 đang chạy trên làn phía nam, cách đó một cây số về phía tây. Rồi màn hình khách hiện «tài xế đã tới»: xe đang đứng ngay đối diện, cách khách một dải phân cách và hai làn đường. Cả hai nhìn thấy nhau mà không ai sang được. Chỗ quay đầu gần nhất nằm 600 m về phía đông, và để về lại đúng chỗ khách đứng, tài xế còn phải đi ngược lại 600 m nữa.

Hai người ở gần nhau trên bản đồ nhưng xa nhau trên mặt đường. Service định vị của Tới Nơi tồn tại để xử lý đúng khoảng cách đó: nhận một cái ghim, một đội xe, một mạng đường, rồi trả lời câu hỏi làm sao để **tổng quãng đường và thời gian của cả khách lẫn tài xế** nhỏ nhất. Bài này là bài mở đầu: bài này chưa có dòng code nào, chỉ đặt bài toán, định nghĩa các chỉ số và vạch bản đồ cho cả loạt 64 bài. Đây là bài 01 của lộ trình **Nền tảng định vị** qua [Tới Nơi](/articles/du-an-mau-toi-noi).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Có một khách cần đón. Tài xế A cách ghim 1.000 m theo đường chim bay, tài xế B cách 1.400 m. Hệ thống nên gán chuyến cho ai?</p>
<details><summary>Xem đáp án</summary><p><b>Chưa đủ dữ kiện để nói.</b> Đường chim bay không phải đường xe đi. Nếu A bị dải phân cách hoặc đường một chiều chặn thì quãng đường thật của A có thể dài hơn B. Ở phần «Tài xế gần nhất đo khoảng cách bằng gì» bên dưới, một ví dụ tự dựng cho A đi 2.200 m còn B đi 1.800 m, và kết quả đo hệ số vòng trên bản đồ TP.HCM của lab cho thấy chuyện đó không hiếm.</p></details>
</div>

## Ghim ở đâu và xe dừng được ở đâu là hai chuyện khác nhau

Khách chạm vào bản đồ và để lại một **toạ độ**. Điều khách muốn là **một chỗ đứng có xe tới**. Giữa hai thứ này có ba khoảng hở, và cả ba đều có mặt trong tình huống mở bài.

1. **Phía đường.** Đại lộ có dải phân cách chia thành hai chiều chạy riêng. Ghim nằm bên chiều nào thì xe phải đi trên chiều đó mới dừng sát lề được. Ghim bên phía bắc mà xe đang ở làn phía nam là sai phía.
2. **Hướng tiếp cận.** Ở Việt Nam xe chạy bên phải, nên xe thường đón khách ở lề phải của chiều đang chạy. Cùng một điểm trên bản đồ, đi từ đông sang tây là tiện, đi từ tây sang đông là phải quay đầu.
3. **Chỗ dừng được.** Cầu vượt, hầm, đoạn cấm dừng hay lối vào hẻm nhỏ đều có thể nằm đúng chỗ ghim mà xe không đứng được.

Hình 1 vẽ đúng tình huống mở bài theo tỉ lệ ngang. Tài xế xuất phát ở x = −1.000 m, ghim P0 ở x = 0. Dải phân cách chỉ hở một chỗ ở x = 600 m. Bốn điểm P1 đến P4 là các chỗ khác trên vỉa hè phía bắc mà khách có thể đi bộ tới.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 304" role="img" aria-label="Hình 1. Đường có dải phân cách. Tài xế đi trên làn phía nam về hướng đông, khách ghim ở vỉa hè phía bắc. Dải phân cách chỉ hở ở chỗ quay đầu cách ghim 600 mét về phía đông, nên tài xế phải đi 1.000 cộng 600 cộng 600 bằng 2.200 mét. Năm điểm đón ứng viên P0 đến P4 nằm trên vỉa hè phía bắc, cách ghim 0, 150, 300, 450 và 600 mét."><defs><marker id="location-bai-toan-quang-duong-khach-va-tai-xe-0-m0" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><g transform="translate(0 -56)"><rect x="70" y="120" width="780" height="110" class="f-surface s-line"/><rect x="70" y="98" width="780" height="22" class="f-bg s-line"/><rect x="70" y="166" width="753" height="18" class="f-bg s-dead"/><rect x="837" y="166" width="13" height="18" class="f-bg s-dead"/><text x="90" y="179" class="t-small">dải phân cách</text><text x="90" y="113" class="t-small">vỉa hè phía bắc</text><text x="90" y="136" class="t-small">làn bắc, xe chạy về tây</text><text x="250" y="224" class="t-small">làn nam, xe chạy về đông</text><circle cx="560" cy="109" r="6" class="f-pk s-pk" stroke-width="1"/><text x="560" y="88" class="t-b" text-anchor="middle">P0 (ghim)</text><circle cx="627.5" cy="109" r="6" class="f-phys s-phys" stroke-width="1"/><text x="627.5" y="88" class="t-b" text-anchor="middle">P1</text><circle cx="695" cy="109" r="6" class="f-phys s-phys" stroke-width="1"/><text x="695" y="88" class="t-b" text-anchor="middle">P2</text><circle cx="762.5" cy="109" r="6" class="f-phys s-phys" stroke-width="1"/><text x="762.5" y="88" class="t-b" text-anchor="middle">P3</text><circle cx="830" cy="109" r="6" class="f-phys s-phys" stroke-width="1"/><text x="830" y="88" class="t-b" text-anchor="middle">P4</text><rect x="99" y="201" width="22" height="12" rx="3" class="f-warn s-warn"/><text x="99" y="250" class="t-small">tài xế (x = −1.000 m)</text><path d="M123 207 L826 207 C855 207 855 146 826 146 L568 146" class="f-none s-warn dash" stroke-width="3" marker-end="url(#location-bai-toan-quang-duong-khach-va-tai-xe-0-m0)"/><text x="846" y="250" class="t-small" text-anchor="end">chỗ quay đầu</text><path d="M830 186 L830 234" class="f-none s-dead dash" stroke-width="1"/><path d="M110 276 L560 276" class="f-none s-muted" stroke-width="1"/><path d="M110 270 L110 282" class="f-none s-muted" stroke-width="1"/><path d="M560 270 L560 282" class="f-none s-muted" stroke-width="1"/><path d="M830 270 L830 282" class="f-none s-muted" stroke-width="1"/><path d="M560 276 L830 276" class="f-none s-muted" stroke-width="1"/><text x="335" y="296" class="t-mono" text-anchor="middle">1.000 m</text><text x="695" y="296" class="t-small" text-anchor="middle">600 m tới chỗ quay đầu</text><text x="20" y="326" class="t-b">Tới P0 theo đường: 1.000 + 600 + 600 = 2.200 m, trong khi ghim nằm ngay bên kia dải phân cách.</text><text x="20" y="346" class="t-small">Ví dụ tự dựng với số đơn giản, không phải số đo.</text></g></svg>
</div>
<figcaption>Hình 1. Cùng một con đường, nhìn từ trên xuống. Hình chữ nhật cam là tài xế, đường cam đứt nét là lộ trình của anh ta tới ghim; chấm P0 là ghim, các chấm P1 đến P4 là ứng viên dọc vỉa hè phía bắc. Khoảng cách vẽ đúng tỉ lệ ngang.</figcaption>
</figure>

Quãng đường tài xế đi tới ghim P0 là 1.000 m để tới chỗ hở, thêm 600 m quay đầu thành 1.600 m, rồi 600 m ngược lại: tổng **2.200 m xe chạy mà không có khách**. Dân vận tải gọi đó là km rỗng, và loạt bài này gọi là `deadhead_m`.

Để so các phương án với nhau, cần thêm hai con số tự đặt cho dễ tính nhẩm: khách đi bộ 1 m/s và xe chạy 5 m/s. Đây không phải số đo, chỉ là hai số tròn cho một phép tính tay. Bảng dưới dùng chúng cho năm điểm đón nằm cách ghim 0, 150, 300, 450 và 600 m về phía đông trên vỉa hè phía bắc. Điểm càng xa ghim thì khách đi bộ càng nhiều, nhưng xe đỡ phải đi ngược lại.

| Ứng viên | `walk_m` | `deadhead_m` | Khách tới (s) | Xe tới (s) | Gặp nhau (s) | `meet_gap_s` |
|---|---:|---:|---:|---:|---:|---:|
| P0 (ghim) | 0 | 2.200 | 0 | 440 | 440 | 440 |
| P1 | 150 | 2.050 | 150 | 410 | 410 | 260 |
| P2 | 300 | 1.900 | 300 | 380 | **380** | 80 |
| P3 | 450 | 1.750 | 450 | 350 | 450 | 100 |
| P4 | 600 | 1.600 | 600 | 320 | 600 | 280 |

*Bảng tự dựng với các số đơn giản (đi bộ 1 m/s, xe 5 m/s, bỏ qua thời gian quay đầu), không phải kết quả đo.* «Gặp nhau» là thời điểm cả hai có mặt, tức giá trị lớn hơn trong hai cột «tới». `meet_gap_s` là chênh lệch giữa hai cột đó: ai tới sớm thì đứng chờ người kia. Ở P0 khách là người chờ (440 s); ở P3 và P4 thì tài xế chờ.

Bảng có hai điểm đáng chú ý. Thứ nhất, cột `walk_m` cộng với `deadhead_m` luôn bằng 2.200 m ở mọi dòng. Cộng mét thì năm phương án **hoà nhau**, và đó là lý do hàm mục tiêu của loạt bài tính bằng **thời gian**, không bằng mét: một mét đi bộ tốn nhiều giây hơn một mét xe chạy, nên hai loại mét này không cộng thẳng vào nhau được. Thứ hai, thời điểm gặp nhau không nhỏ nhất ở hai đầu mà ở giữa.

## Hai cách tối ưu một phía, và vì sao cả hai đều sai

Thử đặt mỗi bên vào ghế người quyết định.

**Chỉ nghĩ cho khách: «đón đúng chỗ ghim».** Cách này cho `walk_m` bằng 0 và không ai bị bắt đi đâu cả, nghe rất tử tế. Trong bảng, đó là dòng P0: tài xế chạy 2.200 m, quãng dài nhất trong năm phương án, và chính khách cũng chờ lâu nhất, 440 s so với 380 s ở P2. Khách được lợi một thứ (không phải đi bộ) và chịu thiệt một thứ khác (chờ lâu hơn) mà không hề biết.

**Chỉ nghĩ cho tài xế: «chọn chỗ đón rẻ nhất cho xe».** Dòng P4 có `deadhead_m` nhỏ nhất, 1.600 m. Nhưng khách phải đi bộ 600 m dọc vỉa hè, lên xe lúc giây thứ 600, muộn nhất cả bảng, và tài xế đứng chờ 280 s ở lề đường. Tối ưu kiểu này nhìn như tiết kiệm km rỗng, nhưng hai người gặp nhau muộn nhất. Nếu không có rào chắn, kiểu tối ưu này còn dễ trượt sang việc bắt khách đi bộ qua đường lớn để xe đỡ vòng. Loạt bài này cấm điều đó (quy tắc biên tập số 4): mỗi bài tối ưu phải nói bên nào được lợi, bên nào chịu thiệt, và không đẩy thiệt hại sang khách bằng cách bắt họ qua đường lớn.

Cả hai cách đều chọn một đầu của bảng. Điểm có thời gian gặp nhau ngắn nhất, P2, nằm ở giữa. So với P0, khách đi bộ thêm 300 m nhưng lên xe sớm hơn 60 s; tài xế bớt 300 m km rỗng và cũng tới sớm hơn 60 s. Ở ví dụ này không ai mất thời gian, chỉ khách mất 300 m đi bộ, nên 300 m đó phải được **giới hạn** (đó là ràng buộc cứng, sẽ nói ở dưới) và hiển thị rõ cho khách. Bài về điểm hẹn động (LOC-36, phần P04) sẽ bàn cách xin khách đồng ý.

Cũng chưa có lý do để tin P2 là điểm tốt nhất trong mọi hoàn cảnh. Nếu khách đi nhanh gấp đôi, điểm tốt nhất đổi chỗ. Bài tập 1 ở cuối bài tính thử. Điều bài này muốn chứng minh không phải «P2 thắng», mà là: **không có điểm nào tối ưu cho cả hai đầu; muốn chọn phải có một thước đo chung**.

## Tài xế «gần nhất» đo khoảng cách bằng gì?

Cùng lỗi ấy xuất hiện ở bước trước đó, lúc chọn tài xế. Cách ngây thơ nhất là sắp tài xế theo khoảng cách chim bay tới ghim và gán cho người đầu danh sách. Hình 2 cho một ví dụ tự dựng: A cách ghim 1.000 m chim bay nhưng đang ở bên kia dải phân cách nên phải đi 2.200 m; B cách 1.400 m chim bay và đi 1.800 m. Xếp theo chim bay, A đứng trước. Xếp theo đường đi, B đứng trước và tiết kiệm 400 m chạy rỗng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 2. Hai tài xế A và B. Theo đường chim bay A cách ghim 1.000 mét và B cách 1.400 mét, nên A đứng trước. Theo đường đi A mất 2.200 mét và B mất 1.800 mét, nên B đứng trước."><text x="130" y="30" class="t-small">Ví dụ tự dựng, không phải số đo</text><text x="20" y="88" class="t-b">Tài xế A</text><rect x="130" y="62" width="240" height="24" class="f-pk s-pk"/><text x="378" y="78" class="t-mono">1.000 m chim bay</text><rect x="130" y="94" width="528" height="24" class="f-warn s-warn"/><text x="666" y="110" class="t-mono">2.200 m theo đường</text><text x="20" y="172" class="t-b">Tài xế B</text><rect x="130" y="146" width="336" height="24" class="f-pk s-pk"/><text x="474" y="162" class="t-mono">1.400 m chim bay</text><rect x="130" y="178" width="432" height="24" class="f-warn s-warn"/><text x="570" y="194" class="t-mono">1.800 m theo đường</text><path d="M130 224 L706 224" class="f-none s-muted" stroke-width="1"/><path d="M130 220 L130 228" class="f-none s-muted" stroke-width="1"/><text x="130" y="244" class="t-small" text-anchor="middle">0</text><path d="M274 220 L274 228" class="f-none s-muted" stroke-width="1"/><text x="274" y="244" class="t-small" text-anchor="middle">600</text><path d="M418 220 L418 228" class="f-none s-muted" stroke-width="1"/><text x="418" y="244" class="t-small" text-anchor="middle">1.200</text><path d="M562 220 L562 228" class="f-none s-muted" stroke-width="1"/><text x="562" y="244" class="t-small" text-anchor="middle">1.800</text><path d="M706 220 L706 228" class="f-none s-muted" stroke-width="1"/><text x="706" y="244" class="t-small" text-anchor="middle">2.400</text><text x="20" y="244" class="t-small">mét</text></svg>
</div>
<figcaption>Hình 2. Xếp theo chim bay thì A đứng trước B (1.000 so với 1.400 m); xếp theo đường đi thì B đứng trước A (1.800 so với 2.200 m).</figcaption>
</figure>

Có thể nghi ngờ rằng đó chỉ là ví dụ chọn sẵn. Số đo thật của lab nằm ở bài [khoảng cách và hệ số vòng](/articles/location-khoang-cach-haversine-equirectangular-sai-so): trên đồ thị đường bộ OSM của TP.HCM (126.699 nút, 319.074 cạnh có hướng), lấy 200 nút nguồn ngẫu nhiên, mỗi nguồn 15 đích mỗi dải, hệ số vòng, tức đường đi ngắn nhất theo đường chia cho khoảng cách chim bay, ở dải 1–2 km của ô tô có **trung bình 1,840, trung vị 1,456, p90 2,481**, và **17,7% số cặp có hệ số từ 2,0 trở lên**. Hai lưu ý để đọc đúng con số này: phép đo bỏ qua cấm rẽ (trên cùng đồ thị, tính cấm rẽ thì đường đi chỉ có thể dài hơn) và đó là số của đồ thị bị cắt theo khung lab, không phải số của cả thành phố.

Con số cho biết đường đi dài gấp đôi chim bay không hiếm. Con số này **chưa** cho biết thứ tự hai tài xế bị đảo bao nhiêu lần, vì việc đó cần simulator chạy cả đội xe, và sẽ được đo ở bài LOC-41. Tương tự, mối nguy khác của cách «gần nhất» là tham lam: gán tài xế gần nhất cho khách này có thể cướp mất người duy nhất gần khách khác. Cả hai ý đều sẽ có số đo ở P05, không đặt thành kết luận ở bài này.

## Một bộ chỉ số thay cho cảm giác

Nói «tối ưu hơn» mà không có thước đo thì chỉ là cảm giác. Mọi thí nghiệm trong loạt bài báo cáo cùng một bộ chỉ số, và mỗi chỉ số gắn với bên chịu hậu quả của nó.

| Chỉ số | Đo gì | Bên bị ảnh hưởng | Cái bẫy khi chỉ nhìn riêng nó |
|---|---|---|---|
| `walk_m` | Mét khách đi bộ từ chỗ đứng tới điểm đón | Khách | Đón đúng ghim thì kết quả bằng 0, đổi lại km rỗng tăng |
| `deadhead_m` | Mét xe chạy rỗng từ chỗ đang đứng tới điểm đón | Tài xế (và chi phí của nền tảng) | Khách phải đi bộ xa |
| `pickup_eta_s` | Thời gian xe tới điểm đón kể từ lúc gán chuyến; báo p50 và p95 | Khách | Trung bình đẹp nhưng đuôi dài bị che |
| `meet_gap_s` | Chênh lệch giữa lúc khách và lúc xe tới điểm hẹn | Cả hai: ai tới sớm thì chờ | Gặp nhau đúng giờ nhưng đường đi dài |
| `trip_m`, `u_turns` | Quãng đường và số lần quay đầu ở đoạn có khách | Khách (cước) và tài xế | Tối ưu đoạn đi tìm khách mà quên đoạn có khách: cước và số lần quay đầu của khách tăng |
| Tỷ lệ huỷ mô phỏng | Phần chuyến bị huỷ trong simulator vì chờ hoặc đi bộ quá ngưỡng | Cả hai và doanh thu | Con số đẹp che nguyên nhân: khách bỏ vì chờ hay vì phải đi bộ |

Hai ghi chú về cách dùng. Một là `pickup_eta_s` luôn đi kèm p95 cạnh p50: khách nhớ chuyến chờ quá lâu, không nhớ trung bình. Hai là ở ví dụ trên `meet_gap_s` lấy giá trị tuyệt đối; cách tính chính thức của các chỉ số sẽ được chốt khi có simulator (LOC-30), và `meet_gap_s` sẽ quay lại ở bài về điểm hẹn động.

Một điều chi phối cả loạt: ngoài hệ số vòng ở trên, **không có số nào trong bài này là số đo**, và toàn bộ loạt bài sẽ không có GPS của người thật. Dữ liệu chuyến, vị trí tài xế và tắc nghẽn do simulator sinh ra với seed cố định. Mọi con số kiểu «giảm x% km rỗng» chỉ xuất hiện khi có lệnh chạy lại để tái hiện kết quả, so với một baseline, và kèm cụm «trên dữ liệu mô phỏng». Baseline mặc định của loạt chính là hai cách bị phê phán ở trên: đón đúng ghim, giao cho tài xế gần nhất.

## Một cái cân cho hai người

Chỉ số chỉ mô tả. Để chọn phải gộp chúng vào một hàm. Với điểm hẹn ứng viên `p`, hàm mục tiêu xuyên suốt của loạt (chốt ở bài LOC-32, dùng lại ở các phần P04, P05, P07) có dạng:

```text
J(p) = a·T_walk(khách → p)                      khách đi bộ
     + b·max(T_walk, T_drive)                   thời gian tới lúc hai người gặp nhau
     + c·T_drive(tài xế → p)                    km rỗng của tài xế
     + d·T_trip(p → điểm trả, gồm quay đầu)     đoạn có khách, ảnh hưởng cả cước
     + phạt(cấm dừng, qua đường lớn, phía đường bất lợi, hẻm không vào được)
```

Mỗi dòng là một người hoặc một hậu quả đang ngồi trên cân. Dòng 1 và dòng 3 là hai đĩa cân: khách đi bộ ở một bên, tài xế chạy rỗng ở bên kia. Dòng 2 là thứ cả hai cùng chịu: thời điểm gặp nhau. Dòng 4 nhắc rằng điểm đón không chỉ quyết định đoạn đi tìm khách; điểm đón còn quyết định cả hướng xe chạy khi đã có khách, và việc quay đầu ở đoạn đó làm chính khách tốn thêm cước. Trong bảng ở phần đầu, cột «Gặp nhau» chính là số hạng `max(T_walk, T_drive)`.

Có hai loại luật khác hẳn nhau và không nên lẫn.

- **Ràng buộc cứng** loại hẳn ứng viên: vượt ngưỡng đi bộ, điểm dừng bất hợp pháp, không có đường bộ an toàn tới điểm đó. Ứng viên vi phạm thì không có điểm số, không phải điểm số thấp.
- **Phạt** cộng thêm vào `J` cho những thứ xấu nhưng chấp nhận được trong chừng mực: qua đường lớn, đón sai phía đường, hẻm xe không vào được.

Bộ trọng số `a, b, c, d` không phải chân lý vật lý. Nó là **chính sách**: «một giây khách đi bộ nặng bằng bao nhiêu giây xe chạy rỗng» là câu trả lời về giá trị, không phải về hình học. Vì vậy mỗi bộ trọng số mang một `policy_version` và được lưu cùng snapshot của quote, đồng thời được hiệu chỉnh bằng simulator thay vì đoán. Ở bài này chưa chọn trọng số nào cả; chỉ cần nhớ rằng thứ tự ưu tiên giữa hai bên được viết ra, đặt tên, đo được và đổi được.

Điểm trả dùng hàm cùng dạng: khách đi bộ từ chỗ xuống xe tới đích thật, cộng với chi phí để tài xế thoát ra và bắt chuyến kế tiếp. Ghép chuyến ở P05 thay «tài xế gần nhất» bằng việc tối thiểu hoá tổng chi phí này trên cả lô chuyến lẫn lô tài xế.

## Bản đồ loạt bài

Loạt bài gồm ba lộ trình, bảy phần, 64 bài (Hình 3). Lộ trình đầu xây nền: toạ độ, khoảng cách, đồ thị đường bộ, chỉ mục không gian, nhận và làm sạch GPS, bám đường, định tuyến, và kết thúc ở một simulator làm thước đo. Lộ trình hai là trọng tâm: chọn điểm đón, điểm trả, ghép chuyến, giảm km rỗng, mọi cải thiện có số đo so với baseline. Lộ trình ba đưa Python vào huấn luyện ETA và dự báo nhu cầu rồi đưa mô hình sang Rust, và hoàn thiện workspace, tải, quan sát, triển khai.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 3. Bản đồ loạt bài gồm 64 bài, 7 phần, 3 lộ trình. Lộ trình Nền tảng định vị gồm P01 đến P03 với 30 bài. Lộ trình Tối ưu quãng đường hai phía gồm P04 và P05 với 20 bài. Lộ trình Dữ liệu và vận hành gồm P06 với 6 bài và P07 với 8 bài."><rect x="22" y="100" width="127.2" height="70" rx="8" class="f-phys-soft s-phys"/><text x="85.6" y="124" class="t-b" text-anchor="middle">P01</text><text x="85.6" y="142" class="t-small" text-anchor="middle">10 bài</text><text x="85.6" y="196" class="t-small" text-anchor="middle">hình học địa lý</text><rect x="153.2" y="100" width="127.2" height="70" rx="8" class="f-phys-soft s-phys"/><text x="216.9" y="124" class="t-b" text-anchor="middle">P02</text><text x="216.9" y="142" class="t-small" text-anchor="middle">10 bài</text><text x="216.9" y="196" class="t-small" text-anchor="middle">nhận GPS, bám đường</text><rect x="284.5" y="100" width="127.2" height="70" rx="8" class="f-phys-soft s-phys"/><text x="348.1" y="124" class="t-b" text-anchor="middle">P03</text><text x="348.1" y="142" class="t-small" text-anchor="middle">10 bài</text><text x="348.1" y="196" class="t-small" text-anchor="middle">định tuyến</text><rect x="415.8" y="100" width="127.2" height="70" rx="8" class="f-pk-soft s-pk"/><text x="479.4" y="124" class="t-b" text-anchor="middle">P04</text><text x="479.4" y="142" class="t-small" text-anchor="middle">10 bài</text><text x="479.4" y="196" class="t-small" text-anchor="middle">điểm đón, điểm trả</text><rect x="547" y="100" width="127.2" height="70" rx="8" class="f-pk-soft s-pk"/><text x="610.6" y="124" class="t-b" text-anchor="middle">P05</text><text x="610.6" y="142" class="t-small" text-anchor="middle">10 bài</text><text x="610.6" y="196" class="t-small" text-anchor="middle">ghép chuyến, km rỗng</text><rect x="678.2" y="100" width="74.8" height="70" rx="8" class="f-warn-soft s-warn"/><text x="715.6" y="124" class="t-b" text-anchor="middle">P06</text><text x="715.6" y="142" class="t-small" text-anchor="middle">6 bài</text><text x="715.6" y="196" class="t-small" text-anchor="middle">Python, dữ liệu</text><rect x="757" y="100" width="101" height="70" rx="8" class="f-warn-soft s-warn"/><text x="807.5" y="124" class="t-b" text-anchor="middle">P07</text><text x="807.5" y="142" class="t-small" text-anchor="middle">8 bài</text><text x="807.5" y="196" class="t-small" text-anchor="middle">vận hành</text><path d="M22 76 L411.8 76" class="f-none s-phys" stroke-width="2"/><text x="216.9" y="50" class="t-b" text-anchor="middle">Nền tảng định vị</text><text x="216.9" y="68" class="t-small" text-anchor="middle">30 bài</text><path d="M415.8 76 L674.2 76" class="f-none s-pk" stroke-width="2"/><text x="545" y="50" class="t-b" text-anchor="middle">Tối ưu quãng đường hai phía</text><text x="545" y="68" class="t-small" text-anchor="middle">20 bài</text><path d="M678.2 76 L858 76" class="f-none s-warn" stroke-width="2"/><text x="768.1" y="50" class="t-b" text-anchor="middle">Dữ liệu và vận hành</text><text x="768.1" y="68" class="t-small" text-anchor="middle">14 bài</text><path d="M407.8 170 L407.8 236" class="f-none s-muted dash" stroke-width="1"/><path d="M419.8 170 L419.8 236" class="f-none s-muted dash" stroke-width="1"/><text x="401.8" y="256" class="t-b" text-anchor="end">LOC-30: simulator là thước đo</text><text x="401.8" y="274" class="t-small" text-anchor="end">cho mọi con số tối ưu</text><text x="425.8" y="256" class="t-b">LOC-32: hàm mục tiêu J, khách và tài xế</text><text x="425.8" y="274" class="t-small">trên một cân, dùng lại ở P04, P05, P07</text><text x="20" y="316" class="t-small">Chiều rộng mỗi hộp tỉ lệ với số bài.</text></svg>
</div>
<figcaption>Hình 3. Bản đồ loạt bài. Màu xanh ngọc là lộ trình nền tảng, xanh dương là lộ trình tối ưu, vàng là lộ trình dữ liệu và vận hành.</figcaption>
</figure>

Lab của loạt là một workspace tích luỹ, bài sau dùng crate của bài trước, đặt trong `examples/toi-noi-location/`. Ngôn ngữ chính là Rust, vì phần định vị nhạy với độ trễ và bộ nhớ; Go và Python được dùng ở những chỗ chúng có lợi thế thật, và bài kế tiếp (ADR) giải thích ranh giới đó. Các loạt Rust, Python và Flutter qua Tới Nơi là phần đọc kèm, chưa nằm trên nhánh này nên bài chỉ nhắc tên mà không liên kết.

Ai chỉ quan tâm tới tối ưu có thể đọc theo đường tắt: bài 01 này, bài [ADR](/articles/location-adr-rust-go-python-csharp-ranh-gioi-service) về ranh giới các ngôn ngữ, rồi bài về chi phí rẽ và quay đầu (LOC-23), ma trận chi phí (LOC-26), bộ mô phỏng (LOC-30), và vào P04. Ai đi từ đầu thì bắt đầu ở [toạ độ WGS84 và lỗi đảo lat/lng](/articles/location-toa-do-wgs84-newtype-lat-lng-do-chinh-xac), lỗi đầu tiên mà một service bản đồ dễ mắc.

## Giới hạn của ví dụ

Ví dụ trong bài này chỉ minh hoạ cho cơ chế. Ví dụ này có đúng một con đường thẳng, một dải phân cách, một chỗ quay đầu, tốc độ đi bộ và tốc độ xe là hai hằng số tự đặt, và bỏ qua hoàn toàn thời gian quay đầu, tắc nghẽn, đèn đỏ, việc khách có thật sự đứng chờ ở ghim hay không. Bảng năm ứng viên không có trong lab: nó không do simulator sinh ra và không nên được trích như một kết quả. Số duy nhất đến từ phép đo thật là hệ số vòng, và nó chỉ đúng với đồ thị OSM trong khung TP.HCM của lab, bỏ qua cấm rẽ. Ở thời điểm viết bài này lab chưa có simulator, chưa có baseline, chưa có con số «tối ưu hơn» nào; những thứ đó là việc của P03 và P04.

## Bài tập

1. Giữ nguyên bảng ở phần đầu nhưng cho khách đi bộ nhanh gấp đôi (2 m/s). Tính thời điểm gặp nhau của năm ứng viên và cho biết điểm nào tốt nhất. Nhận xét về vị trí của điểm đó so với P0 và P4.

<details><summary>Lời giải</summary>

Thời gian khách tới là `x / 2` giây, thời gian xe tới vẫn là `(2.200 − x) / 5`. Thời điểm gặp nhau là giá trị lớn hơn của hai số: P0: max(0; 440) = 440; P1: max(75; 410) = 410; P2: max(150; 380) = 380; P3: max(225; 350) = 350; P4: max(300; 320) = 320. Điểm tốt nhất là **P4**, với 320 s và `meet_gap_s` chỉ còn 20 s. Khi khách đi nhanh, điểm tốt nhất dịch về phía chỗ quay đầu, trùng với điểm có `deadhead_m` nhỏ nhất. Bài học: vị trí «ở giữa» không phải hằng số của bài toán; nó phụ thuộc tốc độ hai bên, nên trọng số và các giả định phải được đo, không được đoán.

</details>

2. Giữ ví dụ Hình 2. Tính hệ số vòng của A và B (đường đi chia chim bay). Rồi cho biết nếu hệ thống chọn theo chim bay thì `deadhead_m` tăng bao nhiêu mét so với chọn theo đường đi.

<details><summary>Lời giải</summary>

A: 2.200 / 1.000 = 2,2. B: 1.800 / 1.400 ≈ 1,29. A có hệ số ≥ 2,0, đúng loại cặp chiếm 17,7% trong dải 1–2 km của ô tô ở kết quả đo của bài 04 (số đó không nói về trường hợp cụ thể này, chỉ cho thấy hệ số ≥ 2,0 không hiếm). Chọn theo chim bay thì gán A, đi 2.200 m; chọn theo đường thì gán B, đi 1.800 m, nên km rỗng tăng thêm 400 m cho mỗi chuyến kiểu này.

</details>

3. Vì sao không thể định nghĩa «điểm tốt nhất» bằng cách tối thiểu hoá `walk_m + deadhead_m`?

<details><summary>Lời giải</summary>

Vì trong ví dụ của bài, tổng này bằng 2.200 m ở mọi ứng viên: khách đi thêm bao nhiêu mét thì xe đỡ đi bấy nhiêu, nên hàm này không phân biệt được các phương án. Nguyên nhân sâu hơn là một mét đi bộ và một mét xe chạy không cùng giá: chúng tốn thời gian khác nhau và gây thiệt hại cho hai người khác nhau. Hàm mục tiêu phải đổi cả hai về cùng một đơn vị (thời gian) và gắn trọng số chính sách cho từng bên.

</details>

Bài tiếp: [bài 02](/articles/location-adr-rust-go-python-csharp-ranh-gioi-service).

## Tài liệu gốc

- [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi): quy mô tải, quy ước mã chuyến, tài xế, khách dùng trong bài.
- `docs/location-master-curriculum.md`, mục 3.4 (hàm mục tiêu và chỉ số) và bảng P01: danh sách 64 bài và đề cương từng bài.
- `docs/toi-noi-overall-architecture.md`, mục 4.5 (Matching & Dispatch) và 4.8 (Location & Tracking): trách nhiệm của hai service mà bài nói tới.
- [OpenStreetMap: bản quyền và giấy phép dữ liệu](https://www.openstreetmap.org/copyright): nguồn dữ liệu bản đồ của lab (ODbL).
