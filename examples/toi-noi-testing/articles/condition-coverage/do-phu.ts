/**
 * TEST-045 — đếm coverage item cho MỘT quyết định nhiều điều kiện.
 * - Condition coverage: mỗi atomic condition có 2 item (true, false) ⇒ 2N item.
 * - Decision coverage: 2 item (quyết định true, quyết định false).
 * - Condition/decision coverage: gộp cả hai.
 */
import { TEN_DIEU_KIEN, type GiaTriDieuKien, type TenDieuKien } from './dieu-kien-voucher';

export interface DongChay {
  giaTri: GiaTriDieuKien;
  /** Điều kiện được tính tới; bỏ trống = coi như mọi điều kiện (cách nhìn bảng chân trị). */
  daDanhGia?: readonly TenDieuKien[];
  ketQua: boolean;
}

export interface DoPhu {
  dieuKien: { phu: number; tong: number; thieu: string[] };
  quyetDinh: { phu: number; tong: number; thieu: string[] };
}

const phanTram = (phu: number, tong: number) => `${((phu / tong) * 100).toFixed(1).replace('.', ',')} %`;
export const hienThi = (d: { phu: number; tong: number }) => `${d.phu}/${d.tong} = ${phanTram(d.phu, d.tong)}`;

export function doPhu(dong: readonly DongChay[]): DoPhu {
  const thay = new Set<string>();
  for (const d of dong) {
    for (const ten of d.daDanhGia ?? TEN_DIEU_KIEN) thay.add(`${ten}=${d.giaTri[ten] ? 'T' : 'F'}`);
  }
  const itemDieuKien = TEN_DIEU_KIEN.flatMap((ten) => [`${ten}=T`, `${ten}=F`]);
  const ketQua = new Set<string>(dong.map((d) => (d.ketQua ? 'quyếtĐịnh=T' : 'quyếtĐịnh=F')));
  const itemQuyetDinh = ['quyếtĐịnh=T', 'quyếtĐịnh=F'];
  return {
    dieuKien: {
      phu: itemDieuKien.filter((i) => thay.has(i)).length,
      tong: itemDieuKien.length,
      thieu: itemDieuKien.filter((i) => !thay.has(i)),
    },
    quyetDinh: {
      phu: itemQuyetDinh.filter((i) => ketQua.has(i)).length,
      tong: itemQuyetDinh.length,
      thieu: itemQuyetDinh.filter((i) => !ketQua.has(i)),
    },
  };
}
