---
slug: mssql-thuc-hanh-tsql-doi-soat
title: "Thực hành: truy vấn vận hành và đối soát thanh toán"
summary: "Kế toán Tới Nơi chốt tháng 10 trên SQL Server 2022: phải thu 182.287.000 đ, tiền về 182.555.000 đ. Bài thực hành cuối Phần 2: bàn giao sổ đối soát bằng T-SQL, expected output đúng 7 chuyến lệch, lời giải thích cho từng khoản trong +268.000 đ và bản JSON cho hệ khác."
category: mssql
level: advanced
tags: [ms-020, mssql, sql-server-2022, t-sql, doi-soat, toi-noi]
language: vi
---

Ngày 31/10, kế toán Tới Nơi chốt sổ tháng 10. Hệ thống chuyến nói phải thu **182.287.000 đ**; cộng giao dịch ví, thẻ và tiền mặt thì ra **182.555.000 đ**. Mức chênh là **+268.000 đ**, nhỏ so với doanh thu, nhưng không ai ký biên bản khi chưa biết số đó gồm những chuyến nào và vì sao. Một câu `SUM` không trả lời được. Thứ cần bàn giao là một bộ truy vấn **đối soát** (reconciliation: so khớp từng khoản giữa hai nguồn ghi nhận rồi giải thích phần lệch): định nghĩa rõ, chạy lại được, có expected output (kết quả mong đợi ghi sẵn để máy so), và giải thích được từng dòng chênh lệch, kể cả những dòng **không** được báo.

Đây là bài thực hành cuối Phần 2. Bạn nhận đề, tự làm, rồi mới mở lời giải mẫu dựa trên lab và tự chấm theo mục 4.

<div class="tkh-callout">
<span class="tkh-callout-title">MS-020 · Phần 2/6 · Nâng cao</span>
<p>Đọc trước: toàn bộ MS-011 … MS-019 (danh sách ở mục 6), đặc biệt <a href="/articles/mssql-null-case-isnull-coalesce">MS-011 · NULL, CASE, ISNULL và COALESCE</a>, <a href="/articles/mssql-group-by-rollup-grouping-sets">MS-012 · GROUP BY, ROLLUP và GROUPING SETS</a>, <a href="/articles/mssql-join-apply-cardinality">MS-013 · JOIN, APPLY và số dòng sau khi nối</a> và <a href="/articles/mssql-ngay-gio-at-time-zone-date-bucket">MS-016 · Ngày giờ và AT TIME ZONE</a>. Lab: <code>examples/toi-noi-mssql/p02</code> (tệp <code>20-doi-soat-thanh-toan.sql</code>). Sau bài này, bạn bàn giao được T-SQL đối soát, expected output và lời giải thích chênh lệch.</p>
</div>

## 1. Đề bài

### Bối cảnh

Dữ liệu P02 là ảnh chụp lúc **01:00 ngày 31/10/2026 giờ VN**: 3.174 chuyến đặt từ 01/10, trong đó 6 chuyến đặt sau nửa đêm 31/10 còn đang chạy. Bảng `toinoi.payments` (thêm ở `p02/00-mo-rong-schema.sql`) có 0..n dòng mỗi chuyến:

| Cột | Kiểu | Giá trị |
|---|---|---|
| `kind` | `varchar(10)` | `charge` (thu), `refund` (hoàn), `adjustment` (điều chỉnh của CSKH, có dấu) |
| `method` | `varchar(10)` | `cash`, `wallet` (ví VíXanh), `card` |
| `status` | `varchar(10)` | `pending`, `succeeded`, `failed` |
| `amount` | `int` | Đồng; charge/refund > 0 |
| `provider_ref` | `varchar(30)` | Mã giao dịch phía cổng; NULL khi và chỉ khi tiền mặt; duy nhất qua filtered unique index |
| `paid_at` | `datetimeoffset(3)` | Thời điểm phát sinh giao dịch |

Kết quả `02-kiem-tra-fixture.sql` (mục H9) trên máy viết bài: 2.826 giao dịch; 485 chuyến không có giao dịch nào, 2.552 chuyến có một, 137 chuyến có hai.

### Câu hỏi của kế toán

1. Tháng 10 phải thu bao nhiêu, thực thu bao nhiêu, chênh bao nhiêu?
2. Chênh lệch nằm ở những chuyến nào, mỗi chuyến thuộc loại sai lệch nào?
3. Những chuyến **trông** bất thường (thẻ lỗi rồi thu lại, trả một phần, điều chỉnh của CSKH, giao dịch treo) có bị báo nhầm không?
4. Vì sao bảng kê của cổng thanh toán theo ngày không khớp số theo ngày đặt chuyến?

### Sản phẩm bàn giao

| Mã | Sản phẩm | Yêu cầu tối thiểu |
|---|---|---|
| D1 | Đặc tả đối soát | Phạm vi, "phải thu", "thực thu", "khớp", phân loại lệch có thứ tự ưu tiên, viết thành lời trước khi viết T-SQL |
| D2 | Sổ đối soát theo chuyến | **Một** định nghĩa dùng chung, đúng một dòng mỗi chuyến trong phạm vi, có bằng chứng số dòng không bị nhân lên hay mất đi |
| D3 | Đối soát tổng và số dòng | Ba tổng của câu 1; cầu nối từ tổng giao dịch tới thực thu; mọi chuyến rơi vào đúng một nhóm |
| D4 | Danh sách chênh lệch | Expected output từng dòng, tổng theo loại cộng lại bằng chênh của D3; bản JSON cho hệ thống khác |
| D5 | Giải thích | Giả thuyết cho từng dòng lệch; ca khó nhưng đúng; chênh do thời điểm; việc cần theo dõi |
| D6 | Bằng chứng | Chạy từ database trống, mỗi con số chính có `lab.expect_value`, phiên bản và những gì **chưa** kiểm |

Chỉ dùng kiến thức P01–P02. Không thêm bảng, không sửa dữ liệu, trừ trong transaction kết thúc bằng `ROLLBACK`.

## 2. Gợi ý trước khi làm

- **Chốt grain của sổ trước** (grain: mỗi dòng kết quả đại diện cho cái gì): một dòng một chuyến. Giao dịch quan hệ một–nhiều với chuyến, nên **gom giao dịch theo chuyến trước rồi mới JOIN**, hoặc lấy tổng qua `APPLY` ([MS-013](/articles/mssql-join-apply-cardinality)). JOIN rồi mới `SUM(fare)` thì chuyến có hai giao dịch bị cộng cước hai lần.
- Chuyến không có giao dịch vẫn phải có dòng: `LEFT JOIN` rồi thay NULL. Chọn `COALESCE` hay `ISNULL` có hệ quả về kiểu dữ liệu ([MS-011](/articles/mssql-null-case-isnull-coalesce)); mục 3.2 cho thấy hệ quả đó trên chính cột bằng chứng.
- T-SQL không có mệnh đề `FILTER` của PostgreSQL: tổng có điều kiện viết `SUM(IIF(điều kiện, giá trị, 0))`. Cộng tiền bằng `SUM(CAST(amount AS bigint))`: `SUM` trên cột `int` trả về `int` và báo lỗi tràn khi vượt 2.147.483.647.
- Phạm vi theo ngày đặt giờ VN, nửa mở, mốc ghi rõ `+07:00` ([MS-016](/articles/mssql-ngay-gio-at-time-zone-date-bucket)).
- Đặt định nghĩa vào **một** chỗ. SQL Server không có view tạm như PostgreSQL; dùng **view** (truy vấn được đặt tên và lưu trong database, đọc như một bảng) tạo bằng `CREATE OR ALTER VIEW` để tệp chạy lại được. `CREATE VIEW` phải là câu đầu tiên của batch, nên trước nó cần `GO`.
- Dòng tổng của báo cáo: `GROUP BY GROUPING SETS ((cột), ())` cùng hàm `GROUPING()` ([MS-012](/articles/mssql-group-by-rollup-grouping-sets)). Liệt kê mã chuyến thành một chuỗi: `STRING_AGG … WITHIN GROUP (ORDER BY …)`.
- Mỗi con số chính gọi `lab.expect_value`, để runner in `PASS`/`FAIL` thay vì bạn phải đọc bằng mắt.

```sh
cd examples/toi-noi-mssql
npm run db:up
LAB_DB=ToiNoi_thu node run.mjs p01/00-reset.sql p01/01-schema.sql \
  p02/00-mo-rong-schema.sql p02/01-fixture.sql p02/02-kiem-tra-fixture.sql p02/20-doi-soat-thanh-toan.sql
```

Lệnh trên chạy trên database riêng `ToiNoi_thu`, khoảng 5 giây; `npm run p02` chạy toàn bộ Phần 2 trên `ToiNoi`. Trong DBStudio, mở tệp và bấm `Ctrl+Shift+Enter` để chạy cả script: mỗi `SELECT` thành một result set riêng, các dòng `PASS`/`FAIL` nằm ở tab "Thông điệp". Muốn chạy lại một bước, đặt con trỏ vào batch đó và bấm `Ctrl+Enter`.

## 3. Lời giải mẫu

Lời giải là `20-doi-soat-thanh-toan.sql`. Mở từng phần **sau khi** đã có bản của bạn. Mọi khối kết quả dưới đây chép từ lần chạy trên máy viết bài (runner in cột cách nhau bằng `|`).

### 3.1 D1 — Đặc tả

<details><summary><b>Lời giải</b></summary>
<table>
<thead><tr><th>Mục</th><th>Định nghĩa</th></tr></thead>
<tbody>
<tr><td>Phạm vi</td><td>Chuyến <b>đặt</b> trong <code>[2026-10-01 00:00:00 +07:00, 2026-10-31 00:00:00 +07:00)</code> và đã kết thúc (<code>completed</code>/<code>cancelled</code>). Chuyến đang chạy chưa đối soát.</td></tr>
<tr><td>Phải thu</td><td><code>completed</code> ⇒ <code>fare</code>; <code>cancelled</code> ⇒ 0 đ</td></tr>
<tr><td>Thực thu</td><td>Σ charge − Σ refund + Σ adjustment, <b>chỉ</b> <code>succeeded</code>; không lọc theo ngày giao dịch, vì tiền của chuyến tính cho chuyến dù thu sau nửa đêm</td></tr>
<tr><td>Khớp</td><td>Thực thu = phải thu</td></tr>
<tr><td>Loại lệch (lấy loại <b>đầu tiên</b> khớp điều kiện)</td><td>1 HOÀN KHÔNG CÓ THU: có refund thành công, không có charge thành công · 2 THU CHUYẾN HUỶ: huỷ mà thực thu khác 0 · 3 THIẾU THANH TOÁN: hoàn thành, cước &gt; 0, không có charge thành công · 4 THU TRÙNG: ≥ 2 charge thành công cùng số tiền · 5 LỆCH SỐ TIỀN: còn lại</td></tr>
</tbody>
</table>
<p>Hai quyết định phải viết ra vì chúng đổi con số: <code>pending</code>/<code>failed</code> không làm đổi tiền, và hai charge thành công <b>khác</b> số tiền là trả một phần (ví + thẻ), không phải thu trùng. Thứ tự ưu tiên tồn tại vì một chuyến có thể khớp nhiều mô tả: chuyến huỷ chỉ có refund vừa "hoàn không có thu" vừa "lệch số tiền", còn người nhận báo cáo cần đúng một nhãn.</p>
</details>

### 3.2 D2 — Sổ đối soát

<details><summary><b>Lời giải</b></summary>
<p>Mục 0 của tệp tạo <code>lab.so_doi_soat</code> bằng <code>CREATE OR ALTER VIEW</code>, gồm CTE <code>pham_vi</code> (chuyến trong phạm vi, ngày đặt giờ VN, phải thu) và CTE <code>gd</code> (giao dịch <b>đã gom theo chuyến</b>), rồi <code>pham_vi LEFT JOIN gd</code>. Phần lõi:</p>
<pre><code class="language-sql">gd AS (
    SELECT p.trip_id,
           SUM(IIF(p.kind = 'charge' AND p.status = 'succeeded', 1, 0))                            AS so_thu_ok,
           SUM(IIF(p.kind = 'charge'     AND p.status = 'succeeded', CAST(p.amount AS bigint), 0)) AS tong_thu,
           SUM(IIF(p.kind = 'refund'     AND p.status = 'succeeded', CAST(p.amount AS bigint), 0)) AS tong_hoan,
           SUM(IIF(p.kind = 'adjustment' AND p.status = 'succeeded', CAST(p.amount AS bigint), 0)) AS tong_dieu_chinh,
           STRING_AGG(CONCAT(p.kind, '/', p.method, '/', p.status, '/', p.amount), ', ')
               WITHIN GROUP (ORDER BY p.paid_at, p.id)                                             AS chi_tiet
    FROM toinoi.payments AS p
    GROUP BY p.trip_id
)
SELECT pv.id, pv.code, …,
       ISNULL(g.tong_thu - g.tong_hoan + g.tong_dieu_chinh, 0)              AS thuc_thu,
       COALESCE(g.chi_tiet, N'(không có giao dịch)')                        AS chi_tiet,
       CASE WHEN … = pv.phai_thu THEN NULL WHEN … THEN N'1 HOÀN KHÔNG CÓ THU' … END AS loai_lech
FROM pham_vi AS pv
LEFT JOIN gd AS g ON g.trip_id = pv.id
CROSS APPLY (SELECT COUNT(*) - COUNT(DISTINCT p.amount) AS so_thu_lap_tien
             FROM toinoi.payments AS p
             WHERE p.trip_id = pv.id AND p.kind = 'charge' AND p.status = 'succeeded') AS lap;</code></pre>
<p>Ba chi tiết đặc thù T-SQL:</p>
<ul>
<li><b>Tổng có điều kiện bằng <code>SUM(IIF(…, giá trị, 0))</code></b>, thay cho <code>FILTER</code> của PostgreSQL. Đếm và cộng tiền dùng cùng một khuôn nên đặt cạnh nhau dễ soát; <code>COUNT(CASE WHEN … THEN 1 END)</code> cho cùng kết quả đếm.</li>
<li><b><code>CROSS APPLY</code> với truy vấn tổng hợp không có <code>GROUP BY</code></b> luôn trả đúng một dòng (chuyến không có charge nào cho <code>0 − 0</code>), nên không làm mất chuyến. <code>so_thu_lap_tien</code> &gt; 0 nghĩa là có hai charge thành công cùng số tiền.</li>
<li><b><code>COALESCE</code>, không <code>ISNULL</code>, cho cột chữ.</b> <code>STRING_AGG</code> trên <code>CONCAT</code> các cột <code>varchar</code> trả về <code>varchar</code>; <code>ISNULL</code> lấy kiểu của tham số <b>đầu</b> nên chuỗi tiếng Việt bị đổi về code page của collation. Lần chạy đầu của lời giải mẫu dùng <code>ISNULL</code> và in ra <code>(không có giao d?ch)</code>. Tệp giữ lại thí nghiệm này (V2):</li>
</ul>
<pre><code class="language-text">voi_isnull|voi_coalesce
----------|------------
(không có giao d?ch)|(không có giao dịch)</code></pre>
<p>Hai bất biến chứng minh sổ đúng grain: số dòng sổ = số chuyến khác nhau = đếm độc lập trên <code>trips</code>, và <b>0</b> giao dịch không thuộc chuyến nào trong sổ (<code>NOT EXISTS</code>). Không có tiền "trôi nổi" của chuyến đang chạy hay tháng khác, nên tổng của sổ là tổng của toàn bộ <code>payments</code>.</p>
<pre><code class="language-text">dong_so|chuyen_khac_nhau|dem_doc_lap
-------|----------------|-----------
3168|3168|3168
PASS 1 dòng sổ | chuyến khác nhau | đếm độc lập = 3168|3168|3168
PASS 1 giao dịch ngoài sổ = 0</code></pre>
<p>Tệp cũng chứng minh vì sao view cần <code>GO</code> phía trước: <code>PASS V1 CREATE VIEW sau một câu khác trong cùng batch — lỗi 111: 'CREATE VIEW' must be the first statement in a query batch.</code> View được để lại sau khi chạy, nên bạn có thể mở <code>lab.so_doi_soat</code> trong DBStudio và lọc tiếp.</p>
</details>

### 3.3 D3 — Đối soát tổng và số dòng

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-text">so_chuyen|tong_phai_thu|tong_thuc_thu|chenh
---------|-------------|-------------|-----
3168|182287000|182555000|268000</code></pre>
<p>Phải thu 182.287.000 đ trùng doanh thu tháng mà <code>01-fixture.sql</code> tự kiểm (<code>PASS doanh thu chuyến hoàn thành (đ) = 182287000</code>), tức hai cách tính độc lập cho cùng một số. Cầu nối từ giao dịch tới thực thu, cộng thẳng trên <code>payments</code> chỉ <code>succeeded</code>:</p>
<pre><code class="language-text">kind|so_gd|tong_tien
----|-----|---------
adjustment|2|-5000
charge|2714|183640000
refund|13|1080000
PASS 1 cầu nối charge − refund + adjustment (đ) = 182555000
PASS 1 giao dịch không đổi tiền = failed:96, pending:1</code></pre>
<p>183.640.000 − 1.080.000 − 5.000 = <b>182.555.000</b>. Cộng thêm 96 giao dịch <code>failed</code> và 1 <code>pending</code> không đổi tiền, mọi dòng của <code>payments</code> đều có chỗ trong cầu nối. Đối soát số dòng: mỗi chuyến rơi vào đúng một nhóm (cột cuối là tổng charge thành công của nhóm), dòng tổng do <code>GROUPING SETS ((nhom), ())</code> sinh ra:</p>
<pre><code class="language-text">nhom|so_chuyen|tong_tien_charge
----|---------|----------------
KHỚP — hoàn thành, có thu|2671|181941000
KHỚP — chuyến huỷ, không giao dịch|447|0
KHỚP — hoàn thành cước 0 đ, không cần thu|31|0
KHỚP — chuyến huỷ, đã hoàn đủ|12|1008000
LỆCH|7|691000
TỔNG|3168|183640000</code></pre>
<p>Dòng TỔNG của cột cuối trùng tổng charge ở cầu nối. Nhóm được gán bằng một <code>CASE</code> nên mỗi chuyến chỉ nhận một nhãn; các nhóm cộng lại 3.168 cho thấy không chuyến nào bị rơi.</p>
<p>Về kiểu dữ liệu: tháng 10 cộng bằng <code>int</code> vẫn vừa, nhưng báo cáo nhiều tháng thì không. Tệp giả lập 12 tháng bằng cách nhân 12 lần tiền thu: <code>PASS 1 SUM(int) tràn khi cộng 12 tháng — lỗi 8115: Arithmetic overflow error converting expression to data type int.</code>, còn bản <code>CAST(… AS bigint)</code> cho 2.203.680.000.</p>
</details>

### 3.4 D4 — Danh sách chênh lệch (expected output)

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-text">code|ngay|tp|kind|status|phai_thu|thuc_thu|chenh|loai_lech|chi_tiet
----|----|--|----|------|--------|--------|-----|---------|--------
100724|2026-10-07|HN|ride|completed|76000|0|-76000|3 THIẾU THANH TOÁN|(không có giao dịch)
101084|2026-10-11|DN|food|completed|98000|0|-98000|3 THIẾU THANH TOÁN|charge/card/failed/98000
101310|2026-10-13|SG|food|completed|85000|170000|85000|4 THU TRÙNG|charge/wallet/succeeded/85000, charge/wallet/succeeded/85000
101987|2026-10-19|SG|ride|cancelled|0|-72000|-72000|1 HOÀN KHÔNG CÓ THU|refund/wallet/succeeded/72000
102443|2026-10-24|HN|parcel|completed|54000|45000|-9000|5 LỆCH SỐ TIỀN|charge/cash/succeeded/45000
102642|2026-10-25|SG|ride|completed|38000|380000|342000|5 LỆCH SỐ TIỀN|charge/card/succeeded/380000
102739|2026-10-26|SG|food|cancelled|0|96000|96000|2 THU CHUYẾN HUỶ|charge/wallet/succeeded/96000</code></pre>
<p>Kỳ vọng không nằm trong đầu người viết: tệp gom cả 7 dòng thành một chuỗi bằng <code>STRING_AGG … WITHIN GROUP (ORDER BY code)</code> rồi so:</p>
<pre><code class="language-text">PASS 2 đáp án mã:loại:chênh = 100724:3:-76000, 101084:3:-98000, 101310:4:85000, 101987:1:-72000, 102443:5:-9000, 102642:5:342000, 102739:2:96000</code></pre>
<p>Theo loại, có dòng tổng:</p>
<pre><code class="language-text">loai|so_chuyen|tong_chenh
----|---------|----------
1 HOÀN KHÔNG CÓ THU|1|-72000
2 THU CHUYẾN HUỶ|1|96000
3 THIẾU THANH TOÁN|2|-174000
4 THU TRÙNG|1|85000
5 LỆCH SỐ TIỀN|2|333000
TỔNG|7|268000</code></pre>
<p>TỔNG khớp chênh của D3, nên không còn đồng nào chưa được giải thích.</p>
<p><b>Bản JSON.</b> Đội thanh toán muốn nhận danh sách ở dạng máy đọc được. <code>FOR JSON PATH</code> biến kết quả thành một mảng JSON, mỗi dòng một object, tên cột thành tên khoá. Tệp gán JSON vào biến <code>nvarchar(max)</code> (không trả thẳng ra lưới: công cụ client có thể cắt chuỗi JSON dài thành nhiều dòng), rồi đọc ngược bằng <code>OPENJSON … WITH</code> để chứng minh không mất dòng:</p>
<pre><code class="language-text">hop_le|do_dai|phan_tu_dau
------|------|-----------
1|1196|{"ma_chuyen":100724,"ngay":"2026-10-07","tp":"HN","dich_vu":"ride","trang_thai":"completed","phai_thu":76000,"thuc_thu":0,"chenh":-76000,"loai_lech":"3 THIẾU THANH TOÁN"}
PASS 2 JSON đọc lại: số dòng | tổng chênh = 7|268000</code></pre>
<p>Cột <code>date</code> thành chuỗi ISO <code>"2026-10-07"</code>, số giữ nguyên là số. Người nhận chỉ cần bảng tính thì export result set ra CSV hoặc XLSX từ DBStudio.</p>
</details>

### 3.5 D5 — Giải thích

<details><summary><b>Lời giải</b></summary>
<p><b>Từng dòng lệch.</b> T-SQL chỉ chứng minh <i>có</i> lệch; nguyên nhân là <b>giả thuyết</b> để bộ phận liên quan xác minh:</p>
<ul>
<li>100724: chuyến hoàn thành không có giao dịch nào; có thể tài xế thu tiền mặt mà không ghi nhận. Hỏi tài xế.</li>
<li>101084: thẻ bị từ chối, chuyến vẫn kết thúc; khách còn nợ, cần quy trình thu lại.</li>
<li>101310: ví trừ hai lần cách 3 giây, hai mã <code>VX-101310-1</code>/<code>-2</code> khác nhau nên unique index trên <code>provider_ref</code> không chặn; app đã gửi lại yêu cầu. Hoàn 85.000 đ.</li>
<li>101987: chuyến huỷ chưa thu đồng nào mà có refund 72.000; hoàn nhầm, cần truy hồi.</li>
<li>102443: tiền mặt ghi 45.000 cho cước 54.000; nghi đảo chữ số khi nhập tay.</li>
<li>102642: thẻ 380.000 cho cước 38.000, thừa một số 0; hoàn 342.000 đ.</li>
<li>102739: chuyến huỷ nhưng ví bị trừ 96.000; hoàn cho khách.</li>
</ul>
<p><b>Ca khó nhưng đúng</b>: không được báo, và là lý do quy tắc phải tính đủ refund, adjustment và chỉ lấy <code>succeeded</code>:</p>
<pre><code class="language-text">code|status|phai_thu|thuc_thu|loai_lech|chi_tiet
----|------|--------|--------|---------|--------
100042|completed|82000|82000|NULL|charge/card/failed/82000, charge/card/succeeded/82000
100048|completed|119000|119000|NULL|charge/wallet/succeeded/104000, charge/card/succeeded/15000
102235|completed|72000|72000|NULL|charge/wallet/succeeded/82000, adjustment/wallet/succeeded/-10000
103162|completed|57000|57000|NULL|charge/card/succeeded/57000, charge/card/pending/57000
103165|completed|28000|28000|NULL|charge/cash/succeeded/23000, adjustment/wallet/succeeded/5000</code></pre>
<p>Thêm hai nhóm cũng không bị báo: <code>PASS 4 chuyến huỷ đã hoàn đủ, không bị báo = 12</code> (như 100366: thu thẻ 103.000 rồi hoàn 103.000) và <code>PASS 4 chuyến trả một phần, không bị báo = 26</code>.</p>
<p><b>Quy tắc ngây thơ.</b> Tệp thử hai quy tắc "đơn giản" hay gặp. A: cộng mọi charge bất kể trạng thái, bỏ qua refund và adjustment, so với phải thu. B: "chuyến hoàn thành có cước phải có đúng một charge thành công, chuyến huỷ không có charge thành công nào".</p>
<pre><code class="language-text">ngay_tho_a|ngay_tho_b|quy_tac_dung
----------|----------|------------
115|42|7
PASS 4 quy tắc ngây thơ B báo | trong đó lệch thật = 42|4
PASS 4 quy tắc ngây thơ B bỏ sót = 101987, 102443, 102642</code></pre>
<p>A báo 115 chuyến. B tệ theo cả hai hướng: 38 trong 42 dòng là báo nhầm (26 chuyến trả một phần, 12 chuyến huỷ đã hoàn đủ), đồng thời bỏ sót 3 dòng lệch thật, vì đếm số giao dịch không nhìn số tiền: 102443 và 102642 có đúng một charge nhưng sai tiền, 101987 không có charge nào. Đối soát báo nhầm hàng chục dòng còn tệ hơn không đối soát, vì người đọc sẽ bỏ qua luôn những dòng thật.</p>
<p><b>Chênh lệch thời điểm, không phải lỗi.</b> Bảng kê của cổng thanh toán gom theo ngày giao dịch; sổ gom theo ngày đặt chuyến. Chuyến đặt cuối ngày được thu sang ngày sau. Tệp lấy biên ngày giờ VN bằng <code>CAST(ngay AS datetime2(3)) AT TIME ZONE 'SE Asia Standard Time'</code>, và đối chiếu bằng cách thứ hai (<code>SWITCHOFFSET(paid_at, '+07:00')</code> rồi lấy ngày) cho cùng kết quả:</p>
<pre><code class="language-text">ngay|theo_ngay_dat_chuyen|theo_ngay_giao_dich
----|--------------------|-------------------
2026-10-15|5862000|5823000
2026-10-16|5244000|5228000</code></pre>
<p>Cả tháng:</p>
<pre><code class="language-text">theo_ngay_dat_chuyen|theo_ngay_giao_dich_thang_10|ghi_nhan_tu_31_10
--------------------|----------------------------|-----------------
182555000|182550000|5000
code|kind|amount|paid_at_vn
----|----|------|----------
103165|adjustment|5000|2026-10-31 00:05:29.000 +07:00</code></pre>
<p>Tính theo ngày giao dịch, tháng 10 ra 182.550.000, ít hơn sổ đúng 5.000: adjustment +5.000 của 103165 (đặt 30/10) được CSKH ghi lúc 00:05 ngày 31/10. Hai cách đều đúng; biên bản phải ghi rõ dùng cách nào.</p>
<p><b>Việc cần theo dõi.</b> Giao dịch còn <code>pending</code> khi chuyến đã hoàn thành hơn 1 giờ, tính tới lúc chụp dữ liệu:</p>
<pre><code class="language-text">code|provider_ref|amount|paid_at_vn
----|------------|------|----------
103162|TH-103162-P|57000|2026-10-30 22:17:37.000 +07:00</code></pre>
<p>Hôm nay giao dịch này không đổi tiền. Tệp thử giả định cổng chuyển nó sang <code>succeeded</code> trong một transaction rồi <code>ROLLBACK</code>: <code>PASS 6 giả định pending → succeeded = 103162:4 THU TRÙNG:57000</code>. Vì vậy cần có người hỏi cổng thanh toán trước khi điều đó xảy ra.</p>
</details>

### 3.6 D6 — Bằng chứng

<details><summary><b>Lời giải</b></summary>
<p>Chạy từ database trống (<code>p01/00-reset.sql</code> xoá và tạo lại database), sáu tệp từ <code>00-reset</code> tới <code>20</code>:</p>
<pre><code class="language-text">-- [lab] File đã chạy: 6
-- [lab] PASS: 61
-- [lab] FAIL: 0
-- [lab] KẾT QUẢ: ĐẠT</code></pre>
<p>61 PASS = 25 của nền (<code>00</code> → <code>02</code>) + 36 của <code>20</code>, gồm 3 lỗi mong đợi qua <code>lab.expect_error</code> (111, 8115, 8710). Chạy riêng <code>20</code> lần thứ hai trên cùng database cho kết quả giống từng dòng, và kiểm tra cuối tệp xác nhận dữ liệu không đổi: <code>PASS cuối file trips | payments | doanh thu = 3174|2826|182287000</code>. Fixture P02 sinh tất định bằng hàm băm số học, không <code>NEWID()</code>/<code>RAND()</code>, nên lần chạy nào cũng ra cùng 7 dòng. Phiên bản: SQL Server <b>16.0.4325.4 (RTM-CU28)</b>, Developer Edition trong Docker, như README của lab.</p>
<p>Phần "chưa kiểm" ghi thẳng: mỗi tệp là một phiên, nên chưa có gì về hai phiên cùng ghi giao dịch trong lúc đối soát chạy (Phần 4); chưa đo tốc độ hay xem execution plan của view (Phần 3), và truy vấn chạy nhanh trên 2.826 giao dịch không nói gì về bảng lớn gấp nghìn lần; chưa chạy trên Standard Edition hay trên Windows.</p>
</details>

## 4. Tiêu chí tự chấm

Mỗi tiêu chí 10 điểm. Từ 80 điểm là đạt để sang Phần 3; tiêu chí 2, 4 và 6 bắt buộc.

| # | Tiêu chí | Đạt khi |
|---|---|---|
| 1 | Đặc tả viết trước | Phạm vi, phải thu, thực thu, khớp, thứ tự phân loại có bằng lời, khớp với T-SQL |
| 2 | Đúng grain | Một dòng một chuyến; bất biến 3.168 = 3.168 = 3.168; 0 giao dịch ngoài sổ |
| 3 | Một định nghĩa | Công thức thực thu và phân loại nằm ở một view, không chép lại ở từng truy vấn; tệp chạy lại được |
| 4 | Ba tổng đúng | 3.168 / 182.287.000 / 182.555.000 / +268.000, tiền cộng bằng `bigint` |
| 5 | Đối soát số dòng | Các nhóm rời nhau, cộng lại bằng phạm vi; có cầu nối từ tổng charge tới thực thu |
| 6 | Đáp án chính xác | Đúng 7 chuyến, đúng loại, tổng theo loại = +268.000 |
| 7 | Không báo nhầm | Mọi ca khó nhưng đúng ở D5 không có nhãn lệch |
| 8 | Thời điểm | Phân biệt được chênh do ngày đặt/ngày giao dịch với lỗi thật |
| 9 | Giải thích dùng được | Mỗi dòng lệch có giả thuyết và hành động đề xuất; có danh sách việc cần theo dõi |
| 10 | Bằng chứng trung thực | Mỗi con số chính có `expect_value`, kết quả chạy thật, phiên bản, điều chưa kiểm |

## 5. Bẫy hay gặp khi tự làm

- **JOIN `trips` với `payments` rồi `SUM(fare)`.** Chuyến có hai giao dịch bị cộng cước hai lần. Gom trước hoặc dùng `APPLY`.
- **`INNER JOIN` với giao dịch đã gom.** 485 chuyến không có giao dịch biến mất, trong đó có 100724, một trong 7 đáp án.
- **Lọc `status = 'succeeded'` ở `WHERE` trước khi gom.** Tổng tiền vẫn đúng, nhưng 101084 (chỉ có giao dịch `failed`) mất khỏi cột bằng chứng.
- **`ISNULL(cột varchar, N'…')`.** Kết quả mang kiểu `varchar` của tham số đầu: tiếng Việt thành dấu `?`, và chuỗi thay thế bị cắt theo độ dài của cột đầu.
- **`SUM` trên `int` cho báo cáo dài kỳ.** Lỗi 8115 xuất hiện đúng ngày dữ liệu đủ lớn, không phải lúc viết truy vấn.
- **Thêm `STRING_AGG` vào câu `GROUPING SETS`/`ROLLUP`.** Lỗi 8710: SQL Server không gộp được chuỗi từ các nhóm con. Liệt kê mã ở một câu `GROUP BY` thường (bài tập 1).
- **`CREATE VIEW` giữa script không có `GO` phía trước.** Lỗi 111. Trong DBStudio, `Ctrl+Enter` chỉ gửi batch đang đứng, nên đặt con trỏ đúng batch của view.
- **Phân loại không có thứ tự, hoặc coi hai charge thành công là thu trùng.** 100048 trả một phần bằng hai phương thức; chỉ cùng số tiền mới là dấu hiệu lặp yêu cầu, và ngay cả thế vẫn chỉ là tín hiệu (bài tập 3).
- **Lọc theo ngày giao dịch cho sổ theo chuyến.** Mất 5.000 của 103165 và gán nhầm tiền chuyến cuối ngày sang hôm sau.

## 6. Tổng kết Phần 2

| Bài | Bạn đã có gì cho sản phẩm cuối phần |
|---|---|
| [MS-011 · NULL, CASE, ISNULL và COALESCE](/articles/mssql-null-case-isnull-coalesce) | Phân biệt "không có giao dịch" với "giao dịch có cột NULL"; kiểu trả về của `ISNULL` so với `COALESCE` |
| [MS-012 · GROUP BY, ROLLUP và GROUPING SETS](/articles/mssql-group-by-rollup-grouping-sets) | Tổng có điều kiện, dòng tổng bằng `GROUPING SETS` + `GROUPING()`, mẫu số rõ ràng |
| [MS-013 · JOIN, APPLY và số dòng sau khi nối](/articles/mssql-join-apply-cardinality) | Gom trước rồi JOIN, `CROSS/OUTER APPLY`; bất biến số dòng của sổ |
| [MS-014 · Subquery, EXISTS, CTE và bảng tạm](/articles/mssql-subquery-exists-cte-temp-table) | Tách đối soát thành CTE có kiểm kê; `NOT EXISTS` cho "giao dịch ngoài sổ" |
| [MS-015 · Window function và mệnh đề WINDOW](/articles/mssql-window-function-window-clause) | Luỹ kế và so sánh dòng trước, nền cho báo cáo theo ngày |
| [MS-016 · Ngày giờ, AT TIME ZONE, DATETRUNC và DATE_BUCKET](/articles/mssql-ngay-gio-at-time-zone-date-bucket) | Phạm vi nửa mở `+07:00`; ngày đặt khác ngày giao dịch |
| [MS-017 · Upsert: MERGE và các mẫu an toàn hơn](/articles/mssql-upsert-merge) | Nhập lại dữ liệu không nhân đôi, điều kiện để chạy lại đối soát |
| [MS-018 · Phân trang lịch sử chuyến: OFFSET/FETCH và keyset](/articles/mssql-phan-trang-keyset) | Thứ tự xác định có khoá phụ; danh sách dài không lặp, không sót |
| [MS-019 · Recursive CTE với cây khu vực](/articles/mssql-recursive-cte-cay-khu-vuc) | Cộng dồn theo cây có câu đối chiếu với cách đếm thẳng |
| MS-020 · Bài này | Bàn giao D1–D6 có bằng chứng |

Sản phẩm của phần đúng như khung: bộ truy vấn T-SQL có expected output, đối soát số dòng và tổng tiền. Những gì **chưa** có cũng rõ: chưa đo tốc độ hay index cho các truy vấn này (Phần 3), chưa có hai phiên cùng ghi một giao dịch (Phần 4), chưa có quyền truy cập riêng cho bảng tiền (Phần 5).

**So với PostgreSQL.** Bài cùng vai trò ở series PostgreSQL là [PG-020 · Thực hành: truy vấn vận hành và đối soát thanh toán](/articles/postgresql-thuc-hanh-sql-doi-soat), trên cùng dữ liệu (fixture SQL Server trùng từng dòng với bản PostgreSQL), nên mọi con số chính trùng nhau. Khác ở cách viết: PostgreSQL có `FILTER`, view tạm và `string_agg(… ORDER BY …)`; T-SQL dùng `SUM(IIF(…))`, view thường tạo bằng `CREATE OR ALTER` trong batch riêng, và `STRING_AGG … WITHIN GROUP`. T-SQL thêm hai việc phải để ý mà PostgreSQL không có: `ISNULL` đổi kiểu theo tham số đầu, và `SUM(int)` trả `int` nên có thể tràn (PostgreSQL trả `bigint`). Bản JSON bằng `FOR JSON PATH` là phần riêng của bài này.

## 7. Bài tập mở rộng

Lời giải nằm ở mục 7 của `20-doi-soat-thanh-toan.sql`, chạy cùng lệnh ở mục 2. Mọi thay đổi dữ liệu nằm trong transaction kết thúc bằng `ROLLBACK`.

**Bài 1.** Giám đốc thành phố muốn biết phần chênh của mình. Lập đối soát theo thành phố có dòng tổng, chứng minh nó khớp mục 3.3, và liệt kê mã chuyến lệch của từng thành phố.

<details><summary><b>Lời giải</b></summary>
<pre><code class="language-sql">SELECT IIF(GROUPING(c.code) = 1, 'TONG', c.code) AS tp, COUNT(*) AS so_chuyen,
       SUM(s.phai_thu) AS phai_thu, SUM(s.thuc_thu) AS thuc_thu, SUM(s.chenh) AS chenh,
       SUM(IIF(s.loai_lech IS NOT NULL, 1, 0)) AS so_lech
FROM lab.so_doi_soat AS s
JOIN toinoi.cities AS c ON c.id = s.city_id
GROUP BY GROUPING SETS ((c.code), ())
ORDER BY GROUPING(c.code), c.code;</code></pre>
<pre><code class="language-text">tp|so_chuyen|phai_thu|thuc_thu|chenh|so_lech
--|---------|--------|--------|-----|-------
DN|459|26416000|26318000|-98000|1
HN|1106|62991000|62906000|-85000|2
SG|1603|92880000|93331000|451000|4
TONG|3168|182287000|182555000|268000|7</code></pre>
<p>Dòng TONG trùng mục 3.3 từng số. Thêm cột <code>STRING_AGG(…)</code> vào chính câu này thì lỗi: <code>PASS BT1 STRING_AGG trong GROUPING SETS — lỗi 8710: Aggregate functions that are used with CUBE, ROLLUP, or GROUPING SET queries must provide for the merging of subaggregates. …</code>. Liệt kê mã bằng một câu <code>GROUP BY c.code</code> thường:</p>
<pre><code class="language-text">tp|ma_lech
--|-------
DN|101084
HN|100724, 102443
SG|101310, 101987, 102642, 102739</code></pre>
<p>SG gánh +451.000 nhưng đó là tổng của bốn dòng ngược dấu (−72.000, +85.000, +96.000, +342.000). Con số ròng nhỏ có thể che các dòng lớn, nên báo cáo theo thành phố phải đi kèm danh sách.</p>
</details>

**Bài 2.** CSKH muốn giảm 10.000 đ cho chuyến 100048 (cước 119.000). Ba cách ghi: (a) chỉ thêm một refund 10.000 vào ví; (b) chỉ thêm adjustment −10.000; (c) sửa cước còn 109.000 **và** thêm adjustment −10.000. Mỗi cách làm đối soát báo gì?

<details><summary><b>Lời giải</b></summary>
<p>Mỗi cách chạy trong <code>BEGIN TRANSACTION … ROLLBACK TRANSACTION</code>, đọc sổ trước khi rollback. Cột kết quả: phải thu : thực thu : chênh : loại của 100048, rồi số dòng lệch : tổng chênh của cả sổ.</p>
<pre><code class="language-text">cach|ket_qua
----|-------
a|119000:109000:-10000:5 LỆCH SỐ TIỀN | 8:258000
b|119000:109000:-10000:5 LỆCH SỐ TIỀN | 8:258000
c|109000:109000:0:khớp | 7:268000
PASS BT2 sau ROLLBACK cước:số giao dịch của 100048 = 119000:2</code></pre>
<p>Refund ở (a) không thành loại 1 vì chuyến có charge thành công. Chỉ (c) khớp, đúng cách của ca 102235. Bài học: giảm giá phải đổi <b>cả</b> phải thu lẫn tiền; đối soát chính là phép kiểm tra hai nơi đó có nhất quán hay không. Chọn refund hay adjustment là quy ước kế toán cần chốt trong đặc tả; đối soát không tự quyết thay. Lưu ý: <code>INSERT</code> bị rollback vẫn tiêu giá trị IDENTITY của <code>payments</code> (như MS-010 đã thấy với <code>trips</code>), nên mã giao dịch không bảo đảm liên tục.</p>
</details>

**Bài 3.** Tín hiệu THU TRÙNG của sổ chỉ nhìn "cùng số tiền". Siết thành "cùng số tiền, cùng phương thức, cách nhau ≤ 60 giây". Trên fixture, đáp án có đổi không?

<details><summary><b>Lời giải</b></summary>
<p>Tự JOIN các charge thành công của cùng chuyến, cùng số tiền và phương thức; điều kiện <code>b.paid_at &gt; a.paid_at OR (b.paid_at = a.paid_at AND b.id &gt; a.id)</code> để mỗi cặp xuất hiện một lần (T-SQL không so sánh bộ giá trị như <code>(x, y) &gt; (…)</code> của PostgreSQL):</p>
<pre><code class="language-text">code|method|amount|cach_giay
----|------|------|---------
101310|wallet|85000|3</code></pre>
<p>Chỉ có một cặp, nên đáp án vẫn 7 dòng: fixture không phân biệt được hai quy tắc. Muốn phân biệt, phải dựng thêm ca, chẳng hạn hai lần thu cùng số tiền cách nhau một ngày (tín hiệu lỏng báo THU TRÙNG, tín hiệu chặt để rơi về LỆCH SỐ TIỀN). Nhãn nào đúng tuỳ người xử lý: "thu trùng" giao cho đội cổng thanh toán, "lệch số tiền" giao cho vận hành. Cách chắc chắn hơn mọi tín hiệu suy luận là khoá idempotency phía ứng dụng, chủ đề của Phần 4.</p>
</details>

## 8. Đọc tiếp

- [Microsoft Learn · STRING_AGG](https://learn.microsoft.com/sql/t-sql/functions/string-agg-transact-sql): `WITHIN GROUP (ORDER BY …)`, kiểu trả về theo kiểu đầu vào.
- [Microsoft Learn · GROUP BY](https://learn.microsoft.com/sql/t-sql/queries/select-group-by-transact-sql): `GROUPING SETS`, `ROLLUP`, hàm `GROUPING`.
- [Microsoft Learn · Format query results as JSON with FOR JSON](https://learn.microsoft.com/sql/relational-databases/json/format-query-results-as-json-with-for-json-sql-server) và [OPENJSON](https://learn.microsoft.com/sql/t-sql/functions/openjson-transact-sql).
- [Microsoft Learn · CREATE VIEW](https://learn.microsoft.com/sql/t-sql/statements/create-view-transact-sql): `CREATE OR ALTER`, quy tắc một batch.
- Lab: `examples/toi-noi-mssql/p02`: `01-fixture.sql` (mục "SAI LỆCH CÀI SẴN" và các ca khó), `20-doi-soat-thanh-toan.sql`.

Bài tiếp theo: Phần 3 · Lưu trữ, index và execution plan (đang viết).
