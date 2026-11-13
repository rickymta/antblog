# SYS-07 — Báo cáo Performance Test

Ngày 13/11/2026, kết thúc 20:52:47 giờ Việt Nam (13:52:47 UTC trong JSON). Working tree sau SYS-06 `5d60302` với script mới. Windows 10.0.26300, Node 24.15.0, Intel Core i7-10700K 3,80 GHz, 16 logical CPUs, RAM 34.245.918.720 bytes (~31,9 GiB). Không ghi nhận CPU background của các ứng dụng khác; đây là hạn chế kiểm soát môi trường.

Lệnh tại lab: `npx tsx articles/system-testing-performance-test/baseline.ts`, exit 0. Ba vòng, mỗi vòng 50 warm-up và 300 measured requests. Tổng 1.050 request, 900 mẫu đo, không lỗi. Gate giả định local error=0 và p95<1.000 ms đạt ở cả ba vòng. `npm run typecheck` tại lab đạt, exit 0; kiểm lại JSON độc lập xác nhận đủ 900 mẫu, failure count và p95 khớp summary.

| Vòng | Thời gian đo ms | p95 ms | p99 ms | Max ms | Valid RPS | CPU chung ms | RSS đầu/cuối bytes |
|---|---:|---:|---:|---:|---:|---:|---|
| 1 | 226,0935 | 1,1388 | 1,6418 | 5,4078 | 1.326,88 | 313 | 139264000 / 164614144 |
| 2 | 188,4008 | 0,9504 | 1,7715 | 6,4522 | 1.592,35 | 234 | 171724800 / 177557504 |
| 3 | 154,7974 | 0,6909 | 1,3307 | 6,3169 | 1.938,02 | 218 | 178257920 / 182837248 |

Evidence raw: `examples/toi-noi-testing/articles/system-testing-performance-test/baseline-observed.json`. Script chạy lại ghi file dưới reports, không thay bản evidence đã commit. Percentiles nearest-rank trên mọi attempt đo; response body được kiểm total=35.000 và quoteId dạng chuỗi.

CPU user+system có thể lớn hơn wall time vì process có hoạt động trên nhiều thread; đó không phải số CPU riêng của server. RSS đầu/cuối không phải peak. Client/server cùng process, closed model concurrency 1, memory store, clock/weather giả, request loopback; mỗi cửa sổ dưới 0,3 giây. Không dùng RPS này làm capacity bền vững hoặc khẳng định đạt NFR production.

Chưa chạy tải mở, tải dài, PostgreSQL thật, browser, provider ngoài hoặc stress. Chưa đo độ biến thiên giữa process/máy độc lập. Không thay src sản phẩm và không rebuild web/admin. Đây là baseline phương pháp đo, không phải kết quả chứng nhận hiệu năng hệ thống.
