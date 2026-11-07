// Sinh icon giữ chỗ cho ba flavor của hai app: nền một màu theo flavor, hình trắng ở giữa
// (tròn cho app khách, vuông cho app tài xế), dải đen ở đáy với dev và staging (prod không có
// dải). Không phải icon thiết kế thật.
//
//   node examples/toi-noi-flutter/50-flavors-build/tool/make-placeholder-icons.mjs
//
// Ghi PNG RGB 8 bit bằng zlib của Node, không cần thư viện ngoài.
import fs from 'node:fs';
import path from 'node:path';
import zlib from 'node:zlib';
import { fileURLToPath } from 'node:url';

const lab = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const shapes = { customer: 'circle', driver: 'square' };

const flavors = {
  dev: { color: [240, 140, 0], ribbon: true },
  staging: { color: [25, 113, 194], ribbon: true },
  prod: { color: [47, 158, 68], ribbon: false },
};
const densities = { mdpi: 48, hdpi: 72, xhdpi: 96, xxhdpi: 144, xxxhdpi: 192 };

const crcTable = Array.from({ length: 256 }, (_, n) => {
  let c = n;
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c >>> 0;
});
function crc32(buf) {
  let c = 0xffffffff;
  for (const byte of buf) c = crcTable[(c ^ byte) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}
function chunk(type, data) {
  const body = Buffer.concat([Buffer.from(type, 'ascii'), data]);
  const out = Buffer.alloc(body.length + 8);
  out.writeUInt32BE(data.length, 0);
  body.copy(out, 4);
  out.writeUInt32BE(crc32(body), body.length + 4);
  return out;
}

function png(size, { color, ribbon }, shape) {
  const row = 1 + size * 3;
  const raw = Buffer.alloc(row * size);
  const c = (size - 1) / 2;
  for (let y = 0; y < size; y++) {
    raw[y * row] = 0; // filter: none
    for (let x = 0; x < size; x++) {
      let px = color;
      const inside =
        shape === 'circle'
          ? Math.hypot(x - c, y - c) <= size * 0.28
          : Math.abs(x - c) <= size * 0.24 && Math.abs(y - c) <= size * 0.24;
      if (inside) px = [255, 255, 255];
      if (ribbon && y >= size * 0.78) px = [20, 20, 20];
      raw.set(px, y * row + 1 + x * 3);
    }
  }
  const header = Buffer.alloc(13);
  header.writeUInt32BE(size, 0);
  header.writeUInt32BE(size, 4);
  header[8] = 8; // bit depth
  header[9] = 2; // colour type: RGB
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', header),
    chunk('IDAT', zlib.deflateSync(raw, { level: 9 })),
    chunk('IEND', Buffer.alloc(0)),
  ]);
}

function write(file, data) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, data);
  console.log(path.relative(lab, file));
}

const iosContents = (file) =>
  JSON.stringify(
    {
      images: [{ filename: file, idiom: 'universal', platform: 'ios', size: '1024x1024' }],
      info: { author: 'xcode', version: 1 },
    },
    null,
    2,
  ) + '\n';

for (const [app, shape] of Object.entries(shapes)) {
  const root = path.join(lab, 'native-config', app);
  for (const [flavor, spec] of Object.entries(flavors)) {
    // Android: prod dùng thư mục main (icon mặc định), dev và staging ghi đè bằng src/<flavor>.
    const androidRoot =
      flavor === 'prod'
        ? path.join(root, 'android/app/src/main/res')
        : path.join(root, `android/app/src/${flavor}/res`);
    for (const [density, size] of Object.entries(densities)) {
      write(path.join(androidRoot, `mipmap-${density}`, 'ic_launcher.png'), png(size, spec, shape));
    }
    // iOS: prod là AppIcon, dev và staging là AppIcon-<flavor>.
    const set = flavor === 'prod' ? 'AppIcon' : `AppIcon-${flavor}`;
    const dir = path.join(root, 'ios/Runner/Assets.xcassets', `${set}.appiconset`);
    write(path.join(dir, 'icon-1024.png'), png(1024, spec, shape));
    write(path.join(dir, 'Contents.json'), iosContents('icon-1024.png'));
  }
}
