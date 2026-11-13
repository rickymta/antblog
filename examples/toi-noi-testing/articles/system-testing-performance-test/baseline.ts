import { mkdir, writeFile } from 'node:fs/promises';
import { cpus, platform, release, totalmem } from 'node:os';
import { performance } from 'node:perf_hooks';
import { once } from 'node:events';
import { createApp } from '../../src/api/app';
import { createMemorySystem } from '../../src/infra/compose';
import { FixedClock } from '../../src/infra/clock';
import { vnTime } from '../../src/domain/time';

const system = createMemorySystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)) });
const server = createApp(system.deps, { logErrors: false }).listen(0, '127.0.0.1');
await once(server, 'listening');
const address = server.address();
if (!address || typeof address === 'string') throw new Error('Không lấy được cổng lab');
const url = `http://127.0.0.1:${address.port}/api/quotes`;

async function sample() {
  const start = performance.now();
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
  return { ms: performance.now() - start, status, valid, error };
}

const rounds = [];
try {
  for (let round = 1; round <= 3; round++) {
    await system.reset();
    for (let i = 0; i < 50; i++) {
      if (!(await sample()).valid) throw new Error('Warm-up không đạt oracle; dừng đo');
    }
    await system.reset(); // warm-up không làm tăng kích thước dataset đo
    const cpuStart = process.cpuUsage();
    const rssStart = process.memoryUsage().rss;
    const start = performance.now();
    const samples = [];
    for (let i = 0; i < 300; i++) samples.push(await sample());
    const elapsedMs = performance.now() - start;
    const cpu = process.cpuUsage(cpuStart);
    const sorted = samples.map(s => s.ms).sort((a, b) => a - b);
    const percentile = (p: number) => sorted[Math.ceil(p * sorted.length) - 1]!;
    const failures = samples.filter(s => !s.valid).length;
    rounds.push({ round, requests: samples.length, failures, errorRate: failures / samples.length,
      elapsedMs, completedRps: samples.length / (elapsedMs / 1000),
      validRps: (samples.length - failures) / (elapsedMs / 1000),
      latencyMsAllAttempts: { p50: percentile(0.5), p95: percentile(0.95), p99: percentile(0.99), max: sorted.at(-1)! },
      combinedCpuMs: (cpu.user + cpu.system) / 1000,
      rssStartBytes: rssStart, rssEndBytes: process.memoryUsage().rss,
      illustrativeGatePassed: failures === 0 && percentile(0.95) < 1_000,
      samples,
    });
  }
  const report = {
    measuredAtUtc: new Date().toISOString(),
    environment: { node: process.version, os: platform(), release: release(), cpu: cpus()[0]?.model,
      logicalCpus: cpus().length, totalMemoryBytes: totalmem() },
    design: { endpoint: 'POST /api/quotes', store: 'memory', model: 'closed', concurrency: 1,
      warmupPerRound: 50, samplesPerRound: 300, rounds: 3, timeoutMs: 2_000,
      latencyBoundary: 'before fetch through body JSON parsing and oracle, includes client + loopback + server',
      gate: 'illustrative local only: zero errors and p95 < 1000 ms; not production SLO',
      limitation: 'client/server share process, CPU and event loop; RSS snapshots are not peak memory' },
    rounds,
  };
  await mkdir('reports/performance', { recursive: true });
  await writeFile('reports/performance/sys07-baseline.json', JSON.stringify(report, null, 2) + '\n');
  console.log(JSON.stringify({ ...report, rounds: rounds.map(({ samples, ...summary }) => summary) }, null, 2));
  if (rounds.some(r => !r.illustrativeGatePassed)) process.exitCode = 1;
} finally {
  await new Promise<void>((resolve, reject) => server.close(err => err ? reject(err) : resolve()));
}
