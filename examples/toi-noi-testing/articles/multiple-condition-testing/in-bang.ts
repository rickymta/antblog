/**
 * TEST-046 — in bảng chân trị 32 dòng: mong đợi (R-ACCEPT-02), code thật, bản sai thứ tự.
 * Chạy: npx tsx articles/multiple-condition-testing/in-bang.ts
 */
import { bangMucCode } from './bang-chan-tri';
import { chayDong } from './dung-tinh-huong';
import { acceptTripSaiThuTu } from './nhan-chuyen-sai-thu-tu';

const bang = bangMucCode();
console.log(' #  Off Busy City Xe  Srch | mong đợi            | code thật           | bản sai thứ tự');
for (const d of bang) {
  const o = Object.values(d.to).map((b) => (b ? 'T' : 'F'));
  const dau = `${String(d.so).padStart(2)}   ${o[0]}    ${o[1]}    ${o[2]}   ${o[3]}   ${o[4]}  | `;
  if (!d.khaThi) {
    console.log(`${dau}bất khả thi: ${d.lyDoBatKhaThi}`);
    continue;
  }
  const that = await chayDong(d.to);
  const sai = await chayDong(d.to, acceptTripSaiThuTu);
  const danh = (kq: string) => (kq === d.mongDoi ? kq : `${kq} ✗`).padEnd(19);
  console.log(`${dau}${d.mongDoi.padEnd(19)} | ${danh(that)} | ${danh(sai)}`);
}
const khaThi = bang.filter((d) => d.khaThi).length;
console.log(`\nTổ hợp: ${bang.length} · bất khả thi: ${bang.length - khaThi} · khả thi đã chạy: ${khaThi}`);
