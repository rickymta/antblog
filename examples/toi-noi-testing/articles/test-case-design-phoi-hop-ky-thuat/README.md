# TCD-06 · Chọn và phối hợp kỹ thuật cho kiểm tra yêu cầu báo giá

Bài: [`/articles/test-case-design-phoi-hop-ky-thuat`](../../../../api/seed/content/test-case-design-phoi-hop-ky-thuat.md). Chạy từ `examples/toi-noi-testing`:

```bash
npx vitest run articles/test-case-design-phoi-hop-ky-thuat
```

Kết quả khi viết bài (vitest 3.2.7, Node 22): **3 tệp, 49 test case đạt**.

| Tệp | Nội dung | Số test case |
|---|---|---|
| `mo-hinh.ts` | Mô hình EP (20 phân vùng), BVA 2-value trên giá trị thô (12 giá trị), bảng quyết định thứ tự lỗi R-VALID-01 cho bưu kiện (6 cột); oracle số nguyên từ SPEC; phân loại coverage item; sinh bộ EP (13), BVA (12), DT (6), kho ca dương ghép (42); rút gọn tham lam | — |
| `ban-loi.ts` | Bản chép lớp kiểm tra có lỗi Q (quên cận dưới quãng đường) và lỗi X (quên R-PARCEL-02) | — |
| `mo-hinh.test.ts` | Oracle tính tay; độ phủ từng bộ; kho 73 ca rút còn 18 ca giữ 38/38 item; không ca thừa; 19 ca nếu không ghép; 17 ca nếu đặt biên quãng đường lên nền bưu kiện | 21 |
| `api.test.ts` | 18 ca của bộ cuối qua `POST /api/quotes` (supertest); đối chiếu oracle trên cả 73 ca; ca gộp lỗi MASK-01; lỗi định dạng không che nhau trừ khi `kind` sai | 22 |
| `che-loi.test.ts` | MASK-01 chỉ phủ C2 + R2 và xanh cả trên bản lỗi; bộ EP bắt Q bỏ X, bộ DT bắt X bỏ Q, bộ BVA và bộ cuối bắt cả hai | 6 |

Điều kiện chung: 09:00 giờ VN 16/11/2026, không mưa, không voucher, khách `C001`. Giới hạn: chỉ dịch vụ `ride` và `parcel`; không xét voucher và đồ ăn.
