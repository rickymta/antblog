/**
 * Bốn bản cài đặt SAI của vòng cộng tiền món (SPEC R-FOOD-02: tiền món = Σ đơn giá × số lượng),
 * mỗi bản mang một dạng lỗi vòng lặp kinh điển. Chỉ dùng để minh hoạ loop testing — không sửa src/.
 */
import type { FoodItem } from '../../src/domain/food';

export const MAX_ITEMS = 20; // SPEC R-FOOD-02: 1–20 dòng món

export type SubtotalFn = (items: readonly FoodItem[]) => number;

const line = (item: FoodItem) => item.unitPrice * item.quantity;

/** L1 — bắt đầu sai chỉ số: bỏ dòng đầu tiên. */
export const skipFirstLine: SubtotalFn = (items) => {
  let sum = 0;
  for (let i = 1; i < items.length; i++) sum += line(items[i]!);
  return sum;
};

/** L2 — khởi tạo biến cộng dồn BÊN TRONG vòng: chỉ còn dòng cuối. */
export const resetInsideLoop: SubtotalFn = (items) => {
  let result = 0;
  for (const item of items) {
    let sum = 0;
    sum += line(item);
    result = sum;
  }
  return result;
};

/** L3 — chặn trên lệch một: «tối đa 20 dòng» viết thành `< MAX_ITEMS - 1`. */
export const capOffByOne: SubtotalFn = (items) => {
  let sum = 0;
  for (let i = 0; i < Math.min(items.length, MAX_ITEMS - 1); i++) sum += line(items[i]!);
  return sum;
};

/** L4 — «phòng thủ» bằng cách lặng lẽ cắt bớt: dòng thứ 21 trở đi không được cộng, cũng không bị từ chối. */
export const silentTruncation: SubtotalFn = (items) => {
  let sum = 0;
  for (let i = 0; i < Math.min(items.length, MAX_ITEMS); i++) sum += line(items[i]!);
  return sum;
};
