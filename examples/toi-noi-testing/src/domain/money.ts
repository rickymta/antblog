/**
 * Tiền và số thập phân (SPEC R-GEN-01).
 *
 * Bên trong công thức cước, tiền được tính bằng số nguyên "milli-đồng" (1/1000 đồng) để
 * các bước trung gian chính xác tuyệt đối (×1,25 và ×percent% không sinh sai số dấu phẩy động).
 * Với đầu vào hợp lệ mọi bước đều ra số nguyên: cước cơ bản luôn là bội số của 50 đ
 * (quãng đường làm tròn 0,1 km) nên ×125/100 và ×percent/100 chia hết.
 */

export const MILLI_PER_DONG = 1000;

export function toMilli(dong: number): number {
  return Math.round(dong * MILLI_PER_DONG);
}

export function toDong(milli: number): number {
  return milli / MILLI_PER_DONG;
}

/** percent% của một số tiền milli-đồng. */
export function percentOf(milli: number, percent: number): number {
  return (milli * percent) / 100;
}

/** Làm tròn LÊN bội số 1.000 đ; nhận milli-đồng, trả đồng (SPEC R-FARE-08). */
export function roundUpToThousandDong(milli: number): number {
  const thousandsOfDong = Math.ceil(milli / (1000 * MILLI_PER_DONG));
  return thousandsOfDong * 1000;
}

/**
 * Làm tròn tới 0,1 theo kiểu half-up (0,45 ⇒ 0,5; 0,44 ⇒ 0,4).
 * Đi qua toPrecision để 8,35 × 10 = 83,49999… vẫn được hiểu là 83,5.
 */
export function roundToTenth(value: number): number {
  const tenths = Number((value * 10).toPrecision(12));
  return Math.round(tenths) / 10;
}

/** Định dạng kiểu Việt Nam: 52000 ⇒ "52.000 đ" (dùng cho thông điệp lỗi). */
export function formatDong(dong: number): string {
  return `${dong.toLocaleString('vi-VN')} đ`;
}
