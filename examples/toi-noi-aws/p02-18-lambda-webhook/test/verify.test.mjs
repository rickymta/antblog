import test from 'node:test';
import assert from 'node:assert/strict';
import { createHmac } from 'node:crypto';
import { verifySignature } from '../src/webhook/verify.mjs';

const secret = 'test-secret-khong-dung-that';
const body = '{"eventId":"evt-1","type":"payment.succeeded","tripId":"1001","amountVnd":52500}';
const now = Date.UTC(2026, 10, 6, 4, 30, 0);
const t = Math.floor(now / 1000);
const sign = (ts, b, s = secret) => createHmac('sha256', s).update(`${ts}.${b}`).digest('hex');

test('chữ ký đúng được chấp nhận', () => {
  const header = `t=${t},v1=${sign(t, body)}`;
  assert.deepEqual(verifySignature({ header, rawBody: body, secret, now }), { ok: true });
});

test('thiếu header bị từ chối', () => {
  assert.equal(verifySignature({ header: undefined, rawBody: body, secret, now }).reason, 'missing-signature');
});

test('body bị sửa bị từ chối', () => {
  const header = `t=${t},v1=${sign(t, body)}`;
  const tampered = body.replace('52500', '1');
  assert.equal(verifySignature({ header, rawBody: tampered, secret, now }).reason, 'signature-mismatch');
});

test('sai secret bị từ chối', () => {
  const header = `t=${t},v1=${sign(t, body, 'secret-khac')}`;
  assert.equal(verifySignature({ header, rawBody: body, secret, now }).reason, 'signature-mismatch');
});

test('timestamp cũ hơn 5 phút bị từ chối (chống replay)', () => {
  const old = t - 301;
  const header = `t=${old},v1=${sign(old, body)}`;
  assert.equal(verifySignature({ header, rawBody: body, secret, now }).reason, 'stale-timestamp');
});

test('chữ ký hex lỗi hoặc ngắn bị từ chối, không ném lỗi', () => {
  const header = `t=${t},v1=zz`;
  assert.equal(verifySignature({ header, rawBody: body, secret, now }).ok, false);
});
