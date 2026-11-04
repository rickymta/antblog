/**
 * Máy chủ lab Tới Nơi. Biến môi trường:
 *   PORT=3100                     cổng nghe
 *   TOINOI_DB=memory|pglite       kho dữ liệu (mặc định memory)
 *   TOINOI_START_AT=<ISO 8601>    dời đồng hồ tới mốc này rồi chạy tiếp (vd 2026-11-04T11:30:00+07:00)
 *   TOINOI_RAIN=SG,HN             thành phố đang mưa (thời tiết giả)
 *   TRIP_RATE_LIMIT_MAX=5         số POST /api/trips mỗi khách trong cửa sổ; 0 = tắt
 *   TRIP_RATE_LIMIT_WINDOW_MS=60000
 *   NODE_ENV=test hoặc TOINOI_TEST_API=1   bật API kiểm thử /api/test/* (mặc định TẮT)
 */
import { DEFAULT_CONFIG } from './app/deps';
import { expireStaleSearches } from './app/trips';
import { createApp } from './api/app';
import { isCityCode } from './domain/types';
import { AdjustableClock } from './infra/clock';
import { createMemorySystem, createPgliteSystem, type ComposeOptions } from './infra/compose';
import { RandomIdGenerator } from './infra/ids';

const env = process.env;
const port = Number(env.PORT ?? 3100);
const clock = new AdjustableClock(env.TOINOI_START_AT ? new Date(env.TOINOI_START_AT) : undefined);

const options: ComposeOptions = {
  clock,
  ids: new RandomIdGenerator(),
  rainingCities: (env.TOINOI_RAIN ?? '').split(',').map((c) => c.trim()).filter(isCityCode),
  config: {
    tripRateLimit: {
      max: Number(env.TRIP_RATE_LIMIT_MAX ?? DEFAULT_CONFIG.tripRateLimit.max),
      windowMs: Number(env.TRIP_RATE_LIMIT_WINDOW_MS ?? DEFAULT_CONFIG.tripRateLimit.windowMs),
    },
  },
};
const system = env.TOINOI_DB === 'pglite' ? await createPgliteSystem(options) : createMemorySystem(options);

const testApiEnabled = env.NODE_ENV === 'test' || env.TOINOI_TEST_API === '1';
const app = createApp(system.deps, {
  testControl: testApiEnabled
    ? { reset: system.reset, setNow: (at) => clock.set(at), weather: system.weather, payments: system.payments }
    : undefined,
});

// Quét chuyến tìm tài xế quá 5 phút (SPEC R-STATE-05).
const sweeper = setInterval(() => {
  expireStaleSearches(system.deps).catch((error: unknown) => console.error('Lỗi quét chuyến hết hạn', error));
}, 15_000);

const server = app.listen(port, () => {
  console.log(
    `Tới Nơi lab chạy tại http://localhost:${port} (kho: ${env.TOINOI_DB === 'pglite' ? 'pglite' : 'memory'}, ` +
      `giờ hệ thống: ${clock.now().toISOString()}, API test: ${testApiEnabled ? 'BẬT' : 'tắt'})`,
  );
});

function shutdown(): void {
  clearInterval(sweeper);
  server.close(() => process.exit(0));
}
process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);
