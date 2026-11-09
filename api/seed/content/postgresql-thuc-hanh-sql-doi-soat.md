---
slug: postgresql-thuc-hanh-sql-doi-soat
title: "Thực hành: truy vấn vận hành và đối soát thanh toán"
summary: "Kế toán Tới Nơi chốt tháng 10: hệ chuyến báo phải thu 182.287.000 đ, tiền về thực tế 182.555.000 đ. Bài thực hành cuối Phần 2: bàn giao sổ đối soát, expected output đúng 7 chuyến lệch và lời giải thích cho từng đồng chênh +268.000 đ."
category: postgresql
level: advanced
tags: [postgresql, pg-020, thuc-hanh, doi-soat, payments, reconciliation, toi-noi]
language: vi
---

Ngày 31/10, kế toán Tới Nơi chốt sổ tháng 10. Hệ thống chuyến nói phải thu **182.287.000 đ**; cộng giao dịch ví, thẻ và tiền mặt thì ra **182.555.000 đ**. Chênh **+268.000 đ** — nhỏ so với doanh thu, nhưng không ai ký biên bản khi chưa biết số đó gồm những chuyến nào và vì sao. Một câu `SUM` không trả lời được. Thứ cần bàn giao là một **bộ truy vấn đối soát**: định nghĩa rõ, chạy lại được, có expected output, và giải thích được từng dòng chênh lệch — kể cả những dòng **không** được báo.

Đây là bài thực hành cuối Phần 2. Bạn nhận đề, tự làm, rồi mới mở lời giải mẫu dựa trên lab và tự chấm theo mục 4.

<div class="tkh-callout">
<span class="tkh-callout-title">PG-020 · Phần 2/6 · Nâng cao</span>
<p>Đọc trước: toàn bộ PG-011 … PG-019 (danh sách ở mục 6), đặc biệt <a href="/articles/postgresql-join-quan-he-cardinality">JOIN và số dòng sau khi nối bảng</a> và <a href="/articles/postgresql-subquery-exists-cte">Subquery, EXISTS và CTE</a>. Lab: <code>examples/toi-noi-postgresql/p02</code> (tệp <code>20-doi-soat-thanh-toan.sql</code>). Sau bài này, bạn bàn giao được SQL đối soát, expected output và lời giải thích chênh lệch.</p>
</div>

## 1. Đề bài

### Bối cảnh

Dữ liệu P02 là ảnh chụp lúc **01:00 ngày 31/10/2026 giờ VN**: 3.174 chuyến đặt từ 01/10, trong đó 6 chuyến đặt sau nửa đêm 31/10 còn đang chạy. Bảng `payments` (thêm ở `p02/00-mo-rong-schema.sql`) có 0..n dòng mỗi chuyến:

| Cột | Giá trị |
|---|---|
| `kind` | `charge` (thu), `refund` (hoàn), `adjustment` (điều chỉnh của CSKH, có dấu) |
| `method` | `cash`, `wallet` (ví VíXanh), `card` |
| `status` | `pending`, `succeeded`, `failed` |
| `amount` | Đồng, số nguyên; charge/refund > 0 |
| `provider_ref` | Mã giao dịch phía cổng, UNIQUE; NULL khi và chỉ khi tiền mặt |
| `paid_at` | Thời điểm phát sinh giao dịch |

Tổng `payments` có 2.826 dòng: 485 chuyến không có giao dịch nào, 2.552 chuyến có một, 137 chuyến có hai.

### Câu hỏi của kế toán

1. Tháng 10 phải thu bao nhiêu, thực thu bao nhiêu, chênh bao nhiêu?
2. Chênh nằm ở những chuyến nào, mỗi chuyến thuộc loại sai gì?
3. Những chuyến **trông** bất thường (thẻ lỗi rồi thu lại, trả một phần, điều chỉnh của CSKH, giao dịch treo) có bị báo nhầm không?
4. Vì sao bảng kê ngân hàng theo ngày không khớp số theo ngày đặt chuyến?

### Sản phẩm bàn giao

| Mã | Sản phẩm | Yêu cầu tối thiểu |
|---|---|---|
| D1 | Đặc tả đối soát | Phạm vi, "phải thu", "thực thu", "khớp", phân loại lệch có thứ tự ưu tiên — viết thành lời trước khi viết SQL |
| D2 | Sổ đối soát theo chuyến | **Một** định nghĩa dùng chung, đúng một dòng mỗi chuyến trong phạm vi, có bằng chứng không nở/mất dòng |
| D3 | Đối soát tổng và số dòng | Ba tổng của câu 1; mọi chuyến rơi vào đúng một nhóm, các nhóm cộng lại bằng phạm vi |
| D4 | Danh sách chênh lệch | Expected output từng dòng, tổng theo loại cộng lại bằng chênh của D3 |
| D5 | Giải thích | Giả thuyết cho từng dòng lệch; ca khó nhưng đúng; chênh do thời điểm; việc cần theo dõi |
| D6 | Bằng chứng | Chạy từ database trống, kỳ vọng ghi bằng máy kiểm được, phiên bản và những gì **chưa** kiểm |

Chỉ dùng kiến thức P01–P02. Không thêm bảng, không sửa dữ liệu (trừ trong giao dịch có `ROLLBACK`).

## 2. Gợi ý trước khi làm

- **Chốt hạt của sổ trước** (grain — một dòng đại diện cho cái gì): một dòng một chuyến. Giao dịch là một–nhiều, nên **gom giao dịch theo chuyến trước rồi mới JOIN** — JOIN rồi `sum(fare)` là nhân tiền như [PG-013](/articles/postgresql-join-quan-he-cardinality) (2.707 chuyến thành 2.800 dòng, doanh thu thành 191.644.000).
- Chuyến không có giao dịch vẫn phải có dòng: `LEFT JOIN` + `coalesce` ([PG-011](/articles/postgresql-null-case-coalesce)). Tính nhiều tổng có điều kiện trong một lần quét bằng `FILTER` ([PG-012](/articles/postgresql-aggregate-group-by-having)).
- Phạm vi theo ngày đặt giờ VN, nửa hở, mốc ghi `+07` ([PG-016](/articles/postgresql-date-time-timezone)). Lọc `::date` theo phiên là đếm sai ở phiên UTC.
- Tách bước bằng CTE, mỗi bước có câu kiểm kê ([PG-014](/articles/postgresql-subquery-exists-cte)); câu "có/không có giao dịch" dùng `EXISTS`/`NOT EXISTS`, đừng `NOT IN` trên cột có thể NULL.
- Đặt định nghĩa vào **một** chỗ (VIEW tạm sống trong phiên), các truy vấn sau chỉ đọc từ đó.
- Kỳ vọng ghi thành chỉ thị `-- expect-line:` / `-- expect-value:` của runner, để máy so chứ không phải mắt.

```bash
cd examples/toi-noi-postgresql
npm ci
npm run p02      # p01/00 + p01/01 rồi p02/00 → 20 trên PGlite (PostgreSQL 17.5)
```

## 3. Lời giải mẫu

Lời giải là `20-doi-soat-thanh-toan.sql`. Mở từng phần **sau khi** đã có bản của bạn.

### 3.1 D1 — Đặc tả

<details><summary><b>Lời giải</b></summary>
<table>
<thead><tr><th>Mục</th><th>Định nghĩa</th></tr></thead>
<tbody>
<tr><td>Phạm vi</td><td>Chuyến <b>đặt</b> trong <code>[2026-10-01 00:00+07, 2026-10-31 00:00+07)</code> và đã kết thúc (<code>completed</code>/<code>cancelled</code>) — 3.168 chuyến. Chuyến đang chạy chưa đối soát.</td></tr>
<tr><td>Phải thu</td><td><code>completed</code> ⇒ <code>fare</code>; <code>cancelled</code> ⇒ 0 đ</td></tr>
<tr><td>Thực thu</td><td>Σ charge − Σ refund + Σ adjustment, <b>chỉ</b> <code>succeeded</code>; không lọc theo ngày giao dịch — tiền của chuyến tính cho chuyến, dù thu sau nửa đêm</td></tr>
<tr><td>Khớp</td><td>Thực thu = phải thu</td></tr>
<tr><td>Loại lệch (lấy loại <b>đầu tiên</b> đúng)</td><td>1 HOÀN KHÔNG CÓ THU — có refund thành công, không có charge thành công · 2 THU CHUYẾN HUỶ — huỷ mà thực thu &gt; 0 · 3 THIẾU THANH TOÁN — hoàn thành, cước &gt; 0, không có charge thành công · 4 THU TRÙNG — ≥ 2 charge thành công cùng số tiền · 5 LỆCH SỐ TIỀN — còn lại</td></tr>
</tbody>
</table>
<p>Hai quyết định phải viết ra vì chúng đổi con số: <code>pending</code>/<code>failed</code> không làm đổi tiền, và hai charge thành công <b>khác</b> số tiền là trả một phần (ví + thẻ), không phải thu trùng. Thứ tự ưu tiên tồn tại vì một chuyến có thể đúng nhiều mô tả cùng lúc — chuyến huỷ chỉ có refund vừa "hoàn không có thu" vừa "lệch số tiền"; người nhận báo cáo cần đúng một nhãn.</p>
</details>

### 3.2 D2 — Sổ đối soát

<details><summary><b>Lời giải</b></summary>
<p>Mục 0 của tệp tạo <code>CREATE TEMP VIEW so_doi_soat</code> gồm hai CTE: <code>pham_vi</code> (chuyến trong phạm vi, ngày đặt giờ VN, phải thu) và <code>gd</code> (giao dịch <b>đã gom theo chuyến</b>), rồi <code>pham_vi LEFT JOIN gd</code>. Phần lõi của <code>gd</code>:</p>
<pre><code class="language-sql">SELECT p.trip_id,
       count(*) FILTER (WHERE kind = 'charge' AND status = 'succeeded')                 AS so_thu_ok,
       count(*) FILTER (WHERE kind = 'charge' AND status = 'succeeded')
         - count(DISTINCT amount) FILTER (WHERE kind = 'charge' AND status = 'succeeded') AS so_thu_lap_tien,
       coalesce(sum(amount) FILTER (WHERE kind = 'charge'     AND status = 'succeeded'), 0) AS tong_thu,
       coalesce(sum(amount) FILTER (WHERE kind = 'refund'     AND status = 'succeeded'), 0) AS tong_hoan,
       coalesce(sum(amount) FILTER (WHERE kind = 'adjustment' AND status = 'succeeded'), 0) AS tong_dieu_chinh,
       string_agg(kind || '/' || method || '/' || status || '/' || amount, ', ' ORDER BY paid_at, id) AS chi_tiet
FROM payments AS p
GROUP BY p.trip_id</code></pre>
<p>View còn tính <code>thuc_thu</code>, <code>chenh = thuc_thu − phai_thu</code> và <code>loai_lech</code> bằng một <code>CASE</code> theo đúng thứ tự D1 (<code>NULL</code> khi khớp). <code>so_thu_lap_tien</code> = số charge thành công trừ số mức tiền khác nhau: &gt; 0 nghĩa là có hai charge cùng số tiền. Cột <code>chi_tiet</code> không dùng để tính; nó là bằng chứng đọc được cho người nhận báo cáo.</p>
<p>Hai bất biến chứng minh sổ đúng hạt:</p>
<pre><code class="language-text"> dong_so | chuyen_khac_nhau | dem_doc_lap
---------+------------------+-------------
    3168 |             3168 |        3168</code></pre>
<p>Số dòng sổ = số chuyến khác nhau = đếm độc lập trên <code>trips</code>; và <b>0</b> giao dịch không thuộc chuyến nào trong sổ (<code>NOT EXISTS</code>) — không có tiền "trôi nổi" của chuyến đang chạy hay tháng khác, nên tổng của sổ là tổng của toàn bộ <code>payments</code>.</p>
</details>

### 3.3 D3 — Đối soát tổng và số dòng

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-text"> so_chuyen | tong_phai_thu | tong_thuc_thu | chenh
-----------+---------------+---------------+--------
      3168 |     182287000 |     182555000 | 268000</code></pre>
<p>Phải thu 182.287.000 đ trùng doanh thu tháng của PG-012 và con số "đúng" của PG-013 — ba cách tính độc lập cho cùng một số. Đối soát số dòng: mỗi chuyến rơi vào đúng một nhóm (cột cuối là tổng charge thành công của nhóm):</p>
<pre><code class="language-text">                   nhom                    | so_chuyen | tong_tien_charge
-------------------------------------------+-----------+------------------
 KHỚP — hoàn thành, có thu                 |      2671 |        181941000
 KHỚP — chuyến huỷ, không giao dịch        |       447 |                0
 KHỚP — hoàn thành cước 0 đ, không cần thu |        31 |                0
 KHỚP — chuyến huỷ, đã hoàn đủ             |        12 |          1008000
 LỆCH                                      |         7 |           691000
 TỔNG                                      |      3168 |        183640000</code></pre>
<p>Cầu nối từ tiền thu tới thực thu (thí nghiệm ngoài lab, cộng thẳng trên <code>payments</code> chỉ <code>succeeded</code>): 2.714 charge = 183.640.000 − 13 refund 1.080.000 + 2 adjustment −5.000 = <b>182.555.000</b>. Ngoài ra có 96 giao dịch <code>failed</code> và 1 <code>pending</code> không đổi tiền. Ba con số cùng đi qua một chuỗi cộng trừ kiểm được — đó là thứ kế toán ký.</p>
</details>

### 3.4 D4 — Danh sách chênh lệch (expected output)

<details><summary><b>Lời giải</b></summary>
<p>Đáp án đúng 7 dòng, mỗi dòng có chỉ thị <code>expect-line</code> trong tệp (cột <code>chi_tiet</code> rút gọn ở đây):</p>
<table>
<thead><tr><th>Mã</th><th>Ngày</th><th>TP</th><th>DV</th><th>Trạng thái</th><th>Phải thu</th><th>Thực thu</th><th>Chênh</th><th>Loại</th><th>Giao dịch</th></tr></thead>
<tbody>
<tr><td>100724</td><td>07/10</td><td>HN</td><td>ride</td><td>completed</td><td>76.000</td><td>0</td><td>−76.000</td><td>3 THIẾU THANH TOÁN</td><td>(không có giao dịch)</td></tr>
<tr><td>101084</td><td>11/10</td><td>DN</td><td>food</td><td>completed</td><td>98.000</td><td>0</td><td>−98.000</td><td>3 THIẾU THANH TOÁN</td><td>thẻ <code>failed</code> 98.000</td></tr>
<tr><td>101310</td><td>13/10</td><td>SG</td><td>food</td><td>completed</td><td>85.000</td><td>170.000</td><td>+85.000</td><td>4 THU TRÙNG</td><td>ví 85.000 × 2</td></tr>
<tr><td>101987</td><td>19/10</td><td>SG</td><td>ride</td><td>cancelled</td><td>0</td><td>−72.000</td><td>−72.000</td><td>1 HOÀN KHÔNG CÓ THU</td><td>ví refund 72.000</td></tr>
<tr><td>102443</td><td>24/10</td><td>HN</td><td>parcel</td><td>completed</td><td>54.000</td><td>45.000</td><td>−9.000</td><td>5 LỆCH SỐ TIỀN</td><td>tiền mặt 45.000</td></tr>
<tr><td>102642</td><td>25/10</td><td>SG</td><td>ride</td><td>completed</td><td>38.000</td><td>380.000</td><td>+342.000</td><td>5 LỆCH SỐ TIỀN</td><td>thẻ 380.000</td></tr>
<tr><td>102739</td><td>26/10</td><td>SG</td><td>food</td><td>cancelled</td><td>0</td><td>96.000</td><td>+96.000</td><td>2 THU CHUYẾN HUỶ</td><td>ví 96.000</td></tr>
</tbody>
</table>
<p>Theo loại (<code>GROUP BY ROLLUP</code>): −72.000 · +96.000 · −174.000 (2 chuyến) · +85.000 · +333.000 (2 chuyến) · <b>TỔNG 7 chuyến, +268.000</b> — khớp chênh của D3, nên không còn đồng nào chưa được giải thích.</p>
</details>

### 3.5 D5 — Giải thích

<details><summary><b>Lời giải</b></summary>
<p><b>Từng dòng lệch.</b> SQL chỉ chứng minh <i>có</i> lệch; nguyên nhân là <b>giả thuyết</b> để bộ phận liên quan xác minh:</p>
<ul>
<li>100724: chuyến hoàn thành không có giao dịch nào — có thể tài xế thu tiền mặt mà không ghi nhận. Hỏi tài xế.</li>
<li>101084: thẻ bị từ chối, chuyến vẫn kết thúc — khách còn nợ; cần quy trình thu lại.</li>
<li>101310: ví trừ hai lần cách 3 giây, hai mã <code>VX-101310-1</code>/<code>-2</code> khác nhau nên UNIQUE <code>provider_ref</code> không chặn — app gửi lại yêu cầu. Hoàn 85.000 đ.</li>
<li>101987: chuyến huỷ chưa thu đồng nào mà có refund 72.000 — hoàn nhầm, cần truy hồi.</li>
<li>102443: tiền mặt ghi 45.000 cho cước 54.000 — nghi đảo chữ số khi nhập tay.</li>
<li>102642: thẻ 380.000 cho cước 38.000 — thừa một số 0; hoàn 342.000 đ.</li>
<li>102739: chuyến huỷ nhưng ví bị trừ 96.000 — hoàn cho khách.</li>
</ul>
<p><b>Ca khó nhưng đúng</b> — không được báo, và là lý do quy tắc phải tính đủ refund, adjustment, chỉ lấy <code>succeeded</code>: 100042 thẻ lỗi rồi thu lại thành công 82.000; 100048 trả một phần ví 104.000 + thẻ 15.000 = 119.000; 102235 ví trừ 82.000, CSKH sửa cước còn 72.000 và ghi adjustment −10.000; 103162 có thêm một charge <code>pending</code> 57.000 bên cạnh lần thu thành công; 103165 tiền mặt 23.000 + adjustment +5.000 = 28.000; 12 chuyến huỷ đã trừ thẻ rồi hoàn đủ (như 100366, 103.000 thu và hoàn); 31 chuyến cước 0 đ không có giao dịch.</p>
<p><b>Quy tắc ngây thơ</b> — cộng mọi charge bất kể trạng thái, bỏ qua refund và adjustment — báo lệch <b>115</b> chuyến, so với 7 của quy tắc đúng. Đối soát báo nhầm trăm dòng còn tệ hơn không đối soát: người đọc sẽ bỏ qua cả 7 dòng thật.</p>
<p><b>Chênh lệch thời điểm, không phải lỗi.</b> Bảng kê cổng thanh toán gom theo ngày giao dịch; sổ gom theo ngày đặt chuyến. Chuyến đặt cuối ngày được thu sang ngày sau:</p>
<pre><code class="language-text">    ngay    | theo_ngay_dat_chuyen | theo_ngay_giao_dich
------------+----------------------+---------------------
 2026-10-15 |              5862000 |             5823000
 2026-10-16 |              5244000 |             5228000</code></pre>
<p>Cả tháng: theo ngày giao dịch trong tháng 10 ra 182.550.000, ít hơn sổ đúng 5.000 — adjustment +5.000 của 103165 (đặt 30/10) được CSKH ghi lúc 00:05 ngày 31/10. Hai cách đều đúng; biên bản phải ghi rõ dùng cách nào.</p>
<p><b>Việc cần theo dõi.</b> Charge <code>TH-103162-P</code> 57.000 còn <code>pending</code> lúc chụp dữ liệu dù chuyến đã hoàn thành hơn 1 giờ. Không đổi tiền hôm nay, nhưng nếu cổng chuyển nó sang <code>succeeded</code>, 103162 sẽ thành THU TRÙNG.</p>
</details>

### 3.6 D6 — Bằng chứng

<details><summary><b>Lời giải</b></summary>
<p>Chạy từ database trống (PGlite tạo database mới trong bộ nhớ mỗi lần), tổng kết của <code>npm run p02</code>:</p>
<pre><code class="language-text">-- [lab] Câu lệnh đã chạy: 266
-- [lab] Lỗi mong đợi (expect-error): 7
-- [lab] Kiểm số dòng (expect-rows): 37
-- [lab] Kiểm giá trị (expect-value): 13
-- [lab] Kiểm nội dung (expect-line): 109
-- [lab] Kiểm md5 kết quả (expect-md5): 3
-- [lab] FAIL: 0
-- [lab] KẾT QUẢ: ĐẠT</code></pre>
<p>Mọi bảng kết quả của <code>20</code> đều có kỳ vọng; đổi một giao dịch là runner báo <code>FAIL</code> và in dòng khác. <code>npm test</code> báo <code>pass 32</code>, <code>fail 0</code>, gồm so <code>p02/OUTPUT.txt</code> từng byte. Fixture sinh tất định bằng hàm băm số học, không <code>random()</code>, nên lần chạy nào cũng cùng 7 dòng. Phần "chưa kiểm" ghi thẳng: mới chạy trên PGlite 0.3.16 (PostgreSQL 17.5), <b>chưa</b> chạy trên server thật; một kết nối nên chưa có gì về hai phiên cùng ghi giao dịch; chưa đo tốc độ — truy vấn chạy nhanh trên 2.826 giao dịch không nói gì về bảng lớn gấp nghìn lần.</p>
</details>

## 4. Tiêu chí tự chấm

Mỗi tiêu chí 10 điểm. Từ 80 điểm là đạt để sang Phần 3; tiêu chí 2, 4 và 6 bắt buộc.

| # | Tiêu chí | Đạt khi |
|---|---|---|
| 1 | Đặc tả viết trước | Phạm vi, phải thu, thực thu, khớp, thứ tự phân loại có bằng lời, khớp với SQL |
| 2 | Đúng hạt | Một dòng một chuyến; bất biến số dòng 3.168 = 3.168 = 3.168 |
| 3 | Một định nghĩa | Công thức thực thu và phân loại nằm ở một chỗ, không chép lại ở từng truy vấn |
| 4 | Ba tổng đúng | 3.168 / 182.287.000 / 182.555.000 / +268.000 |
| 5 | Đối soát số dòng | Các nhóm rời nhau, cộng lại bằng phạm vi; có cầu nối từ tổng charge tới thực thu |
| 6 | Đáp án chính xác | Đúng 7 chuyến, đúng loại, tổng theo loại = +268.000 |
| 7 | Không báo nhầm | Mọi ca khó nhưng đúng ở D5 không có nhãn lệch |
| 8 | Thời điểm | Phân biệt được chênh do ngày đặt/ngày giao dịch với lỗi thật |
| 9 | Giải thích dùng được | Mỗi dòng lệch có giả thuyết và hành động đề xuất; có danh sách việc cần theo dõi |
| 10 | Bằng chứng trung thực | Kỳ vọng máy kiểm được, output thật, phiên bản, điều chưa kiểm |

## 5. Bẫy hay gặp khi tự làm

- **JOIN `trips` với `payments` rồi `sum(fare)`.** Chuyến có hai giao dịch bị cộng cước hai lần; `sum(DISTINCT fare)` thì gộp nhầm các chuyến cùng giá (PG-013).
- **`INNER JOIN` với giao dịch đã gom.** 485 chuyến không có giao dịch biến mất, trong đó có 100724 — một trong 7 đáp án.
- **Lọc `status = 'succeeded'` ở `WHERE` trước khi gom.** Tổng tiền vẫn đúng, nhưng chuyến chỉ có giao dịch `failed` (101084) mất khỏi cột bằng chứng.
- **Phân loại không có thứ tự.** Chuyến huỷ chỉ có refund đúng cả loại 1 và loại 5; không có thứ tự là đếm hai lần hoặc tuỳ hứng.
- **Coi hai charge thành công là thu trùng.** 100048 trả một phần bằng hai phương thức; chỉ cùng số tiền mới là dấu hiệu lặp yêu cầu — và ngay cả thế vẫn chỉ là tín hiệu (bài tập 3).
- **Lọc theo ngày giao dịch cho sổ theo chuyến.** Mất 5.000 của 103165 và gán nhầm tiền chuyến cuối ngày sang hôm sau.
- **Expected output chỉ để trong đầu.** Không có `expect-line`, lần sửa sau không ai biết đáp án đã đổi.

## 6. Tổng kết Phần 2

| Bài | Bạn đã có gì cho sản phẩm cuối phần |
|---|---|
| [PG-011 · NULL, CASE và COALESCE](/articles/postgresql-null-case-coalesce) | Phân biệt "không có giao dịch" với "giao dịch có cột NULL"; `coalesce` sau LEFT JOIN |
| [PG-012 · COUNT, SUM, GROUP BY và HAVING](/articles/postgresql-aggregate-group-by-having) | `FILTER`, `ROLLUP` + `grouping()`, mẫu số rõ ràng — tổng theo loại và nhóm |
| [PG-013 · JOIN và số dòng sau khi nối bảng](/articles/postgresql-join-quan-he-cardinality) | Gom trước rồi JOIN; bất biến số dòng của sổ |
| [PG-014 · Subquery, EXISTS và CTE](/articles/postgresql-subquery-exists-cte) | Tách đối soát thành bước có kiểm kê; `NOT EXISTS` thay `NOT IN` |
| [PG-015 · Window function và cửa sổ tính toán](/articles/postgresql-window-function-frame) | Luỹ kế và so sánh dòng trước — nền cho báo cáo theo ngày |
| [PG-016 · Ngày giờ và múi giờ trong truy vấn](/articles/postgresql-date-time-timezone) | Phạm vi nửa hở `+07`; ngày đặt khác ngày giao dịch |
| [PG-017 · UPSERT và xung đột khoá duy nhất](/articles/postgresql-upsert-on-conflict) | Nhập lại dữ liệu không nhân đôi — điều kiện để chạy lại đối soát |
| [PG-018 · Phân trang lịch sử chuyến bằng keyset](/articles/postgresql-pagination-keyset) | Thứ tự xác định có khoá phụ; danh sách dài không lặp, không sót |
| [PG-019 · Recursive CTE với cây khu vực](/articles/postgresql-recursive-cte-cay-khu-vuc) | Cộng dồn theo cây có câu đối chiếu "qua đường vòng = đếm thẳng" |

Sản phẩm của phần đúng như khung: bộ truy vấn có expected output, đối soát số dòng và tổng tiền. Những gì **chưa** có cũng rõ: chưa đo tốc độ hay index cho các truy vấn này (Phần 3); chưa có hai phiên cùng ghi một giao dịch hay cùng UPSERT một khoá (Phần 4); chưa có quyền truy cập riêng cho bảng tiền (Phần 5).

## 7. Bài tập mở rộng

Lời giải không nằm trong luồng chính của lab; tệp thử dựng lại `so_doi_soat` nguyên văn rồi chạy bằng `node run.mjs p01/00-reset.sql p01/01-schema.sql p02/00-mo-rong-schema.sql p02/01-fixture.sql p02/02-kiem-tra-fixture.sql <tệp>`.

**Bài 1.** Giám đốc thành phố muốn biết phần chênh của mình. Lập đối soát theo thành phố có dòng tổng, và chứng minh nó khớp mục 3.3.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT coalesce(c.code, 'TỔNG') AS tp, count(*) AS so_chuyen,
       sum(s.phai_thu) AS phai_thu, sum(s.thuc_thu) AS thuc_thu, sum(s.chenh) AS chenh,
       count(*) FILTER (WHERE s.loai_lech IS NOT NULL) AS so_lech
FROM so_doi_soat AS s
JOIN cities AS c ON c.id = s.city_id
GROUP BY ROLLUP (c.code)
ORDER BY grouping(c.code), c.code COLLATE "C";</code></pre>
<pre><code class="language-text">  tp  | so_chuyen | phai_thu  | thuc_thu  | chenh  | so_lech
------+-----------+-----------+-----------+--------+---------
 DN   |       459 |  26416000 |  26318000 | -98000 |       1
 HN   |      1106 |  62991000 |  62906000 | -85000 |       2
 SG   |      1603 |  92880000 |  93331000 | 451000 |       4
 TỔNG |      3168 | 182287000 | 182555000 | 268000 |       7</code></pre>
<p>Dòng TỔNG trùng mục 3.3 từng số. SG gánh +451.000 nhưng đó là tổng của bốn dòng ngược dấu (−72.000, +85.000, +96.000, +342.000): con số ròng nhỏ có thể che các dòng lớn, nên báo cáo theo thành phố phải đi kèm danh sách.</p>
</details>

**Bài 2.** CSKH muốn giảm 10.000 đ cho chuyến 100048 (cước 119.000). Ba cách ghi: (a) chỉ thêm một refund 10.000 vào ví; (b) chỉ thêm adjustment −10.000; (c) sửa cước còn 109.000 **và** thêm adjustment −10.000. Đối soát báo gì mỗi cách? Thử trong `BEGIN … ROLLBACK`.

<details><summary><b>Lời giải</b></summary>
<p>(a) và (b) đều cho <code>100048 | 119000 | 109000 | -10000 | 5 LỆCH SỐ TIỀN</code>; với (a), sổ có 8 dòng lệch, tổng chênh 258.000. Refund không thành loại 1 vì chuyến có charge thành công. (c) khớp: <code>109000 | 109000 | 0 | NULL</code>, sổ trở lại 7 dòng, +268.000 — đúng cách của ca 102235. Bài học: giảm giá phải đổi <b>cả</b> phải thu lẫn tiền; đối soát chính là phép kiểm hai nơi đó nhất quán. Loại giao dịch (refund hay adjustment) là quy ước kế toán cần chốt trong đặc tả, đối soát không tự quyết thay.</p>
</details>

**Bài 3.** Tín hiệu THU TRÙNG của lab chỉ nhìn "cùng số tiền". Siết thành "cùng số tiền, cùng phương thức, cách nhau ≤ 60 giây". Trên fixture, đáp án có đổi không? Dựng một ca làm hai tín hiệu khác nhau.

<details><summary><b>Lời giải</b></summary>
<p>Tự JOIN các charge thành công của cùng chuyến, cùng số tiền, <code>(b.paid_at, b.id) &gt; (a.paid_at, a.id)</code> để mỗi cặp xuất hiện một lần. Trên fixture chỉ có <b>một</b> cặp: <code>101310 | wallet | 85000 | 00:00:03</code>, cả hai tín hiệu đều bật — đáp án vẫn 7 dòng. Fixture không phân biệt được hai quy tắc, nên phải thêm ca: trong giao dịch, trừ thêm thẻ 15.000 cho 100048 lúc 03/10 09:00. Sổ báo <code>100048 | 15000 | 4 THU TRÙNG</code>, còn tín hiệu chặt cho <code>f</code> (cách nhau <code>1 day 19:30:02</code>) ⇒ sẽ rơi về LỆCH SỐ TIỀN. Nhãn nào đúng tuỳ người xử lý: "thu trùng" giao cho đội cổng thanh toán (lỗi lặp yêu cầu), "lệch số tiền" giao cho vận hành. Cách chắc chắn hơn mọi tín hiệu suy luận là khoá idempotency phía ứng dụng — chủ đề của Phần 4 và 6.</p>
</details>

## 8. Đọc tiếp

- [Tài liệu PostgreSQL 17 — Aggregate Functions](https://www.postgresql.org/docs/17/functions-aggregate.html): `string_agg … ORDER BY`, `count(DISTINCT)`, mệnh đề `FILTER`.
- [Tài liệu PostgreSQL 17 — GROUPING SETS, CUBE và ROLLUP](https://www.postgresql.org/docs/17/queries-table-expressions.html#QUERIES-GROUPING-SETS).
- [Tài liệu PostgreSQL 17 — CREATE VIEW](https://www.postgresql.org/docs/17/sql-createview.html): view `TEMP` sống trong phiên.
- Lab: `examples/toi-noi-postgresql/p02/README.md` (bảng ca khó và đáp án), `20-doi-soat-thanh-toan.sql`, `OUTPUT.txt`, `test/p02.test.mjs`.

Phần 3 · Lưu trữ, index và kế hoạch thực thi sắp ra mắt. Quay lại đầu phần: [PG-011: NULL, CASE và COALESCE trong dữ liệu chuyến](/articles/postgresql-null-case-coalesce).
