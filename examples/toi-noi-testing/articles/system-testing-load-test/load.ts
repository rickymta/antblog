import { once } from 'node:events';
import { mkdir, writeFile } from 'node:fs/promises';
import { cpus, platform, release, totalmem } from 'node:os';
import { performance } from 'node:perf_hooks';
import { setTimeout as sleep } from 'node:timers/promises';
import { createApp } from '../../src/api/app';
import { createPgliteSystem } from '../../src/infra/compose';
import { FixedClock } from '../../src/infra/clock';
import { vnTime } from '../../src/domain/time';

const system = await createPgliteSystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)) });
const server = createApp(system.deps, { logErrors: false }).listen(0, '127.0.0.1');
await once(server, 'listening');
const address = server.address();
if (!address || typeof address === 'string') throw new Error('Missing port');
const url = `http://127.0.0.1:${address.port}/api/quotes`;
const stages = [{ name: 'low', rps: 10, seconds: 3 }, { name: 'target', rps: 30, seconds: 8 }, { name: 'return-low', rps: 10, seconds: 3 }];
const maxInFlight = 20;
const maxLagMs = 100;
const results = [];
let failureCount = 0;

async function send() {
  const started = performance.now();
  let status = 0;
  let valid = false;
  let error: string | null = null;
  try {
    const res = await fetch(url, {
      method: 'POST', headers: { 'content-type': 'application/json', 'x-user': 'customer:C001' },
      body: JSON.stringify({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 }),
      signal: AbortSignal.timeout(2_000),
    });
    status = res.status;
    const body = await res.json() as { quote?: { total?: number; quoteId?: string } };
    valid = status === 200 && body.quote?.total === 35_000 && typeof body.quote.quoteId === 'string';
    if (!valid) error = 'HTTP_OR_BUSINESS_ORACLE';
  } catch (cause) { error = cause instanceof Error ? cause.name : 'UNKNOWN'; }
  if (!valid) failureCount++;
  return { latencyMs: performance.now() - started, status, valid, error };
}

try {
  for (let i = 0; i < 20; i++) if (!(await send()).valid) throw new Error('Warm-up failed');
  await system.reset();
  for (const stage of stages) {
    const planned = stage.rps * stage.seconds;
    const start = performance.now();
    const active = new Set<Promise<void>>();
    const samples: Array<{ slot: number; lagMs: number; startOffsetMs: number; endOffsetMs: number; latencyMs: number; status: number; valid: boolean; error: string | null }> = [];
    const dropped: Array<{ slot: number; lagMs: number; reason: string }> = [];
    let peakInFlight = 0;
    for (let slot = 0; slot < planned; slot++) {
      const due = start + slot * 1_000 / stage.rps;
      while (performance.now() < due) await sleep(Math.max(1, due - performance.now()));
      const lagMs = performance.now() - due;
      const reason = failureCount >= 10 ? 'failure-stop' : lagMs > maxLagMs ? 'scheduler-late' : active.size >= maxInFlight ? 'in-flight-cap' : null;
      if (reason) { dropped.push({ slot, lagMs, reason }); continue; }
      const startOffsetMs = performance.now() - start;
      const task = send().then(result => { samples.push({ slot, lagMs, startOffsetMs, endOffsetMs: performance.now() - start, ...result }); });
      active.add(task);
      peakInFlight = Math.max(peakInFlight, active.size);
      void task.then(() => active.delete(task));
    }
    const end = start + stage.seconds * 1_000;
    while (performance.now() < end) await sleep(Math.max(1, end - performance.now()));
    await Promise.all(active); // drain before the next step; report includes drain time
    const elapsedMsIncludingDrain = performance.now() - start;
    const sorted = samples.map(s => s.latencyMs).sort((a, b) => a - b);
    const p = (q: number) => sorted.length ? sorted[Math.ceil(q * sorted.length) - 1]! : null;
    const errors = samples.filter(s => !s.valid).length;
    const p95 = p(0.95);
    results.push({ ...stage, planned, sent: samples.length, dropped: dropped.length, errors,
      errorRateSent: samples.length ? errors / samples.length : null,
      droppedRatePlanned: dropped.length / planned, peakInFlight,
      scheduledWindowSeconds: stage.seconds, elapsedMsIncludingDrain,
      sentPerScheduledSecond: samples.length / stage.seconds,
      completedInWindow: samples.filter(s => s.endOffsetMs < stage.seconds * 1_000).length,
      validRpsIncludingDrain: (samples.length - errors) / (elapsedMsIncludingDrain / 1_000),
      p50Ms: p(0.5), p95Ms: p95, p99Ms: p(0.99), maxSchedulerLagMs: Math.max(0, ...samples.map(s => s.lagMs), ...dropped.map(s => s.lagMs)),
      illustrativeGatePassed: dropped.length === 0 && errors === 0 && p95 !== null && p95 < 500,
      samples: samples.sort((a, b) => a.slot - b.slot), droppedSlots: dropped,
    });
  }
  const report = { measuredAtUtc: new Date().toISOString(),
    environment: { node: process.version, os: platform(), release: release(), cpu: cpus()[0]?.model, logicalCpus: cpus().length, totalMemoryBytes: totalmem() },
    design: { endpoint: 'POST /api/quotes', store: 'PGlite', model: 'open scheduled slots', warmup: 20,
      maxInFlight, maxSchedulerLagMs: maxLagMs, requestTimeoutMs: 2_000, stopSendingAfterFailures: 10,
      gate: 'illustrative: every stage zero drops, zero errors, p95 < 500 ms',
      limitations: 'client/server/PGlite same process; steps drain before next step; short educational workload, not production capacity' }, results };
  await mkdir('reports/load', { recursive: true });
  await writeFile('reports/load/sys08-load.json', JSON.stringify(report, null, 2) + '\n');
  console.log(JSON.stringify({ ...report, results: results.map(({ samples, droppedSlots, ...summary }) => summary) }, null, 2));
  if (results.some(r => !r.illustrativeGatePassed)) process.exitCode = 1;
} finally {
  await new Promise<void>((resolve, reject) => server.close(err => err ? reject(err) : resolve()));
  await system.db!.close();
}
