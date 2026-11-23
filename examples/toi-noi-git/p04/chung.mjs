// Tiện ích chung cho các bài Phần 4 (cùng ba repository với Phần 3).
export { P, checkRefs3 as checkRefs4 } from '../p03/chung.mjs';

/** Mã ngắn của một commit trong thư mục hiện tại của phiên. */
export const short = (s, rev) => s.value(`rev-parse --short ${rev}`);
