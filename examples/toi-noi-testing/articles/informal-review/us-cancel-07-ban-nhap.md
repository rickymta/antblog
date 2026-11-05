# US-CANCEL-07 — Khách huỷ chuyến và xem phí

*Bản nháp 0.1 — PO viết trong buổi refinement. Đây là work product CÓ LỖI dùng để luyện informal review; nguồn chuẩn để đối chiếu là `SPEC.md` mục 10 (R-CANCEL-01…06) và R-API-11.*

**Là** khách hàng, **tôi muốn** huỷ chuyến và biết mình mất bao nhiêu tiền, **để** quyết định có huỷ hay không.

## Tiêu chí chấp nhận

- **AC1.** Khách huỷ khi đang tìm tài xế thì không mất phí.
- **AC2.** Khách huỷ trong vòng 2 phút sau khi tài xế nhận thì miễn phí.
- **AC3.** Sau 2 phút, phí huỷ là 10.000 đ.
- **AC4.** Nếu tài xế đến trễ quá 10 phút thì khách được miễn phí.
- **AC5.** Trước khi xác nhận huỷ, app hiện số phí khách sẽ mất.
- **AC6.** Sau khi huỷ, app hiện phí và lý do.
- **AC7.** Khách có thể huỷ chuyến bất cứ lúc nào trước khi hoàn thành.
