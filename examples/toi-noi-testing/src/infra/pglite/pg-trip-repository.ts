import type { PGlite, Transaction } from '@electric-sql/pglite';
import type { TripRepository } from '../../app/ports';
import type { Trip, TripEventRecord } from '../../domain/trip';
import type { TripStatus } from '../../domain/trip-state';
import type { CityCode, Vehicle } from '../../domain/types';

type Db = PGlite | Transaction;

/** Một dòng `trips` như PostgreSQL trả về (snake_case; numeric là chuỗi). */
interface TripRow {
  id: string;
  quote_id: string;
  pricing_version: string;
  kind: Trip['kind'];
  city_code: CityCode;
  customer_id: string;
  driver_id: string | null;
  vehicle_type: Vehicle;
  status: TripStatus;
  distance_km: string;
  parcel_weight_kg: string | null;
  items: Trip['items'];
  items_subtotal: number | null;
  delivery_fee: number | null;
  peak: boolean;
  raining: boolean;
  voucher_code: string | null;
  discount: string;
  fare: number;
  fare_breakdown: Trip['fareBreakdown'];
  payment_method: Trip['paymentMethod'];
  payment_status: Trip['paymentStatus'];
  requested_at: Date;
  searching_since: Date | null;
  accepted_at: Date | null;
  eta_minutes: number | null;
  completed_at: Date | null;
  cancelled_at: Date | null;
  cancelled_by: Trip['cancelledBy'];
  cancel_reason: string | null;
  cancellation_fee: number;
  updated_at: Date;
}

const COLUMNS = [
  'id', 'quote_id', 'pricing_version', 'kind', 'city_code', 'customer_id', 'driver_id', 'vehicle_type', 'status',
  'distance_km', 'parcel_weight_kg', 'items', 'items_subtotal', 'delivery_fee', 'peak', 'raining', 'voucher_code',
  'discount', 'fare', 'fare_breakdown', 'payment_method', 'payment_status', 'requested_at', 'searching_since',
  'accepted_at', 'eta_minutes', 'completed_at', 'cancelled_at', 'cancelled_by', 'cancel_reason', 'cancellation_fee',
  'updated_at',
] as const;

function toParams(t: Trip): unknown[] {
  return [
    t.id, t.quoteId, t.pricingVersion, t.kind, t.cityCode, t.customerId, t.driverId, t.vehicle, t.status,
    t.distanceKm, t.parcelWeightKg, t.items === null ? null : JSON.stringify(t.items), t.itemsSubtotal, t.deliveryFee,
    t.peak, t.raining, t.voucherCode, t.discount, t.fare, JSON.stringify(t.fareBreakdown), t.paymentMethod,
    t.paymentStatus, t.requestedAt, t.searchingSince, t.acceptedAt, t.etaMinutes, t.completedAt, t.cancelledAt,
    t.cancelledBy, t.cancelReason, t.cancellationFee, t.updatedAt,
  ];
}

function fromRow(r: TripRow): Trip {
  return {
    id: r.id,
    quoteId: r.quote_id,
    pricingVersion: r.pricing_version,
    kind: r.kind,
    cityCode: r.city_code,
    customerId: r.customer_id,
    driverId: r.driver_id,
    vehicle: r.vehicle_type,
    status: r.status,
    distanceKm: Number(r.distance_km),
    parcelWeightKg: r.parcel_weight_kg === null ? null : Number(r.parcel_weight_kg),
    items: r.items,
    itemsSubtotal: r.items_subtotal,
    deliveryFee: r.delivery_fee,
    peak: r.peak,
    raining: r.raining,
    voucherCode: r.voucher_code,
    discount: Number(r.discount),
    fare: r.fare,
    fareBreakdown: r.fare_breakdown,
    paymentMethod: r.payment_method,
    paymentStatus: r.payment_status,
    requestedAt: r.requested_at,
    searchingSince: r.searching_since,
    acceptedAt: r.accepted_at,
    etaMinutes: r.eta_minutes,
    completedAt: r.completed_at,
    cancelledAt: r.cancelled_at,
    cancelledBy: r.cancelled_by,
    cancelReason: r.cancel_reason,
    cancellationFee: r.cancellation_fee,
    updatedAt: r.updated_at,
  };
}

const INSERT_SQL = `INSERT INTO trips (${COLUMNS.join(', ')}) VALUES (${COLUMNS.map((_, i) => `$${i + 1}`).join(', ')})`;
// $1 = id; $2..$32 = các cột còn lại; $33 = trạng thái mong đợi.
const UPDATE_SQL = `UPDATE trips SET ${COLUMNS.slice(1).map((c, i) => `${c} = $${i + 2}`).join(', ')}
                    WHERE id = $1 AND status = $${COLUMNS.length + 1} RETURNING id`;

async function insertEvent(db: Db, e: TripEventRecord): Promise<void> {
  await db.query('INSERT INTO trip_events (trip_id, status, occurred_at) VALUES ($1, $2, $3)', [e.tripId, e.status, e.occurredAt]);
}

export class PgTripRepository implements TripRepository {
  constructor(private readonly db: PGlite) {}

  async create(trip: Trip, events: TripEventRecord[]): Promise<void> {
    await this.db.transaction(async (tx) => {
      await tx.query(INSERT_SQL, toParams(trip));
      for (const e of events) await insertEvent(tx, e);
    });
  }

  async findById(id: string): Promise<Trip | null> {
    const { rows } = await this.db.query<TripRow>('SELECT * FROM trips WHERE id = $1', [id]);
    return rows[0] ? fromRow(rows[0]) : null;
  }

  async updateIfStatus(trip: Trip, expected: TripStatus): Promise<boolean> {
    return this.db.transaction(async (tx) => {
      const { rows } = await tx.query(UPDATE_SQL, [...toParams(trip), expected]);
      if (rows.length === 0) return false;
      if (trip.status !== expected) await insertEvent(tx, { tripId: trip.id, status: trip.status, occurredAt: trip.updatedAt });
      return true;
    });
  }

  /** Chỉ một tài xế thắng: UPDATE có điều kiện `status = 'searching'` (SPEC R-ACCEPT-03). */
  async acceptIfSearching(input: { tripId: string; driverId: string; acceptedAt: Date; etaMinutes: number }): Promise<Trip | null> {
    return this.db.transaction(async (tx) => {
      const { rows } = await tx.query<TripRow>(
        `UPDATE trips
            SET status = 'accepted', driver_id = $2, accepted_at = $3, eta_minutes = $4, updated_at = $3
          WHERE id = $1 AND status = 'searching'
          RETURNING *`,
        [input.tripId, input.driverId, input.acceptedAt, input.etaMinutes],
      );
      if (!rows[0]) return null;
      await insertEvent(tx, { tripId: input.tripId, status: 'accepted', occurredAt: input.acceptedAt });
      return fromRow(rows[0]);
    });
  }

  async listSearching(filter: { cityCode: CityCode; vehicle: Vehicle }): Promise<Trip[]> {
    const { rows } = await this.db.query<TripRow>(
      `SELECT * FROM trips WHERE status = 'searching' AND city_code = $1 AND vehicle_type = $2 ORDER BY requested_at, id`,
      [filter.cityCode, filter.vehicle],
    );
    return rows.map(fromRow);
  }

  async listSearchingSince(cutoff: Date): Promise<Trip[]> {
    const { rows } = await this.db.query<TripRow>(
      `SELECT * FROM trips WHERE status = 'searching' AND searching_since <= $1 ORDER BY searching_since, id`,
      [cutoff],
    );
    return rows.map(fromRow);
  }

  async countVoucherUses(input: { customerId: string; voucherCode: string; from: Date; to: Date }): Promise<number> {
    const { rows } = await this.db.query<{ n: number }>(
      `SELECT count(*)::int AS n FROM trips
        WHERE customer_id = $1 AND voucher_code = $2 AND status <> 'cancelled'
          AND requested_at >= $3 AND requested_at < $4`,
      [input.customerId, input.voucherCode, input.from, input.to],
    );
    return rows[0]?.n ?? 0;
  }

  async listEvents(tripId: string): Promise<TripEventRecord[]> {
    const { rows } = await this.db.query<{ trip_id: string; status: TripStatus; occurred_at: Date }>(
      'SELECT trip_id, status, occurred_at FROM trip_events WHERE trip_id = $1 ORDER BY id',
      [tripId],
    );
    return rows.map((r) => ({ tripId: r.trip_id, status: r.status, occurredAt: r.occurred_at }));
  }
}
