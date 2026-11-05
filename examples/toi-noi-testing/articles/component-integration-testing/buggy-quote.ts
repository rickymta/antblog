/**
 * TEST-013 — năm bản «ghép lỗi» của priceQuote (src/app/quote.ts).
 *
 * Mỗi bản dùng lại ĐÚNG các component thật của src/domain (pricing.ts, voucher.ts, fare.ts,
 * time.ts, money.ts) — không component nào bị sửa. Defect nằm ở CÁCH GHÉP: truyền sai đơn vị,
 * truyền giá trị đã làm tròn, bỏ bước chuẩn hoá, tự tính giờ, gọi sai thứ tự.
 * Chỉ hỗ trợ `ride`/`parcel` (đủ cho bài). Không sửa src/.
 */
import type { AppDeps } from '../../src/app/deps';
import { assertVoucherQuota, priceQuote, type QuoteDetails, type QuoteInput } from '../../src/app/quote';
import { DomainError } from '../../src/domain/errors';
import { FARE_TABLE } from '../../src/domain/fare';
import { roundUpToThousandDong, toDong, toMilli } from '../../src/domain/money';
import { normalizePriceRequest, priceRequest, type NormalizedRequest } from '../../src/domain/pricing';
import { isPeakTime } from '../../src/domain/time';
import { assertVoucherEligible, normalizeVoucherCode, voucherDiscountMilli, type Voucher } from '../../src/domain/voucher';

export type PriceFn = typeof priceQuote;

export type DefectId = 'D1-don-vi-tien' | 'D2-lam-tron-giua-chung' | 'D3-ma-chua-chuan-hoa' | 'D4-gio-may-chu' | 'D5-thu-tu-kiem-tra';

type VoucherLoader = (deps: AppDeps, raw: string | null | undefined, req: NormalizedRequest, at: Date) => Promise<Voucher | null>;

/** Nạp voucher đúng như src/app/quote.ts (chuẩn hoá mã R-VOUCHER-02 rồi kiểm thời hạn, phạm vi). */
const loadVoucher: VoucherLoader = async (deps, raw, req, at) => {
  const code = normalizeVoucherCode(raw);
  if (code === null) return null;
  const voucher = await deps.vouchers.findByCode(code);
  if (!voucher) throw new DomainError('VOUCHER_NOT_FOUND', `Không tìm thấy voucher ${code}.`);
  assertVoucherEligible(voucher, { kind: req.kind, cityCode: req.cityCode, at });
  return voucher;
};

/** D3 — tầng ứng dụng tưởng repository tự chuẩn hoá mã nên chuyển thẳng chuỗi khách gõ. */
const loadVoucherRaw: VoucherLoader = async (deps, raw, req, at) => {
  if (raw === null || raw === undefined) return null; // "" và " giam20 " lọt qua
  const voucher = await deps.vouchers.findByCode(raw);
  if (!voucher) throw new DomainError('VOUCHER_NOT_FOUND', `Không tìm thấy voucher ${raw}.`);
  assertVoucherEligible(voucher, { kind: req.kind, cityCode: req.cityCode, at });
  return voucher;
};

/** D4 — tự viết lại «giờ cao điểm» bằng giờ của máy chủ; container chạy UTC nên giờ máy = giờ UTC. */
const peakByServerClock = (at: Date): boolean => {
  const hour = at.getUTCHours();
  return (hour >= 11 && hour < 13) || (hour >= 17 && hour < 19);
};

/**
 * D1, D2 — «tái cấu trúc»: tính cước KHÔNG voucher qua priceRequest, rồi tầng ứng dụng tự trừ voucher.
 * `amountOf` chọn số tiền A đưa cho voucherDiscountMilli — chỗ hai defect khác nhau.
 */
function appLayerVoucher(amountOf: (fare: QuoteDetails['fare']) => number): PriceFn {
  return async (deps, _customerId, input, at) => {
    const request = normalizePriceRequest(input);
    const voucher = await loadVoucher(deps, input.voucherCode, request, at);
    const peak = isPeakTime(at);
    const raining = await deps.weather.isRaining(request.cityCode, at);
    const { fare } = priceRequest(request, { peak, raining, voucher: null });
    const discountMilli = voucher ? voucherDiscountMilli(voucher, amountOf(fare)) : 0;
    const min = toMilli(FARE_TABLE[request.vehicle].minimumFare);
    const total = roundUpToThousandDong(Math.max(toMilli(fare.beforeDiscount) - discountMilli, min));
    return details(request, peak, raining, voucher, { ...fare, discount: toDong(discountMilli), total });
  };
}

function details(
  request: NormalizedRequest,
  peak: boolean,
  raining: boolean,
  voucher: Voucher | null,
  fare: QuoteDetails['fare'],
): QuoteDetails {
  return {
    kind: request.kind,
    cityCode: request.cityCode,
    vehicle: request.vehicle,
    distanceKm: request.distanceKm,
    parcelWeightKg: request.parcelWeightKg,
    items: null,
    peak,
    raining,
    voucherCode: voucher?.code ?? null,
    fare,
    itemsSubtotal: null,
    itemsDiscount: null,
    deliveryFee: null,
    discount: fare.discount,
    total: fare.total,
  };
}

/** Khung chung cho D3, D4, D5: giống quote.ts, chỉ thay đúng một mắt xích. */
function wired(opts: { loader?: VoucherLoader; peakOf?: (at: Date) => boolean; quotaFirst?: boolean }): PriceFn {
  const loader = opts.loader ?? loadVoucher;
  const peakOf = opts.peakOf ?? isPeakTime;
  return async (deps, customerId, input: QuoteInput, at) => {
    const request = normalizePriceRequest(input);
    const voucher = await loader(deps, input.voucherCode, request, at);
    if (voucher && opts.quotaFirst) await assertVoucherQuota(deps, customerId, voucher, at); // D5: hỏi lượt TRƯỚC
    const peak = peakOf(at);
    const raining = await deps.weather.isRaining(request.cityCode, at);
    const price = priceRequest(request, { peak, raining, voucher }); // VOUCHER_MIN_AMOUNT ném ở đây
    if (voucher && !opts.quotaFirst) await assertVoucherQuota(deps, customerId, voucher, at);
    return details(request, peak, raining, voucher, price.fare);
  };
}

export const defectivePriceQuote: Record<DefectId, PriceFn> = {
  // FareBreakdown trả ĐỒNG, voucherDiscountMilli nhận MILLI-ĐỒNG (src/domain/money.ts).
  'D1-don-vi-tien': appLayerVoucher((fare) => fare.beforeDiscount),
  // A lấy từ tổng đã làm tròn lên 1.000 đ và đã nâng mức tối thiểu — trái R-GEN-01, R-FARE-06.
  'D2-lam-tron-giua-chung': appLayerVoucher((fare) => toMilli(fare.total)),
  'D3-ma-chua-chuan-hoa': wired({ loader: loadVoucherRaw }),
  'D4-gio-may-chu': wired({ peakOf: peakByServerClock }),
  'D5-thu-tu-kiem-tra': wired({ quotaFirst: true }),
};
