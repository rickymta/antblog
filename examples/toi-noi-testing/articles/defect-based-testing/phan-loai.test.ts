/**
 * TEST-054 — phân loại 30 defect lịch sử: cùng một dữ liệu, nhìn theo module và theo mô hình lỗi cho hai bức tranh khác nhau.
 * Chạy: npx vitest run articles/defect-based-testing/phan-loai.test.ts
 */
import { describe, expect, it } from 'vitest';
import { formatTable, groupBy, loadDefects } from './phan-loai';

const defects = loadDefects();

describe('TEST-054 · phân loại defect lịch sử', () => {
  it('đủ 30 defect, mọi defect có đủ các thuộc tính phân loại', () => {
    expect(defects).toHaveLength(30);
    for (const d of defects) {
      expect([d.module, d.cum, d.odc, d.ctfl, d.muc_do, d.dua_vao, d.phat_hien].every((v) => v !== '')).toBe(true);
    }
  });

  it('theo module: đếm số lượng thì Pricing đứng đầu (góc nhìn TEST-003), tính trọng số mức độ thì Trip vượt lên', () => {
    const byCount = groupBy(defects, 'module', 'count');
    console.log(formatTable('\n== Theo module (sắp theo SL) ==', byCount));
    expect(byCount[0]?.key).toBe('Pricing');
    expect(groupBy(defects, 'module', 'weighted')[0]?.key).toBe('Trip');
  });

  it('theo mô hình lỗi (sắp theo điểm có trọng số): ba cụm đầu là trang-thai, gio-vn, lam-tron', () => {
    const byCluster = groupBy(defects, 'cum');
    console.log(formatTable('\n== Theo mô hình lỗi (cum, sắp theo điểm) ==', byCluster));
    console.log(formatTable('\n== Theo nhóm CTFL 4.4.1 ==', groupBy(defects, 'ctfl')));
    console.log(formatTable('\n== Theo loại ODC ==', groupBy(defects, 'odc')));
    expect(byCluster.slice(0, 3).map((s) => s.key)).toEqual(['trang-thai', 'gio-vn', 'lam-tron']);
    const gio = byCluster.find((s) => s.key === 'gio-vn')!;
    expect(gio.modules.length).toBeGreaterThanOrEqual(4); // cụm cắt ngang nhiều module — nhìn theo module sẽ không thấy
  });
});
