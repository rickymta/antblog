import type { FoodItem } from '../../src/domain/food';

/**
 * n dòng món, dòng thứ i (1-based) có đơn giá 30.000 + 1.000 × (i − 1), số lượng 1.
 * Mỗi dòng góp một số tiền KHÁC nhau ⇒ bỏ dòng đầu, bỏ dòng cuối, chỉ giữ dòng cuối… đều cho tổng khác.
 */
export function lines(n: number): FoodItem[] {
  return Array.from({ length: n }, (_, k) => ({ name: `Món ${k + 1}`, unitPrice: 30_000 + 1_000 * k, quantity: 1 }));
}

/** Oracle dạng đóng (không dùng vòng lặp): Σ (30.000 + 1.000k), k = 0…n−1. */
export function expectedSubtotal(n: number): number {
  return 30_000 * n + (1_000 * n * (n - 1)) / 2;
}

/** n dòng ở giá trần của R-FOOD-02: đơn giá 5.000.000 đ, số lượng 50 ⇒ 250.000.000 đ mỗi dòng. */
export function maxLines(n: number): FoodItem[] {
  return Array.from({ length: n }, (_, k) => ({ name: `Món ${k + 1}`, unitPrice: 5_000_000, quantity: 50 }));
}

/** Số lần lặp theo chiến lược Beizer cho vòng có max = 20 dòng: bỏ qua, 1, 2, điển hình (5), max−1, max, max+1. */
export const LOOP_COUNTS = [0, 1, 2, 5, 19, 20, 21] as const;
