// Dedicated operations lab: real TLS, dump/restore and archived-WAL recovery.
// Never attaches to an existing server. Temporary clusters are disposed in finally.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync, appendFileSync, mkdirSync, renameSync, cpSync, existsSync, chmodSync } from 'node:fs';
import { dirname, join, resolve, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { randomBytes } from 'node:crypto';
import { performance } from 'node:perf_hooks';
import { findPgBin, exe } from '../pg/lib/bin.mjs';
import { createCluster } from '../pg/lib/cluster.mjs';
import { PsqlSession } from '../pg/lib/session.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const pg = findPgBin();
assert(!pg.error, pg.error);
const openssl = process.env.OPENSSL_BIN || (process.platform === 'win32' ? 'C:/Program Files/Git/usr/bin/openssl.exe' : 'openssl');
const output = [`P05 operations lab · PostgreSQL ${pg.version}`, `Node ${process.version} · ${process.platform}/${process.arch}`, 'Fixture: 100 trips, 2 tenants, total fare 3000000 VND.'];
const note = (s) => { output.push(s); console.log(s); };
const delay = (ms) => new Promise(r => setTimeout(r, ms));
const slash = p => p.replaceAll('\\', '/');
const conf = s => "'" + s.replaceAll("'", "''") + "'";
let source, recovery;
const sessions = [];
let passed = false;
function tool(c, name, args, extra = {}) {
  const r = spawnSync(exe(pg.dir, name), args, { env: c.env, encoding: 'utf8', windowsHide: true, timeout: 60000, ...extra });
  if (r.error || r.status !== 0) throw Error(`${name}: ${r.error?.message || r.stderr}`);
  return r.stdout.trim();
}
function sql(c, text, { db = 'postgres', user = 'postgres', env = {}, fail = false } = {}) {
  const r = spawnSync(exe(pg.dir, 'psql'), ['-X', '-h', '127.0.0.1', '-p', String(c.port), '-U', user, '-d', db, '-At', '-v', 'ON_ERROR_STOP=1', '-v', 'VERBOSITY=verbose'], { input: text, env: { ...c.env, ...env }, encoding: 'utf8', windowsHide: true, timeout: 15000 });
  if (r.error) throw r.error;
  if (fail) { assert.notEqual(r.status, 0, 'Expected SQL rejection'); return r.stderr; }
  assert.equal(r.status, 0, r.stderr);
  return r.stdout.trim();
}
function scalar(text, options) { return sql(source, text, options).split('\n').at(-1).trim(); }
function check(label, actual, expected) { assert.equal(actual, expected, label); note(`PASS ${label}: ${actual}`); }
async function session(label) {
  const s = await new PsqlSession(source, 'postgres', label).open(); sessions.push(s); return s;
}
async function step(s, text) { const r = await s.run(text); assert(!r.error, r.stderr); return r; }
async function until(fn, message, seconds = 15) {
  const end = Date.now() + seconds * 1000;
  while (Date.now() < end) { if (fn()) return; await delay(100); }
  throw Error(message);
}
try {
  source = await createCluster(pg, { settings: { autovacuum: 'on', archive_mode: 'on', archive_command: '', wal_level: 'replica', log_checkpoints: 'on' } });
  sql(source, readFileSync(join(here, '00-schema.sql'), 'utf8'));
  check('041 report reads', scalar('SET ROLE tn_report; SELECT count(*) FROM ops.trips;'), '100');
  assert.match(sql(source, 'SET ROLE tn_report; DELETE FROM ops.trips;', { fail: true }), /42501/);
  assert.match(sql(source, 'SET ROLE tn_app; ALTER TABLE ops.trips ADD COLUMN bad int;', { fail: true }), /42501/);
  check('041 application is not owner', scalar("SELECT tableowner FROM pg_tables WHERE schemaname='ops' AND tablename='trips'"), 'tn_owner');
  sql(source, 'SET ROLE tn_owner; CREATE TABLE ops.next_report(id int); RESET ROLE;');
  check('041 future table default privileges', scalar("SELECT has_table_privilege('tn_report','ops.next_report','SELECT')"), 't');

  // PG-042: self-signed lab certificate used as explicit trust anchor; no key in git.
  const cert = join(source.dataDir, 'server.crt'), key = join(source.dataDir, 'server.key');
  const ssl = spawnSync(openssl, ['req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-keyout', key, '-out', cert, '-days', '2', '-subj', '/CN=localhost', '-addext', 'subjectAltName=DNS:localhost'], { encoding: 'utf8', windowsHide: true, timeout: 30000, env: { ...process.env, MSYS_NO_PATHCONV: '1' } });
  assert.equal(ssl.status, 0, ssl.stderr); chmodSync(key, 0o600);
  const password = randomBytes(24).toString('hex');
  sql(source, `SET password_encryption='scram-sha-256'; ALTER ROLE tn_app PASSWORD '${password}';`);
  writeFileSync(join(source.dataDir, 'pg_hba.conf'), 'local all postgres trust\nhost all postgres 127.0.0.1/32 trust\nhost all postgres ::1/128 trust\nhostssl all tn_app 127.0.0.1/32 scram-sha-256\nhost all all 127.0.0.1/32 reject\nhost all all ::1/128 reject\nhost replication postgres 127.0.0.1/32 trust\n');
  const archive = join(source.root, 'archive'); mkdirSync(archive);
  const copier = slash(join(here, 'wal-copy.mjs'));
  const archiveCommand = `"${slash(process.execPath)}" "${copier}" archive "%p" "${slash(archive)}/%f"`;
  appendFileSync(join(source.dataDir, 'postgresql.conf'), `\nssl=on\narchive_command=${conf(archiveCommand)}\n`);
  source.restart();
  // host=localhost validates SAN; hostaddr avoids DNS/network ambiguity.
  const app = (mode, pass = password, host = 'localhost') => spawnSync(exe(pg.dir, 'psql'), ['-X', '-At', '-d', `host=${host} hostaddr=127.0.0.1 port=${source.port} user=tn_app dbname=postgres sslmode=${mode} sslrootcert='${slash(cert)}' connect_timeout=3`, '-c', 'SELECT ssl FROM pg_stat_ssl WHERE pid=pg_backend_pid()'], { env: { ...source.env, PGPASSWORD: pass }, encoding: 'utf8', timeout: 10000, windowsHide: true });
  check('042 TLS verify-full', app('verify-full').stdout.trim(), 't');
  for (const [label, r, pattern] of [['plaintext', app('disable'), /rejects connection/], ['wrong password', app('verify-full', 'wrong'), /password authentication failed/], ['wrong hostname', app('verify-full', password, 'wrong.invalid'), /does not match host name/]]) {
    assert.notEqual(r.status, 0); assert.match(r.stderr, pattern); note(`PASS 042 rejects ${label}`);
  }

  // PG-043: actual blocked backend, identified through pg_blocking_pids.
  const a = await session('p05-blocker'), b = await session('p05-waiter');
  await step(a, 'BEGIN; UPDATE ops.trips SET fare=fare WHERE id=1;');
  const blocked = b.run('UPDATE ops.trips SET fare=fare WHERE id=1;');
  await until(() => scalar(`SELECT ${a.pid}=ANY(pg_blocking_pids(${b.pid}))`) === 't', 'waiter never blocked');
  check('043 identifies lock wait', scalar(`SELECT wait_event_type FROM pg_stat_activity WHERE pid=${b.pid}`), 'Lock');
  await step(a, 'ROLLBACK;'); assert(!(await blocked).error);
  await a.close(); await b.close();

  // PG-044: deterministic pgstattuple inspection; per-table autovac off only for experiment.
  sql(source, 'CREATE EXTENSION pgstattuple; CREATE TABLE ops.churn(id int, payload text) WITH (autovacuum_enabled=false); INSERT INTO ops.churn SELECT n, repeat(md5(n::text),10) FROM generate_series(1,5000)n;');
  sql(source, 'DELETE FROM ops.churn WHERE id % 2=0;');
  const before = Number(scalar("SELECT dead_tuple_count FROM pgstattuple('ops.churn')")); assert(before > 0);
  sql(source, 'VACUUM (ANALYZE) ops.churn;');
  check('044 dead tuples after vacuum', scalar("SELECT dead_tuple_count FROM pgstattuple('ops.churn')"), '0');
  sql(source, 'ALTER TABLE ops.churn RESET (autovacuum_enabled);');
  note(`PASS 044 dead tuples before vacuum: ${before}; cluster autovacuum remains on`);

  // PG-045: local warm-cache micro experiment, not a production benchmark.
  check('045 durability settings', scalar("SELECT current_setting('fsync')||'/'||current_setting('synchronous_commit')||'/'||current_setting('full_page_writes')"), 'on/on/on');
  const timings = [];
  for (let i=0; i<3; i++) {
    const lsn = scalar('SELECT pg_current_wal_insert_lsn();');
    const start = performance.now(); sql(source, 'UPDATE ops.trips SET fare=fare+1;');
    const updateMs = performance.now()-start;
    const bytes = scalar(`SELECT pg_wal_lsn_diff(pg_current_wal_insert_lsn(),'${lsn}')::bigint`);
    const cp = performance.now(); sql(source, 'CHECKPOINT;');
    timings.push({ updateMs: +updateMs.toFixed(1), checkpointMs: +(performance.now()-cp).toFixed(1), walBytes: Number(bytes) });
  }
  note('MEASURE 045 warm local 100-row updates, includes psql startup: '+JSON.stringify(timings));
  sql(source, 'UPDATE ops.trips SET fare=30000;');

  // PG-046: custom-format dump including ownership and ACL; roles already exist in same cluster.
  const dump = join(source.root, 'ops.dump');
  tool(source, 'pg_dump', ['-h','127.0.0.1','-p',String(source.port),'-U','postgres','-d','postgres','-Fc','-n','ops','-f',dump]);
  sql(source, 'CREATE DATABASE p05_restored;');
  tool(source, 'pg_restore', ['-h','127.0.0.1','-p',String(source.port),'-U','postgres','-d','p05_restored','--exit-on-error',dump]);
  const digest = "SELECT count(*)||'/'||sum(fare)||'/'||md5(string_agg(id||':'||tenant_id||':'||fare||':'||status,',' ORDER BY id)) FROM ops.trips";
  const expected = scalar(digest);
  check('046 restored data digest', scalar(digest, {db:'p05_restored'}), expected);
  check('046 restored owner', scalar("SELECT tableowner FROM pg_tables WHERE schemaname='ops' AND tablename='trips'",{db:'p05_restored'}), 'tn_owner');
  check('046 restored report ACL', scalar("SELECT has_table_privilege('tn_report','ops.trips','SELECT')",{db:'p05_restored'}), 't');
  assert.match(sql(source, 'SET ROLE tn_report; UPDATE ops.trips SET fare=0;', {db:'p05_restored',fail:true}), /42501/);

  // PG-047: bounded two-session application queue; measure checkout wait separately.
  const workers = await Promise.all([session('pool-1'),session('pool-2')]);
  const waits=[]; let next=0; const enqueued=performance.now();
  await Promise.all(workers.map(async s => { while(next<6) { const job=next++; waits[job]=+(performance.now()-enqueued).toFixed(1); await step(s,'SELECT pg_sleep(0.05);'); } }));
  note('MEASURE 047 6 jobs / 2 existing connections / 50ms service; queue waits ms: '+JSON.stringify(waits));
  const timeout = await workers[0].run("SET statement_timeout='20ms'; SELECT pg_sleep(0.2);");
  check('047 statement timeout SQLSTATE', timeout.sqlstate, '57014');
  for (const s of workers) await s.close();

  // PG-049 first: include RLS in the physical backup and prove it survives recovery.
  sql(source, `ALTER TABLE ops.trips ENABLE ROW LEVEL SECURITY;
    CREATE POLICY tenant_scope ON ops.trips TO tn_app
    USING (tenant_id = NULLIF(current_setting('app.tenant_id',true),'')::int)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id',true),'')::int);`);
  check('049 no tenant denies rows', scalar('SET ROLE tn_app; SELECT count(*) FROM ops.trips;'), '0');
  check('049 tenant 1', scalar("SET ROLE tn_app; SET app.tenant_id='1'; SELECT count(*) FROM ops.trips;"), '50');
  assert.match(sql(source, "SET ROLE tn_app; SET app.tenant_id='1'; INSERT INTO ops.trips VALUES(101,2,30000,'completed');", {fail:true}), /42501/);
  check('049 owner bypass', scalar('SET ROLE tn_owner; SELECT count(*) FROM ops.trips;'), '100');
  sql(source, 'ALTER TABLE ops.trips FORCE ROW LEVEL SECURITY;');
  check('049 forced owner without policy', scalar('SET ROLE tn_owner; SELECT count(*) FROM ops.trips;'), '0');
  check('049 superuser still bypasses FORCE', scalar('SELECT count(*) FROM ops.trips;'), '100');

  // PG-048/050: restore a base backup + archive up to a named point before deletion.
  const backup = join(source.root, 'base');
  tool(source, 'pg_basebackup', ['-h','127.0.0.1','-p',String(source.port),'-U','postgres','-D',backup,'-Fp','-X','stream','--checkpoint=fast']);
  sql(source, "INSERT INTO ops.trips VALUES(101,1,45000,'completed');");
  const recoveryExpected = scalar(digest);
  sql(source, "SELECT pg_create_restore_point('p05_before_delete');");
  sql(source, 'DELETE FROM ops.trips;');
  check('048 source damaged deliberately', scalar('SELECT count(*) FROM ops.trips;'), '0');
  const segment = scalar('SELECT pg_walfile_name(pg_current_wal_insert_lsn());');
  sql(source, 'SELECT pg_switch_wal();');
  await until(() => existsSync(join(archive, segment)), 'WAL archive did not arrive', 30);
  recovery = await createCluster(pg, {settings:{autovacuum:'on'}}); recovery.stop();
  assert.equal(relative(recovery.root, recovery.dataDir), 'data');
  renameSync(recovery.dataDir, join(recovery.root, 'unused-initial-data'));
  cpSync(backup, recovery.dataDir, {recursive:true});
  const restoreCommand = `"${slash(process.execPath)}" "${copier}" restore "${slash(archive)}/%f" "%p"`;
  appendFileSync(join(recovery.dataDir,'postgresql.conf'), `\nport=${recovery.port}\narchive_mode=off\nssl=off\nrestore_command=${conf(restoreCommand)}\nrecovery_target_name='p05_before_delete'\nrecovery_target_action='pause'\n`);
  writeFileSync(join(recovery.dataDir,'recovery.signal'),'');
  const rt=performance.now(); recovery.start();
  await until(() => sql(recovery,"SELECT pg_is_wal_replay_paused();") === 't','recovery did not pause',30);
  check('048 recovered data including post-backup insert', sql(recovery,digest), recoveryExpected);
  check('048 RLS survives PITR', sql(recovery,"SET ROLE tn_app; SET app.tenant_id='1'; SELECT count(*) FROM ops.trips;").split('\n').at(-1).trim(), '51');
  note(`MEASURE 048 start-to-verified-paused-recovery ms: ${(performance.now()-rt).toFixed(1)} (excludes base-copy and incident detection)`);
  sql(recovery,'SELECT pg_wal_replay_resume();');
  await until(() => sql(recovery,'SELECT pg_is_in_recovery();')==='f','recovery did not finish');
  check('050 writable restored cluster', sql(recovery,"INSERT INTO ops.trips VALUES(102,2,40000,'completed') RETURNING id;").split('\n')[0].trim(), '102');
  passed=true; note('PASS P05: all assertions passed; both temporary clusters cleaned up.');
} finally {
  for (const s of sessions) { try { await s.close(); } catch {} }
  recovery?.destroySync(); source?.destroySync();
  if (passed) writeFileSync(join(here,'OUTPUT.txt'),output.join('\n')+'\n');
}
