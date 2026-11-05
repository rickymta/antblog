/**
 * Script lưu trữ dữ liệu khi ngừng phục vụ một thành phố — bài TEST-020 «Maintenance Testing».
 *
 * CTFL v4.0.1 mục 2.3: khi một hệ thống (ở đây: một phần dịch vụ) bị ngừng, có thể cần kiểm thử
 * việc LƯU TRỮ dữ liệu (data archiving) và thủ tục KHÔI PHỤC, TRUY XUẤT sau khi lưu trữ.
 * Đây là «thay đổi» cần kiểm thử của bài; nó chỉ ĐỌC database của lab, không sửa src/.
 *
 * Quy tắc giả định (cần nghiệp vụ duyệt): chỉ lưu trữ chuyến đã kết thúc (completed/cancelled);
 * chuyến còn đang chạy bị CHẶN và liệt kê để vận hành xử lý trước khi ngừng dịch vụ.
 */
import { PGlite } from '@electric-sql/pglite';

export interface ArchivedTrip {
  id: string;
  kind: string;
  status: string;
  customerId: string;
  fare: number;
  cancellationFee: number;
  pricingVersion: string;
  requestedAt: string;
  events: { status: string; occurredAt: string }[];
}

export interface CityArchive {
  cityCode: string;
  trips: ArchivedTrip[];
  blocked: string[]; // chuyến chưa kết thúc — không lưu trữ
  checksum: { count: number; fareSum: number };
}

const TERMINAL = new Set(['completed', 'cancelled']);

interface TripRow {
  id: string;
  kind: string;
  status: string;
  customer_id: string;
  fare: number;
  cancellation_fee: number;
  pricing_version: string;
  requested_at: Date;
}

/** Đọc chuyến của một thành phố từ database đang vận hành và đóng gói thành bản lưu trữ. */
export async function exportCityArchive(db: PGlite, cityCode: string): Promise<CityArchive> {
  const { rows } = await db.query<TripRow>(
    `SELECT id, kind, status, customer_id, fare, cancellation_fee, pricing_version, requested_at
       FROM trips WHERE city_code = $1 ORDER BY id`,
    [cityCode],
  );
  const trips: ArchivedTrip[] = [];
  const blocked: string[] = [];
  for (const row of rows) {
    if (!TERMINAL.has(row.status)) {
      blocked.push(row.id);
      continue;
    }
    const events = await db.query<{ status: string; occurred_at: Date }>(
      'SELECT status, occurred_at FROM trip_events WHERE trip_id = $1 ORDER BY id',
      [row.id],
    );
    trips.push({
      id: row.id,
      kind: row.kind,
      status: row.status,
      customerId: row.customer_id,
      fare: row.fare,
      cancellationFee: row.cancellation_fee,
      pricingVersion: row.pricing_version,
      requestedAt: row.requested_at.toISOString(),
      events: events.rows.map((e) => ({ status: e.status, occurredAt: e.occurred_at.toISOString() })),
    });
  }
  const fareSum = trips.reduce((sum, t) => sum + t.fare, 0);
  return { cityCode, trips, blocked, checksum: { count: trips.length, fareSum } };
}

/** Nạp bản lưu trữ vào kho lưu trữ (một PostgreSQL riêng, lược đồ khác database vận hành). */
export async function restoreArchive(archive: CityArchive): Promise<PGlite> {
  const store = await PGlite.create();
  await store.exec(`
    CREATE TABLE archived_trips (
      id text PRIMARY KEY, city_code text NOT NULL, status text NOT NULL,
      fare integer NOT NULL, payload jsonb NOT NULL
    );`);
  for (const trip of archive.trips) {
    await store.query('INSERT INTO archived_trips (id, city_code, status, fare, payload) VALUES ($1, $2, $3, $4, $5)', [
      trip.id,
      archive.cityCode,
      trip.status,
      trip.fare,
      JSON.stringify(trip),
    ]);
  }
  return store;
}

/** Truy xuất một chuyến đã lưu trữ — ví dụ khi khách khiếu nại sau ngày ngừng dịch vụ. */
export async function findArchivedTrip(store: PGlite, id: string): Promise<ArchivedTrip | null> {
  const { rows } = await store.query<{ payload: ArchivedTrip }>('SELECT payload FROM archived_trips WHERE id = $1', [id]);
  return rows[0]?.payload ?? null;
}
