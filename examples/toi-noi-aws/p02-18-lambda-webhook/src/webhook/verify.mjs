import { createHmac, timingSafeEqual } from 'node:crypto';

/**
 * Xác minh chữ ký webhook của cổng thanh toán giả lập VíXanh.
 * Header: x-vixanh-signature: t=<unix giây>,v1=<hex HMAC-SHA256>
 * Chuỗi được ký: `${t}.${rawBody}` với rawBody là đúng byte nhận được (chưa parse JSON).
 */
export function verifySignature({ header, rawBody, secret, now = Date.now(), toleranceSec = 300 }) {
  if (!header) return { ok: false, reason: 'missing-signature' };

  const parts = {};
  for (const piece of header.split(',')) {
    const [k, v] = piece.trim().split('=');
    if (k && v) parts[k] = v;
  }

  const t = Number(parts.t);
  if (!Number.isInteger(t) || !parts.v1) return { ok: false, reason: 'bad-format' };
  if (Math.abs(now / 1000 - t) > toleranceSec) return { ok: false, reason: 'stale-timestamp' };

  const expected = createHmac('sha256', secret).update(`${t}.${rawBody}`).digest();
  const given = Buffer.from(parts.v1, 'hex');
  if (given.length !== expected.length || !timingSafeEqual(given, expected)) {
    return { ok: false, reason: 'signature-mismatch' };
  }
  return { ok: true };
}
