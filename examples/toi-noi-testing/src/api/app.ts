import { fileURLToPath } from 'node:url';
import express, { type NextFunction, type Request, type Response } from 'express';
import type { ZodType } from 'zod';
import type { Actor, AppDeps } from '../app/deps';
import { createQuote, type StoredQuote } from '../app/quote';
import {
  acceptTrip,
  cancelByCustomer,
  cancelByDriver,
  createTrip,
  driverStep,
  getTrip,
  listAvailableTrips,
} from '../app/trips';
import { ApiError, toErrorResponse, validationError, type JsonResponse } from './http-errors';
import { runIdempotent } from './idempotency';
import { createRateLimiter } from './rate-limit';
import { acceptSchema, cancelSchema, createTripSchema, quoteSchema } from './schemas';
import { createTestRouter, type TestControl } from './test-routes';

export interface AppOptions {
  /** API kiểm thử `/api/test/*` — CHỈ truyền khi NODE_ENV=test hoặc cờ TOINOI_TEST_API=1 (SPEC R-TEST-01). */
  testControl?: TestControl;
  /** Ghi lỗi 500 ra console (mặc định bật). */
  logErrors?: boolean;
}

const USER_HEADER = /^(customer|driver):([A-Za-z0-9_-]{1,64})$/;
const WEB_DIR = fileURLToPath(new URL('../web', import.meta.url));

/** Tạo ứng dụng Express — không mở cổng, để test dùng supertest trực tiếp. */
export function createApp(deps: AppDeps, options: AppOptions = {}) {
  const app = express();
  app.disable('x-powered-by');
  app.use(express.json({ limit: '10kb' }));

  const tripLimiter = createRateLimiter({ ...deps.config.tripRateLimit, clock: deps.clock });
  const idem = { store: deps.idempotency, clock: deps.clock };

  // ---- xác thực & vai trò (SPEC R-API-03, R-API-04)
  const authenticate = async (req: Request, res: Response, next: NextFunction) => {
    const match = USER_HEADER.exec(req.get('x-user') ?? '');
    const role = match?.[1] as Actor['role'] | undefined;
    const id = match?.[2];
    const exists = role === 'customer' ? await deps.customers.findById(id!) : role === 'driver' ? await deps.drivers.findById(id!) : null;
    if (!role || !id || !exists) throw new ApiError('UNAUTHENTICATED', 'Thiếu hoặc sai header x-user.');
    res.locals.actor = { role, id } satisfies Actor;
    next();
  };
  const only = (role: Actor['role']) => (_req: Request, res: Response, next: NextFunction) => {
    if (actorOf(res).role !== role) {
      throw new ApiError('FORBIDDEN', role === 'customer' ? 'Chỉ khách hàng được dùng chức năng này.' : 'Chỉ tài xế được dùng chức năng này.');
    }
    next();
  };
  const rateLimitTrips = (_req: Request, res: Response, next: NextFunction) => {
    const decision = tripLimiter.check(actorOf(res).id);
    if (!decision.allowed) {
      res.set('Retry-After', String(decision.retryAfterSeconds));
      throw new ApiError('RATE_LIMITED', 'Bạn tạo chuyến quá nhanh, vui lòng thử lại sau.');
    }
    next();
  };

  // ---- endpoint
  app.get('/api/health', (_req, res) => {
    res.json({ status: 'ok' });
  });

  app.post('/api/quotes', authenticate, only('customer'), async (req, res) => {
    const input = parse(quoteSchema, req.body);
    const quote = await createQuote(deps, actorOf(res), input);
    res.json({ quote: quoteDto(quote) });
  });

  app.post('/api/trips', authenticate, only('customer'), rateLimitTrips, async (req, res) => {
    const actor = actorOf(res);
    const result = await runIdempotent(req, idem, `${actor.role}:${actor.id}:POST /api/trips`, async () => {
      const trip = await createTrip(deps, actor, parse(createTripSchema, req.body));
      return { status: 201, body: { trip } };
    });
    send(res, result);
  });

  app.get('/api/trips/:id', authenticate, async (req, res) => {
    res.json({ trip: await getTrip(deps, actorOf(res), req.params.id as string) });
  });

  app.post('/api/trips/:id/accept', authenticate, only('driver'), async (req, res) => {
    const { etaMinutes } = parse(acceptSchema, req.body);
    res.json({ trip: await acceptTrip(deps, actorOf(res), req.params.id as string, etaMinutes) });
  });

  for (const [path, event] of [['arrive', 'arrive'], ['pickup', 'pickup'], ['start', 'start'], ['complete', 'complete']] as const) {
    app.post(`/api/trips/:id/${path}`, authenticate, only('driver'), async (req, res) => {
      res.json({ trip: await driverStep(deps, actorOf(res), req.params.id as string, event) });
    });
  }

  app.post('/api/trips/:id/cancel', authenticate, async (req, res) => {
    const actor = actorOf(res);
    const tripId = req.params.id as string;
    const result = await runIdempotent(req, idem, `${actor.role}:${actor.id}:POST /api/trips/${tripId}/cancel`, async () => {
      const { reason } = parse(cancelSchema, req.body ?? {});
      if (actor.role === 'customer') {
        return { status: 200, body: await cancelByCustomer(deps, actor, tripId, reason) };
      }
      return { status: 200, body: { trip: await cancelByDriver(deps, actor, tripId) } };
    });
    send(res, result);
  });

  app.get('/api/driver/available-trips', authenticate, only('driver'), async (_req, res) => {
    res.json({ trips: await listAvailableTrips(deps, actorOf(res)) });
  });

  if (options.testControl) app.use('/api/test', createTestRouter(deps, options.testControl, () => tripLimiter.reset()));

  app.use('/api', (_req, _res, next) => next(new ApiError('NOT_FOUND', 'Không có đường dẫn này.')));
  app.use(express.static(WEB_DIR));

  // ---- lỗi (SPEC R-API-02): không lộ stack trace ra ngoài
  app.use((error: unknown, _req: Request, res: Response, _next: NextFunction) => {
    const response = toErrorResponse(error);
    if (response.status >= 500 && options.logErrors !== false) console.error(error);
    res.status(response.status).json(response.body);
  });

  return app;
}

function actorOf(res: Response): Actor {
  return res.locals.actor as Actor;
}

function parse<T>(schema: ZodType<T>, body: unknown): T {
  const result = schema.safeParse(body);
  if (!result.success) throw validationError(result.error);
  return result.data;
}

function send(res: Response, result: JsonResponse & { replayed: boolean }): void {
  if (result.replayed) res.set('Idempotent-Replayed', 'true');
  res.status(result.status).json(result.body);
}

/** Báo giá trả cho client (SPEC R-API-07). */
export function quoteDto(q: StoredQuote) {
  return {
    quoteId: q.id,
    pricingVersion: q.pricingVersion,
    createdAt: q.createdAt,
    expiresAt: q.expiresAt,
    ...q.details,
  };
}
