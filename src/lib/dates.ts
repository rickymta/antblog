const VN_OFFSET_MS = 7 * 3600 * 1000;
const DAY_MS = 24 * 3600 * 1000;

/** Ngày lịch Việt Nam (UTC+7) dạng `YYYY-MM-DD`. */
export function vnDate(d: Date = new Date()): string {
  return new Date(d.getTime() + VN_OFFSET_MS).toISOString().slice(0, 10);
}

/** Cộng/trừ ngày trên chuỗi `YYYY-MM-DD`. */
export function addDays(date: string, days: number): string {
  return new Date(Date.parse(`${date}T00:00:00Z`) + days * DAY_MS).toISOString().slice(0, 10);
}

/** Mốc UTC của 00:00 giờ VN ngày `date`. */
export function vnDayStart(date: string): Date {
  return new Date(Date.parse(`${date}T00:00:00Z`) - VN_OFFSET_MS);
}

/** Danh sách ngày liên tiếp từ `from` tới `to` (bao gồm hai đầu). */
export function dateRange(from: string, to: string): string[] {
  const out: string[] = [];
  for (let d = from; d <= to; d = addDays(d, 1)) out.push(d);
  return out;
}
