import { readFileSync } from 'node:fs';
import { PGlite } from '@electric-sql/pglite';
import type { SeedData } from '../seed';

const SCHEMA_SQL = readFileSync(new URL('./schema.sql', import.meta.url), 'utf8');

/**
 * PostgreSQL chạy trong tiến trình (WASM). Mỗi lần gọi tạo một database trống trong bộ nhớ.
 * Giới hạn: MỘT kết nối — các câu lệnh chạy tuần tự, không mô phỏng được hai giao dịch song song thật.
 */
export async function createPgliteDatabase(): Promise<PGlite> {
  const db = await PGlite.create();
  await db.exec(SCHEMA_SQL);
  return db;
}

/** Xoá dữ liệu nghiệp vụ (giữ bảng cities) rồi nạp dữ liệu mẫu. */
export async function resetPgliteData(db: PGlite, seed: SeedData): Promise<void> {
  await db.exec('TRUNCATE trip_events, trips, quotes, idempotency_keys, vouchers, drivers, customers RESTART IDENTITY CASCADE');
  await seedPglite(db, seed);
}

export async function seedPglite(db: PGlite, seed: SeedData): Promise<void> {
  await db.transaction(async (tx) => {
    for (const c of seed.customers) {
      await tx.query('INSERT INTO customers (id, phone, full_name) VALUES ($1, $2, $3)', [c.id, c.phone, c.fullName]);
    }
    for (const d of seed.drivers) {
      await tx.query('INSERT INTO drivers (id, full_name, city_code, vehicle_type, status) VALUES ($1, $2, $3, $4, $5)', [
        d.id,
        d.fullName,
        d.cityCode,
        d.vehicle,
        d.status,
      ]);
    }
    for (const v of seed.vouchers) {
      await tx.query(
        `INSERT INTO vouchers (code, percent, max_discount, min_amount, kinds, city_codes, valid_from, valid_to, per_customer_per_day)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)`,
        [v.code, v.percent, v.maxDiscount, v.minAmount, v.kinds, v.cityCodes, v.validFrom, v.validTo, v.perCustomerPerDay],
      );
    }
  });
}
