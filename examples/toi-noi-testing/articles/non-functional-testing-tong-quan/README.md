# Non-functional Testing: tổng quan theo ISO/IEC 25010 (TEST-018)

Thư mục của bài [Non-functional Testing: tổng quan theo ISO/IEC 25010](https://antblog.xyz/articles/non-functional-testing-tong-quan).

| Tệp | Nội dung |
|---|---|
| `phan-loai-r-nfr.ts` | Lời giải bài thực hành: gán R-NFR-01…07 vào đặc tính ISO/IEC 25010:2023 |
| `phan-loai-r-nfr.test.ts` | Kiểm lời giải phủ đủ mọi `R-NFR-xx` đang có trong `SPEC.md` và chỉ dùng 9 đặc tính của bản 2023 |
| `nfr-tu-test-chuc-nang.test.ts` | Ba kiểm tra phi chức năng dựng từ một test chức năng (báo giá bưu kiện 6,1 kg ⇒ 62.000 đ): NF-01 time behaviour, NF-02 confidentiality, NF-03 resistance |

## Lệnh chạy

```bash
npx vitest run articles/non-functional-testing-tong-quan
```

Kết quả mong đợi: 2 file, **5 test đạt**; NF-01 in một dòng `NF-01: n=200 p50=… p95=… max=…`.

**Giới hạn:** NF-01 đo trong một tiến trình, trên một máy, kho trong bộ nhớ, không mạng, không MapViet — chỉ là cảnh báo sớm khi một thay đổi làm báo giá chậm hẳn đi. Nó **không** phải performance test và không chứng minh R-NFR-03 đạt trên hệ thống thật (xem Series 09). Số đo phụ thuộc máy; khi trích dẫn phải ghi cấu hình máy.
