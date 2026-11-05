/**
 * Hàm làm tròn 0,1 half-up (SPEC R-FARE-02 cho quãng đường, R-PARCEL-01 cho khối lượng)
 * qua các bản build giả định của bài TEST-019. Bản rc3 (đúng) chính là `roundToTenth`
 * trong src/domain/money.ts — không chép lại ở đây.
 *
 * Lưu ý kỹ thuật: tệp này được nạp BÊN TRONG vi.mock của src/domain/money.ts,
 * nên tuyệt đối không import src/domain/money (sẽ thành mock vòng tròn).
 */

/**
 * Build rc1 — CÓ DEFECT. `toFixed` làm tròn theo giá trị nhị phân thật của số:
 * 7,35 được lưu là 7,34999999999999964… nên ra "7.3" thay vì 7,4.
 * Lệch ở 398/996 giá trị dạng x,x5 trong [0,45; 99,95] (ca CF-06).
 */
export function roundTenthRc1(value: number): number {
  return Number(value.toFixed(1));
}

/**
 * Build rc2 — «sửa» lần 1, SAI. Developer hiểu «half-up» thành «làm tròn lên»:
 * 7,35 ⇒ 7,4 (đúng, confirmation test đạt) nhưng 0,44 ⇒ 0,5 và 100,04 ⇒ 100,1 (sai).
 */
export function roundTenthRc2(value: number): number {
  return Math.ceil(value * 10) / 10;
}
