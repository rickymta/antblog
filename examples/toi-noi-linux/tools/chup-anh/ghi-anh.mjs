// Ghi ảnh WebP, thử lại vài lần: trên Windows, file ảnh cũ có thể đang bị một tiến trình khác giữ
// (trình duyệt xem thử, trình quét virus) đúng lúc ghi đè, và sharp báo "unable to open for write".
import { createRequire } from 'node:module';
import { join } from 'node:path';

import { ROOT } from '../../lib/lab.mjs';

const sharp = createRequire(join(ROOT, '..', '..', 'package.json'))('sharp');

export async function ghiWebp(png, out) {
  const buf = await sharp(png).webp({ quality: 90, effort: 6 }).toBuffer();
  for (let lan = 1; ; lan++) {
    try {
      await sharp(buf).toFile(out);
      return;
    } catch (e) {
      if (lan >= 6) throw e;
      await new Promise((ok) => setTimeout(ok, 1000 * lan));
    }
  }
}
