/**
 * TEST-032 · Ba bản cài đặt CÓ LỖI CỐ Ý ở biên (bản đúng: src/domain/fare.ts, src/domain/parcel.ts).
 * Mỗi hàm trả `true` nếu giá trị được CHẤP NHẬN. Chỉ dùng trong bva-so-sanh.test.ts.
 */
import { roundToTenth } from '../../src/domain/money';

export type Accepts = (raw: number) => boolean;

/** Lỗi B — kiểm khoảng TRƯỚC khi làm tròn (sai thứ tự R-FARE-02). */
export const quangDuongKiemTruocLamTron: Accepts = (raw) => Number.isFinite(raw) && raw >= 0.5 && raw <= 100;

/** Lỗi T — làm tròn bằng toFixed(1): làm tròn theo giá trị nhị phân thật, nên 100,05 ⇒ "100.0". */
export const quangDuongToFixed: Accepts = (raw) => {
  const km = Number(raw.toFixed(1));
  return Number.isFinite(km) && km >= 0.5 && km <= 100;
};

/** Lỗi K — «khối lượng 0 là sai» viết thành `kg === 0` thay vì `kg <= 0`: số âm lọt qua. */
export const khoiLuongBangKhong: Accepts = (raw) => {
  const kg = roundToTenth(raw);
  return Number.isFinite(kg) && kg !== 0 && kg <= 30;
};
