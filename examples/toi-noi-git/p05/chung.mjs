// Tiện ích chung cho các bài Phần 5.
import { join } from 'node:path';

import { places5 } from '../lib/history5.mjs';
import { WORK } from '../lib/lab.mjs';

export { checkRefs3 as checkRefs5 } from '../p03/chung.mjs';

/** Đường dẫn các repository của Phần 5 (Minh ở work/bao-gia). */
export const P = places5(join(WORK, 'bao-gia'));
export const LIB_SHOW = '<lab>/work/remote/tien-te.git';
export const REMOTE_URL = `file:///${P.remote.replace(/^\//, '')}`;
export const REMOTE_SHOW = 'file:///<lab>/work/remote/bao-gia.git';
