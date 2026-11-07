/**
 * TEST-040 · Sinh bảng 10.000 yêu cầu báo giá bưu kiện «giống dữ liệu thật» (GIẢ ĐỊNH, không phải số liệu thật):
 * quãng đường lệch phải quanh 2–15 km, khối lượng xe máy thường dưới 8 kg, ô tô 2–28 kg, khoảng 1–2% dòng sai
 * (khách nhập 0 kg, đơn liên tỉnh > 100 km…). Hạt giống cố định ⇒ chạy lại ra đúng tệp cũ.
 *
 *   npx tsx articles/data-based-test-techniques-domain-testing/sinh-bang.ts
 */
import { writeFileSync } from 'node:fs';

function mulberry32(seed: number) {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const rand = mulberry32(40_2026);
const normal = () => Math.sqrt(-2 * Math.log(1 - rand())) * Math.cos(2 * Math.PI * rand());
const two = (x: number) => Math.round(x * 100) / 100;

function distanceKm(): number {
  const u = rand();
  if (u < 0.004) return two(0.05 + rand() * 0.4); // nhập nhầm quá ngắn
  if (u < 0.01) return two(100 + rand() * 80); // đơn liên tỉnh
  return two(Math.min(99.9, 0.6 + Math.exp(1.6 + 0.7 * normal())));
}

function weightKg(vehicle: 'bike' | 'car'): number {
  const u = rand();
  if (u < 0.005) return 0; // quên nhập khối lượng
  if (u < 0.01) return two(30 + rand() * 20); // hàng quá nặng
  if (vehicle === 'bike') return two(Math.min(19.9, 0.2 + Math.exp(0.6 + 0.8 * normal())));
  return two(2 + rand() * 26);
}

const lines = ['ma,quang_duong_km,khoi_luong_kg,loai_xe'];
for (let i = 1; i <= 10_000; i++) {
  const vehicle = rand() < 0.7 ? 'bike' : 'car';
  lines.push(`P${String(i).padStart(5, '0')},${distanceKm()},${weightKg(vehicle)},${vehicle}`);
}
writeFileSync(new URL('./bang-10000.csv', import.meta.url), `${lines.join('\n')}\n`);
console.log(`đã ghi ${lines.length - 1} dòng`);
