/**
 * BẢN CÀI ĐẶT CÓ LỖI — chỉ dùng để minh hoạ defect report DEF-0042 trong bài
 * «Testware và các sản phẩm kiểm thử». KHÔNG phải mã của lab (mã đúng: src/domain/parcel.ts).
 *
 * Lập trình viên hiểu «mỗi kg bắt đầu» là «phần nguyên của phần vượt, cộng thêm 1».
 * Cách đếm này đúng với 5,1 hay 6,1 kg nhưng thừa 1 kg ở mọi khối lượng NGUYÊN trên 5 kg.
 */
export function buggyParcelSurcharge(weightKg: number): number {
  if (weightKg <= 5) return 0;
  return (Math.floor(weightKg - 5) + 1) * 2_000;
}
