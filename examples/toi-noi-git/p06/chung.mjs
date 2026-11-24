// Tiện ích chung cho các bài Phần 6.
import { spawnSync } from 'node:child_process';
import { join } from 'node:path';

import { places6 } from '../lib/history6.mjs';
import { WORK, gitEnv, normalize, shPath } from '../lib/lab.mjs';

export { checkRefs3 as checkRefs6 } from '../p03/chung.mjs';

/** Đường dẫn các repository của Phần 6 (Minh ở work/bao-gia). */
export const P = places6(join(WORK, 'bao-gia'));
export const REMOTE_URL = `file:///${P.remote.replace(/^\//, '')}`;
export const REMOTE_SHOW = 'file:///<lab>/work/remote/bao-gia.git';

/** Clone trong bài không tải file LFS (GIT-045), pack một luồng để kích thước ổn định. */
export const NO_LFS = { env: { GIT_LFS_SKIP_SMUDGE: '1', GIT_CONFIG_COUNT: '1', GIT_CONFIG_KEY_0: 'pack.threads', GIT_CONFIG_VALUE_0: '1' } };

/** Chạy `sh <args>` trong thư mục hiện tại của phiên, in lệnh và kết quả như s.git. */
export function sh(s, args, { expectFail = false, env } = {}) {
  const r = spawnSync(shPath(), args, { cwd: s.cwd, env: gitEnv({ env }), encoding: 'utf8' });
  if (r.error) throw r.error;
  const out = normalize((r.stdout ?? '') + (r.stderr ?? ''));
  s.print(`$ sh ${args.join(' ')}`);
  if (out) s.print(out);
  if (r.status !== 0 && !expectFail) throw new Error(`sh ${args.join(' ')} thoát mã ${r.status}\n${out}`);
  if (r.status === 0 && expectFail) throw new Error(`sh ${args.join(' ')} lẽ ra phải lỗi`);
  return { out: normalize(r.stdout ?? ''), status: r.status };
}
