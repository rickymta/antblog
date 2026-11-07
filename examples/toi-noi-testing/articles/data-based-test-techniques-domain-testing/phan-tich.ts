/**
 * TEST-040 · So sánh các bộ dữ liệu kiểm: bộ domain testing (vài chục điểm) và bảng 10.000 dòng.
 * In bảng: npx tsx articles/data-based-test-techniques-domain-testing/phan-tich.ts
 */
import { readFileSync } from 'node:fs';
import { domainPoints, expectedOutcome, RAW, ROUNDED, type Partition, type Point, type Vehicle } from './mien-buu-kien';
import { makeImpl, MUTANTS, type Impl } from './trien-khai';

export function loadTable(): Point[] {
  const text = readFileSync(new URL('./bang-10000.csv', import.meta.url), 'utf8');
  return text.trim().split('\n').slice(1).map((line) => {
    const [ma, d, w, v] = line.split(',');
    return { d: Number(d), w: Number(w), v: v as Vehicle, kind: ma! };
  });
}

export const SUITES = {
  'Simplified 0,1': domainPoints(ROUNDED, 0.1, false),
  'Reliable 0,1': domainPoints(ROUNDED, 0.1, true),
  'Simplified thô 0,01': domainPoints(RAW, 0.01, false),
} as const;

/** Điểm mà bản cài đặt trả khác oracle. */
export function failures(points: Point[], impl: Impl): Point[] {
  return points.filter((p) => JSON.stringify(impl(p.d, p.w, p.v)) !== JSON.stringify(expectedOutcome(p.d, p.w, p.v)));
}

export function killMatrix(table: Point[]) {
  const sets: Record<string, Point[]> = { ...SUITES, 'Bảng 10.000 dòng': table };
  return Object.entries(MUTANTS).map(([id, m]) => ({
    id,
    moTa: m.moTa,
    ...Object.fromEntries(Object.entries(sets).map(([name, pts]) => [name, failures(pts, makeImpl(m.cfg)).length])),
  })) as { id: string; moTa: string; [suite: string]: number | string }[];
}

/** Coverage item ON/OFF (mức thô) nào có ít nhất một dòng của bảng chạm đúng — cùng giá trị biên, cùng loại xe, biến kia nằm trong vùng. */
export function tableHits(table: Point[], parts: Partition[] = RAW) {
  const items = domainPoints(parts, 0.01, false);
  return items.map((item) => {
    const part = parts.find((p) => item.kind.includes(p.id))!;
    const x = item.kind.split(' ').at(-1)!.startsWith('d') ? 'd' : 'w';
    const other = x === 'd' ? 'w' : 'd';
    const inside = (p: Point) =>
      part.borders.filter((bd) => bd.x === other).every((bd) =>
        bd.op === '<' ? p[other] < bd.c : bd.op === '<=' ? p[other] <= bd.c : bd.op === '>' ? p[other] > bd.c : p[other] >= bd.c);
    const hits = table.filter((r) => r.v === item.v && r[x] === item[x] && inside(r)).length;
    return { item: `${item.kind} ⇒ ${x}=${item[x]}`, hits };
  });
}

if (import.meta.url === `file://${process.argv[1]?.replace(/\\/g, '/').replace(/^\/?/, '/')}` || process.argv[1]?.endsWith('phan-tich.ts')) {
  const table = loadTable();
  console.log('Số điểm:', Object.entries(SUITES).map(([k, v]) => `${k} = ${v.length}`).join(' · '), '· bảng =', table.length);
  console.table(killMatrix(table));
  const hits = tableHits(table);
  console.log(`Coverage item ON/OFF mức thô được bảng chạm: ${hits.filter((h) => h.hits > 0).length}/${hits.length}`);
  for (const h of hits) console.log(`  ${h.hits.toString().padStart(4)}  ${h.item}`);
}
