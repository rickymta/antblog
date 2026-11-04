import { z } from 'zod';

/** Kiểm định dạng đầu vào (SPEC R-API-07…11, R-FOOD-02). Trường lạ bị bỏ qua (R-API-14). */

const cityCode = z.string().min(1).max(10);
const vehicle = z.enum(['bike', 'car']);
const distanceKm = z.number();
const voucherCode = z.string().max(32).nullish();

const foodItem = z.object({
  name: z.string().trim().min(1).max(100),
  unitPrice: z.number().int().min(1_000).max(5_000_000).multipleOf(1_000),
  quantity: z.number().int().min(1).max(50),
});

export const quoteSchema = z.discriminatedUnion('kind', [
  z.object({ kind: z.literal('ride'), cityCode, vehicle, distanceKm, voucherCode }),
  z.object({ kind: z.literal('parcel'), cityCode, vehicle, distanceKm, parcelWeightKg: z.number(), voucherCode }),
  z.object({
    kind: z.literal('food'),
    cityCode,
    vehicle: z.literal('bike').optional(),
    distanceKm,
    items: z.array(foodItem).min(1).max(20),
    voucherCode,
  }),
]);

export const createTripSchema = z.object({
  quoteId: z.string().min(1).max(100),
  paymentMethod: z.enum(['cash', 'vixanh']).optional(),
});

export const acceptSchema = z.object({
  etaMinutes: z.number().int().min(1).max(60),
});

export const cancelSchema = z.object({
  reason: z.string().trim().max(200).optional(),
});

export const testClockSchema = z.object({ now: z.iso.datetime({ offset: true }) });
export const testWeatherSchema = z.object({ raining: z.array(z.string()) });
export const testPaymentSchema = z.object({ mode: z.enum(['none', 'decline', 'error', 'timeout', 'lost_response']) });
