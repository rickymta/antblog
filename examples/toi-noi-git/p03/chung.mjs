// Tiện ích chung cho các bài Phần 3.
import { places, refsOf3 } from '../lib/history3.mjs';
import { WORK } from '../lib/lab.mjs';
import { join } from 'node:path';

/** Đường dẫn các repository của Phần 3 trong work/ (Minh ở work/bao-gia). */
export const P = places(join(WORK, 'bao-gia'));
/** Đường dẫn remote hiển thị trong lệnh (đường dẫn thật được thay bằng <lab>). */
export const R_SHOW = '<lab>/work/remote/bao-gia.git';
export const FORK_SHOW = '<lab>/work/remote/lan-bao-gia.git';

/** So nhánh, tag, remote-tracking của Minh, của remote chung và của Lan với lịch sử chuẩn. */
export function checkRefs3(s, expected) {
  const now = { minh: refsOf3(P.minh), remote: Object.keys(expected.remote).length ? refsOf3(P.remote) : {}, lan: expected.lan && Object.keys(expected.lan).length ? refsOf3(P.lan) : {} };
  for (const who of ['minh', 'remote', 'lan']) {
    const a = now[who];
    const b = expected[who];
    const diff = Object.keys({ ...a, ...b })
      .filter((k) => a[k] !== b[k])
      .map((k) => `${k}: ${a[k]?.slice(0, 7) ?? '—'} ≠ ${b[k]?.slice(0, 7) ?? '—'}`)
      .join('; ');
    s.check(`ref của ${who === 'minh' ? 'Minh' : who === 'lan' ? 'Lan' : 'remote'} trùng lịch sử chuẩn`, diff === '', diff);
  }
}
