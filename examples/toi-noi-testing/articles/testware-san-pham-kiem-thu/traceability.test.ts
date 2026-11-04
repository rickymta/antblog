/**
 * Kiểm ma trận truy vết hai chiều giữa test basis (SPEC.md) và testware (TD-PARCEL-01):
 * - xuôi: mọi yêu cầu R-PARCEL-xx của SPEC có ít nhất một test case;
 * - ngược: mọi mã yêu cầu mà testware trích dẫn đều tồn tại trong SPEC.
 */
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import data from './test-data/td-parcel-01.json';

const spec = readFileSync(new URL('../../SPEC.md', import.meta.url), 'utf8');
const specParcelIds = [...new Set(spec.match(/R-PARCEL-\d{2}/g) ?? [])].sort();

/** Ma trận: mã yêu cầu → danh sách test case phủ nó. */
function matrix(): Record<string, string[]> {
  const m: Record<string, string[]> = Object.fromEntries(specParcelIds.map((id) => [id, [] as string[]]));
  for (const row of data.rows) {
    for (const req of row.requirements) {
      const list = (m[req] ??= []);
      if (!list.includes(row.testCase)) list.push(row.testCase);
    }
  }
  return m;
}

describe('Truy vết R-PARCEL ↔ TD-PARCEL-01', () => {
  it('xuôi: mọi yêu cầu R-PARCEL của SPEC đều có test case', () => {
    expect(specParcelIds).toEqual(['R-PARCEL-01', 'R-PARCEL-02', 'R-PARCEL-03', 'R-PARCEL-04']);
    expect(matrix()).toEqual({
      'R-PARCEL-01': ['TC-PARCEL-001', 'TC-PARCEL-002', 'TC-PARCEL-004'],
      'R-PARCEL-02': ['TC-PARCEL-003'],
      'R-PARCEL-03': ['TC-PARCEL-001', 'TC-PARCEL-002', 'TC-PARCEL-003', 'TC-PARCEL-004'],
      'R-PARCEL-04': ['TC-PARCEL-005'],
    });
  });

  it('ngược: testware không trích dẫn mã yêu cầu không có trong SPEC', () => {
    const cited = new Set(data.rows.flatMap((r) => r.requirements));
    expect([...cited].filter((id) => !specParcelIds.includes(id))).toEqual([]);
  });
});
