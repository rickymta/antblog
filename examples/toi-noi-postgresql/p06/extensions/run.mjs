// PG-057 · PostGIS and pgvector evaluated as two independent experiments.
// Runs on PGlite (PostgreSQL compiled to WASM, in-process) because the Windows
// PostgreSQL 18 build used by the other labs ships neither extension.
// Deterministic data, no network. Writes OUTPUT.txt next to this file on success.
import assert from 'node:assert/strict';
import { writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';
import { PGlite } from '@electric-sql/pglite';
import { postgis } from '@electric-sql/pglite-postgis';
import { vector } from '@electric-sql/pglite-pgvector';

const here = fileURLToPath(new URL('.', import.meta.url));
const out = [];
const note = s => { out.push(s); console.log(s); };
const block = (label, text) => { const lines = String(text).split('\n').map(l => '  ' + l); out.push(`--- ${label}`, ...lines); console.log(`--- ${label}\n${lines.join('\n')}`); };
const plan = async (db, q) => (await db.query('EXPLAIN (COSTS OFF) ' + q)).rows.map(r => r['QUERY PLAN']).join('\n');
const db = new PGlite({ extensions: { postgis, vector } });
let passed = false;
try {
  await db.exec('CREATE EXTENSION postgis; CREATE EXTENSION vector;');
  const v = (await db.query("SELECT current_setting('server_version') AS pg, (SELECT extversion FROM pg_extension WHERE extname='postgis') AS postgis, (SELECT extversion FROM pg_extension WHERE extname='vector') AS vector")).rows[0];
  note(`PG-057 extension lab · PGlite PostgreSQL ${v.pg} · postgis ${v.postgis} · vector ${v.vector} · Node ${process.version}`);

  // --- Experiment 1: nearby drivers. 5,000 drivers on a deterministic grid around District 1 (HCMC).
  await db.exec(`
    CREATE TABLE drivers (id int PRIMARY KEY, location geography(Point, 4326) NOT NULL);
    INSERT INTO drivers
    SELECT n, ST_SetSRID(ST_MakePoint(106.60 + (n % 100) * 0.002, 10.70 + (n / 100) * 0.002), 4326)::geography
    FROM generate_series(0, 4999) n;
    ANALYZE drivers;`);
  const pickup = "'SRID=4326;POINT(106.70 10.77)'::geography";
  const within = `SELECT count(*)::int AS n FROM drivers WHERE ST_DWithin(location, ${pickup}, 500)`;
  const before = (await db.query(within)).rows[0].n;
  block('057 EXPLAIN ST_DWithin without spatial index', await plan(db, within));
  await db.exec('CREATE INDEX drivers_location_gist ON drivers USING gist (location); ANALYZE drivers;');
  const after = (await db.query(within)).rows[0].n;
  assert.equal(after, before);
  note(`PASS 057 drivers within 500 m of pickup (seq scan = GiST): ${after}`);
  block('057 EXPLAIN ST_DWithin with GiST', await plan(db, within));
  const nearest = (await db.query(`SELECT id, round(ST_Distance(location, ${pickup}))::int AS m FROM drivers ORDER BY location <-> ${pickup} LIMIT 3`)).rows;
  block('057 three nearest drivers (KNN <->)', nearest.map(r => `id ${r.id} · ${r.m} m`).join('\n'));
  const degreeTrap = (await db.query("SELECT count(*)::int AS n FROM drivers WHERE ST_DWithin(location::geometry, 'SRID=4326;POINT(106.70 10.77)'::geometry, 500)")).rows[0].n;
  note(`PASS 057 geometry in SRID 4326 measures degrees: radius 500 matches ${degreeTrap} of 5000 drivers`);
  assert.equal(degreeTrap, 5000);

  // --- Experiment 2: similar support tickets. 2,000 deterministic 16-d vectors, exact vs HNSW.
  await db.exec(`
    CREATE TABLE tickets (id int PRIMARY KEY, embedding vector(16) NOT NULL);
    INSERT INTO tickets
    SELECT n, (SELECT array_agg(sin(n * 0.37 + d * 1.71 + (n % 7) * d)::real ORDER BY d) FROM generate_series(1, 16) d)::vector
    FROM generate_series(1, 2000) n;
    ANALYZE tickets;`);
  // Query vectors come from the same generator but are NOT rows of the table (ids 5001..5020),
  // so no WHERE filter is needed: a filter applied after an HNSW scan can return fewer than k rows.
  const queries = Array.from({ length: 20 }, (_, i) => 5001 + i * 37);
  const qvec = (n) => `(SELECT array_agg(sin(${n} * 0.37 + d * 1.71 + (${n} % 7) * d)::real ORDER BY d) FROM generate_series(1, 16) d)::vector`;
  const topk = async (n) => (await db.query(`SELECT id FROM tickets ORDER BY embedding <=> ${qvec(n)} LIMIT 10`)).rows.map(r => r.id);
  const exact = {};
  for (const q of queries) exact[q] = await topk(q);
  assert(queries.every(q => exact[q].length === 10));
  block('057 EXPLAIN cosine top-10 without vector index', await plan(db, `SELECT id FROM tickets ORDER BY embedding <=> ${qvec(5001)} LIMIT 10`));
  // HNSW assigns graph layers randomly: build the index several times and report the spread.
  const builds = 3, efs = [10, 40, 200];
  const runs = Object.fromEntries(efs.map(ef => [ef, []]));
  let rowsSeen = Infinity;
  for (let b = 0; b < builds; b++) {
    await db.exec('DROP INDEX IF EXISTS tickets_embedding_hnsw; CREATE INDEX tickets_embedding_hnsw ON tickets USING hnsw (embedding vector_cosine_ops) WITH (m = 4, ef_construction = 16);');
    if (b === 0) block('057 EXPLAIN cosine top-10 with HNSW', await plan(db, `SELECT id FROM tickets ORDER BY embedding <=> ${qvec(5001)} LIMIT 10`));
    for (const ef of efs) {
      await db.exec(`SET hnsw.ef_search = ${ef};`);
      let hit = 0;
      for (const q of queries) { const got = await topk(q); rowsSeen = Math.min(rowsSeen, got.length); hit += got.filter(id => exact[q].includes(id)).length; }
      runs[ef].push(+(hit / (queries.length * 10)).toFixed(3));
    }
  }
  const recall = efs.map(ef => ({ ef_search: ef, recall_per_build: runs[ef], min: Math.min(...runs[ef]), max: Math.max(...runs[ef]) }));
  note(`MEASURE 057 HNSW (m=4, ef_construction=16) recall@10 vs exact, ${queries.length} queries not in table, ${builds} index builds: ` + JSON.stringify(recall));
  note(`PASS 057 every HNSW query returned 10 rows (min rows seen: ${rowsSeen})`);
  assert.equal(rowsSeen, 10);
  // Within one build the graph is fixed, so a wider search must not find fewer true neighbours.
  for (let b = 0; b < builds; b++) assert(runs[200][b] >= runs[10][b], `build ${b + 1}: recall fell as ef_search grew`);
  passed = true;
  note('PASS PG-057: both experiments completed.');
} finally {
  await db.close();
  if (passed) writeFileSync(join(here, 'OUTPUT.txt'), out.join('\n') + '\n');
}
