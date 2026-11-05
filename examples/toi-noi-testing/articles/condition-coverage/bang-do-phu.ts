/**
 * TEST-045 — in bảng giá trị điều kiện và coverage của ba bộ test.
 * Chạy: npx tsx articles/condition-coverage/bang-do-phu.ts
 */
import { BO_A, BO_B, BO_C, HN11 } from './bo-test';
import { chayThat, giaTriTheoSpec, quyetDinh, TEN_DIEU_KIEN, type DuLieuTest } from './dieu-kien-voucher';
import { doPhu, hienThi } from './do-phu';

const tf = (b: boolean) => (b ? 'T' : 'F');

for (const [ten, bo] of [['Bộ A', BO_A], ['Bộ B', BO_B], ['Bộ C', BO_C]] as [string, DuLieuTest[]][]) {
  console.log(`\n${ten}   trongHan dungDichVu dungThanhPho duTien | quyết định | code thật (điều kiện đã tính)`);
  for (const t of bo) {
    const g = giaTriTheoSpec(HN11, t);
    const r = chayThat(HN11, t);
    const cot = TEN_DIEU_KIEN.map((k) => (r.daDanhGia.includes(k) ? ` ${tf(g[k])} ` : `(${tf(g[k])})`));
    console.log(`  ${t.id}    ${cot.join('        ')}      |     ${tf(quyetDinh(g))}      | ${r.ma ?? `OK giảm ${r.tienGiamDong} đ`}`);
  }
  const logic = doPhu(bo.map((t) => ({ giaTri: giaTriTheoSpec(HN11, t), ketQua: quyetDinh(giaTriTheoSpec(HN11, t)) })));
  const that = doPhu(bo.map((t) => {
    const r = chayThat(HN11, t);
    return { giaTri: giaTriTheoSpec(HN11, t), daDanhGia: r.daDanhGia, ketQua: r.apDuoc };
  }));
  console.log(`  condition coverage — bảng chân trị: ${hienThi(logic.dieuKien)} · code thật: ${hienThi(that.dieuKien)}`);
  console.log(`  decision coverage  — ${hienThi(logic.quyetDinh)}${logic.quyetDinh.thieu.length ? ` (thiếu ${logic.quyetDinh.thieu.join(', ')})` : ''}`);
}
console.log('\n(T)/(F) = giá trị theo SPEC nhưng code thật KHÔNG tính tới vì đã ném lỗi trước đó.');
