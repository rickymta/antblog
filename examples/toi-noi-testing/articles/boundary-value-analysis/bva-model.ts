/**
 * TEST-032 · Sinh coverage item cho BVA 2-value và 3-value (CTFL v4.0.1 mục 4.2.2).
 * Mọi giá trị biểu diễn bằng SỐ NGUYÊN BƯỚC theo độ chính xác của miền (0,01 km → 1 bước)
 * để không dính sai số dấu phẩy động khi tìm «hàng xóm gần nhất».
 */

export interface OrderedPartition {
  label: string;
  min: number | null; // bước; null = không chặn dưới
  max: number | null; // bước; null = không chặn trên
}

export interface OrderedDomain {
  name: string;
  scale: number; // số bước trong một đơn vị: 100 ⇒ độ chính xác 0,01; 1 ⇒ 1 giây
  partitions: OrderedPartition[]; // liền nhau, tăng dần
}

/** Giá trị biên = min và max của từng phân vùng (bỏ đầu không chặn). */
export function boundaryValues(domain: OrderedDomain): number[] {
  const values = domain.partitions.flatMap((p) => [p.min, p.max]).filter((v): v is number => v !== null);
  return [...new Set(values)].sort((a, b) => a - b);
}

function partitionOf(domain: OrderedDomain, step: number): number {
  return domain.partitions.findIndex((p) => (p.min === null || step >= p.min) && (p.max === null || step <= p.max));
}

/** 2-value: mỗi giá trị biên + hàng xóm gần nhất thuộc phân vùng LIỀN KỀ. */
export function twoValueItems(domain: OrderedDomain): number[] {
  const items = new Set<number>();
  for (const b of boundaryValues(domain)) {
    items.add(b);
    for (const n of [b - 1, b + 1]) if (partitionOf(domain, n) !== partitionOf(domain, b)) items.add(n);
  }
  return [...items].sort((a, b) => a - b);
}

/** 3-value: mỗi giá trị biên + CẢ HAI hàng xóm. */
export function threeValueItems(domain: OrderedDomain): number[] {
  const items = new Set<number>();
  for (const b of boundaryValues(domain)) [b - 1, b, b + 1].forEach((v) => items.add(v));
  return [...items].sort((a, b) => a - b);
}

export const toNumber = (domain: OrderedDomain, step: number) => step / domain.scale;

// ---- Các miền của bài (SPEC R-FARE-02, R-PARCEL-01, R-PARCEL-02, R-FARE-04)

/** Quãng đường THÔ client gửi, độ chính xác giả định 0,01 km. */
export const DISTANCE_RAW: OrderedDomain = {
  name: 'quãng đường thô (0,01 km)',
  scale: 100,
  partitions: [
    { label: 'D1 < 0,5 sau làm tròn', min: null, max: 44 },
    { label: 'D2 hợp lệ', min: 45, max: 10_004 },
    { label: 'D3 > 100 sau làm tròn', min: 10_005, max: null },
  ],
};

/** Quãng đường SAU làm tròn, độ chính xác 0,1 km. */
export const DISTANCE_ROUNDED: OrderedDomain = {
  name: 'quãng đường sau làm tròn (0,1 km)',
  scale: 10,
  partitions: [
    { label: 'D1', min: null, max: 4 },
    { label: 'D2', min: 5, max: 1_000 },
    { label: 'D3', min: 1_001, max: null },
  ],
};

export const WEIGHT_RAW: OrderedDomain = {
  name: 'khối lượng thô (0,01 kg)',
  scale: 100,
  partitions: [
    { label: 'W1 ≤ 0 sau làm tròn', min: null, max: 4 },
    { label: 'W2–W3 hợp lệ', min: 5, max: 3_004 },
    { label: 'W4 > 30 sau làm tròn', min: 3_005, max: null },
  ],
};

export const WEIGHT_ROUNDED: OrderedDomain = {
  name: 'khối lượng sau làm tròn (0,1 kg)',
  scale: 10,
  partitions: [
    { label: 'W1', min: null, max: 0 },
    { label: 'W2–W3', min: 1, max: 300 },
    { label: 'W4', min: 301, max: null },
  ],
};

/** Ngưỡng xe máy R-PARCEL-02 (khối lượng sau làm tròn). Chỉ mô hình biên ngưỡng — biên 0,1 và 30,0 đã ở WEIGHT_ROUNDED. */
export const BIKE_LIMIT: OrderedDomain = {
  name: 'ngưỡng xe máy (0,1 kg)',
  scale: 10,
  partitions: [
    { label: 'xe máy được', min: null, max: 200 },
    { label: 'bắt buộc ô tô', min: 201, max: null },
  ],
};

const hms = (h: number, m = 0, s = 0) => h * 3600 + m * 60 + s;

/** Giờ trong ngày theo giờ VN, độ chính xác 1 giây. Đêm (19:00 → 10:59:59 hôm sau) vòng qua nửa đêm. */
export const PEAK_TIME: OrderedDomain = {
  name: 'giờ đặt (1 giây)',
  scale: 1,
  partitions: [
    { label: 'sáng (thường)', min: null, max: hms(10, 59, 59) },
    { label: 'cao điểm trưa', min: hms(11), max: hms(12, 59, 59) },
    { label: 'chiều (thường)', min: hms(13), max: hms(16, 59, 59) },
    { label: 'cao điểm tối', min: hms(17), max: hms(18, 59, 59) },
    { label: 'tối (thường)', min: hms(19), max: null },
  ],
};

export const fmtTime = (sec: number) =>
  [Math.floor(sec / 3600), Math.floor((sec % 3600) / 60), sec % 60].map((n) => String(n).padStart(2, '0')).join(':');
