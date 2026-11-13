/**
 * SYS-16 · Deployment test: chạy máy chủ lab như một bản triển khai (tiến trình riêng, cấu hình qua biến môi trường)
 * rồi kiểm qua HTTP thật — readiness, cấu hình an toàn, post-deploy smoke và bằng chứng cấu hình đang chạy.
 * Không sửa src/; không cần DB ngoài (TOINOI_DB=pglite chạy trong tiến trình máy chủ).
 */
import { spawn, type ChildProcess } from 'node:child_process';
import { createServer } from 'node:net';
import { fileURLToPath } from 'node:url';
import { afterEach, describe, expect, it } from 'vitest';

const LAB_ROOT = fileURLToPath(new URL('../../', import.meta.url));
const READY_DEADLINE_MS = 15_000;

interface Deployment {
  baseUrl: string;
  process: ChildProcess;
  log: () => string;
  readyAfterMs: number;
}

const running: ChildProcess[] = [];

async function freePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const probe = createServer();
    probe.once('error', reject);
    probe.listen(0, '127.0.0.1', () => {
      const address = probe.address();
      const port = typeof address === 'object' && address ? address.port : 0;
      probe.close(() => resolve(port));
    });
  });
}

/** Khởi động `src/server.ts` với cấu hình cho trước; chờ /api/health 200 trong hạn chót, không sleep cố định. */
async function deploy(extraEnv: Record<string, string>): Promise<Deployment> {
  const port = await freePort();
  const env: NodeJS.ProcessEnv = { ...process.env };
  delete env.NODE_ENV; // Vitest tự đặt NODE_ENV=test — phải xoá, nếu không API kiểm thử sẽ bật.
  delete env.TOINOI_TEST_API;
  Object.assign(env, { PORT: String(port), ...extraEnv });

  const child = spawn(process.execPath, ['--import', 'tsx', 'src/server.ts'], { cwd: LAB_ROOT, env });
  running.push(child);
  let output = '';
  child.stdout?.on('data', (chunk) => { output += String(chunk); });
  child.stderr?.on('data', (chunk) => { output += String(chunk); });

  const baseUrl = `http://127.0.0.1:${port}`;
  const started = Date.now();
  while (Date.now() - started < READY_DEADLINE_MS) {
    if (child.exitCode !== null) throw new Error(`Máy chủ thoát sớm (exit ${child.exitCode}):\n${output}`);
    try {
      const res = await fetch(`${baseUrl}/api/health`);
      if (res.status === 200) return { baseUrl, process: child, log: () => output, readyAfterMs: Date.now() - started };
    } catch {
      // Chưa nghe cổng — thử lại sau 100 ms.
    }
    await new Promise((r) => setTimeout(r, 100));
  }
  throw new Error(`Không sẵn sàng sau ${READY_DEADLINE_MS} ms:\n${output}`);
}

const PROD_LIKE = {
  NODE_ENV: 'production',
  TOINOI_DB: 'pglite',
  TOINOI_START_AT: '2026-11-04T09:00:00+07:00',
};

afterEach(async () => {
  for (const child of running.splice(0)) {
    if (child.exitCode === null) {
      const exited = new Promise((r) => child.once('exit', r));
      child.kill();
      await exited;
    }
  }
});

describe('SYS-16 · triển khai máy chủ lab và kiểm sau triển khai', () => {
  it('DP-01 · R-API-15: tiến trình mới sẵn sàng trong hạn chót và health đúng hợp đồng', async () => {
    const d = await deploy(PROD_LIKE);
    const res = await fetch(`${d.baseUrl}/api/health`);
    expect(res.status).toBe(200);
    expect(await res.json()).toEqual({ status: 'ok' });
    expect(d.readyAfterMs).toBeLessThan(READY_DEADLINE_MS);
    console.info(`[SYS-16] DP-01 readyAfterMs=${d.readyAfterMs}`);
  }, 30_000);

  it('DP-02 · R-TEST-01/R-NFR-06: cấu hình triển khai không mở API kiểm thử', async () => {
    const d = await deploy(PROD_LIKE);
    const res = await fetch(`${d.baseUrl}/api/test/reset`, { method: 'POST' });
    expect(res.status).toBe(404);
    expect(await res.json()).toMatchObject({ error: { code: 'NOT_FOUND' } });
  }, 30_000);

  it('DP-03 · bằng chứng cấu hình: log khởi động ghi đúng kho dữ liệu, giờ và trạng thái API test', async () => {
    const d = await deploy(PROD_LIKE);
    expect(d.log()).toContain('kho: pglite');
    expect(d.log()).toContain('API test: tắt');
    expect(d.log()).toMatch(/giờ hệ thống: 2026-11-04T02:00:\d{2}/);
  }, 30_000);

  it('DP-04 · post-deploy smoke qua HTTP thật: báo giá → đặt → hoàn tất → đọc lại', async () => {
    const d = await deploy(PROD_LIKE);
    const call = async (method: string, path: string, user: string, body?: unknown, key?: string) => {
      const headers: Record<string, string> = { 'content-type': 'application/json', 'x-user': user };
      if (key) headers['idempotency-key'] = key;
      const res = await fetch(`${d.baseUrl}${path}`, { method, headers, body: body ? JSON.stringify(body) : undefined });
      // eslint-disable-next-line @typescript-eslint/no-explicit-any -- JSON phản hồi được assert từng trường bên dưới
      return { status: res.status, body: (await res.json()) as any };
    };
    const quote = await call('POST', '/api/quotes', 'customer:C001', { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 });
    expect(quote.status).toBe(200);
    expect(quote.body.quote.total).toBe(35_000);
    const created = await call('POST', '/api/trips', 'customer:C001',
      { quoteId: quote.body.quote.quoteId, paymentMethod: 'vixanh' }, 'sys16-post-deploy-001');
    expect(created.status).toBe(201);
    const id: string = created.body.trip.id;
    expect((await call('POST', `/api/trips/${id}/accept`, 'driver:D101', { etaMinutes: 5 })).status).toBe(200);
    for (const step of ['arrive', 'pickup', 'start', 'complete']) {
      expect((await call('POST', `/api/trips/${id}/${step}`, 'driver:D101', {})).status).toBe(200);
    }
    const read = await call('GET', `/api/trips/${id}`, 'customer:C001');
    expect(read.status).toBe(200);
    expect(read.body.trip).toMatchObject({ id, status: 'completed', fare: 35_000, paymentStatus: 'paid', pricingVersion: 'fare-2026.1' });
  }, 30_000);

  it('DP-05 · phép kiểm cấu hình bắt được bản triển khai sai: bật TOINOI_TEST_API=1 thì API kiểm thử lộ ra', async () => {
    const d = await deploy({ ...PROD_LIKE, TOINOI_TEST_API: '1' });
    const res = await fetch(`${d.baseUrl}/api/test/reset`, { method: 'POST' });
    // Đây là bản triển khai SAI có chủ đích: phép kiểm DP-02 chạy trên nó phải đỏ.
    expect(res.status).not.toBe(404);
    expect(d.log()).toContain('API test: BẬT');
  }, 30_000);
});
