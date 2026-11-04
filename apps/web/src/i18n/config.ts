/**
 * Bật giao diện song ngữ (nút VI/EN + cookie `tkh-locale`) — `NEXT_PUBLIC_ENABLE_I18N=true` LÚC BUILD (biến NEXT_PUBLIC_*
 * được nhúng vào bundle). Mặc định TẮT (tạm ẩn theo yêu cầu): không có nút VI/EN ở đâu cả, ngôn ngữ luôn `vi`,
 * máy chủ KHÔNG đọc cookie ⇒ HTML không phụ thuộc cookie của request (trang bài/lộ trình được ISR, cache được).
 * Tài nguyên tiếng Anh + mã nút vẫn giữ nguyên: bật lại cờ là chạy như trước.
 */
// So sánh TRỰC TIẾP giá trị nhúng lúc build (không trim/lowercase): bundler gập thành hằng `false` ⇒ cắt được nhánh chết
// (tài nguyên tiếng Anh không vào bundle trình duyệt khi tắt cờ). Giá trị hợp lệ: đúng chuỗi `true`.
export const I18N_ENABLED = process.env.NEXT_PUBLIC_ENABLE_I18N === 'true';
