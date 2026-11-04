import type { PGlite } from '@electric-sql/pglite';
import type {
  CustomerRepository,
  DriverRepository,
  IdempotencyBegin,
  IdempotencyStore,
  QuoteRepository,
  StoredResponse,
  VoucherRepository,
} from '../../app/ports';
import type { QuoteDetails, StoredQuote } from '../../app/quote';
import type { Customer, Driver } from '../../domain/trip';
import type { Voucher } from '../../domain/voucher';

export class PgQuoteRepository implements QuoteRepository {
  constructor(private readonly db: PGlite) {}

  async save(q: StoredQuote): Promise<void> {
    await this.db.query(
      `INSERT INTO quotes (id, customer_id, pricing_version, details, created_at, expires_at, used_by_trip_id)
       VALUES ($1, $2, $3, $4, $5, $6, $7)`,
      [q.id, q.customerId, q.pricingVersion, JSON.stringify(q.details), q.createdAt, q.expiresAt, q.usedByTripId],
    );
  }

  async findById(id: string): Promise<StoredQuote | null> {
    const { rows } = await this.db.query<{
      id: string;
      customer_id: string;
      pricing_version: string;
      details: QuoteDetails;
      created_at: Date;
      expires_at: Date;
      used_by_trip_id: string | null;
    }>('SELECT * FROM quotes WHERE id = $1', [id]);
    const r = rows[0];
    if (!r) return null;
    return {
      id: r.id,
      customerId: r.customer_id,
      pricingVersion: r.pricing_version,
      details: r.details,
      createdAt: r.created_at,
      expiresAt: r.expires_at,
      usedByTripId: r.used_by_trip_id,
    };
  }

  async markUsed(quoteId: string, tripId: string): Promise<boolean> {
    const { rows } = await this.db.query(
      'UPDATE quotes SET used_by_trip_id = $2 WHERE id = $1 AND used_by_trip_id IS NULL RETURNING id',
      [quoteId, tripId],
    );
    return rows.length === 1;
  }
}

export class PgDriverRepository implements DriverRepository {
  constructor(private readonly db: PGlite) {}

  async findById(id: string): Promise<Driver | null> {
    const { rows } = await this.db.query<{ id: string; full_name: string; city_code: Driver['cityCode']; vehicle_type: Driver['vehicle']; status: Driver['status'] }>(
      'SELECT id, full_name, city_code, vehicle_type, status FROM drivers WHERE id = $1',
      [id],
    );
    const r = rows[0];
    return r ? { id: r.id, fullName: r.full_name, cityCode: r.city_code, vehicle: r.vehicle_type, status: r.status } : null;
  }

  async tryMarkBusy(id: string): Promise<boolean> {
    const { rows } = await this.db.query(`UPDATE drivers SET status = 'busy' WHERE id = $1 AND status = 'online' RETURNING id`, [id]);
    return rows.length === 1;
  }

  async markOnline(id: string): Promise<void> {
    await this.db.query(`UPDATE drivers SET status = 'online' WHERE id = $1 AND status = 'busy'`, [id]);
  }
}

export class PgCustomerRepository implements CustomerRepository {
  constructor(private readonly db: PGlite) {}

  async findById(id: string): Promise<Customer | null> {
    const { rows } = await this.db.query<{ id: string; phone: string; full_name: string }>(
      'SELECT id, phone, full_name FROM customers WHERE id = $1',
      [id],
    );
    const r = rows[0];
    return r ? { id: r.id, phone: r.phone, fullName: r.full_name } : null;
  }
}

export class PgVoucherRepository implements VoucherRepository {
  constructor(private readonly db: PGlite) {}

  async findByCode(code: string): Promise<Voucher | null> {
    const { rows } = await this.db.query<{
      code: string;
      percent: number;
      max_discount: number;
      min_amount: number;
      kinds: Voucher['kinds'];
      city_codes: Voucher['cityCodes'];
      valid_from: string;
      valid_to: string;
      per_customer_per_day: number;
    }>(
      `SELECT code, percent, max_discount, min_amount, kinds, city_codes,
              to_char(valid_from, 'YYYY-MM-DD') AS valid_from, to_char(valid_to, 'YYYY-MM-DD') AS valid_to,
              per_customer_per_day
         FROM vouchers WHERE code = $1`,
      [code],
    );
    const r = rows[0];
    if (!r) return null;
    return {
      code: r.code,
      percent: r.percent,
      maxDiscount: r.max_discount,
      minAmount: r.min_amount,
      kinds: r.kinds,
      cityCodes: r.city_codes,
      validFrom: r.valid_from,
      validTo: r.valid_to,
      perCustomerPerDay: r.per_customer_per_day,
    };
  }
}

export class PgIdempotencyStore implements IdempotencyStore {
  constructor(private readonly db: PGlite) {}

  async begin(scope: string, key: string, fingerprint: string, now: Date): Promise<IdempotencyBegin> {
    const inserted = await this.db.query(
      `INSERT INTO idempotency_keys (scope, key, fingerprint, created_at) VALUES ($1, $2, $3, $4)
       ON CONFLICT (scope, key) DO NOTHING RETURNING key`,
      [scope, key, fingerprint, now],
    );
    if (inserted.rows.length === 1) return { state: 'new' };

    const { rows } = await this.db.query<{ fingerprint: string; status_code: number | null; body: unknown }>(
      'SELECT fingerprint, status_code, body FROM idempotency_keys WHERE scope = $1 AND key = $2',
      [scope, key],
    );
    const r = rows[0];
    if (!r) return this.begin(scope, key, fingerprint, now); // vừa bị release — giữ chỗ lại
    if (r.fingerprint !== fingerprint) return { state: 'mismatch' };
    if (r.status_code === null) return { state: 'in_progress' };
    return { state: 'replay', response: { status: r.status_code, body: r.body } };
  }

  async complete(scope: string, key: string, response: StoredResponse): Promise<void> {
    await this.db.query('UPDATE idempotency_keys SET status_code = $3, body = $4 WHERE scope = $1 AND key = $2', [
      scope,
      key,
      response.status,
      JSON.stringify(response.body),
    ]);
  }

  async release(scope: string, key: string): Promise<void> {
    await this.db.query('DELETE FROM idempotency_keys WHERE scope = $1 AND key = $2', [scope, key]);
  }
}
