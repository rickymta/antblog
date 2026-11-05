/**
 * TEST-018 · Kiểm chính lời giải phân loại: phủ ĐỦ mọi R-NFR đang có trong SPEC.md
 * và chỉ dùng tên đặc tính của ISO/IEC 25010:2023.
 */
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { ISO25010_2023, PHAN_LOAI } from './phan-loai-r-nfr';

const spec = readFileSync(fileURLToPath(new URL('../../SPEC.md', import.meta.url)), 'utf8');
const nfrIds = [...new Set(spec.match(/R-NFR-\d{2}/g) ?? [])].sort();

describe('Phân loại R-NFR theo ISO/IEC 25010:2023', () => {
  it('SPEC có 7 yêu cầu R-NFR và lời giải phân loại đúng 7 mã đó', () => {
    expect(nfrIds).toHaveLength(7);
    expect(Object.keys(PHAN_LOAI).sort()).toEqual(nfrIds);
  });

  it('mọi đặc tính được dùng đều là một trong 9 đặc tính của bản 2023', () => {
    for (const m of Object.values(PHAN_LOAI)) {
      const used = [...(m.chinh ? [m.chinh.dacTinh] : []), ...m.phu];
      for (const c of used) expect(ISO25010_2023).toContain(c);
    }
  });
});
