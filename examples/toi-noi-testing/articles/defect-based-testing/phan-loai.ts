/**
 * TEST-054 — đọc 30 defect lịch sử (giả định) và đếm theo từng thuộc tính phân loại.
 * Mỗi defect được phân loại theo NHIỀU trục độc lập (kiểu ODC — Chillarege 1992):
 *   module (ở đâu) · cum (mô hình lỗi / fault model) · odc (loại defect) · ctfl (nhóm của CTFL 4.4.1)
 *   · muc_do · dua_vao (pha đưa defect vào) · phat_hien (nơi phát hiện) · tai_dien_cua (lặp lại defect nào).
 */
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

export interface Defect {
  id: string;
  tom_tat: string;
  module: string;
  cum: string;
  odc: string;
  ctfl: string;
  muc_do: 'critical' | 'major' | 'minor' | 'trivial';
  dua_vao: string;
  phat_hien: string;
  tai_dien_cua: string;
}

/** Trọng số mức độ (giả định của bài) — TA-5.3.1: cụm nhỏ nhưng nghiêm trọng quan trọng hơn cụm lớn nhưng nhẹ. */
export const WEIGHT: Record<Defect['muc_do'], number> = { critical: 10, major: 5, minor: 2, trivial: 1 };

export function loadDefects(): Defect[] {
  const path = fileURLToPath(new URL('./lich-su-defect.csv', import.meta.url));
  const [header, ...rows] = readFileSync(path, 'utf8').trim().split(/\r?\n/);
  const keys = header!.split(';');
  return rows.map((row) => Object.fromEntries(row.split(';').map((v, i) => [keys[i], v])) as unknown as Defect);
}

export interface ClusterStat {
  key: string;
  count: number;
  weighted: number;
  critical: number;
  production: number;
  recurring: number;
  modules: string[];
}

/** Gom theo một thuộc tính; sắp theo số lượng (`count`) hoặc theo điểm có trọng số mức độ (`weighted`). */
export function groupBy(defects: Defect[], attr: keyof Defect, sortBy: 'count' | 'weighted' = 'weighted'): ClusterStat[] {
  const map = new Map<string, Defect[]>();
  for (const d of defects) map.set(d[attr], [...(map.get(d[attr]) ?? []), d]);
  return [...map.entries()]
    .map(([key, list]) => ({
      key,
      count: list.length,
      weighted: list.reduce((s, d) => s + WEIGHT[d.muc_do], 0),
      critical: list.filter((d) => d.muc_do === 'critical').length,
      production: list.filter((d) => d.phat_hien === 'production').length,
      recurring: list.filter((d) => d.tai_dien_cua !== '').length,
      modules: [...new Set(list.map((d) => d.module))].sort(),
    }))
    .sort((a, b) => b[sortBy] - a[sortBy] || b.count - a.count || b.weighted - a.weighted || a.key.localeCompare(b.key));
}

export function formatTable(title: string, stats: ClusterStat[]): string {
  const lines = [`${title}`, 'nhóm                  SL  điểm  crit  prod  tái_diễn  module'];
  for (const s of stats) {
    lines.push(
      `${s.key.padEnd(21)} ${String(s.count).padStart(2)}  ${String(s.weighted).padStart(4)}  ${String(s.critical).padStart(4)}` +
        `  ${String(s.production).padStart(4)}  ${String(s.recurring).padStart(8)}  ${s.modules.join(', ')}`,
    );
  }
  return lines.join('\n');
}
