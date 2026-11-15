// P06 lab: partition, JSONB, materialized view, logical/physical replication, failover,
// split-brain + pg_rewind, and a 17 → 18 upgrade by dump/restore and by pg_upgrade.
// Every cluster is temporary, localhost-only and destroyed in finally.
//
// OUTPUT.txt: PASS/MEASURE lines are the gate; `--- 0xx label` blocks are real psql
// output or server/tool log excerpts quoted by the articles. Ports, PIDs, temp paths
// and timestamps are masked; LSNs and WAL file names are those of the recorded run.
//
// PG-057 (PostGIS, pgvector) runs separately on PGlite: see extensions/run.mjs.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { appendFileSync, existsSync, readFileSync, renameSync, statSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { performance } from 'node:perf_hooks';
import { createCluster, freePort } from '../pg/lib/cluster.mjs';
import { findPgBin, exe } from '../pg/lib/bin.mjs';
import { PsqlSession } from '../pg/lib/session.mjs';

const here = fileURLToPath(new URL('.', import.meta.url));
const pg = findPgBin();
assert(!pg.error, pg.error);
const oldDir = process.env.PG17_BIN || process.env.PG_OLD_BIN;
assert(oldDir && existsSync(exe(oldDir, 'pg_ctl')), 'Set PG17_BIN to PostgreSQL 17 binaries (folder with initdb, pg_ctl, postgres) for PG-059.');
const oldVersion = spawnSync(exe(oldDir, 'postgres'), ['--version'], { encoding: 'utf8' }).stdout.match(/(\d+)\.(\d+)/);
assert(oldVersion && oldVersion[1] === '17', 'PG17_BIN must point to PostgreSQL 17');
const pg17 = { dir: oldDir, version: oldVersion[0], major: 17, source: 'PG17_BIN' };

const clusters = [];
const sessions = [];
const output = [`P06 extension & integration lab · PostgreSQL ${pg.version} (+ ${pg17.version} for PG-059)`, `Node ${process.version} · ${process.platform}/${process.arch}`];
const masks = [];
const delay = ms => new Promise(r => setTimeout(r, ms));
const note = s => { output.push(s); console.log(s); };
function clean(s) {
  let t = String(s).replace(/\r\n/g, '\n');
  for (const [from, to] of masks) t = t.replaceAll(from, to);
  return t.split('\n')
    .filter(l => l.trim() && !/^\s*LOCATION:/.test(l))
    .map(l => l.replace(/^psql:<stdin>:\d+: /, '').replace(/port[= ]\d{4,5}/g, m => m.slice(0, 5) + '<PORT>')
      .replace(/[A-Z]:[\\/][^\s"']*toinoi-pg-[^\s"'\\/]+/g, '<TMP>').replace(/^\S+ \S+ \S+ \[\d+\] /, ''))
    .join('\n');
}
function evidence(label, text) {
  const lines = clean(text).split('\n');
  output.push(`--- ${label}`, ...lines.map(l => '  ' + l));
  console.log(`--- ${label}\n${lines.map(l => '  ' + l).join('\n')}`);
}
function check(label, actual, expected) { assert.equal(actual, expected, label); note(`PASS ${label}: ${actual}`); }
async function cluster(bin = pg, settings = {}, name = '') {
  const c = await createCluster(bin, { settings });
  c.name = name; clusters.push(c);
  return c;
}
function tool(c, name, args, { bin = pg, fail = false, cwd } = {}) {
  const r = spawnSync(exe(bin.dir, name), args, { env: c.env, encoding: 'utf8', timeout: 120000, windowsHide: true, cwd });
  if (r.error) throw r.error;
  if (fail) { assert.notEqual(r.status, 0, `${name} should fail`); return r.stdout + r.stderr; }
  assert.equal(r.status, 0, `${name}: ${r.stdout}\n${r.stderr}`);
  return (r.stdout + r.stderr).trim();
}
const addr = c => ['-h', '127.0.0.1', '-p', String(c.port), '-U', 'postgres'];
let lastStderr = '';
function sql(c, text, { fail = false, db = 'postgres' } = {}) {
  const r = spawnSync(exe(pg.dir, 'psql'), [...addr(c), '-d', db, '-XqAt', '-v', 'ON_ERROR_STOP=1', '-v', 'VERBOSITY=verbose'], { input: text, env: c.env, encoding: 'utf8', timeout: 30000, windowsHide: true });
  if (r.error) throw r.error;
  lastStderr = r.stderr;
  if (fail) { assert.notEqual(r.status, 0, 'Expected SQL error: ' + text); return r.stderr; }
  assert.equal(r.status, 0, r.stderr);
  return r.stdout.replace(/\r\n/g, '\n').trim();
}
const one = (c, text, o) => sql(c, text, o).split('\n').at(-1).trim();
function show(c, text, { db = 'postgres' } = {}) {
  const r = spawnSync(exe(pg.dir, 'psql'), [...addr(c), '-d', db, '-Xq', '-P', 'pager=off', '-P', 'null=NULL', '-v', 'ON_ERROR_STOP=1'], { input: text, env: c.env, encoding: 'utf8', timeout: 30000, windowsHide: true });
  assert.equal(r.status, 0, r.stderr);
  return r.stdout.replace(/\r\n/g, '\n').trimEnd();
}
function serverLog(c, pattern) {
  return existsSync(c.logFile) ? readFileSync(c.logFile, 'utf8').split(/\r?\n/).filter(l => pattern.test(l)).join('\n') : '';
}
async function until(fn, message, seconds = 20) {
  const end = Date.now() + seconds * 1000;
  while (Date.now() < end) { try { if (fn()) return; } catch {} await delay(100); }
  throw Error('Timed out: ' + message);
}
async function session(c, label) { const s = await new PsqlSession(c, 'postgres', label).open(); sessions.push(s); return s; }
const walFrom = (c, text) => { const l = one(c, 'SELECT pg_current_wal_insert_lsn()'); sql(c, text); return Number(one(c, `SELECT pg_wal_lsn_diff(pg_current_wal_insert_lsn(), '${l}')::bigint`)); };
const ledger = "SELECT count(*)||'|'||sum(fare) FROM fares";
// Re-run initdb in an existing cluster folder with extra flags, keeping the lab settings block.
function reinit(c, bin, extra) {
  c.stop();
  const conf = readFileSync(join(c.dataDir, 'postgresql.conf'), 'utf8');
  const block = conf.slice(conf.indexOf('# --- Cấu hình lab'));
  renameSync(c.dataDir, join(c.root, 'replaced-data'));
  tool(c, 'initdb', ['-D', c.dataDir, '-U', 'postgres', '--auth=trust', '-E', 'UTF8', '--no-sync', '--locale=C', `--lc-ctype=${c.ctype ?? 'C.UTF-8'}`, '--locale-provider=builtin', '--builtin-locale=C.UTF-8', ...extra], { bin });
  appendFileSync(join(c.dataDir, 'postgresql.conf'), '\n' + block);
}

let passed = false;
try {
  // wal_keep_size keeps the segments pg_rewind needs to walk back to the last common checkpoint.
  const primary = await cluster(pg, { wal_level: 'logical', autovacuum: 'on', wal_keep_size: '512MB' }, 'primary');
  masks.push([String(primary.port), '<PRIMARY>']);
  sql(primary, readFileSync(join(here, 'fixture.sql'), 'utf8'));
  note('Fixture: gps 1464 points (20 drivers, Oct–Nov 2026 UTC) partitioned by month + gps_plain copy; fares 2 rows / 150000 VND.');

  // ---------------- PG-051 · partition basics
  const filter = "driver_id = 1 AND seen_at >= '2026-11-01 UTC' AND seen_at < '2026-12-01 UTC'";
  check('051 rows per partition oct/nov', one(primary, "SELECT count(*) FILTER (WHERE tableoid = 'gps_oct'::regclass)||'/'||count(*) FILTER (WHERE tableoid = 'gps_nov'::regclass) FROM gps"), '744/720');
  check('051 boundary point 2026-11-01 00:00 UTC lands in', one(primary, "SELECT tableoid::regclass FROM gps WHERE seen_at = '2026-11-01 UTC'"), 'gps_nov');
  for (const t of ['gps', 'gps_plain']) {
    check(`051 driver 1 in November (${t})`, one(primary, `SELECT count(*) FROM ${t} WHERE ${filter}`), '36');
    const plan = show(primary, `EXPLAIN (ANALYZE, BUFFERS, COSTS OFF, TIMING OFF, SUMMARY OFF) SELECT * FROM ${t} WHERE ${filter};`);
    if (t === 'gps') { assert.match(plan, /gps_nov/); assert.doesNotMatch(plan, /gps_oct/); }
    evidence(`051 EXPLAIN driver+month on ${t}`, plan);
  }
  const driverOnly = show(primary, 'EXPLAIN (COSTS OFF) SELECT * FROM gps WHERE driver_id = 1;');
  assert.match(driverOnly, /gps_oct/); assert.match(driverOnly, /gps_nov/);
  evidence('051 EXPLAIN driver only (no partition key)', driverOnly);
  const noPartition = sql(primary, "INSERT INTO gps VALUES (1, '2026-12-01 UTC', '{\"accuracy_m\":5}');", { fail: true });
  assert.match(noPartition, /23514/);
  evidence('051 INSERT outside every partition', noPartition);
  const badUnique = sql(primary, 'CREATE TABLE gps_bad (driver_id int, seen_at timestamptz, UNIQUE (driver_id)) PARTITION BY RANGE (seen_at);', { fail: true });
  assert.match(badUnique, /0A000/);
  evidence('051 UNIQUE without partition key', badUnique);
  evidence('051 relation sizes', show(primary, "SELECT c.relname, pg_size_pretty(pg_total_relation_size(c.oid)) AS total FROM pg_class c WHERE c.relname IN ('gps_oct','gps_nov','gps_plain') ORDER BY 1;"));

  // ---------------- PG-053 · JSONB vs columns
  check('053 attrs @> accuracy 5', one(primary, "SELECT count(*) FROM gps WHERE attrs @> '{\"accuracy_m\":5}'"), '147');
  check('053 typo in key is silent', one(primary, "SELECT count(*) FROM gps WHERE (attrs->>'accurracy_m')::int = 5"), '0');
  check('053 ->> returns text, -> returns jsonb', one(primary, "SELECT pg_typeof(attrs->>'accuracy_m')||' / '||pg_typeof(attrs->'accuracy_m') FROM gps LIMIT 1"), 'text / jsonb');
  const notObject = sql(primary, "INSERT INTO gps VALUES (1, '2026-11-15 00:30 UTC', '[5]');", { fail: true });
  assert.match(notObject, /23514/);
  evidence('053 CHECK jsonb_typeof = object', notObject);
  const wrongType = sql(primary, "SELECT count(*) FROM gps WHERE (attrs->>'source')::int = 1;", { fail: true });
  assert.match(wrongType, /22P02/);
  evidence('053 cast of a text value at query time', wrongType);
  sql(primary, 'CREATE INDEX gps_attrs_gin ON gps USING gin (attrs jsonb_path_ops);');
  evidence('053 EXPLAIN @> with GIN, default planner choice', show(primary, `EXPLAIN (COSTS OFF) SELECT count(*) FROM gps WHERE attrs @> '{"accuracy_m":5}';`));
  const forced = show(primary, `SET enable_seqscan = off; EXPLAIN (COSTS OFF) SELECT count(*) FROM gps WHERE attrs @> '{"accuracy_m":5}';`);
  assert.match(forced, /Bitmap Index Scan/);
  evidence('053 EXPLAIN @> with GIN, enable_seqscan=off (demonstration only)', forced);
  sql(primary, "ALTER TABLE gps ADD COLUMN accuracy_m int GENERATED ALWAYS AS ((attrs->>'accuracy_m')::int) STORED;");
  check('053 generated column agrees with JSONB', one(primary, 'SELECT count(*) FROM gps WHERE accuracy_m = 5'), '147');
  evidence('053 index sizes', show(primary, "SELECT indexrelid::regclass AS index, pg_size_pretty(pg_relation_size(indexrelid)) AS size FROM pg_index WHERE indrelid IN ('gps_oct'::regclass, 'gps_nov'::regclass) ORDER BY 1;"));

  // ---------------- PG-054 · pruning and retention
  const generic = show(primary, "SET plan_cache_mode = force_generic_plan; PREPARE by_window(timestamptz, timestamptz) AS SELECT count(*) FROM gps WHERE seen_at >= $1 AND seen_at < $2; EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF) EXECUTE by_window('2026-11-05 UTC', '2026-11-06 UTC');");
  assert.match(generic, /Subplans Removed: 1/);
  evidence('054 generic plan, run-time pruning', generic);
  const fnFilter = show(primary, "EXPLAIN (COSTS OFF) SELECT count(*) FROM gps WHERE date_trunc('month', seen_at AT TIME ZONE 'UTC') = '2026-11-01';");
  assert.match(fnFilter, /gps_oct/);
  evidence('054 EXPLAIN function on partition key (no pruning)', fnFilter);
  sql(primary, "CREATE TABLE gps_dec PARTITION OF gps FOR VALUES FROM ('2026-12-01 UTC') TO ('2027-01-01 UTC');");
  sql(primary, "INSERT INTO gps VALUES (1, '2026-12-01 UTC', '{\"source\":\"phone\",\"accuracy_m\":5}');");
  check('054 December partition prepared ahead receives', one(primary, 'SELECT count(*) FROM gps_dec'), '1');
  const deleteWal = walFrom(primary, "DELETE FROM gps_plain WHERE seen_at < '2026-11-01 UTC';");
  const detachWal = walFrom(primary, 'ALTER TABLE gps DETACH PARTITION gps_oct CONCURRENTLY;');
  check('054 after DETACH: gps / detached gps_oct', one(primary, "SELECT (SELECT count(*) FROM gps)||'/'||(SELECT count(*) FROM gps_oct)"), '721/744');
  check('054 detached table is a normal table', one(primary, "SELECT relispartition FROM pg_class WHERE oid = 'gps_oct'::regclass"), 'f');
  const dropWal = walFrom(primary, 'DROP TABLE gps_oct;');
  note(`MEASURE 054 WAL bytes · DELETE 744 rows from gps_plain: ${deleteWal} · DETACH CONCURRENTLY: ${detachWal} · DROP detached partition: ${dropWal}`);
  check('054 gps_plain after DELETE', one(primary, 'SELECT count(*) FROM gps_plain'), '720');

  // ---------------- PG-056 · materialized view
  sql(primary, "INSERT INTO fares VALUES (3, 'HCM', 50000);");
  check('056 report before REFRESH (HCM)', one(primary, "SELECT trips||'/'||total FROM fare_report WHERE city = 'HCM'"), '1/85000');
  check('056 base table HCM', one(primary, "SELECT count(*)||'/'||sum(fare) FROM fares WHERE city = 'HCM'"), '2/135000');
  const concurrentNoIndex = sql(primary, 'REFRESH MATERIALIZED VIEW CONCURRENTLY fare_report;', { fail: true });
  assert.match(concurrentNoIndex, /55000/);
  evidence('056 CONCURRENTLY without unique index', concurrentNoIndex);
  const refreshedAt0 = one(primary, 'SELECT max(refreshed_at) FROM fare_report');
  {
    const w = await session(primary, 'refresh'), r = await session(primary, 'reader');
    await w.run('BEGIN; REFRESH MATERIALIZED VIEW fare_report;');
    const read = r.run('SELECT count(*) FROM fare_report;');
    await until(() => one(primary, `SELECT wait_event_type FROM pg_stat_activity WHERE pid = ${r.pid}`) === 'Lock', 'reader not blocked by plain REFRESH');
    note('PASS 056 plain REFRESH blocks readers: reader wait_event_type = Lock');
    const who = `CASE l.pid WHEN ${w.pid} THEN 'refresh' ELSE 'reader' END`;
    evidence('056 locks during plain REFRESH', show(primary, `SELECT ${who} AS s, l.mode, l.granted FROM pg_locks l WHERE l.relation = 'fare_report'::regclass AND l.pid IN (${w.pid}, ${r.pid}) ORDER BY 1 DESC, 2;`));
    await w.run('COMMIT;'); assert(!(await read).error);
    sql(primary, 'CREATE UNIQUE INDEX fare_report_city ON fare_report (city);');
    sql(primary, "INSERT INTO fares VALUES (99, 'DN', 1000);");
    await w.run('BEGIN; REFRESH MATERIALIZED VIEW CONCURRENTLY fare_report;');
    const t0 = performance.now();
    const read2 = await r.run('SELECT count(*) FROM fare_report;');
    assert(!read2.error);
    note(`PASS 056 CONCURRENTLY: reader answered while refresh transaction open (${(performance.now() - t0) < 2000 ? 'no wait' : 'waited'})`);
    evidence('056 locks during REFRESH CONCURRENTLY', show(primary, `SELECT 'refresh' AS s, l.mode, l.granted FROM pg_locks l WHERE l.relation = 'fare_report'::regclass AND l.pid = ${w.pid} ORDER BY 2;`));
    await w.run('COMMIT;');
    await w.close(); await r.close();
  }
  sql(primary, 'DELETE FROM fares WHERE id = 99; REFRESH MATERIALIZED VIEW CONCURRENTLY fare_report;');
  check('056 report after REFRESH (HCM)', one(primary, "SELECT trips||'/'||total FROM fare_report WHERE city = 'HCM'"), '2/135000');
  check('056 refreshed_at advanced', one(primary, `SELECT max(refreshed_at) > '${refreshedAt0}' FROM fare_report`), 't');
  check('056 reconcile report vs base', one(primary, 'SELECT (SELECT sum(total) FROM fare_report) = (SELECT sum(fare) FROM fares)'), 't');
  evidence('056 fare_report', show(primary, 'SELECT city, trips, total FROM fare_report ORDER BY city;'));

  // ---------------- PG-055 · logical replication / CDC
  const subscriber = await cluster(pg, {}, 'subscriber');
  masks.push([String(subscriber.port), '<SUBSCRIBER>']);
  sql(primary, 'CREATE PUBLICATION fare_pub FOR TABLE fares;');
  sql(subscriber, 'CREATE TABLE fares (id int PRIMARY KEY, city text NOT NULL, fare int NOT NULL CHECK (fare >= 0));');
  sql(subscriber, `CREATE SUBSCRIPTION fare_sub CONNECTION 'host=127.0.0.1 port=${primary.port} dbname=postgres user=postgres' PUBLICATION fare_pub;`);
  await until(() => one(subscriber, "SELECT string_agg(srsubstate::text, '') FROM pg_subscription_rel") === 'r', 'initial copy not ready');
  check('055 initial copy', one(subscriber, ledger), one(primary, ledger));
  evidence('055 slot on publisher', show(primary, 'SELECT slot_name, plugin, slot_type, active, wal_status FROM pg_replication_slots;'));
  evidence('055 walsender for subscription', show(primary, "SELECT application_name, state FROM pg_stat_replication WHERE application_name = 'fare_sub';"));
  sql(subscriber, 'ALTER SUBSCRIPTION fare_sub DISABLE;');
  await until(() => one(primary, "SELECT active FROM pg_replication_slots WHERE slot_name = 'fare_sub'") === 'f', 'slot still active');
  sql(primary, 'UPDATE fares SET fare = 55000 WHERE id = 3;');
  sql(primary, 'SELECT pg_switch_wal(); CHECKPOINT;');
  check('055 disabled subscriber still sees', one(subscriber, 'SELECT fare FROM fares WHERE id = 3'), '50000');
  const retained = Number(one(primary, "SELECT pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn)::bigint FROM pg_replication_slots WHERE slot_name = 'fare_sub'"));
  assert(retained > 0);
  note(`MEASURE 055 WAL bytes held back by inactive slot fare_sub: ${retained}`);
  evidence('055 inactive slot', show(primary, "SELECT slot_name, active, wal_status, pg_size_pretty(pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn)) AS retained FROM pg_replication_slots;"));
  sql(subscriber, 'ALTER SUBSCRIPTION fare_sub ENABLE;');
  await until(() => one(subscriber, 'SELECT fare FROM fares WHERE id = 3') === '55000', 'subscriber did not catch up');
  note('PASS 055 subscriber caught up after ENABLE: 55000');
  // DDL is not replicated: a new column on the publisher stops apply until the subscriber has it.
  sql(primary, 'ALTER TABLE fares ADD COLUMN tip int NOT NULL DEFAULT 0;');
  sql(primary, 'UPDATE fares SET tip = 5000 WHERE id = 1;');
  await until(() => /missing replicated column/.test(readFileSync(subscriber.logFile, 'utf8')), 'apply error not logged');
  check('055 subscriber unchanged while apply fails', one(subscriber, "SELECT count(*) FROM information_schema.columns WHERE table_name = 'fares' AND column_name = 'tip'"), '0');
  evidence('055 subscriber log: DDL drift', serverLog(subscriber, /missing replicated column/).split('\n').slice(0, 1).join('\n'));
  sql(subscriber, 'ALTER TABLE fares ADD COLUMN tip int NOT NULL DEFAULT 0;');
  await until(() => one(subscriber, 'SELECT tip FROM fares WHERE id = 1') === '5000', 'apply did not resume after DDL');
  note('PASS 055 apply resumed after matching DDL on subscriber: tip 5000');
  check('055 apply errors counted', one(subscriber, "SELECT apply_error_count > 0 FROM pg_stat_subscription_stats WHERE subname = 'fare_sub'"), 't');
  sql(subscriber, 'DROP SUBSCRIPTION fare_sub;');
  check('055 DROP SUBSCRIPTION removed the slot', one(primary, 'SELECT count(*) FROM pg_replication_slots'), '0');

  // ---------------- PG-052 · primary / standby
  const standby = await cluster(pg, {}, 'standby');
  masks.push([String(standby.port), '<STANDBY>']);
  standby.stop();
  renameSync(standby.dataDir, join(standby.root, 'initial-data'));
  tool(primary, 'pg_basebackup', [...addr(primary), '-D', standby.dataDir, '-R', '-X', 'stream', '--checkpoint=fast']);
  appendFileSync(join(standby.dataDir, 'postgresql.conf'), `\nport = ${standby.port}\n`);
  check('052 standby.signal written by -R', String(existsSync(join(standby.dataDir, 'standby.signal'))), 'true');
  evidence('052 postgresql.auto.conf from pg_basebackup -R', readFileSync(join(standby.dataDir, 'postgresql.auto.conf'), 'utf8').split(/\r?\n/).filter(l => /primary_conninfo/.test(l)).map(l => l.replace(/passfile=\S+ /, '').replace(/ (sslmode|sslnegotiation|sslcompression|sslcertmode|sslsni|ssl_min_protocol_version|gssencmode|krbsrvname|gssdelegation|target_session_attrs|load_balance_hosts|channel_binding|requiressl)=\S+/g, '')).join('\n'));
  standby.start();
  await until(() => one(primary, 'SELECT count(*) FROM pg_stat_replication WHERE state = \'streaming\' AND application_name <> \'fare_sub\'') === '1', 'standby not streaming');
  check('052 standby in recovery', one(standby, 'SELECT pg_is_in_recovery()'), 't');
  evidence('052 pg_stat_replication on primary', show(primary, 'SELECT application_name, state, sync_state, sent_lsn = replay_lsn AS caught_up FROM pg_stat_replication;'));
  const roWrite = sql(standby, "INSERT INTO fares VALUES (9, 'HN', 1);", { fail: true });
  assert.match(roWrite, /25006/);
  evidence('052 write on standby', roWrite);
  sql(standby, 'SELECT pg_wal_replay_pause();');
  await until(() => one(standby, 'SELECT pg_get_wal_replay_pause_state()') === 'paused', 'replay not paused');
  sql(primary, "INSERT INTO fares VALUES (4, 'HN', 30000);");
  const commitLsn = one(primary, 'SELECT pg_current_wal_flush_lsn()');
  await until(() => one(standby, `SELECT pg_last_wal_receive_lsn() >= '${commitLsn}'`) === 't', 'standby did not receive WAL');
  check('052 standby received WAL but row not visible', one(standby, 'SELECT count(*) FROM fares WHERE id = 4'), '0');
  note(`MEASURE 052 standby received-but-not-replayed bytes: ${one(standby, 'SELECT pg_wal_lsn_diff(pg_last_wal_receive_lsn(), pg_last_wal_replay_lsn())::bigint')}`);
  evidence('052 pg_stat_replication while replay paused', show(primary, "SELECT application_name, flush_lsn >= '" + commitLsn + "' AS flushed_on_standby, replay_lsn >= '" + commitLsn + "' AS replayed FROM pg_stat_replication WHERE application_name <> 'fare_sub';"));
  sql(standby, 'SELECT pg_wal_replay_resume();');
  await until(() => one(standby, `SELECT pg_last_wal_replay_lsn() >= '${commitLsn}'`) === 't', 'replay did not reach commit LSN');
  check('052 read-your-writes after waiting for commit LSN', one(standby, 'SELECT count(*) FROM fares WHERE id = 4'), '1');

  // ---------------- PG-058 · failover, data loss, split-brain, pg_rewind
  sql(standby, "ALTER SYSTEM SET primary_conninfo = ''; SELECT pg_reload_conf();");
  await until(() => one(standby, 'SELECT count(*) FROM pg_stat_wal_receiver') === '0', 'walreceiver still running');
  note('PASS 058 standby cut off from primary (network partition simulated)');
  sql(primary, "INSERT INTO fares VALUES (5, 'HN', 20000);");
  check('058 old primary accepted write while standby cut off', one(primary, 'SELECT count(*) FROM fares WHERE id = 5'), '1');
  const primaryFlush = one(primary, 'SELECT pg_current_wal_flush_lsn()');
  primary.stop('immediate');
  const standbyReceived = one(standby, 'SELECT pg_last_wal_receive_lsn()');
  note(`MEASURE 058 WAL not received by standby at failover: ${one(standby, `SELECT pg_wal_lsn_diff('${primaryFlush}', '${standbyReceived}')::bigint`)} bytes (primary flush ${primaryFlush}, standby received ${standbyReceived})`);
  check('058 pg_promote', one(standby, 'SELECT pg_promote(true, 30)'), 't');
  check('058 committed row lost by async failover', one(standby, 'SELECT count(*) FROM fares WHERE id = 5'), '0');
  check('058 new primary timeline', one(standby, 'SELECT substr(pg_walfile_name(pg_current_wal_lsn()), 1, 8)'), '00000002');
  const multi = (first, second) => spawnSync(exe(pg.dir, 'psql'), ['-XAt', '-d', `host=127.0.0.1,127.0.0.1 port=${first.port},${second.port} dbname=postgres user=postgres target_session_attrs=read-write connect_timeout=3`, '-c', 'SELECT inet_server_port()'], { env: standby.env, encoding: 'utf8', timeout: 20000, windowsHide: true }).stdout.trim();
  check('058 target_session_attrs=read-write picks', multi(primary, standby) === String(standby.port) ? 'standby (new primary)' : 'other', 'standby (new primary)');
  sql(standby, "INSERT INTO fares VALUES (6, 'HCM', 15000);");
  // Split-brain: the old primary comes back without fencing and is writable on timeline 1.
  primary.start();
  check('058 old primary restarted, not in recovery', one(primary, 'SELECT pg_is_in_recovery()'), 'f');
  sql(primary, "INSERT INTO fares VALUES (7, 'HN', 99000);");
  evidence('058 split-brain: two writable primaries', [
    'old primary (timeline ' + one(primary, 'SELECT substr(pg_walfile_name(pg_current_wal_lsn()), 1, 8)') + '): ' + sql(primary, "SELECT string_agg(id::text, ',' ORDER BY id) FROM fares"),
    'new primary (timeline ' + one(standby, 'SELECT substr(pg_walfile_name(pg_current_wal_lsn()), 1, 8)') + '): ' + sql(standby, "SELECT string_agg(id::text, ',' ORDER BY id) FROM fares"),
  ].join('\n'));
  check('058 same connection string now picks', multi(primary, standby) === String(primary.port) ? 'old primary' : 'other', 'old primary');
  // Fence the old primary, rewind it onto the new timeline and rejoin as standby.
  primary.stop('fast');
  const rewind = tool(primary, 'pg_rewind', ['--target-pgdata', primary.dataDir, '--source-server', `host=127.0.0.1 port=${standby.port} dbname=postgres user=postgres`, '--write-recovery-conf', '--progress']);
  evidence('058 pg_rewind', rewind.split(/\r?\n/).filter(l => /diverged|rewinding|Done|last common/.test(l)).join('\n'));
  // pg_rewind copies configuration files from the source too: restore this node's own port.
  appendFileSync(join(primary.dataDir, 'postgresql.conf'), `\nport = ${primary.port}\n`);
  primary.start();
  await until(() => one(primary, 'SELECT count(*) FROM fares WHERE id = 6') === '1', 'rewound node did not follow new primary', 30);
  check('058 rewound old primary is standby', one(primary, 'SELECT pg_is_in_recovery()'), 't');
  check('058 rows 5 and 7 discarded by rewind', one(primary, 'SELECT count(*) FROM fares WHERE id IN (5, 7)'), '0');
  check('058 ledger on new primary', one(standby, ledger), '5|250000');

  // ---------------- PG-060 · capstone reconciliation after failover
  check('060 report projection stale after failover', one(standby, "SELECT (SELECT sum(total) FROM fare_report)||' vs '||(SELECT sum(fare) FROM fares)"), '200000 vs 250000');
  sql(standby, 'REFRESH MATERIALIZED VIEW CONCURRENTLY fare_report;');
  check('060 report reconciled after REFRESH', one(standby, 'SELECT (SELECT sum(total) FROM fare_report) = (SELECT sum(fare) FROM fares)'), 't');
  evidence('060 final ledger on new primary', show(standby, 'SELECT id, city, fare, tip FROM fares ORDER BY id;'));
  evidence('060 final report', show(standby, 'SELECT city, trips, total FROM fare_report ORDER BY city;'));
  check('060 replica follows new primary', one(primary, ledger), '5|250000');

  // ---------------- PG-059 · upgrade 17 → 18
  const old = await cluster(pg17, { effective_io_concurrency: 0, shared_preload_libraries: "''" }, 'pg17');
  masks.push([String(old.port), '<PG17>']);
  check('059 source version', one(old, 'SHOW server_version'), pg17.version);
  sql(old, "CREATE TABLE tickets (id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY, fare int NOT NULL, meta jsonb NOT NULL DEFAULT '{}'); INSERT INTO tickets (fare) VALUES (85000), (65000); ANALYZE tickets;");
  const ticketDigest = "SELECT count(*)||'/'||sum(fare)||'/'||md5(string_agg(id||':'||fare, ',' ORDER BY id)) FROM tickets";
  const sourceDigest = one(old, ticketDigest);
  // Path A: dump with the NEWER pg_dump, restore into an 18 cluster.
  const dump = join(old.root, 'tickets.dump');
  tool(old, 'pg_dump', [...addr(old), '-Fc', '-f', dump, 'postgres']);
  note(`MEASURE 059 dump size bytes: ${statSync(dump).size}`);
  const target = await cluster(pg, {}, 'target18');
  tool(target, 'pg_restore', [...addr(target), '-d', 'postgres', '--exit-on-error', '--single-transaction', dump]);
  check('059 A dump/restore digest', one(target, ticketDigest), sourceDigest);
  check('059 A identity continues after restore', one(target, 'INSERT INTO tickets (fare) VALUES (50000) RETURNING id'), '3');
  check('059 A source untouched (rollback target lacks new write)', one(old, 'SELECT count(*) FROM tickets'), '2');
  check('059 A statistics after restore (before ANALYZE)', one(target, "SELECT count(*) FROM pg_stats WHERE tablename = 'tickets'"), '0');
  // Path B: pg_upgrade. PG18 initdb enables data checksums by default; the 17 cluster has none.
  old.stop();
  const fresh = await cluster(pg, {}, 'upgrade18');
  fresh.stop();
  const checkOut = tool(fresh, 'pg_upgrade', ['-b', pg17.dir, '-B', pg.dir, '-d', old.dataDir, '-D', fresh.dataDir, '-U', 'postgres', '-p', String(old.port), '-P', String(fresh.port), '--check'], { fail: true, cwd: fresh.root });
  assert.match(checkOut, /checksum/i);
  evidence('059 B pg_upgrade --check against default 18 initdb', checkOut.split(/\r?\n/).filter(l => /checksum|Checking|Failure|fatal/i.test(l)).slice(-4).join('\n'));
  reinit(fresh, pg, ['--no-data-checksums']);
  const upgradeOut = tool(fresh, 'pg_upgrade', ['-b', pg17.dir, '-B', pg.dir, '-d', old.dataDir, '-D', fresh.dataDir, '-U', 'postgres', '-p', String(old.port), '-P', String(fresh.port)], { cwd: fresh.root });
  evidence('059 B pg_upgrade (copy mode)', upgradeOut.split(/\r?\n/).filter(l => /Upgrade Complete|statistics|Copying user relation|Performing Upgrade/i.test(l)).slice(0, 6).join('\n'));
  fresh.start();
  check('059 B upgraded version', one(fresh, 'SHOW server_version_num').slice(0, 2), '18');
  check('059 B digest after pg_upgrade', one(fresh, ticketDigest), sourceDigest);
  check('059 B planner statistics carried over', String(Number(one(fresh, "SELECT count(*) FROM pg_stats WHERE tablename = 'tickets'")) > 0), 'true');
  check('059 B identity continues', one(fresh, 'INSERT INTO tickets (fare) VALUES (50000) RETURNING id'), '3');
  old.start();
  check('059 B copy mode: old 17 cluster still starts with', one(old, "SELECT count(*)||' rows on '||current_setting('server_version') FROM tickets"), `2 rows on ${pg17.version}`);

  passed = true;
  note('PASS P06: all assertions passed; temporary clusters cleaned up.');
} finally {
  for (const s of sessions) { try { await s.close(); } catch {} }
  for (const c of clusters.reverse()) { try { c.destroySync(); } catch {} }
  if (passed) writeFileSync(join(here, 'OUTPUT.txt'), output.join('\n') + '\n');
}
