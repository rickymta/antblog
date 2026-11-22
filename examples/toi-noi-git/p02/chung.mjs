// Tiện ích chung cho các bài Phần 2.
import { refsOf } from '../lib/history2.mjs';

/** So mọi nhánh và tag của repository hiện tại với lịch sử chuẩn sau n bước. */
export function checkRefs(s, expected, label = 'nhánh và tag trùng lịch sử chuẩn') {
  const now = refsOf(s.cwd);
  const a = JSON.stringify(now);
  const b = JSON.stringify(expected);
  const diff = Object.keys({ ...now, ...expected })
    .filter((k) => now[k] !== expected[k])
    .map((k) => `${k}: ${now[k]?.slice(0, 7) ?? '—'} ≠ ${expected[k]?.slice(0, 7) ?? '—'}`)
    .join('; ');
  s.check(label, a === b, diff);
}
