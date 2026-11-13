/**
 * Dựng banner 1600×400 cho từng chủ đề: minh hoạ vector gốc (không logo/nhãn hiệu thật) + WebP.
 * Không dùng mạng hay DB, đầu ra tất định. Chạy: `npm run banners:generate -w @tkh/web`.
 *
 * Bố cục: nửa trái (x < 720) để TRỐNG — HTML phủ chữ lên đó; toàn bộ hình nằm trong x ≥ 780 để còn nguyên khi thẻ
 * cắt theo phần bên phải (16:7 neo phải ⇒ thấy từ x ≈ 686). Không đóng tên chủ đề vào ảnh (chữ được địa phương hoá ở HTML).
 */
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import sharp from 'sharp';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const output = path.join(root, 'public/images/categories');
await mkdir(output, { recursive: true });

const W = 1600;
const H = 400;
const NAVY = '#0A0F1E';

/** Màu chủ đạo từng chủ đề — trùng `color` trong api/seed/categories.json (đổi một bên thì đổi cả bên kia rồi dựng lại). */
const COLORS = {
  database: '#2563EB',
  programming: '#6366F1',
  csharp: '#7C3AED',
  rust: '#C2410C',
  flutter: '#0891B2',
  python: '#CA8A04',
  'location-routing': '#059669',
  'software-design': '#DB2777',
  'design-pattern': '#10B981',
  architecture: '#8B5CF6',
  'system-design': '#0EA5E9',
  devops: '#F97316',
  'devops-aws': '#D97706',
  'devops-azure': '#0284C7',
  security: '#EF4444',
  testing: '#14B8A6',
  'business-analysis': '#EAB308',
};

// ───────────────────────── màu & hình nguyên thuỷ ─────────────────────────
const hex = (h) => [1, 3, 5].map(i => parseInt(h.slice(i, i + 2), 16));
const mix = (a, b, t) => {
  const [x, y] = [hex(a), hex(b)];
  return '#' + x.map((v, i) => Math.round(v + (y[i] - v) * t).toString(16).padStart(2, '0')).join('');
};
const n = (v) => Math.round(v * 100) / 100;
const escape = (s) => String(s).replace(/[&<>"']/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&apos;' }[ch]));

function palette(color) {
  return {
    c: color,
    a: mix(color, '#ffffff', 0.62), // sáng: nét chính
    m: mix(color, '#ffffff', 0.3), // vừa: mảng nhấn
    dk: mix(color, NAVY, 0.55), // tối: nền khối
    dim: '#94A3B8',
    ok: '#4ADE80',
    bad: '#F87171',
    warn: '#FACC15',
  };
}

/** Hình chữ nhật bo góc. o: {f fill, fo fill-opacity, s stroke, so stroke-opacity, r bo góc, sw độ dày nét, d nét đứt, t transform}. */
const R = (x, y, w, h, o = {}) => `<rect x="${n(x)}" y="${n(y)}" width="${n(w)}" height="${n(h)}" rx="${o.r ?? 10}" fill="${o.f ?? 'none'}" fill-opacity="${o.fo ?? 1}" stroke="${o.s ?? 'none'}" stroke-opacity="${o.so ?? 1}" stroke-width="${o.sw ?? 2}"${o.d ? ` stroke-dasharray="${o.d}"` : ''}${o.t ? ` transform="${o.t}"` : ''}/>`;
const C = (x, y, r, o = {}) => `<circle cx="${n(x)}" cy="${n(y)}" r="${n(r)}" fill="${o.f ?? 'none'}" fill-opacity="${o.fo ?? 1}" stroke="${o.s ?? 'none'}" stroke-opacity="${o.so ?? 1}" stroke-width="${o.sw ?? 2}"${o.d ? ` stroke-dasharray="${o.d}"` : ''}/>`;
const E = (x, y, rx, ry, o = {}) => `<ellipse cx="${n(x)}" cy="${n(y)}" rx="${n(rx)}" ry="${n(ry)}" fill="${o.f ?? 'none'}" fill-opacity="${o.fo ?? 1}" stroke="${o.s ?? 'none'}" stroke-opacity="${o.so ?? 1}" stroke-width="${o.sw ?? 2}"/>`;
const P = (d, o = {}) => `<path d="${d}" fill="${o.f ?? 'none'}" fill-opacity="${o.fo ?? 1}" stroke="${o.s ?? 'none'}" stroke-opacity="${o.so ?? 1}" stroke-width="${o.sw ?? 2}" stroke-linecap="${o.cap ?? 'round'}" stroke-linejoin="round"${o.d ? ` stroke-dasharray="${o.d}"` : ''}${o.t ? ` transform="${o.t}"` : ''}/>`;
const L = (x1, y1, x2, y2, s, o = {}) => P(`M${n(x1)} ${n(y1)}L${n(x2)} ${n(y2)}`, { s, ...o });
const T = (x, y, s, size, fill, o = {}) => `<text x="${n(x)}" y="${n(y)}" font-family="Arial, sans-serif" font-size="${size}" font-weight="${o.w ?? 700}" fill="${fill}" fill-opacity="${o.fo ?? 1}" text-anchor="${o.anchor ?? 'start'}"${o.t ? ` transform="${o.t}"` : ''}>${escape(s)}</text>`;
const ticks = (x, y, w, h, k) => R(x, y, w, h, { f: '#ffffff', fo: 0.06, s: k.a, so: 0.55, r: 14 });
/** Thanh giả dòng mã: rows = [[thụt lề, độ dài, màu]]. */
const bars = (x, y, rows, gap = 22, h = 9, fo = 0.9) => rows.map(([ind, w, col], i) => R(x + ind * 22, y + i * gap, w, h, { f: col, fo, r: 4.5 })).join('');
/** Cửa sổ trình soạn thảo: thân đặc + thanh tiêu đề bo góc trên + viền. */
const win = (x, y, w, h, k, fill) => R(x, y, w, h, { f: fill, r: 14 }) + P(`M${x} ${y + 34}V${y + 14}a14 14 0 0 1 14 -14H${x + w - 14}a14 14 0 0 1 14 14V${y + 34}Z`, { f: k.m, fo: 0.2 }) + R(x, y, w, h, { s: k.a, so: 0.8, r: 14 }) + L(x, y + 34, x + w, y + 34, k.a, { sw: 1.5, so: 0.45 }) + dots3(x + 24, y + 17, k);
const dots3 = (x, y, k) => C(x, y, 5, { f: k.bad, fo: 0.85 }) + C(x + 17, y, 5, { f: k.warn, fo: 0.85 }) + C(x + 34, y, 5, { f: k.ok, fo: 0.85 });
const check = (x, y, col, sc = 1, sw = 5) => P(`M${n(x)} ${n(y)}l${n(8 * sc)} ${n(8 * sc)} ${n(17 * sc)} ${n(-19 * sc)}`, { s: col, sw });
const cross = (x, y, col, sc = 1, sw = 5) => P(`M${n(x)} ${n(y)}l${n(16 * sc)} ${n(16 * sc)}M${n(x + 16 * sc)} ${n(y)}l${n(-16 * sc)} ${n(16 * sc)}`, { s: col, sw });
/** Mũi tên: nét + đầu tam giác tại (x2,y2) theo hướng từ (x1,y1). */
const arrow = (x1, y1, x2, y2, col, o = {}) => {
  const ang = Math.atan2(y2 - y1, x2 - x1);
  const s = o.size ?? 11;
  const p = (da, len) => `${n(x2 - Math.cos(ang + da) * len)} ${n(y2 - Math.sin(ang + da) * len)}`;
  return L(x1, y1, x2 - Math.cos(ang) * s * 0.6, y2 - Math.sin(ang) * s * 0.6, col, { sw: o.sw ?? 3, d: o.d }) + P(`M${n(x2)} ${n(y2)}L${p(0.45, s)}L${p(-0.45, s)}Z`, { f: col, s: col, sw: 1 });
};
const elbow = (pts, col, o = {}) => P('M' + pts.map(([x, y]) => `${n(x)} ${n(y)}`).join('L'), { s: col, sw: o.sw ?? 3, d: o.d });
const gear = (cx, cy, r, teeth, k, o = {}) => {
  const pts = [];
  for (let i = 0; i < teeth * 4; i++) {
    const ang = (i / (teeth * 4)) * Math.PI * 2 - Math.PI / 2;
    const rr = [r, r, r * 0.78, r * 0.78][i % 4];
    pts.push(`${n(cx + Math.cos(ang) * rr)} ${n(cy + Math.sin(ang) * rr)}`);
  }
  return P('M' + pts.join('L') + 'Z', { f: o.f ?? k.m, fo: o.fo ?? 0.18, s: o.s ?? k.a, sw: o.sw ?? 2.5 }) + C(cx, cy, r * 0.32, { s: o.s ?? k.a, sw: 2.5 });
};
const cyl = (cx, top, w, h, k, bands = 1, o = {}) => {
  const rx = w / 2;
  const ry = w * 0.16;
  let s = P(`M${n(cx - rx)} ${n(top + ry)}v${n(h)}a${n(rx)} ${n(ry)} 0 0 0 ${n(w)} 0v${n(-h)}`, { f: o.f ?? mix(k.c, NAVY, 0.8), fo: o.fo ?? 1, s: o.s ?? k.a, sw: 2.5 });
  for (let i = 1; i < bands; i++) s += P(`M${n(cx - rx)} ${n(top + ry + (h * i) / bands)}a${n(rx)} ${n(ry)} 0 0 0 ${n(w)} 0`, { s: o.s ?? k.a, so: 0.7, sw: 2 });
  s += E(cx, top + ry, rx, ry, { f: o.top ?? k.m, fo: 0.25, s: o.s ?? k.a, sw: 2.5 });
  return s;
};
const pin = (x, y, k, fill) => P(`M${x} ${y}C${x - 14} ${y - 20} ${x - 22} ${y - 32} ${x - 22} ${y - 42}A22 22 0 1 1 ${x + 22} ${y - 42}C${x + 22} ${y - 32} ${x + 14} ${y - 20} ${x} ${y}Z`, { f: fill, s: '#ffffff', so: 0.85, sw: 2.5 }) + C(x, y - 42, 8, { f: NAVY });
const cloud = (x, y, s, k, o = {}) => P(`M${n(x)} ${n(y + 40 * s)}h${n(100 * s)}a${n(26 * s)} ${n(26 * s)} 0 0 0 ${n(4 * s)} ${n(-52 * s)}a${n(36 * s)} ${n(36 * s)} 0 0 0 ${n(-66 * s)} ${n(-14 * s)}a${n(30 * s)} ${n(30 * s)} 0 0 0 ${n(-50 * s)} ${n(20 * s)}a${n(23 * s)} ${n(23 * s)} 0 0 0 ${n(12 * s)} ${n(46 * s)}z`, { f: o.f ?? k.m, fo: o.fo ?? 0.14, s: o.s ?? k.a, sw: o.sw ?? 2.5 });
const node = (x, y, w, h, k, o = {}) => R(x, y, w, h, { f: o.f ?? '#ffffff', fo: o.fo ?? 0.08, s: o.s ?? k.a, so: o.so ?? 0.85, r: o.r ?? 10, sw: o.sw ?? 2.5, d: o.d });

// ───────────────────────── minh hoạ từng chủ đề ─────────────────────────
const art = {
  database(k) {
    let s = L(955, 190, 1050, 150, k.a, { sw: 2.5, d: '6 6', so: 0.7 }) + L(955, 240, 1050, 300, k.a, { sw: 2.5, d: '6 6', so: 0.7 });
    s += cyl(870, 218, 170, 70, k, 1) + C(920, 294, 5, { f: k.warn }) + cyl(870, 140, 170, 70, k, 1) + C(920, 216, 5, { f: k.ok }) + cyl(870, 62, 170, 70, k, 1) + C(920, 138, 5, { f: k.ok });
    // bảng
    s += ticks(1060, 60, 470, 190, k);
    s += R(1060, 60, 470, 40, { f: k.m, fo: 0.3, r: 14 });
    [0, 1, 2, 3].forEach(i => { s += R(1082 + i * 112, 76, 70 + (i % 2) * 14, 9, { f: '#fff', fo: 0.85, r: 4.5 }); });
    for (let r = 0; r < 4; r++) {
      const y = 114 + r * 34;
      if (r === 1) s += R(1072, y - 10, 446, 30, { f: k.m, fo: 0.2, s: k.a, so: 0.8, r: 8 });
      for (let c = 0; c < 4; c++) s += R(1082 + c * 112, y, 40 + ((r * 3 + c * 5) % 4) * 14, 8, { f: k.a, fo: r === 1 ? 0.95 : 0.45, r: 4 });
    }
    // trang dữ liệu + chỉ mục
    for (let i = 0; i < 6; i++) {
      s += R(1060 + i * 80, 285, 66, 54, { f: i === 1 ? k.m : '#fff', fo: i === 1 ? 0.28 : 0.07, s: k.a, so: i === 1 ? 1 : 0.6, r: 8 });
      s += L(1072 + i * 80, 303, 1112 + i * 80, 303, k.a, { sw: 2.5, so: 0.7 }) + L(1072 + i * 80, 318, 1100 + i * 80, 318, k.a, { sw: 2.5, so: 0.5 });
    }
    s += P('M1295 250V270H1100V285M1295 270H1420V285', { s: k.a, sw: 2, d: '4 5', so: 0.8 });
    return s;
  },

  programming(k) {
    const wnd = (x, y, w, h, rows) => win(x, y, w, h, k, mix(k.c, NAVY, 0.82)) + bars(x + 24, y + 56, rows, 24, 9);
    const dimc = 'rgba(148,163,184,.75)';
    let s = wnd(800, 50, 400, 250, [[0, 90, k.a], [1, 160, dimc], [1, 120, k.m], [2, 200, dimc], [2, 90, k.a], [1, 60, dimc], [0, 40, k.a], [0, 130, dimc]]);
    s += wnd(980, 120, 420, 220, [[0, 70, k.m], [0, 150, dimc], [1, 190, k.a], [1, 110, dimc], [2, 140, k.m], [1, 80, dimc], [0, 30, k.m]]);
    s += wnd(1170, 196, 360, 154, [[0, 100, k.a], [1, 200, dimc], [1, 150, k.m], [0, 50, k.a]]);
    s += T(1472, 250, '{ }', 30, k.a) + T(1250, 100, '=>', 24, k.a, { fo: 0.7 });
    return s;
  },

  csharp(k) {
    const ed = mix(k.c, NAVY, 0.84);
    let s = win(800, 50, 480, 290, k, ed);
    s += R(826, 94, 120, 26, { f: k.m, fo: 0.25, r: 8 }) + R(956, 94, 90, 26, { f: '#fff', fo: 0.06, r: 8 });
    const rows = [
      [0, [[60, k.m], [50, k.a], [90, k.dim]]],
      [0, [[12, k.a]]],
      [1, [[60, k.m], [80, k.a], [10, k.dim]]],
      [1, [[12, k.a]]],
      [2, [[60, k.m], [40, k.a], [60, k.dim], [20, k.m]]],
      [2, [[60, k.m], [60, k.a], [80, k.dim]]],
      [2, [[40, k.m], [60, k.a], [14, k.dim], [110, k.m]]],
      [1, [[12, k.a]]],
      [0, [[12, k.a]]],
    ];
    rows.forEach(([ind, toks], i) => {
      const y = 140 + i * 22;
      s += T(826, y + 9, String(i + 1), 12, k.dim, { fo: 0.6, w: 500 });
      let x = 856 + ind * 26;
      toks.forEach(([w, col]) => { s += R(x, y, w, 9, { f: col, fo: 0.9, r: 4.5 }); x += w + 10; });
    });
    // chip kiểu
    s += R(1320, 70, 210, 270, { f: '#fff', fo: 0.05, s: k.a, so: 0.4, r: 14, d: '7 7' });
    const chips = [['int', 1340, 96, 80], ['string', 1432, 96, 76], ['bool', 1340, 148, 80], ['T', 1432, 148, 56], ['Task', 1340, 200, 100], ['List<T>', 1340, 252, 150]];
    chips.forEach(([label, x, y, w], i) => { s += R(x, y, w, 36, { f: i % 2 ? k.m : '#fff', fo: i % 2 ? 0.22 : 0.08, s: k.a, so: 0.8, r: 18, sw: 2 }) + T(x + w / 2, y + 24, label, 16, k.a, { anchor: 'middle' }); });
    s += L(1280, 120, 1320, 120, k.a, { d: '5 5', sw: 2.5, so: 0.8 });
    return s;
  },

  rust(k) {
    const block = (x, y, w, label, hl) => R(x, y, w, 56, { f: hl ? k.m : '#fff', fo: hl ? 0.25 : 0.07, s: k.a, so: 0.9, r: 8 }) + L(x + w / 2, y, x + w / 2, y + 56, k.a, { so: 0.5, sw: 2 }) + T(x + w / 4, y + 36, label, 20, k.a, { anchor: 'middle' });
    let s = '';
    // ô nhớ stack
    s += T(810, 92, 'STACK', 13, k.dim, { fo: 0.8, w: 700 });
    s += block(810, 104, 130, 'a', true) + block(810, 176, 130, 'b', false) + block(810, 248, 130, 'c', false);
    // heap
    s += T(1100, 92, 'HEAP', 13, k.dim, { fo: 0.8 });
    s += R(1100, 104, 190, 200, { f: '#fff', fo: 0.05, s: k.a, so: 0.5, r: 12, d: '7 7' });
    for (let i = 0; i < 4; i++) for (let j = 0; j < 3; j++) s += R(1116 + j * 56, 120 + i * 44, 48, 34, { f: (i + j) % 5 === 0 ? k.m : '#fff', fo: (i + j) % 5 === 0 ? 0.35 : 0.07, s: k.a, so: 0.6, r: 6, sw: 1.5 });
    // sở hữu (nét liền) và mượn & (nét đứt)
    s += arrow(940, 132, 1112, 142, k.m, { sw: 3.5 });
    s += P('M940 204C1000 204 1030 168 1096 166', { s: k.a, sw: 2.5, d: '7 6' }) + arrow(1090, 166, 1114, 163, k.a, { sw: 2.5, size: 10 });
    s += arrow(940, 276, 1108, 246, k.a, { sw: 2.5, d: '7 6' });
    s += T(1010, 126, 'owner', 14, k.m, { anchor: 'middle' }) + T(1020, 200, '&', 22, k.a, { anchor: 'middle' }) + T(1016, 280, '&mut', 14, k.a, { anchor: 'middle' });
    // lifetime
    s += P('M1318 112v190', { s: k.m, sw: 3 }) + L(1318, 112, 1330, 112, k.m, { sw: 3 }) + L(1318, 302, 1330, 302, k.m, { sw: 3 });
    s += T(1344, 214, "'a", 30, k.m);
    // bánh răng
    s += gear(1430, 128, 62, 10, k) + gear(1488, 262, 42, 8, k, { fo: 0.25 });
    return s;
  },

  flutter(k) {
    const phone = (x, y, w, h, tilt, body) => `<g transform="rotate(${tilt} ${x + w / 2} ${y + h / 2})">` + R(x, y, w, h, { f: mix(k.c, NAVY, 0.84), s: k.a, so: 0.9, r: 28, sw: 3 }) + R(x + w / 2 - 26, y + 10, 52, 8, { f: k.a, fo: 0.5, r: 4 }) + body + `</g>`;
    let s = phone(810, 58, 150, 290, -6,
      R(826, 92, 118, 70, { f: k.m, fo: 0.35, r: 12 }) + R(826, 174, 56, 56, { f: '#fff', fo: 0.1, s: k.a, so: 0.5, r: 10 }) + R(888, 174, 56, 56, { f: '#fff', fo: 0.1, s: k.a, so: 0.5, r: 10 }) + R(826, 242, 118, 12, { f: k.a, fo: 0.6, r: 6 }) + R(826, 264, 80, 12, { f: k.a, fo: 0.35, r: 6 }) + C(885, 316, 14, { f: k.m, fo: 0.6 }));
    s += phone(1000, 70, 150, 290, 5,
      C(1075, 136, 30, { f: k.m, fo: 0.4, s: k.a, so: 0.8 }) + R(1018, 186, 114, 14, { f: k.a, fo: 0.6, r: 7 }) + R(1018, 210, 114, 10, { f: k.dim, fo: 0.4, r: 5 }) + R(1018, 230, 80, 10, { f: k.dim, fo: 0.4, r: 5 }) + R(1018, 266, 114, 34, { f: k.m, fo: 0.5, r: 17 }));
    // cây widget
    const nd = (cx, cy, w, fill) => R(cx - w / 2, cy - 18, w, 36, { f: fill ? k.m : '#fff', fo: fill ? 0.3 : 0.08, s: k.a, so: 0.9, r: 10 }) + R(cx - w / 2 + 12, cy - 4, w - 24, 8, { f: k.a, fo: 0.7, r: 4 });
    s += elbow([[1380, 100], [1380, 124], [1300, 124], [1300, 148]], k.a, { sw: 2.5 }) + elbow([[1380, 124], [1460, 124], [1460, 148]], k.a, { sw: 2.5 });
    s += elbow([[1300, 184], [1300, 206], [1240, 206], [1240, 232]], k.a, { sw: 2.5 }) + elbow([[1300, 206], [1350, 206], [1350, 232]], k.a, { sw: 2.5 }) + elbow([[1460, 184], [1460, 206], [1500, 206], [1500, 232]], k.a, { sw: 2.5 }) + elbow([[1460, 206], [1430, 206], [1430, 232]], k.a, { sw: 2.5 });
    s += nd(1380, 82, 110, true) + nd(1300, 166, 96, false) + nd(1460, 166, 96, true) + nd(1240, 250, 70, false) + nd(1350, 250, 70, true) + nd(1430, 250, 56, false) + nd(1500, 250, 56, false);
    s += T(1380, 330, 'build()', 15, k.a, { anchor: 'middle', fo: 0.8 });
    return s;
  },

  python(k) {
    const ed = mix(k.c, NAVY, 0.84);
    let s = win(800, 50, 450, 300, k, ed);
    // ô 1: mã
    s += R(822, 98, 406, 62, { f: '#fff', fo: 0.05, s: k.a, so: 0.4, r: 10 }) + T(836, 122, 'In [1]', 12, k.dim, { w: 500 }) + bars(900, 112, [[0, 120, k.m], [0, 210, k.dim], [1, 80, k.a]], 16, 8);
    // ô 2: biểu đồ
    s += R(822, 172, 406, 112, { f: '#fff', fo: 0.05, s: k.a, so: 0.4, r: 10 }) + T(836, 192, 'Out', 12, k.dim, { w: 500 });
    [34, 58, 44, 74, 62, 90, 70].forEach((h, i) => { s += R(886 + i * 44, 270 - h, 28, h, { f: i === 5 ? k.m : k.a, fo: i === 5 ? 0.95 : 0.4, r: 5 }); });
    s += P('M890 245L934 222L978 232L1022 196L1066 208L1110 178L1154 196', { s: k.a, sw: 2.5, so: 0.9 });
    // ô 3
    s += R(822, 296, 406, 40, { f: '#fff', fo: 0.05, s: k.a, so: 0.4, r: 10 }) + bars(842, 312, [[0, 150, k.m]], 16, 8);
    // pipeline
    const stage = (y, i) => R(1330, y, 190, 62, { f: i === 1 ? k.m : '#fff', fo: i === 1 ? 0.24 : 0.07, s: k.a, so: 0.9, r: 12 }) + bars(1352, y + 20, [[0, 100, k.a], [0, 60, k.dim]], 18, 8);
    s += stage(64, 0) + stage(168, 1) + stage(272, 2);
    s += arrow(1425, 126, 1425, 166, k.a, { sw: 2.5 }) + arrow(1425, 230, 1425, 270, k.a, { sw: 2.5 });
    s += elbow([[1250, 200], [1290, 200], [1290, 95], [1330, 95]], k.a, { sw: 2.5, d: '6 5' }) + elbow([[1250, 200], [1290, 200], [1290, 303], [1330, 303]], k.a, { sw: 2.5, d: '6 5' });
    s += C(1500, 199, 6, { f: k.ok });
    return s;
  },

  'location-routing'(k) {
    const x0 = 800, y0 = 40, w = 740, h = 320;
    const clip = `<clipPath id="mp"><rect x="${x0}" y="${y0}" width="${w}" height="${h}" rx="16"/></clipPath>`;
    let g = R(x0, y0, w, h, { f: mix(k.c, NAVY, 0.86), r: 16 });
    // khối nhà
    const xs = [x0 + 0, x0 + 120, x0 + 250, x0 + 380, x0 + 520, x0 + 640];
    const ys = [y0 + 0, y0 + 90, y0 + 180, y0 + 260];
    // đường phố
    xs.forEach(x => { g += L(x + 70, y0, x + 70, y0 + h, k.a, { sw: 14, so: 0.1 }) + L(x + 70, y0, x + 70, y0 + h, k.a, { sw: 1.5, d: '10 10', so: 0.4 }); });
    ys.forEach(y => { g += L(x0, y + 45, x0 + w, y + 45, k.a, { sw: 14, so: 0.1 }) + L(x0, y + 45, x0 + w, y + 45, k.a, { sw: 1.5, d: '10 10', so: 0.4 }); });
    g += L(x0, y0 + 300, x0 + w, y0 + 20, k.a, { sw: 16, so: 0.12 });
    // công viên + sông
    g += R(x0 + 160, y0 + 110, 60, 50, { f: k.ok, fo: 0.22, r: 8 }) + R(x0 + 560, y0 + 210, 80, 60, { f: k.ok, fo: 0.2, r: 8 });
    g += P(`M${x0 + 400} ${y0 + h}C${x0 + 420} ${y0 + 240} ${x0 + 500} ${y0 + 250} ${x0 + 520} ${y0 + 170}S${x0 + 650} ${y0 + 80} ${x0 + w} ${y0 + 60}`, { s: '#38BDF8', so: 0.25, sw: 18 });
    // tuyến phụ
    g += P(`M${x0 + 190} ${y0 + 305}H${x0 + 320}V${y0 + 225}H${x0 + 450}`, { s: k.dim, so: 0.55, sw: 4, d: '3 9' });
    // tuyến chính
    const route = `M${x0 + 80} ${y0 + 265}H${x0 + 190}V${y0 + 135}H${x0 + 320}V${y0 + 225}H${x0 + 450}V${y0 + 135}H${x0 + 590}V${y0 + 70}H${x0 + 660}`;
    g += P(route, { s: '#000', so: 0.4, sw: 12 }) + P(route, { s: k.m, sw: 7 }) + P(route, { s: '#fff', sw: 2, d: '2 14', so: 0.9 });
    // điểm dừng
    [[190, 135], [320, 225], [450, 135], [590, 70]].forEach(([x, y]) => { g += C(x0 + x, y0 + y, 8, { f: NAVY, s: '#fff', sw: 3 }); });
    // ô lưới phân vùng
    g += R(x0 + 280, y0 + 20, 90, 60, { f: k.m, fo: 0.12, s: k.a, so: 0.55, r: 6, d: '5 5', sw: 1.5 });
    let s = `<defs>${clip}</defs><g clip-path="url(#mp)">${g}</g>` + R(x0, y0, w, h, { s: k.a, so: 0.75, r: 16, sw: 2.5 });
    s += pin(x0 + 80, y0 + 265, k, k.ok) + pin(x0 + 660, y0 + 70, k, k.bad);
    // tài xế
    s += C(x0 + 320, y0 + 225, 22, { f: k.c, fo: 0.3, s: k.a, so: 0.7 }) + P(`M${x0 + 320} ${y0 + 213}l11 22l-11 -5l-11 5z`, { f: '#fff', s: '#fff', sw: 1.5 });
    return s;
  },

  'software-design'(k) {
    let s = R(800, 50, 730, 300, { f: mix(k.c, NAVY, 0.8), s: k.a, so: 0.8, r: 14 });
    // lưới bản vẽ
    for (let x = 820; x <= 1510; x += 30) s += L(x, 62, x, 338, k.a, { sw: x % 150 === 20 ? 1.2 : 0.7, so: x % 150 === 20 ? 0.3 : 0.14 });
    for (let y = 70; y <= 340; y += 30) s += L(812, y, 1518, y, k.a, { sw: (y - 70) % 150 === 0 ? 1.2 : 0.7, so: (y - 70) % 150 === 0 ? 0.3 : 0.14 });
    // khối thành phần
    s += R(850, 100, 190, 100, { f: k.m, fo: 0.14, s: k.a, so: 0.95, r: 6, sw: 2.5 }) + R(1120, 100, 150, 100, { f: '#fff', fo: 0.06, s: k.a, so: 0.95, r: 6 }) + R(1120, 250, 150, 70, { f: '#fff', fo: 0.06, s: k.a, so: 0.95, r: 6 });
    s += arrow(1040, 150, 1120, 150, k.a, { sw: 2.5 }) + arrow(1195, 200, 1195, 250, k.a, { sw: 2.5 });
    s += bars(870, 120, [[0, 100, k.a], [0, 130, k.dim], [0, 60, k.dim]], 22, 8) + bars(1140, 120, [[0, 80, k.a], [0, 100, k.dim]], 22, 8) + bars(1140, 270, [[0, 90, k.a], [0, 60, k.dim]], 22, 8);
    // kích thước
    const dim = (x1, y, x2, label) => L(x1, y, x2, y, k.m, { sw: 1.8 }) + L(x1, y - 7, x1, y + 7, k.m, { sw: 1.8 }) + L(x2, y - 7, x2, y + 7, k.m, { sw: 1.8 }) + R((x1 + x2) / 2 - 22, y - 11, 44, 22, { f: mix(k.c, NAVY, 0.8), r: 4 }) + T((x1 + x2) / 2, y + 5, label, 13, k.m, { anchor: 'middle', w: 600 });
    s += dim(850, 232, 1040, '190') + dim(1120, 226, 1270, '150');
    s += L(828, 100, 828, 200, k.m, { sw: 1.8 }) + L(821, 100, 835, 100, k.m, { sw: 1.8 }) + L(821, 200, 835, 200, k.m, { sw: 1.8 });
    // compa + bán kính
    s += C(1400, 170, 80, { s: k.a, so: 0.8, sw: 2.5 }) + C(1400, 170, 4, { f: k.a }) + L(1400, 170, 1456, 114, k.m, { sw: 2.5 }) + T(1432, 160, 'R', 18, k.m);
    s += C(1400, 170, 46, { s: k.a, so: 0.4, d: '5 5', sw: 1.5 });
    // thước tam giác
    s += P('M1330 330L1330 262L1420 330Z', { f: k.m, fo: 0.18, s: k.a, so: 0.8, sw: 2.2 }) + P('M1342 316v-36l40 36z', { s: k.a, so: 0.45, sw: 1.5 });
    s += T(1480, 322, 'A-A', 14, k.a, { anchor: 'middle', fo: 0.8 });
    return s;
  },

  'design-pattern'(k) {
    const klass = (x, y, w, name, rows, hl) => R(x, y, w, 24 + rows * 18 + 24, { f: mix(k.c, NAVY, 0.82), s: k.a, so: 0.9, r: 8 }) + R(x, y, w, 30, { f: hl ? k.m : k.c, fo: hl ? 0.45 : 0.3, r: 8 }) + R(x, y + 22, w, 8, { f: hl ? k.m : k.c, fo: hl ? 0.45 : 0.3, r: 0 }) + T(x + w / 2, y + 21, name, 14, '#fff', { anchor: 'middle' }) + L(x, y + 30, x + w, y + 30, k.a, { sw: 1.5, so: 0.8 }) + bars(x + 12, y + 44, Array.from({ length: rows }, (_, i) => [0, w * (0.35 + ((i * 17) % 4) * 0.1), i % 2 ? k.dim : k.a]), 18, 7) + L(x, y + 30 + rows * 18 + 12, x + w, y + 30 + rows * 18 + 12, k.a, { sw: 1.5, so: 0.5 });
    let s = '';
    // kế thừa/hiện thực (tam giác rỗng)
    const tri = (x, y) => P(`M${x} ${y}l-10 18h20z`, { f: NAVY, s: k.a, sw: 2.2 });
    s += elbow([[1000, 232], [1000, 252], [900, 252], [900, 270]], k.a, { sw: 2.2, d: '6 5' }) + elbow([[1000, 252], [1100, 252], [1100, 270]], k.a, { sw: 2.2, d: '6 5' });
    s += tri(1000, 214);
    s += klass(920, 60, 160, '<<interface>>', 2, true);
    s += klass(820, 270, 160, 'ConcreteA', 2, false) + klass(1020, 270, 160, 'ConcreteB', 2, false);
    // lớp client — hợp thành
    s += klass(1310, 100, 190, 'Context', 3, false);
    s += arrow(1310, 130, 1086, 106, k.a, { sw: 2.2, d: '6 5' });
    s += P('M1405 214l-12 18l12 18l12 -18z', { f: k.m, fo: 0.8, s: k.a, sw: 2 }) + L(1405, 250, 1405, 290, k.a, { sw: 2.2 });
    s += klass(1330, 290, 150, 'Part', 1, false);
    return s;
  },

  architecture(k) {
    const hexp = (cx, cy, r) => 'M' + [0, 1, 2, 3, 4, 5].map(i => `${n(cx + r * Math.cos(Math.PI / 3 * i + Math.PI / 6))} ${n(cy + r * Math.sin(Math.PI / 3 * i + Math.PI / 6))}`).join('L') + 'Z';
    const cx = 1060, cy = 200;
    let s = '';
    // adapter bên ngoài
    s += P(hexp(cx, cy, 170), { f: k.m, fo: 0.06, s: k.a, so: 0.5, sw: 2, d: '8 7' });
    s += P(hexp(cx, cy, 128), { f: k.m, fo: 0.14, s: k.a, so: 0.95, sw: 3 });
    s += P(hexp(cx, cy, 66), { f: '#fff', fo: 0.1, s: k.a, so: 0.9, sw: 2.5 });
    s += bars(cx - 28, cy - 14, [[0, 56, k.a], [0, 38, k.dim]], 18, 8);
    // đường nối cổng → adapter
    const pts = [[cx + 111, cy - 64, 1250, 100], [cx + 111, cy, 1260, 200], [cx + 111, cy + 64, 1250, 300], [cx - 111, cy + 64, 880, 300], [cx - 111, cy, 870, 200], [cx - 111, cy - 64, 880, 100]];
    pts.forEach(([x1, y1, x2, y2]) => { s += L(x1, y1, x2, y2, k.a, { sw: 2.5, d: '6 6', so: 0.75 }) + C(x1, y1, 7, { f: NAVY, s: k.a, sw: 2.5 }); });
    [[1250, 100, 1], [1260, 200, 0], [1250, 300, 1], [880, 300, 0], [870, 200, 1], [880, 100, 0]].forEach(([x, y, sq]) => { s += sq ? R(x - 18, y - 18, 36, 36, { f: '#fff', fo: 0.08, s: k.a, r: 8 }) : C(x, y, 19, { f: '#fff', fo: 0.08, s: k.a }); });
    // chồng tầng
    for (let i = 0; i < 4; i++) {
      const y = 100 + i * 62;
      s += R(1360, y, 170, 44, { f: i === 1 ? k.m : '#fff', fo: i === 1 ? 0.28 : 0.07, s: k.a, so: 0.9, r: 10 }) + bars(1380, y + 18, [[0, 80 - i * 8, k.a]], 18, 8, 0.8);
      if (i < 3) s += arrow(1445, y + 46, 1445, y + 62, k.a, { sw: 2, size: 8 });
    }
    return s;
  },

  'system-design'(k) {
    let s = '';
    const box = (x, y, w, h, fill) => node(x, y, w, h, k, { f: fill ? k.m : '#fff', fo: fill ? 0.25 : 0.08 });
    s += C(830, 200, 22, { f: '#fff', fo: 0.1, s: k.a }) + C(830, 194, 7, { f: k.a }) + P('M816 214a14 11 0 0 1 28 0', { s: k.a, sw: 2.5 });
    s += arrow(854, 200, 900, 200, k.a, { sw: 2.5 });
    // LB
    s += P('M960 150L1010 200L960 250L910 200Z', { f: k.m, fo: 0.3, s: k.a, sw: 2.5 }) + L(950, 200, 970, 200, '#fff', { sw: 3 });
    [95, 200, 305].forEach((y, i) => { s += L(1010, 200, 1080, y, k.a, { sw: 2.5, d: i === 1 ? undefined : '6 6', so: 0.85 }); });
    [78, 183, 288].forEach((y, i) => { s += box(1080, y, 120, 34, i === 1) + C(1100, y + 17, 5, { f: k.ok }) + R(1116, y + 13, 58, 8, { f: k.a, fo: 0.7, r: 4 }); });
    // cache + queue
    [95, 305].forEach((y, i) => { s += L(1200, y, 1290, i ? 305 : 110, k.a, { sw: 2.2, d: '6 6', so: 0.7 }); });
    s += R(1290, 80, 110, 58, { f: k.warn, fo: 0.14, s: k.warn, so: 0.9, r: 12 }) + bars(1308, 98, [[0, 60, k.warn], [0, 40, k.warn]], 18, 8, 0.8);
    s += L(1200, 200, 1290, 200, k.a, { sw: 2.5 });
    s += R(1290, 176, 210, 50, { f: '#fff', fo: 0.06, s: k.a, so: 0.9, r: 25 }) + [0, 1, 2, 3].map(i => R(1306 + i * 44, 188, 32, 26, { f: i === 0 ? k.m : '#fff', fo: i === 0 ? 0.6 : 0.12, s: k.a, so: 0.7, r: 6, sw: 1.5 })).join('');
    s += L(1200, 305, 1290, 300, k.a, { sw: 2.2, d: '6 6', so: 0.7 });
    s += cyl(1345, 262, 100, 60, k, 1);
    s += P('M1500 226V300H1410', { s: k.a, sw: 2.2, d: '6 6', so: 0.7 });
    return s;
  },

  devops(k) {
    const cx = 1170, cy = 200, a = 340;
    const pt = (t) => [cx + (a * Math.cos(t)) / (1 + Math.sin(t) ** 2), cy + (a * Math.sin(t) * Math.cos(t)) / (1 + Math.sin(t) ** 2) * 1.35];
    let d = '';
    for (let i = 0; i <= 120; i++) { const [x, y] = pt((i / 120) * Math.PI * 2); d += (i ? 'L' : 'M') + `${n(x)} ${n(y)}`; }
    let s = P(d + 'Z', { s: k.a, sw: 3, d: '10 8', so: 0.75 });
    // mũi tên hướng
    [0.52, 2.6, 3.66, 5.76].forEach(t => {
      const [x, y] = pt(t), [x2, y2] = pt(t + 0.02);
      const ang = Math.atan2(y2 - y, x2 - x);
      s += P(`M${n(x + Math.cos(ang) * 8)} ${n(y + Math.sin(ang) * 8)}L${n(x - Math.cos(ang - 0.9) * 10)} ${n(y - Math.sin(ang - 0.9) * 10)}L${n(x - Math.cos(ang + 0.9) * 10)} ${n(y - Math.sin(ang + 0.9) * 10)}Z`, { f: k.a, s: k.a, sw: 1 });
    });
    const glyph = {
      code: (x, y) => P(`M${x - 4} ${y - 8}l-8 8l8 8M${x + 4} ${y - 8}l8 8l-8 8`, { s: k.a, sw: 3 }),
      build: (x, y) => R(x - 9, y - 9, 18, 18, { s: k.a, sw: 2.5, r: 3 }) + L(x - 9, y - 3, x + 9, y - 3, k.a, { sw: 2 }),
      test: (x, y) => check(x - 10, y, k.ok, 0.9, 3.5),
      pack: (x, y) => P(`M${x} ${y - 11}l11 6v12l-11 6l-11 -6v-12z M${x - 11} ${y - 5}l11 6l11 -6M${x} ${y + 1}v12`, { s: k.a, sw: 2.2 }),
      ship: (x, y) => P(`M${x} ${y + 10}V${y - 10}M${x - 9} ${y - 2}L${x} ${y - 11}L${x + 9} ${y - 2}`, { s: k.a, sw: 3 }),
      watch: (x, y) => E(x, y, 13, 8, { s: k.a, sw: 2.5 }) + C(x, y, 3.5, { f: k.a }),
    };
    [[0, 'code'], [Math.PI / 4, 'build'], [3 * Math.PI / 4, 'test'], [Math.PI, 'pack'], [5 * Math.PI / 4, 'ship'], [7 * Math.PI / 4, 'watch']].forEach(([t, g]) => {
      const [x, y] = pt(t);
      s += C(x, y, 28, { f: mix(k.c, NAVY, 0.7), s: k.a, sw: 2.8 }) + glyph[g](x, y);
    });
    // tâm: bánh răng nhỏ
    s += gear(cx, cy, 26, 8, k, { fo: 0.4 });
    return s;
  },

  'devops-aws'(k) {
    let s = R(800, 40, 740, 320, { s: k.a, so: 0.55, r: 18, d: '10 8', sw: 2 });
    s += cloud(822, 52, 0.42, k, { sw: 2 });
    s += R(840, 100, 480, 240, { f: k.m, fo: 0.07, s: k.a, so: 0.9, r: 14, sw: 2.5 }) + R(852, 108, 36, 16, { f: k.a, fo: 0.5, r: 4 });
    // LB phía trên
    s += R(1010, 116, 120, 34, { f: k.m, fo: 0.3, s: k.a, r: 10 }) + L(1030, 133, 1110, 133, '#fff', { sw: 3, so: 0.8 });
    // hai AZ
    [0, 1].forEach(i => {
      const x = 860 + i * 230;
      s += R(x, 170, 210, 160, { f: '#fff', fo: 0.04, s: k.a, so: 0.55, r: 12, d: '6 6', sw: 2 });
      s += R(x + 14, 184, 182, 60, { f: k.c, fo: 0.18, s: k.a, so: 0.8, r: 10 });
      for (let j = 0; j < 3; j++) s += R(x + 26 + j * 56, 200, 38, 28, { f: '#fff', fo: 0.1, s: k.a, r: 6, sw: 2 }) + L(x + 34 + j * 56, 214, x + 56 + j * 56, 214, k.a, { sw: 2.5, so: 0.8 });
      s += R(x + 14, 256, 182, 60, { f: '#fff', fo: 0.05, s: k.a, so: 0.6, r: 10 }) + cyl(x + 56, 264, 44, 34, k, 1) + R(x + 112, 276, 70, 8, { f: k.a, fo: 0.5, r: 4 });
      s += L(1070, 150, x + 105, 172, k.a, { sw: 2.2, d: '5 5', so: 0.8 });
    });
    // dịch vụ ngoài VPC: bucket, hàng đợi, hàm
    const bucket = (x, y) => P(`M${x} ${y}h70l-9 62h-52z`, { f: k.warn, fo: 0.12, s: k.warn, so: 0.9, sw: 2.5 }) + E(x + 35, y, 35, 9, { f: k.warn, fo: 0.25, s: k.warn, sw: 2.5 }) + L(x + 14, y + 30, x + 56, y + 30, k.warn, { sw: 2, so: 0.6 });
    s += bucket(1400, 90) + R(1390, 214, 100, 38, { f: '#fff', fo: 0.07, s: k.a, r: 19 }) + [0, 1, 2].map(i => C(1412 + i * 28, 233, 7, { f: i === 0 ? k.m : k.a, fo: i === 0 ? 0.9 : 0.4 })).join('') + P('M1420 284h40M1445 270l16 14l-16 14', { s: k.a, sw: 3 });
    s += L(1320, 200, 1390, 120, k.a, { sw: 2.2, d: '5 6', so: 0.7 }) + L(1320, 230, 1390, 233, k.a, { sw: 2.2, d: '5 6', so: 0.7 }) + L(1320, 270, 1400, 284, k.a, { sw: 2.2, d: '5 6', so: 0.7 });
    return s;
  },

  'devops-azure'(k) {
    const cx = 1160, cy = 200;
    let s = '';
    const spokes = [[800, 60, 'vm', 950, 110], [1390, 60, 'db', 1390, 110], [800, 250, 'app', 950, 290], [1390, 250, 'net', 1390, 290]];
    // vòng đồng tâm + trục nối hub → spoke (nét liền/đứt xen kẽ)
    s += C(cx, cy, 150, { s: k.a, so: 0.25, sw: 1.8, d: '4 8' }) + C(cx, cy, 100, { s: k.a, so: 0.35, sw: 1.8 });
    spokes.forEach(([, , , ex, ey], i) => { s += L(cx, cy, ex, ey, k.a, { sw: 2.8, so: 0.85, d: i % 2 ? '7 6' : undefined }); });
    // hub
    s += P('M' + [0, 1, 2, 3, 4, 5].map(i => `${n(cx + 62 * Math.cos(Math.PI / 3 * i))} ${n(cy + 62 * Math.sin(Math.PI / 3 * i))}`).join('L') + 'Z', { f: mix(k.c, NAVY, 0.55), s: k.a, sw: 3 });
    s += cloud(cx - 40, cy - 30, 0.4, k, { f: '#fff', fo: 0.14, s: '#fff', sw: 2.5 });
    // spoke
    const kinds = {
      vm: (x, y) => R(x + 20, y + 33, 110, 34, { f: '#fff', fo: 0.1, s: k.a, r: 8 }) + R(x + 34, y + 46, 40, 8, { f: k.a, fo: 0.7, r: 4 }) + C(x + 110, y + 50, 5, { f: k.ok }),
      db: (x, y) => cyl(x + 75, y + 20, 64, 44, k, 2),
      app: (x, y) => R(x + 20, y + 30, 110, 40, { f: '#fff', fo: 0.1, s: k.a, r: 8 }) + P(`M${x + 52} ${y + 40}l-10 10l10 10M${x + 98} ${y + 40}l10 10l-10 10`, { s: k.a, sw: 3 }),
      net: (x, y) => C(x + 75, y + 50, 26, { s: k.a, sw: 2.5 }) + E(x + 75, y + 50, 11, 26, { s: k.a, sw: 2 }) + L(x + 49, y + 50, x + 101, y + 50, k.a, { sw: 2 }),
    };
    spokes.forEach(([x, y, kind]) => { s += R(x, y, 150, 100, { f: mix(k.c, NAVY, 0.7), s: k.a, so: 0.85, r: 14, sw: 2.5 }) + kinds[kind](x, y); });
    // ô nhận dạng + cổng
    s += P(`M${cx - 18} ${cy - 150}h36v26a18 18 0 0 1 -36 0z`, { f: k.warn, fo: 0.18, s: k.warn, sw: 2.5 });
    s += C(cx, cy - 134, 5, { f: k.warn });
    s += R(cx - 40, cy + 126, 80, 24, { f: '#fff', fo: 0.08, s: k.a, so: 0.7, r: 12, sw: 2 }) + bars(cx - 24, cy + 134, [[0, 48, k.a]], 18, 7, 0.7);
    return s;
  },

  security(k) {
    const cx = 1080, cy = 200;
    let s = C(cx, cy, 160, { s: k.a, so: 0.25, sw: 1.8, d: '3 9' }) + C(cx, cy, 120, { s: k.a, so: 0.3, sw: 1.8 });
    s += P(`M${cx} ${cy - 135}L${cx + 100} ${cy - 100}V${cy + 10}C${cx + 100} ${cy + 80} ${cx + 50} ${cy + 120} ${cx} ${cy + 140}C${cx - 50} ${cy + 120} ${cx - 100} ${cy + 80} ${cx - 100} ${cy + 10}V${cy - 100}Z`, { f: k.m, fo: 0.18, s: k.a, sw: 4 });
    s += P(`M${cx} ${cy - 110}L${cx + 78} ${cy - 84}V${cy + 8}C${cx + 78} ${cy + 66} ${cx + 38} ${cy + 98} ${cx} ${cy + 114}Z`, { f: k.m, fo: 0.2 });
    // ổ khoá trong khiên
    s += R(cx - 28, cy - 12, 56, 48, { f: '#fff', fo: 0.14, s: '#fff', so: 0.95, r: 8, sw: 3 }) + P(`M${cx - 16} ${cy - 12}v-14a16 16 0 0 1 32 0v14`, { s: '#fff', sw: 4 }) + C(cx, cy + 10, 6, { f: '#fff' }) + L(cx, cy + 14, cx, cy + 24, '#fff', { sw: 4 });
    // chìa khoá
    const kx = 1300, ky = 214;
    s += `<g transform="rotate(-24 ${kx + 80} ${ky})">` + C(kx, ky, 44, { f: k.warn, fo: 0.14, s: k.warn, sw: 4 }) + C(kx - 10, ky, 12, { s: k.warn, sw: 3 }) + P(`M${kx + 44} ${ky}H${kx + 190}M${kx + 140} ${ky}v30M${kx + 168} ${ky}v22`, { s: k.warn, sw: 5 }) + `</g>`;
    // khoá nhỏ + mạch
    s += P('M1220 100H1330V70H1440M1450 320H1360V290', { s: k.a, sw: 2.2, d: '6 6', so: 0.7 }) + C(1220, 100, 6, { f: NAVY, s: k.a, sw: 2.5 }) + C(1440, 70, 6, { f: k.ok }) + C(1450, 320, 6, { f: k.bad });
    s += R(1440, 108, 70, 46, { f: '#fff', fo: 0.06, s: k.a, so: 0.7, r: 10 }) + bars(1452, 124, [[0, 46, k.a], [0, 30, k.dim]], 16, 7);
    s += P(`M1462 322l8 8l14 -16`, { s: k.ok, sw: 3.5 });
    return s;
  },

  testing(k) {
    let s = ticks(800, 50, 380, 300, k);
    const rows = [['ok', 150], ['ok', 190], ['bad', 120], ['ok', 170], ['run', 140]];
    rows.forEach(([st, w], i) => {
      const y = 76 + i * 54;
      s += R(822, y, 38, 38, { f: st === 'ok' ? k.ok : st === 'bad' ? k.bad : '#fff', fo: st === 'run' ? 0.06 : 0.18, s: st === 'ok' ? k.ok : st === 'bad' ? k.bad : k.a, so: 0.9, r: 9, sw: 2.5 });
      s += st === 'ok' ? check(831, y + 21, k.ok, 0.75, 4.5) : st === 'bad' ? cross(832, y + 11, k.bad, 0.9, 4.5) : C(841, y + 19, 7, { s: k.a, sw: 3, d: '5 4' });
      s += R(878, y + 8, w, 8, { f: k.a, fo: 0.75, r: 4 }) + R(878, y + 24, w - 40, 6, { f: k.dim, fo: 0.45, r: 3 });
      s += R(1090, y + 8, 64, 22, { f: st === 'bad' ? k.bad : st === 'ok' ? k.ok : k.a, fo: 0.18, s: st === 'bad' ? k.bad : st === 'ok' ? k.ok : k.a, so: 0.7, r: 11, sw: 1.8 });
    });
    // bảng chạy + đồng hồ độ phủ
    s += R(1230, 50, 300, 300, { f: mix(k.c, NAVY, 0.84), s: k.a, so: 0.7, r: 14 });
    const arc = (r, frac, col, sw) => { const a = Math.PI * 2 * frac - Math.PI / 2; return P(`M1380 ${200 - r}A${r} ${r} 0 ${frac > 0.5 ? 1 : 0} 1 ${n(1380 + r * Math.cos(a))} ${n(200 + r * Math.sin(a))}`, { s: col, sw }); };
    s += C(1380, 170, 62, { s: k.a, so: 0.2, sw: 12 }) + `<g transform="translate(0 -30)">${arc(62, 0.82, k.ok, 12)}</g>`;
    s += check(1364, 176, k.ok, 1.1, 6);
    [[0, k.ok, 120], [1, k.ok, 150], [2, k.bad, 90]].forEach(([i, col, w]) => { s += C(1262, 268 + i * 24, 5, { f: col }) + R(1282, 264 + i * 24, w, 8, { f: k.a, fo: 0.6, r: 4 }) + R(1440, 264 + i * 24, 66, 8, { f: col, fo: 0.4, r: 4 }); });
    return s;
  },

  'business-analysis'(k) {
    let s = '';
    const lane = (y) => R(800, y, 740, 124, { f: '#fff', fo: 0.03, s: k.a, so: 0.35, r: 10, d: '7 7', sw: 1.8 });
    s += lane(46) + lane(190);
    // luồng
    s += R(824, 90, 70, 36, { f: k.m, fo: 0.3, s: k.a, r: 18 });
    s += arrow(894, 108, 944, 108, k.a, { sw: 2.5 });
    s += R(944, 84, 130, 48, { f: '#fff', fo: 0.08, s: k.a, r: 10 }) + bars(962, 100, [[0, 90, k.a], [0, 60, k.dim]], 16, 7);
    s += arrow(1074, 108, 1124, 108, k.a, { sw: 2.5 });
    s += P('M1170 62L1216 108L1170 154L1124 108Z', { f: k.warn, fo: 0.16, s: k.warn, sw: 2.8 });
    s += T(1170, 114, '?', 22, k.warn, { anchor: 'middle' });
    s += arrow(1216, 108, 1280, 108, k.a, { sw: 2.5 }) + T(1248, 98, 'Y', 13, k.ok, { anchor: 'middle' });
    s += R(1280, 84, 130, 48, { f: '#fff', fo: 0.08, s: k.a, r: 10 }) + bars(1298, 100, [[0, 80, k.a], [0, 50, k.dim]], 16, 7);
    s += elbow([[1170, 154], [1170, 252], [1280, 252]], k.a, { sw: 2.5 }) + T(1186, 196, 'N', 13, k.bad, { anchor: 'middle' });
    s += arrow(1410, 108, 1456, 108, k.a, { sw: 2.5 }) + R(1456, 90, 70, 36, { f: k.m, fo: 0.3, s: k.a, r: 18 });
    s += R(1280, 228, 130, 48, { f: '#fff', fo: 0.08, s: k.a, r: 10 }) + bars(1298, 244, [[0, 90, k.a], [0, 40, k.dim]], 16, 7);
    s += arrow(1410, 252, 1456, 252, k.a, { sw: 2.5 }) + C(1486, 252, 18, { f: k.ok, fo: 0.2, s: k.ok, sw: 3 }) + check(1476, 252, k.ok, 0.6, 3.5);
    // giấy nhớ
    const note = (x, y, rot, col) => `<g transform="rotate(${rot} ${x + 50} ${y + 50})">` + R(x + 4, y + 6, 100, 100, { f: '#000', fo: 0.3, r: 4 }) + R(x, y, 100, 100, { f: col, fo: 0.92, r: 4 }) + R(x + 14, y + 22, 68, 7, { f: NAVY, fo: 0.45, r: 3.5 }) + R(x + 14, y + 42, 50, 7, { f: NAVY, fo: 0.35, r: 3.5 }) + R(x + 14, y + 62, 60, 7, { f: NAVY, fo: 0.35, r: 3.5 }) + `</g>`;
    s += note(822, 222, -6, k.warn) + note(930, 236, 5, mix(k.c, '#ffffff', 0.45)) + note(1034, 222, -4, k.ok);
    return s;
  },

  'data-analysis'(k) {
    let s = win(820, 48, 680, 302, k, '#0f172a');
    s += L(870, 300, 1450, 300, k.a, { so: 0.4 });
    [75, 135, 105, 195, 165, 220].forEach((h, i) => {
      s += R(900 + i * 83, 300 - h, 48, h, { f: k.m, fo: 0.4 + i * 0.1, r: 5 });
    });
    s += P('M924 205 1007 163 1090 178 1173 116 1256 132 1339 92', { s: k.ok, sw: 4 });
    [[924,205],[1007,163],[1090,178],[1173,116],[1256,132],[1339,92]].forEach(([x,y]) => { s += C(x,y,6,{ f:k.ok }); });
    return s;
  },
};

// ───────────────────────── lắp SVG ─────────────────────────
const categories = JSON.parse(await readFile(path.resolve(root, '../../api/seed/categories.json'), 'utf8'));
const slugs = categories.map(c => c.slug);
for (const category of categories) {
  COLORS[category.slug] = category.color;
  if (!art[category.slug]) {
    const base = category.slug === 'go' ? 'programming' : category.parent;
    if (!art[base]) throw new Error('Missing base illustration: ' + category.slug);
    art[category.slug] = k => art[base](k) + T(1510, 370, category.slug.toUpperCase(), 18, k.a, { anchor: 'end' });
  }
}
for (const slug of slugs) {
  if (!art[slug]) throw new Error('Missing illustration: ' + slug);
  const k = palette(COLORS[slug]);
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}">
<defs>
<linearGradient id="bg" x1="0" y1="0" x2="1" y2="0.35"><stop offset="0" stop-color="${mix(k.c, NAVY, 0.58)}"/><stop offset="1" stop-color="${NAVY}"/></linearGradient>
<radialGradient id="glow" cx="0.72" cy="0.5" r="0.45"><stop offset="0" stop-color="${k.c}" stop-opacity="0.3"/><stop offset="1" stop-color="${k.c}" stop-opacity="0"/></radialGradient>
<pattern id="grid" width="32" height="32" patternUnits="userSpaceOnUse"><circle cx="1" cy="1" r="1" fill="${k.a}" opacity=".2"/></pattern>
</defs>
<rect width="${W}" height="${H}" fill="url(#bg)"/><rect width="${W}" height="${H}" fill="url(#glow)"/><rect width="${W}" height="${H}" fill="url(#grid)"/>
${art[slug](k)}
</svg>`;
  await writeFile(path.join(output, `${slug}.svg`), svg);
  await sharp(Buffer.from(svg)).webp({ quality: 86 }).toFile(path.join(output, `${slug}.webp`));
}
console.log(`Generated ${slugs.length} category banners (SVG + WebP, ${W} × ${H}).`);
