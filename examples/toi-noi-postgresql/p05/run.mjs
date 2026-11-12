// Dedicated operations lab: real TLS, dump/restore and archived-WAL recovery.
// Never attaches to an existing server. Temporary clusters are disposed in finally.
//
// OUTPUT.txt has two kinds of lines:
//   PASS/MEASURE …   assertions and observed numbers (the gate)
//   --- 0xx label    evidence block: real psql output / server log excerpts quoted by
//                    the articles. Ports, PIDs, timestamps and passwords are masked;
//                    LSNs and WAL file names are what this run produced.
import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync, appendFileSync, mkdirSync, renameSync, cpSync, existsSync, chmodSync, statSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
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
const masks = [];
// Remove noise that differs on every run and is never the point of the evidence.
function clean(s) {
  let t = String(s).replace(/\r\n/g, '\n');
  for (const [from, to] of masks) t = t.replaceAll(from, to);
  return t.split('\n')
    .filter(l => l.trim() && !/^\s*LOCATION:/.test(l))
    .map(l => l.replace(/^psql:<stdin>:\d+: /, '').replace(/port \d{4,5}/g, 'port <PORT>').replace(/"[^"]*[\/]data[\/]pg_hba\.conf"/g, '"<DATA>/pg_hba.conf"').replace(/^\S+ \S+ \S+ \[\d+\] /, ''))
    .join('\n');
}
function evidence(label, text) {
  const lines = clean(text).split('\n');
  output.push(`--- ${label}`, ...lines.map(l => '  ' + l));
  console.log(`--- ${label}\n${lines.map(l => '  ' + l).join('\n')}`);
}
function tool(c, name, args, extra = {}) {
  const r = spawnSync(exe(pg.dir, name), args, { env: c.env, encoding: 'utf8', windowsHide: true, timeout: 60000, ...extra });
  if (r.error || r.status !== 0) throw Error(`${name}: ${r.error?.message || r.stderr}`);
  return r.stdout.trim();
}
let lastStderr = '';
function sql(c, text, { db = 'postgres', user = 'postgres', env = {}, fail = false } = {}) {
  const r = spawnSync(exe(pg.dir, 'psql'), ['-X', '-h', '127.0.0.1', '-p', String(c.port), '-U', user, '-d', db, '-At', '-v', 'ON_ERROR_STOP=1', '-v', 'VERBOSITY=verbose'], { input: text, env: { ...c.env, ...env }, encoding: 'utf8', windowsHide: true, timeout: 15000 });
  if (r.error) throw r.error;
  lastStderr = r.stderr;
  if (fail) { assert.notEqual(r.status, 0, 'Expected SQL rejection'); return r.stderr; }
  assert.equal(r.status, 0, r.stderr);
  return r.stdout.trim();
}
// Aligned psql output (as a reader sees it), quiet so SET/RESET tags are not printed.
function show(c, text, { db = 'postgres' } = {}) {
  const r = spawnSync(exe(pg.dir, 'psql'), ['-X', '-q', '-h', '127.0.0.1', '-p', String(c.port), '-U', 'postgres', '-d', db, '-P', 'pager=off', '-P', 'null=NULL', '-v', 'ON_ERROR_STOP=1'], { input: text, env: c.env, encoding: 'utf8', windowsHide: true, timeout: 15000 });
  assert.equal(r.status, 0, r.stderr);
  return r.stdout.replace(/\r\n/g, '\n').trimEnd();
}
function scalar(text, options) { return sql(source, text, options).split('\n').at(-1).trim(); }
function check(label, actual, expected) { assert.equal(actual, expected, label); note(`PASS ${label}: ${actual}`); }
function serverLog(c, pattern) {
  if (!existsSync(c.logFile)) return '';
  return readFileSync(c.logFile, 'utf8').split(/\r?\n/).filter(l => pattern.test(l)).join('\n');
}
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

  // PG-041: least privilege — allowed and denied paths, owner, default privileges.
  check('041 report reads', scalar('SET ROLE tn_report; SELECT count(*) FROM ops.trips;'), '100');
  const reportDelete = sql(source, 'SET ROLE tn_report; DELETE FROM ops.trips;', { fail: true });
  assert.match(reportDelete, /42501/);
  evidence('041 tn_report DELETE', reportDelete);
  const appDdl = sql(source, 'SET ROLE tn_app; ALTER TABLE ops.trips ADD COLUMN bad int;', { fail: true });
  assert.match(appDdl, /42501/);
  evidence('041 tn_app ALTER TABLE', appDdl);
  check('041 application is not owner', scalar("SELECT tableowner FROM pg_tables WHERE schemaname='ops' AND tablename='trips'"), 'tn_owner');
  check('041 app cannot CREATE in schema public', scalar("SELECT has_schema_privilege('tn_app','public','CREATE')"), 'f');
  sql(source, 'SET ROLE tn_owner; CREATE TABLE ops.next_report(id int); RESET ROLE;');
  check('041 future table default privileges', scalar("SELECT has_table_privilege('tn_report','ops.next_report','SELECT')"), 't');
  check('041 future table not granted to app', scalar("SELECT has_table_privilege('tn_app','ops.next_report','SELECT')"), 'f');
  evidence('041 roles', show(source, "SELECT rolname, rolcanlogin AS login, rolsuper AS super, rolbypassrls AS bypassrls FROM pg_roles WHERE rolname LIKE 'tn\\_%' ORDER BY 1;"));
  evidence('041 table ACL', show(source, "SELECT c.relname, c.relacl FROM pg_class c WHERE c.oid IN ('ops.trips'::regclass, 'ops.next_report'::regclass) ORDER BY 1;"));
  evidence('041 default ACL', show(source, "SELECT pg_get_userbyid(defaclrole) AS for_role, defaclnamespace::regnamespace AS in_schema, defaclobjtype AS type, defaclacl FROM pg_default_acl;"));

  // PG-042: self-signed lab certificate used as explicit trust anchor; no key in git.
  const cert = join(source.dataDir, 'server.crt'), key = join(source.dataDir, 'server.key');
  const ssl = spawnSync(openssl, ['req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-keyout', key, '-out', cert, '-days', '2', '-subj', '/CN=localhost', '-addext', 'subjectAltName=DNS:localhost'], { encoding: 'utf8', windowsHide: true, timeout: 30000, env: { ...process.env, MSYS_NO_PATHCONV: '1' } });
  assert.equal(ssl.status, 0, ssl.stderr); chmodSync(key, 0o600);
  const password = randomBytes(24).toString('hex');
  masks.push([password, '<PASSWORD>']);
  sql(source, `SET password_encryption='scram-sha-256'; ALTER ROLE tn_app PASSWORD '${password}';`);
  check('042 stored verifier is SCRAM', scalar("SELECT split_part(rolpassword, '$', 1) FROM pg_authid WHERE rolname='tn_app'"), 'SCRAM-SHA-256');
  writeFileSync(join(source.dataDir, 'pg_hba.conf'), 'local all postgres trust\nhost all postgres 127.0.0.1/32 trust\nhost all postgres ::1/128 trust\nhostssl all tn_app 127.0.0.1/32 scram-sha-256\nhost all all 127.0.0.1/32 reject\nhost all all ::1/128 reject\nhost replication postgres 127.0.0.1/32 trust\n');
  const archive = join(source.root, 'archive'); mkdirSync(archive);
  const copier = slash(join(here, 'wal-copy.mjs'));
  const archiveCommand = `"${slash(process.execPath)}" "${copier}" archive "%p" "${slash(archive)}/%f"`;
  appendFileSync(join(source.dataDir, 'postgresql.conf'), `\nssl=on\narchive_command=${conf(archiveCommand)}\n`);
  source.restart();
  evidence('042 pg_hba_file_rules', show(source, 'SELECT rule_number AS rule, type, database, user_name, address, auth_method, error FROM pg_hba_file_rules ORDER BY rule_number;'));
  // host=localhost validates SAN; hostaddr avoids DNS/network ambiguity.
  const appArgs = (mode, host) => `host=${host} hostaddr=127.0.0.1 port=${source.port} user=tn_app dbname=postgres sslmode=${mode} sslrootcert='${slash(cert)}' connect_timeout=3`;
  const app = (mode, pass = password, host = 'localhost', query = 'SELECT ssl FROM pg_stat_ssl WHERE pid=pg_backend_pid()') => spawnSync(exe(pg.dir, 'psql'), ['-X', '-At', '-d', appArgs(mode, host), '-c', query], { env: { ...source.env, PGPASSWORD: pass }, encoding: 'utf8', timeout: 10000, windowsHide: true });
  check('042 TLS verify-full', app('verify-full').stdout.trim(), 't');
  evidence('042 pg_stat_ssl of tn_app session', app('verify-full', password, 'localhost', "SELECT 'ssl='||ssl||' version='||version||' bits='||bits||' user='||current_user FROM pg_stat_ssl WHERE pid=pg_backend_pid()").stdout);
  for (const [label, r, pattern] of [['plaintext', app('disable'), /rejects connection/], ['wrong password', app('verify-full', 'wrong'), /password authentication failed/], ['wrong hostname', app('verify-full', password, 'wrong.invalid'), /does not match host name/]]) {
    assert.notEqual(r.status, 0); assert.match(r.stderr, pattern); note(`PASS 042 rejects ${label}`);
    evidence(`042 client error: ${label}`, r.stderr);
  }
  // verify-ca checks the chain only: the same wrong host name is accepted.
  check('042 verify-ca accepts wrong hostname', app('verify-ca', password, 'wrong.invalid').stdout.trim(), 't');
  evidence('042 server log: rejected attempts', serverLog(source, /FATAL|DETAIL:  Connection matched/));

  // PG-043: actual blocked backend, identified through pg_blocking_pids.
  const a = await session('p05-blocker'), b = await session('p05-waiter'), c = await session('p05-impatient');
  masks.push([`{${a.pid}}`, '{<PID A>}']);
  await step(a, 'BEGIN; UPDATE ops.trips SET fare=fare WHERE id=1;');
  const blocked = b.run('UPDATE ops.trips SET fare=fare WHERE id=1;');
  await until(() => scalar(`SELECT ${a.pid}=ANY(pg_blocking_pids(${b.pid}))`) === 't', 'waiter never blocked');
  check('043 identifies lock wait', scalar(`SELECT wait_event_type FROM pg_stat_activity WHERE pid=${b.pid}`), 'Lock');
  const who = `CASE pid WHEN ${a.pid} THEN 'A' WHEN ${b.pid} THEN 'B' ELSE 'C' END`;
  evidence('043 pg_stat_activity', show(source, `SELECT ${who} AS s, state, wait_event_type AS wtype, wait_event, pg_blocking_pids(pid) AS blocked_by, left(query, 44) AS query FROM pg_stat_activity WHERE pid IN (${a.pid},${b.pid}) ORDER BY 1;`));
  evidence('043 pg_locks not granted', show(source, `SELECT ${who} AS s, locktype, mode, granted FROM pg_locks WHERE pid IN (${a.pid},${b.pid}) AND NOT granted ORDER BY 1;`));
  check('043 tn_monitor sees query text', scalar(`SET ROLE tn_monitor; SELECT query <> '<insufficient privilege>' FROM pg_stat_activity WHERE pid=${b.pid};`), 't');
  check('043 tn_report does not see query text', scalar(`SET ROLE tn_report; SELECT query FROM pg_stat_activity WHERE pid=${b.pid};`), '<insufficient privilege>');
  const impatient = await c.run("SET lock_timeout='200ms'; UPDATE ops.trips SET fare=fare WHERE id=1;");
  check('043 lock_timeout SQLSTATE', impatient.sqlstate, '55P03');
  evidence('043 lock_timeout error', impatient.stderr);
  await step(a, 'ROLLBACK;'); assert(!(await blocked).error);
  // idle in transaction is ended by the server, not left holding locks for ever.
  const idle = await session('p05-idle');
  await step(idle, "SET idle_in_transaction_session_timeout='300ms'; BEGIN; SELECT count(*) FROM ops.trips;");
  await until(() => scalar(`SELECT count(*) FROM pg_stat_activity WHERE pid=${idle.pid}`) === '0', 'idle transaction was not terminated', 10);
  note('PASS 043 idle_in_transaction_session_timeout ended backend: gone');
  evidence('043 server log: idle timeout', serverLog(source, /idle-in-transaction timeout/));
  idle.kill();
  for (const s of [a, b, c]) await s.close();

  // PG-044: deterministic pgstattuple inspection; per-table autovac off only for experiment.
  sql(source, 'CREATE EXTENSION pgstattuple; CREATE TABLE ops.churn(id int, payload text) WITH (autovacuum_enabled=false); INSERT INTO ops.churn SELECT n, repeat(md5(n::text),10) FROM generate_series(1,5000)n;');
  sql(source, 'DELETE FROM ops.churn WHERE id % 2=0;');
  const tupleView = "SELECT table_len, tuple_count AS live, dead_tuple_count AS dead, round(dead_tuple_percent) AS dead_pct, free_space, round(free_percent) AS free_pct FROM pgstattuple('ops.churn');";
  evidence('044 pgstattuple after DELETE', show(source, tupleView));
  const lenBefore = scalar("SELECT table_len FROM pgstattuple('ops.churn')");
  const before = Number(scalar("SELECT dead_tuple_count FROM pgstattuple('ops.churn')")); assert(before > 0);
  sql(source, 'VACUUM (ANALYZE) ops.churn;');
  check('044 dead tuples after vacuum', scalar("SELECT dead_tuple_count FROM pgstattuple('ops.churn')"), '0');
  evidence('044 pgstattuple after VACUUM', show(source, tupleView));
  check('044 VACUUM keeps file size', scalar("SELECT table_len FROM pgstattuple('ops.churn')"), lenBefore);
  note(`PASS 044 dead tuples before vacuum: ${before}; cluster autovacuum remains on`);
  // An old snapshot holds back the xmin horizon: VACUUM runs but cannot remove.
  const holder = await session('p05-old-snapshot');
  await step(holder, 'BEGIN ISOLATION LEVEL REPEATABLE READ; SELECT count(*) FROM ops.churn;');
  sql(source, 'DELETE FROM ops.churn WHERE id <= 1000;');
  sql(source, 'VACUUM (VERBOSE) ops.churn;');
  const held = lastStderr;
  check('044 old snapshot keeps dead tuples', scalar("SELECT dead_tuple_count FROM pgstattuple('ops.churn')"), '500');
  evidence('044 VACUUM VERBOSE while snapshot open', held.split(/\r?\n/).filter(l => /tuples:|removable cutoff/.test(l)).slice(0, 2).join('\n'));
  evidence('044 who holds the horizon', show(source, `SELECT CASE pid WHEN ${holder.pid} THEN 'holder' END AS s, state, backend_xmin IS NOT NULL AS has_xmin, left(query, 40) AS query FROM pg_stat_activity WHERE pid=${holder.pid};`));
  await step(holder, 'COMMIT;'); await holder.close();
  sql(source, 'VACUUM (VERBOSE) ops.churn;');
  evidence('044 VACUUM VERBOSE after snapshot closed', lastStderr.split(/\r?\n/).filter(l => /tuples:/.test(l)).slice(0, 1).join('\n'));
  check('044 dead tuples after holder commits', scalar("SELECT dead_tuple_count FROM pgstattuple('ops.churn')"), '0');
  sql(source, 'ALTER TABLE ops.churn RESET (autovacuum_enabled);');
  evidence('044 autovacuum thresholds', show(source, "SELECT name, setting FROM pg_settings WHERE name IN ('autovacuum','autovacuum_vacuum_threshold','autovacuum_vacuum_scale_factor','autovacuum_vacuum_max_threshold','autovacuum_naptime') ORDER BY name;"));

  // PG-045: local warm-cache micro experiment, not a production benchmark.
  check('045 durability settings', scalar("SELECT current_setting('fsync')||'/'||current_setting('synchronous_commit')||'/'||current_setting('full_page_writes')"), 'on/on/on');
  evidence('045 WAL settings', show(source, "SELECT name, setting, unit FROM pg_settings WHERE name IN ('wal_level','fsync','synchronous_commit','full_page_writes','wal_compression','checkpoint_timeout','max_wal_size','archive_mode') ORDER BY name;"));
  const cpBefore = Number(scalar('SELECT num_requested FROM pg_stat_checkpointer'));
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
  const cpAfter = Number(scalar('SELECT num_requested FROM pg_stat_checkpointer'));
  assert(cpAfter - cpBefore >= 3, 'checkpointer did not count requested checkpoints');
  note(`PASS 045 requested checkpoints counted: +${cpAfter - cpBefore}`);
  // Full-page images: first change of a page after a checkpoint logs the whole page.
  const walOf = (text) => { const l = scalar('SELECT pg_current_wal_insert_lsn();'); sql(source, text); return Number(scalar(`SELECT pg_wal_lsn_diff(pg_current_wal_insert_lsn(),'${l}')::bigint`)); };
  const fpi = [walOf('UPDATE ops.trips SET fare=fare+1;'), walOf('UPDATE ops.trips SET fare=fare+1;')];
  assert(fpi[0] > fpi[1], `expected full-page images after checkpoint: ${fpi}`);
  note(`MEASURE 045 same 100-row UPDATE right after CHECKPOINT vs repeated: walBytes ${JSON.stringify(fpi)}`);
  sql(source, 'UPDATE ops.trips SET fare=30000;');
  evidence('045 server log: checkpoint complete (first 3)', serverLog(source, /checkpoint complete/).split('\n').slice(0, 3).map(l => l.replace(/; (sync|longest|average|distance|estimate|lsn|redo).*$/, '')).join('\n'));

  // PG-046: custom-format dump including ownership and ACL; roles already exist in same cluster.
  const dump = join(source.root, 'ops.dump');
  tool(source, 'pg_dump', ['-h','127.0.0.1','-p',String(source.port),'-U','postgres','-d','postgres','-Fc','-n','ops','-f',dump]);
  note(`MEASURE 046 custom archive size bytes: ${statSync(dump).size}`);
  evidence('046 pg_restore --list (TOC)', tool(source, 'pg_restore', ['--list', dump]).split(/\r?\n/).filter(l => l && !l.startsWith(';')).map(l => l.replace(/^\d+; \d+ \d+ /, '')).join('\n'));
  const globals = tool(source, 'pg_dumpall', ['-h','127.0.0.1','-p',String(source.port),'-U','postgres','--globals-only','--no-role-passwords']);
  check('046 roles live in globals, not in schema dump', String(/CREATE ROLE tn_app;/.test(globals)), 'true');
  evidence('046 pg_dumpall --globals-only (tn_ roles)', globals.split(/\r?\n/).filter(l => /tn_(app|owner|report|monitor)/.test(l) && /ROLE/.test(l)).join('\n'));
  sql(source, 'CREATE DATABASE p05_restored;');
  tool(source, 'pg_restore', ['-h','127.0.0.1','-p',String(source.port),'-U','postgres','-d','p05_restored','--exit-on-error',dump]);
  const digest = "SELECT count(*)||'/'||sum(fare)||'/'||md5(string_agg(id||':'||tenant_id||':'||fare||':'||status,',' ORDER BY id)) FROM ops.trips";
  const expected = scalar(digest);
  check('046 restored data digest', scalar(digest, {db:'p05_restored'}), expected);
  check('046 restored owner', scalar("SELECT tableowner FROM pg_tables WHERE schemaname='ops' AND tablename='trips'",{db:'p05_restored'}), 'tn_owner');
  check('046 restored report ACL', scalar("SELECT has_table_privilege('tn_report','ops.trips','SELECT')",{db:'p05_restored'}), 't');
  assert.match(sql(source, 'SET ROLE tn_report; UPDATE ops.trips SET fare=0;', {db:'p05_restored',fail:true}), /42501/);
  // Same archive restored with --no-owner --no-acl: data equal, permission model gone.
  sql(source, 'CREATE DATABASE p05_restored_bare;');
  tool(source, 'pg_restore', ['-h','127.0.0.1','-p',String(source.port),'-U','postgres','-d','p05_restored_bare','--exit-on-error','--no-owner','--no-acl',dump]);
  check('046 bare restore digest', scalar(digest, {db:'p05_restored_bare'}), expected);
  check('046 bare restore owner', scalar("SELECT tableowner FROM pg_tables WHERE schemaname='ops' AND tablename='trips'",{db:'p05_restored_bare'}), 'postgres');
  check('046 bare restore report ACL', scalar("SELECT has_table_privilege('tn_report','ops.trips','SELECT')",{db:'p05_restored_bare'}), 'f');

  // PG-047: bounded two-session application queue; measure checkout wait separately.
  evidence('047 connection settings', show(source, "SELECT name, setting FROM pg_settings WHERE name IN ('max_connections','superuser_reserved_connections','reserved_connections','idle_in_transaction_session_timeout','statement_timeout','lock_timeout') ORDER BY name;"));
  const workers = await Promise.all([session('pool-1'),session('pool-2')]);
  const waits=[]; let next=0; const enqueued=performance.now();
  await Promise.all(workers.map(async s => { while(next<6) { const job=next++; waits[job]=+(performance.now()-enqueued).toFixed(1); await step(s,'SELECT pg_sleep(0.05);'); } }));
  note('MEASURE 047 6 jobs / 2 existing connections / 50ms service; queue waits ms: '+JSON.stringify(waits));
  const timeout = await workers[0].run("SET statement_timeout='20ms'; SELECT pg_sleep(0.2);");
  check('047 statement timeout SQLSTATE', timeout.sqlstate, '57014');
  evidence('047 statement_timeout error', timeout.stderr);
  for (const s of workers) await s.close();
  // Per-role connection budget: second tn_app connection refused while first is open.
  sql(source, 'ALTER ROLE tn_app CONNECTION LIMIT 1;');
  const holderApp = spawn(exe(pg.dir, 'psql'), ['-X', '-At', '-d', appArgs('verify-full', 'localhost'), '-c', 'SELECT pg_sleep(3)'], { env: { ...source.env, PGPASSWORD: password }, windowsHide: true });
  await until(() => scalar("SELECT count(*) FROM pg_stat_activity WHERE usename='tn_app'") === '1', 'first tn_app connection did not arrive');
  const second = app('verify-full');
  assert.notEqual(second.status, 0); assert.match(second.stderr, /too many connections for role/);
  note('PASS 047 role CONNECTION LIMIT 1 rejects second connection');
  evidence('047 client error: role connection limit', second.stderr);
  await new Promise(r => holderApp.on('close', r));
  sql(source, 'ALTER ROLE tn_app CONNECTION LIMIT -1;');

  // PG-049 first: include RLS in the physical backup and prove it survives recovery.
  sql(source, `ALTER TABLE ops.trips ENABLE ROW LEVEL SECURITY;
    CREATE POLICY tenant_scope ON ops.trips TO tn_app
    USING (tenant_id = NULLIF(current_setting('app.tenant_id',true),'')::int)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id',true),'')::int);`);
  evidence('049 pg_policies', show(source, "SELECT policyname, roles, cmd, qual, with_check FROM pg_policies WHERE tablename='trips';"));
  check('049 no tenant denies rows', scalar('SET ROLE tn_app; SELECT count(*) FROM ops.trips;'), '0');
  check('049 tenant 1', scalar("SET ROLE tn_app; SET app.tenant_id='1'; SELECT count(*) FROM ops.trips;"), '50');
  const wrongInsert = sql(source, "SET ROLE tn_app; SET app.tenant_id='1'; INSERT INTO ops.trips VALUES(101,2,30000,'completed');", {fail:true});
  assert.match(wrongInsert, /42501/);
  evidence('049 INSERT into other tenant', wrongInsert);
  const moveRow = sql(source, "SET ROLE tn_app; SET app.tenant_id='1'; UPDATE ops.trips SET tenant_id=2 WHERE id=1;", {fail:true});
  assert.match(moveRow, /42501/);
  evidence('049 UPDATE moving row to other tenant', moveRow);
  check('049 UPDATE of other tenant row touches', scalar("SET ROLE tn_app; SET app.tenant_id='1'; WITH u AS (UPDATE ops.trips SET fare=fare WHERE id=60 RETURNING 1) SELECT count(*) FROM u;"), '0');
  check('049 SET LOCAL scope inside / after transaction', sql(source, "SET ROLE tn_app; BEGIN; SET LOCAL app.tenant_id='2'; SELECT count(*) FROM ops.trips; COMMIT; SELECT count(*) FROM ops.trips;").split(/\r?\n/).filter(l => /^\d+$/.test(l)).join('/'), '50/0');
  check('049 report role without policy', scalar('SET ROLE tn_report; SELECT count(*) FROM ops.trips;'), '0');
  check('049 owner bypass', scalar('SET ROLE tn_owner; SELECT count(*) FROM ops.trips;'), '100');
  sql(source, 'ALTER TABLE ops.trips FORCE ROW LEVEL SECURITY;');
  check('049 forced owner without policy', scalar('SET ROLE tn_owner; SELECT count(*) FROM ops.trips;'), '0');
  check('049 superuser still bypasses FORCE', scalar('SELECT count(*) FROM ops.trips;'), '100');
  evidence('049 table RLS flags', show(source, "SELECT relname, relrowsecurity AS rls, relforcerowsecurity AS forced FROM pg_class WHERE oid='ops.trips'::regclass;"));

  // PG-048/050: restore a base backup + archive up to a named point before deletion.
  const backup = join(source.root, 'base');
  tool(source, 'pg_basebackup', ['-h','127.0.0.1','-p',String(source.port),'-U','postgres','-D',backup,'-Fp','-X','stream','--checkpoint=fast','--label=p05_base']);
  evidence('048 backup_label', readFileSync(join(backup, 'backup_label'), 'utf8').split(/\r?\n/).filter(l => /^(START WAL|CHECKPOINT|BACKUP METHOD|BACKUP FROM|LABEL|START TIMELINE)/.test(l)).join('\n'));
  sql(source, "INSERT INTO ops.trips VALUES(101,1,45000,'completed');");
  const recoveryExpected = scalar(digest);
  const pointLsn = scalar("SELECT pg_create_restore_point('p05_before_delete');");
  note(`MEASURE 048 restore point p05_before_delete LSN: ${pointLsn}`);
  sql(source, 'DELETE FROM ops.trips;');
  check('048 source damaged deliberately', scalar('SELECT count(*) FROM ops.trips;'), '0');
  const segment = scalar('SELECT pg_walfile_name(pg_current_wal_insert_lsn());');
  sql(source, 'SELECT pg_switch_wal();');
  await until(() => existsSync(join(archive, segment)), 'WAL archive did not arrive', 30);
  await until(() => scalar(`SELECT last_archived_wal >= '${segment}' FROM pg_stat_archiver`) === 't', 'archiver stats did not catch up', 10);
  evidence('048 pg_stat_archiver', show(source, 'SELECT archived_count, last_archived_wal, failed_count FROM pg_stat_archiver;'));
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
  check('048 paused target is still read-only', String(/25006/.test(sql(recovery, "INSERT INTO ops.trips VALUES(999,1,1,'completed');", {fail:true}))), 'true');
  evidence('048 recovery server log', serverLog(recovery, /starting point-in-time recovery|redo starts|consistent recovery state|recovery stopping|pausing at the end of recovery|restored log file/).split('\n').map(l => l.replace(/ at \d+\/[0-9A-F]+/g, ' at <LSN>').replace(/time \d{4}-\S+ \S+/, 'time <T>')).join('\n'));
  sql(recovery,'SELECT pg_wal_replay_resume();');
  await until(() => sql(recovery,'SELECT pg_is_in_recovery();')==='f','recovery did not finish');
  check('050 writable restored cluster', sql(recovery,"INSERT INTO ops.trips VALUES(102,2,40000,'completed') RETURNING id;").split('\n')[0].trim(), '102');
  check('050 promoted onto new timeline', sql(recovery, 'SELECT substr(pg_walfile_name(pg_current_wal_lsn()), 1, 8);'), '00000002');
  check('050 timeline history file', String(existsSync(join(recovery.dataDir, 'pg_wal', '00000002.history'))), 'true');
  evidence('050 00000002.history', readFileSync(join(recovery.dataDir, 'pg_wal', '00000002.history'), 'utf8'));
  const afterResume = serverLog(recovery, /selected new timeline|archive recovery complete|database system is ready/).split('\n');
  evidence('050 recovery server log after resume', afterResume.slice(afterResume.findIndex(l => /selected new timeline/.test(l))).map(l => l.replace(/ at \d+\/[0-9A-F]+/g, ' at <LSN>')).join('\n'));
  passed=true; note('PASS P05: all assertions passed; both temporary clusters cleaned up.');
} finally {
  for (const s of sessions) { try { await s.close(); } catch {} }
  recovery?.destroySync(); source?.destroySync();
  if (passed) writeFileSync(join(here,'OUTPUT.txt'),output.join('\n')+'\n');
}
