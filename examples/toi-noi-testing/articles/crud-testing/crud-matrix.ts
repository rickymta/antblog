/**
 * TEST-039 · Ma trận CRUD của lab Tới Nơi, dựng từ SPEC (CTAL-TA v4.0 mục 3.2.1).
 * Hàng = chức năng (endpoint hoặc job), cột = thực thể dữ liệu, ô = các thao tác C/R/U/D.
 * Đây là bản «theo đặc tả» — viết TRƯỚC khi chạy, không đọc từ mã nguồn.
 */

export type Op = 'C' | 'R' | 'U' | 'D';
export const OPS: readonly Op[] = ['C', 'R', 'U', 'D'];

export const ENTITIES = ['trips', 'trip_events', 'quotes', 'drivers', 'vouchers', 'idempotency_keys'] as const;
export type Entity = (typeof ENTITIES)[number];

export interface CrudFunction {
  id: string;
  name: string;
  actor: 'khách' | 'tài xế' | 'khách/tài xế' | 'hệ thống';
  endpoint: string;
  basis: string;
  ops: Partial<Record<Entity, string>>; // ví dụ { trips: 'RU' }
}

export const SPEC_MATRIX: readonly CrudFunction[] = [
  { id: 'F01', name: 'Báo giá', actor: 'khách', endpoint: 'POST /api/quotes', basis: 'R-API-07, R-QUOTE-01, R-VOUCHER-07',
    ops: { quotes: 'C', vouchers: 'R', trips: 'R' } },
  { id: 'F02', name: 'Đặt chuyến', actor: 'khách', endpoint: 'POST /api/trips', basis: 'R-API-08, R-QUOTE-03, R-STATE-06, R-IDEM',
    ops: { trips: 'CR', trip_events: 'C', quotes: 'RU', vouchers: 'R', idempotency_keys: 'CRUD' } },
  { id: 'F03', name: 'Xem chuyến', actor: 'khách/tài xế', endpoint: 'GET /api/trips/:id', basis: 'R-API-09, R-API-05',
    ops: { trips: 'R', drivers: 'R' } },
  { id: 'F04', name: 'Danh sách chuyến chờ', actor: 'tài xế', endpoint: 'GET /api/driver/available-trips', basis: 'R-API-12, R-ACCEPT-06',
    ops: { trips: 'R', drivers: 'R' } },
  { id: 'F05', name: 'Nhận chuyến', actor: 'tài xế', endpoint: 'POST /api/trips/:id/accept', basis: 'R-ACCEPT-01…07, R-STATE-06',
    ops: { trips: 'RU', trip_events: 'C', drivers: 'RU' } },
  { id: 'F06', name: 'Đến / đón / chạy', actor: 'tài xế', endpoint: 'POST /api/trips/:id/{arrive,pickup,start}', basis: 'R-API-10, R-STATE-02',
    ops: { trips: 'RU', trip_events: 'C' } },
  { id: 'F07', name: 'Hoàn thành', actor: 'tài xế', endpoint: 'POST /api/trips/:id/complete', basis: 'R-API-10, R-ACCEPT-05',
    ops: { trips: 'RU', trip_events: 'C', drivers: 'U' } },
  { id: 'F08', name: 'Khách huỷ', actor: 'khách', endpoint: 'POST /api/trips/:id/cancel', basis: 'R-API-11, R-CANCEL, R-IDEM',
    ops: { trips: 'RU', trip_events: 'C', drivers: 'U', idempotency_keys: 'CRUD' } },
  { id: 'F09', name: 'Tài xế huỷ', actor: 'tài xế', endpoint: 'POST /api/trips/:id/cancel', basis: 'R-STATE-04, R-IDEM',
    ops: { trips: 'RU', trip_events: 'C', drivers: 'U', idempotency_keys: 'CRUD' } },
  { id: 'F10', name: 'Hết giờ tìm tài xế', actor: 'hệ thống', endpoint: 'job quét mỗi 15 giây', basis: 'R-STATE-05',
    ops: { trips: 'RU', trip_events: 'C' } },
];

/** Mọi ô (chức năng, thực thể, thao tác) có trong ma trận — mẫu số của CRUD coverage. */
export function matrixCells(matrix: readonly CrudFunction[]): string[] {
  return matrix.flatMap((f) =>
    Object.entries(f.ops).flatMap(([entity, ops]) => [...(ops ?? '')].map((op) => `${f.id}|${entity}|${op}`)),
  );
}

/** Completeness testing (kiểm tĩnh): thao tác nào KHÔNG có chức năng nào thực hiện, theo từng thực thể. */
export function missingOps(matrix: readonly CrudFunction[]): Record<Entity, Op[]> {
  const result = {} as Record<Entity, Op[]>;
  for (const entity of ENTITIES) {
    const present = new Set(matrix.flatMap((f) => [...(f.ops[entity] ?? '')]));
    result[entity] = OPS.filter((op) => !present.has(op));
  }
  return result;
}
