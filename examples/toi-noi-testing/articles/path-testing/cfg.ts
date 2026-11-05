/**
 * TEST-048 · Control flow graph (CFG) của acceptTrip (src/app/trips.ts dòng 154–184), dựng tay ở mức quyết định.
 * Mỗi `&&` và `??` là một nút quyết định riêng (vì ngắt mạch) — cùng cách đếm với luật `complexity` của ESLint.
 * Lời gọi hàm khác (requireRole, loadDriver, assertDriverAvailable, assertAcceptable) là câu lệnh thường:
 * McCabe không đếm đường ném lỗi bên trong hàm được gọi.
 */
export interface Edge {
  id: string;
  from: string;
  to: string;
}

export const NODES = [
  'S', //   vào: requireRole; findById(trip)
  'D1', //  !trip ?
  'X1', //  throw TRIP_NOT_FOUND
  'D2a', // trip.driverId === actor.id ?
  'D2b', // ACTIVE_STATUSES.includes(trip.status) ?
  'R1', //  return trip (nhận lại — R-ACCEPT-07)
  'L', //   loadDriver; assertDriverAvailable(driver)
  'D3', //  khác thành phố ?
  'X3', //  throw CITY_MISMATCH
  'D4', //  khác loại xe ?
  'X4', //  throw VEHICLE_MISMATCH
  'A', //   assertAcceptable(trip)
  'D5', //  !tryMarkBusy ?
  'D6', //  findById(driver) ?? driver
  'Y6', //  assertDriverAvailable(...); throw DRIVER_BUSY
  'K', //   acceptIfSearching
  'D7', //  !accepted ?
  'M', //   markOnline; findById(latest)
  'D8', //  latest ?
  'Z', //   assertAcceptable(latest)
  'XT', //  throw TRIP_ALREADY_TAKEN
  'R2', //  return accepted
  'E', //   ra
] as const;

const e = (id: string, from: string, to: string): Edge => ({ id, from, to });

export const EDGES: Edge[] = [
  e('S-D1', 'S', 'D1'),
  e('D1:T', 'D1', 'X1'),
  e('D1:F', 'D1', 'D2a'),
  e('D2a:T', 'D2a', 'D2b'),
  e('D2a:F', 'D2a', 'L'),
  e('D2b:T', 'D2b', 'R1'),
  e('D2b:F', 'D2b', 'L'),
  e('L-D3', 'L', 'D3'),
  e('D3:T', 'D3', 'X3'),
  e('D3:F', 'D3', 'D4'),
  e('D4:T', 'D4', 'X4'),
  e('D4:F', 'D4', 'A'),
  e('A-D5', 'A', 'D5'),
  e('D5:T', 'D5', 'D6'),
  e('D5:F', 'D5', 'K'),
  e('D6:co', 'D6', 'Y6'),
  e('D6:null', 'D6', 'Y6'),
  e('K-D7', 'K', 'D7'),
  e('D7:T', 'D7', 'M'),
  e('D7:F', 'D7', 'R2'),
  e('M-D8', 'M', 'D8'),
  e('D8:T', 'D8', 'Z'),
  e('D8:F', 'D8', 'XT'),
  e('Z-XT', 'Z', 'XT'),
  e('X1-E', 'X1', 'E'),
  e('R1-E', 'R1', 'E'),
  e('X3-E', 'X3', 'E'),
  e('X4-E', 'X4', 'E'),
  e('Y6-E', 'Y6', 'E'),
  e('XT-E', 'XT', 'E'),
  e('R2-E', 'R2', 'E'),
];

/** V(G) = E − N + 2 (một lối vào, một lối ra). */
export function cyclomatic(nodes: readonly string[], edges: readonly Edge[]): number {
  return edges.length - nodes.length + 2;
}

/** Đi theo dãy lựa chọn tại các nút quyết định, trả danh sách id cạnh từ S tới E. */
export function walk(choices: Record<string, string>): string[] {
  const path: string[] = [];
  let node = 'S';
  while (node !== 'E') {
    const out = EDGES.filter((x) => x.from === node);
    const edge = out.length === 1 ? out[0] : out.find((x) => x.id === `${node}:${choices[node]}`);
    if (!edge) throw new Error(`Thiếu lựa chọn tại ${node}`);
    path.push(edge.id);
    node = edge.to;
  }
  return path;
}

/** Vectơ đếm số lần mỗi cạnh xuất hiện trong đường đi. */
export function edgeVector(path: readonly string[]): number[] {
  return EDGES.map((x) => path.filter((id) => id === x.id).length);
}

/** Hạng của tập vectơ (khử Gauss trên số thực) — số đường đi độc lập tuyến tính. */
export function rank(vectors: readonly number[][]): number {
  const m = vectors.map((v) => [...v]);
  let r = 0;
  const cols = m[0]?.length ?? 0;
  for (let c = 0; c < cols && r < m.length; c++) {
    const pivot = m.findIndex((row, i) => i >= r && Math.abs(row[c]!) > 1e-9);
    if (pivot < 0) continue;
    [m[r], m[pivot]] = [m[pivot]!, m[r]!];
    for (let i = 0; i < m.length; i++) {
      if (i === r) continue;
      const f = m[i]![c]! / m[r]![c]!;
      m[i] = m[i]!.map((v, k) => v - f * m[r]![k]!);
    }
    r++;
  }
  return r;
}

/** Số đường đi hoàn chỉnh S → E (đồ thị không có chu trình). */
export function countPaths(from = 'S'): number {
  if (from === 'E') return 1;
  return EDGES.filter((x) => x.from === from).reduce((sum, x) => sum + countPaths(x.to), 0);
}
