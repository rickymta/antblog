import { Router } from 'express';
import type { AppDeps } from '../app/deps';
import { expireStaleSearches } from '../app/trips';
import { isCityCode } from '../domain/types';
import type { FakePaymentGateway } from '../infra/fake-payment-gateway';
import type { FakeWeather } from '../infra/fake-weather';
import { ApiError, validationError } from './http-errors';
import { testClockSchema, testPaymentSchema, testWeatherSchema } from './schemas';

/**
 * Móc kiểm thử (SPEC R-TEST). KHÔNG bao giờ gắn vào bản chạy thật:
 * `createApp` chỉ gắn router này khi được truyền `testControl`.
 */
export interface TestControl {
  reset(): Promise<void>;
  setNow(at: Date): void;
  weather: FakeWeather;
  payments: FakePaymentGateway;
}

export function createTestRouter(deps: AppDeps, control: TestControl, resetRateLimit: () => void): Router {
  const router = Router();

  router.post('/reset', async (_req, res) => {
    await control.reset();
    resetRateLimit();
    res.json({ status: 'reset' });
  });

  router.post('/clock', (req, res) => {
    const parsed = testClockSchema.safeParse(req.body);
    if (!parsed.success) throw validationError(parsed.error);
    control.setNow(new Date(parsed.data.now));
    res.json({ now: deps.clock.now() });
  });

  router.post('/weather', (req, res) => {
    const parsed = testWeatherSchema.safeParse(req.body);
    if (!parsed.success) throw validationError(parsed.error);
    const cities = parsed.data.raining.filter(isCityCode);
    if (cities.length !== parsed.data.raining.length) throw new ApiError('VALIDATION_ERROR', 'Mã thành phố không hợp lệ.');
    control.weather.setRaining(cities);
    res.json({ raining: cities });
  });

  router.post('/payment', (req, res) => {
    const parsed = testPaymentSchema.safeParse(req.body);
    if (!parsed.success) throw validationError(parsed.error);
    control.payments.setMode(parsed.data.mode);
    res.json({ mode: parsed.data.mode });
  });

  router.post('/expire-searches', async (_req, res) => {
    res.json({ expired: await expireStaleSearches(deps) });
  });

  return router;
}
