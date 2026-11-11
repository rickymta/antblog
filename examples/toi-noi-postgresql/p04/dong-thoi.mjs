// Đặc tả PHÉP THỬ ĐỒNG THỜI của P04 — chạy bằng `npm run p04:measure`
// (= node pg/run-dong-thoi.mjs p04/dong-thoi.mjs), ghi kết quả ra p04/DO-DAC.md.
// `npm test` chạy bản NHỎ (--nho, vài giây mỗi kịch bản) và chỉ kiểm các bất biến "phải đúng".
//
// Mỗi kịch bản: một database mới chép từ nền p04 mỗi LẦN; `chuan_bi` (SQL) chạy một lần;
// `chay` điều khiển N phiên psql thật (workers) chạy ĐUA nhau; cuối lần, `bat_bien` kiểm bằng SQL
// (hoặc từ bộ đếm phía client: tu_dem). `phai_dung: true` ⇒ vi phạm làm lệnh thoát mã 1.
// Bất biến có `mong_doi` mà không `phai_dung` = MINH HOẠ cách làm sai (ghi số vi phạm, không fail).

const DRIVER = (code) => `d4000000-0000-4000-8000-${String(code).padStart(12, '0')}`;

// ---------------------------------------------------------------------------
// K040 — nhận chuyến: 8 tài xế (9101…9108) cùng bấm "Nhận" MỖI chuyến, chuyến này tới chuyến kia.
const CHUYEN_DAU = 500001;
const taoChuyen = (co) => `
INSERT INTO trips (id, code, kind, city_id, customer_id, status, fare, requested_at, updated_at, pickup_area_id)
OVERRIDING SYSTEM VALUE
SELECT ('a5000000-0000-4000-8000-' || lpad(g::text, 12, '0'))::uuid, ${CHUYEN_DAU - 1} + g, 'ride', 1,
       ('c4000000-0000-4000-8000-' || lpad((1 + g % 20)::text, 12, '0'))::uuid, 'requested', 40000,
       now(), now(), 111
FROM generate_series(1, ${co.chuyen}) AS g;
ANALYZE trips;`;

const NHAN = {
  // (a) đọc rồi ghi, không khoá, không điều kiện.
  a: async (w, ma, d, { giaoDich, dem }) => giaoDich(w, async () => {
    if ((await w.v(`SELECT status FROM trips WHERE code = ${ma};`)) !== 'requested') return { huy: true, thang: false };
    await w.qx(`UPDATE trips SET status = 'accepted', driver_id = '${d}', accepted_at = now(), updated_at = now(), version = version + 1 WHERE code = ${ma};`);
    await w.qx(`INSERT INTO trip_claims (trip_id, driver_id, method) SELECT id, '${d}', 'a' FROM trips WHERE code = ${ma};`);
    return { thang: true };
  }, { dem }),
  // (b) một câu UPDATE có điều kiện + claim trong cùng câu (autocommit).
  b: async (w, ma, d) => {
    const r = await w.qx(`WITH nhan AS (
        UPDATE trips SET status = 'accepted', driver_id = '${d}', accepted_at = now(), updated_at = now(), version = version + 1
        WHERE code = ${ma} AND status = 'requested' RETURNING id)
      INSERT INTO trip_claims (trip_id, driver_id, method) SELECT id, '${d}', 'b' FROM nhan RETURNING id;`);
    return { ok: true, kq: { thang: r.rows.length === 1 } };
  },
  // (c) SELECT … FOR UPDATE, kiểm trong ứng dụng, rồi ghi.
  c: async (w, ma, d, { giaoDich, dem }) => giaoDich(w, async () => {
    if ((await w.v(`SELECT status FROM trips WHERE code = ${ma} FOR UPDATE;`)) !== 'requested') return { huy: true, thang: false };
    await w.qx(`UPDATE trips SET status = 'accepted', driver_id = '${d}', accepted_at = now(), updated_at = now(), version = version + 1 WHERE code = ${ma};`);
    await w.qx(`INSERT INTO trip_claims (trip_id, driver_id, method) SELECT id, '${d}', 'c' FROM trips WHERE code = ${ma};`);
    return { thang: true };
  }, { dem }),
  // (d) khoá chính trip_assignments làm trọng tài (INSERT … ON CONFLICT DO NOTHING).
  d: async (w, ma, d, { giaoDich, dem }) => giaoDich(w, async () => {
    const r = await w.qx(`INSERT INTO trip_assignments (trip_id, driver_id) SELECT id, '${d}' FROM trips WHERE code = ${ma} ON CONFLICT (trip_id) DO NOTHING RETURNING trip_id;`);
    if (r.rows.length === 0) return { huy: true, thang: false };
    await w.qx(`UPDATE trips SET status = 'accepted', driver_id = '${d}', accepted_at = now(), updated_at = now(), version = version + 1 WHERE code = ${ma};`);
    await w.qx(`INSERT INTO trip_claims (trip_id, driver_id, method) SELECT id, '${d}', 'd' FROM trips WHERE code = ${ma};`);
    return { thang: true };
  }, { dem }),
  // (e) đúng code của (a) nhưng chạy REPEATABLE READ + thử lại khi 40001.
  e: async (w, ma, d, { giaoDich, dem }) => giaoDich(w, async () => {
    if ((await w.v(`SELECT status FROM trips WHERE code = ${ma};`)) !== 'requested') return { huy: true, thang: false };
    await w.qx(`UPDATE trips SET status = 'accepted', driver_id = '${d}', accepted_at = now(), updated_at = now(), version = version + 1 WHERE code = ${ma};`);
    await w.qx(`INSERT INTO trip_claims (trip_id, driver_id, method) SELECT id, '${d}', 'e' FROM trips WHERE code = ${ma};`);
    return { thang: true };
  }, { dem, muc: 'REPEATABLE READ', toi_da: 10 }),
};

function kichBanNhan(cach, ten, moTa, saiCoChuDich = false) {
  const trong = (co) => `code BETWEEN ${CHUYEN_DAU} AND ${CHUYEN_DAU + co.chuyen - 1}`;
  return {
    id: `K040-${cach}`,
    bai: 'PG-040',
    ten,
    mo_ta: moTa,
    co: { chuyen: 1000, phien: 8 },
    co_nho: { chuyen: 60, phien: 4 },
    chuan_bi: taoChuyen,
    chay: async ({ workers, co, dem, giaoDich }) => {
      for (let k = 0; k < co.chuyen; k++) {
        const ma = CHUYEN_DAU + k;
        // "Rào": mọi tài xế cùng bắt đầu chuyến k, chờ nhau xong mới sang chuyến k+1 ⇒ tranh
        // chấp tối đa trên từng chuyến.
        const kq = await Promise.all(workers.map((w, i) => NHAN[cach](w, ma, DRIVER(9101 + i), { giaoDich, dem })));
        const thang = kq.filter((r) => r.ok && r.kq?.thang).length;
        dem.tang('luot_thang', thang);
        dem.tang('luot_thua', kq.length - thang);
        if (thang > 1) dem.tang('chuyen_nhieu_nguoi_thang');
      }
    },
    bat_bien: [
      { ten: 'chuyến có > 1 người thắng', phai_dung: !saiCoChuDich, mong_doi: 0,
        sql: (co) => `SELECT count(*) FROM (SELECT trip_id FROM trip_claims GROUP BY trip_id HAVING count(*) > 1) AS x;` },
      { ten: 'chuyến không ai thắng', phai_dung: true, mong_doi: 0,
        sql: (co) => `SELECT count(*) FROM trips AS t WHERE ${trong(co)} AND NOT EXISTS (SELECT 1 FROM trip_claims AS c WHERE c.trip_id = t.id);` },
      { ten: 'người được báo thắng ≠ trips.driver_id', phai_dung: !saiCoChuDich, mong_doi: 0,
        sql: `SELECT count(*) FROM trip_claims AS c JOIN trips AS t ON t.id = c.trip_id WHERE c.driver_id IS DISTINCT FROM t.driver_id;` },
      { ten: 'tổng lượt "bạn đã nhận chuyến"', sql: 'SELECT count(*) FROM trip_claims;' },
    ],
  };
}

// ---------------------------------------------------------------------------
// K036 — hàng đợi việc: N worker lấy việc tới khi hết.
function kichBanHangDoi(che_do, ten, moTa) {
  const menh_de = { skip: 'FOR UPDATE SKIP LOCKED', cho: 'FOR UPDATE', nowait: 'FOR UPDATE NOWAIT' }[che_do];
  return {
    id: `K036-${che_do}`,
    bai: 'PG-036',
    ten,
    mo_ta: moTa,
    co: { viec: 600, phien: 8, xu_ly_ms: 2 },
    co_nho: { viec: 60, phien: 4, xu_ly_ms: 2 },
    chuan_bi: (co) => `
DELETE FROM jobs;
INSERT INTO jobs (kind, priority) SELECT 'gui_thong_bao', 1 + (g * 7) % 9 FROM generate_series(1, ${co.viec}) AS g;
ANALYZE jobs;`,
    chay: async ({ workers, co, dem, giaoDich, sleep }) => {
      const moiWorker = workers.map(() => 0);
      await Promise.all(workers.map(async (w, i) => {
        for (;;) {
          const r = await giaoDich(w, async () => {
            const id = await w.v(`SELECT id FROM jobs WHERE status = 'queued' ORDER BY priority, id LIMIT 1 ${menh_de};`);
            if (id === null) return { huy: true, rong: true };
            // "xử lý" việc: tốn thời gian khi đang giữ khoá dòng.
            await w.qx(`SELECT pg_sleep(${co.xu_ly_ms / 1000});`);
            await w.qx(`INSERT INTO job_runs (job_id, worker) VALUES (${id}, '${w.label}');`);
            await w.qx(`UPDATE jobs SET status = 'done', done_at = now(), attempts = attempts + 1 WHERE id = ${id};`);
            return { id };
          }, { dem, thu_lai_khi: [] });
          if (!r.ok) { await sleep(1); continue; } // NOWAIT: 55P03 ⇒ thử lại sau 1 ms
          if (r.kq?.rong) {
            if ((await w.v(`SELECT count(*) FROM jobs WHERE status = 'queued';`)) === '0') break;
            dem.tang('rong_gia'); // SKIP LOCKED trả rỗng dù còn việc (đang bị người khác khoá)
            await sleep(1);
            continue;
          }
          dem.tang('xu_ly');
          moiWorker[i]++;
        }
      }));
      dem.tang('it_nhat_moi_worker', Math.min(...moiWorker));
      dem.tang('nhieu_nhat_moi_worker', Math.max(...moiWorker));
    },
    chi_so: (g) => ({ 'việc/giây': Math.round(g.dem.xu_ly / g.giay) }),
    bat_bien: [
      { ten: 'việc chưa xong', phai_dung: true, mong_doi: 0, sql: `SELECT count(*) FROM jobs WHERE status <> 'done';` },
      { ten: 'việc xử lý ≠ 1 lần', phai_dung: true, mong_doi: 0,
        sql: `SELECT count(*) FROM jobs AS j WHERE (SELECT count(*) FROM job_runs AS r WHERE r.job_id = j.id) <> 1;` },
    ],
  };
}

// ---------------------------------------------------------------------------
// K037 — chuyển tiền giữa 4 ví (2001–2004), mỗi phiên nhiều lần, cặp ví và chiều ngẫu nhiên.
function kichBanChuyenTien(thu_tu, ten, moTa) {
  return {
    id: `K037-${thu_tu}`,
    bai: 'PG-037',
    ten,
    mo_ta: moTa,
    co: { lan_moi_phien: 150, phien: 8, vi: 4, nghi_giua_ms: 1 },
    co_nho: { lan_moi_phien: 25, phien: 4, vi: 4, nghi_giua_ms: 1 },
    chay: async ({ workers, co, dem, giaoDich, ngau_nhien }) => {
      for (const w of workers) await w.qx(`SET deadlock_timeout = '20ms';`);
      await Promise.all(workers.map(async (w) => {
        for (let k = 0; k < co.lan_moi_phien; k++) {
          const a = 2001 + Math.floor(ngau_nhien() * co.vi);
          let b = 2001 + Math.floor(ngau_nhien() * (co.vi - 1));
          if (b >= a) b++;
          const r = await giaoDich(w, async () => {
            if (thu_tu === 'theo-id') {
              await w.qx(`SELECT id FROM wallets WHERE id IN (${a}, ${b}) ORDER BY id FOR NO KEY UPDATE;`);
            }
            await w.qx(`UPDATE wallets SET balance = balance - 1000 WHERE id = ${a};`);
            await w.qx(`SELECT pg_sleep(${co.nghi_giua_ms / 1000});`);
            await w.qx(`UPDATE wallets SET balance = balance + 1000 WHERE id = ${b};`);
          }, { dem, toi_da: 20 });
          dem.tang(r.ok ? 'chuyen_xong' : 'chuyen_that_bai');
        }
      }));
    },
    bat_bien: [
      { ten: 'tổng tiền 4 ví', phai_dung: true, mong_doi: 800000,
        sql: 'SELECT sum(balance) FROM wallets WHERE id BETWEEN 2001 AND 2004;' },
      { ten: 'giao dịch chuyển thất bại sau 20 lần thử', phai_dung: true, mong_doi: 0,
        tu_dem: (d) => d.chuyen_that_bai ?? 0 },
      { ten: 'số lần bị huỷ vì deadlock (40P01)', phai_dung: thu_tu === 'theo-id', mong_doi: thu_tu === 'theo-id' ? 0 : undefined,
        tu_dem: (d) => d.loi_40P01 ?? 0 },
    ],
  };
}

// ---------------------------------------------------------------------------
// K038 — xin nghỉ trực: mỗi vòng, đặt lại mọi người trực; 2 tài xế mỗi khu cùng xin nghỉ.
function kichBanTruc(id, muc, ten, moTa, { index = false, toi_da = 10 } = {}) {
  return {
    id,
    bai: 'PG-038',
    ten,
    mo_ta: moTa,
    co: { vong: 200, phien: 8 },
    co_nho: { vong: 20, phien: 4 },
    chay: async ({ dieuPhoi, workers, co, dem, giaoDich }) => {
      if (index) for (const w of workers) await w.qx('SET enable_seqscan = off;');
      for (let v = 0; v < co.vong; v++) {
        await dieuPhoi.qx('UPDATE driver_duty SET on_duty = true WHERE NOT on_duty;');
        await Promise.all(workers.map(async (w, i) => {
          const d = DRIVER(9101 + i);
          const r = await giaoDich(w, async () => {
            const con = Number(await w.v(`SELECT count(*) FROM driver_duty WHERE on_duty AND area_id = (SELECT area_id FROM driver_duty WHERE driver_id = '${d}');`));
            if (con < 2) return { huy: true, duoc_nghi: false };
            await w.qx(`UPDATE driver_duty SET on_duty = false, changed_at = now() WHERE driver_id = '${d}';`);
            return { duoc_nghi: true };
          }, { dem, muc, toi_da, co_so_ms: 2, tran_ms: 50 });
          if (!r.ok) dem.tang('bo_cuoc');
          else dem.tang(r.kq.duoc_nghi ? 'duoc_nghi' : 'tu_choi');
        }));
        const vp = Number(await dieuPhoi.v(`SELECT count(*) FROM (SELECT area_id FROM driver_duty GROUP BY area_id HAVING count(*) FILTER (WHERE on_duty) = 0) AS k;`));
        dem.tang('khu_vi_pham', vp);
      }
    },
    bat_bien: [
      { ten: 'khu không còn ai trực (cộng qua các vòng)', phai_dung: muc === 'SERIALIZABLE', mong_doi: 0,
        tu_dem: (d) => d.khu_vi_pham ?? 0 },
      { ten: 'giao dịch bỏ cuộc sau hết lượt thử', tu_dem: (d) => d.bo_cuoc ?? 0 },
    ],
  };
}

// ---------------------------------------------------------------------------
// K039 — idempotency + outbox. Hàm giữ ĐỒNG BỘ với p04/39-idempotency-outbox.sql §2, §6.
const HAM_039 = `
CREATE FUNCTION dat_chuyen(p_key text, p_customer uuid, p_req jsonb) RETURNS jsonb
LANGUAGE plpgsql AS $$
DECLARE
  v_hash text := md5(p_req::text);
  v_key  idempotency_keys%ROWTYPE;
  v_trip trips%ROWTYPE;
BEGIN
  INSERT INTO idempotency_keys (key, customer_id, request_hash)
  VALUES (p_key, p_customer, v_hash)
  ON CONFLICT (key) DO NOTHING;
  IF NOT FOUND THEN
    SELECT * INTO v_key FROM idempotency_keys WHERE key = p_key;
    IF v_key.request_hash <> v_hash OR v_key.customer_id <> p_customer THEN
      RAISE EXCEPTION 'idempotency key % đã dùng cho một request khác', p_key;
    END IF;
    RETURN v_key.response || jsonb_build_object('lap_lai', true);
  END IF;
  INSERT INTO trips (kind, city_id, customer_id, fare, pickup_area_id)
  VALUES (p_req->>'kind', 1, p_customer, (p_req->>'fare')::integer, (p_req->>'pickup_area_id')::integer)
  RETURNING * INTO v_trip;
  INSERT INTO trip_events (trip_id, status) VALUES (v_trip.id, 'requested');
  INSERT INTO outbox (aggregate_type, aggregate_id, event_type, payload)
  VALUES ('trip', v_trip.id, 'trip_requested', jsonb_build_object('code', v_trip.code, 'customer_id', p_customer));
  UPDATE idempotency_keys
  SET trip_id = v_trip.id, completed_at = now(),
      response = jsonb_build_object('trip_code', v_trip.code, 'status', v_trip.status)
  WHERE key = p_key;
  RETURN jsonb_build_object('trip_code', v_trip.code, 'status', v_trip.status, 'lap_lai', false);
END
$$;
CREATE FUNCTION xu_ly_tin(p_seq bigint, p_consumer text) RETURNS text
LANGUAGE plpgsql AS $$
DECLARE
  m broker_messages%ROWTYPE;
BEGIN
  SELECT * INTO m FROM broker_messages WHERE seq = p_seq;
  INSERT INTO processed_messages (consumer, outbox_id) VALUES (p_consumer, m.outbox_id) ON CONFLICT DO NOTHING;
  IF NOT FOUND THEN
    RETURN 'bo_qua';
  END IF;
  INSERT INTO notifications (outbox_id, customer_id, body)
  VALUES (m.outbox_id, (m.payload->>'customer_id')::uuid, 'Đã nhận yêu cầu chuyến ' || (m.payload->>'code'));
  RETURN 'xu_ly';
END
$$;`;

const kichBanOutbox = {
  id: 'K039',
  bai: 'PG-039',
  ten: 'Request lặp + request chết giữa chừng + publisher chết giữa chừng + 2 consumer',
  mo_ta: `
Ba pha. (1) **Request**: mỗi khoá idempotency được HAI phiên gửi đồng thời (bấm hai lần); với xác suất
\`p_chet\`, một request mở giao dịch, gọi \`dat_chuyen()\` rồi CHẾT trước COMMIT (giết tiến trình psql) —
client sau đó gửi lại cùng khoá. (2) **Publisher**: một nửa số phiên lấy outbox bằng \`FOR UPDATE SKIP LOCKED\`
(lô 5), "gửi" sang broker bằng kết nối RIÊNG (autocommit), rồi đánh dấu \`published_at\`; với xác suất \`p_chet\`
publisher chết sau khi gửi, trước khi đánh dấu ⇒ lô đó bị gửi lại. (3) **Consumer**: HAI consumer cùng nhận MỌI
tin của broker (như broker giao lại cho instance khác), chống trùng bằng \`processed_messages\`.`,
  co: { khoa: 400, phien: 8, p_chet: 0.1 },
  co_nho: { khoa: 40, phien: 4, p_chet: 0.2 },
  chuan_bi: HAM_039,
  chay: async ({ workers, co, dem, giaoDich, ngau_nhien, sleep }) => {
    const reqDung = (k) => `{"kind": "ride", "fare": ${30000 + (k % 50) * 1000}, "pickup_area_id": 111}`;
    const khach = (k) => `c4000000-0000-4000-8000-${String(1 + (k % 20)).padStart(12, '0')}`;
    // Pha 1: request. Mỗi vòng: phien/2 khoá, mỗi khoá 2 phiên.
    const cap = workers.length / 2;
    for (let k0 = 0; k0 < co.khoa; k0 += cap) {
      await Promise.all(workers.map(async (w, i) => {
        const k = k0 + (i % cap);
        if (k >= co.khoa) return;
        const sql = `SELECT dat_chuyen('dat-${String(k).padStart(6, '0')}-k039', '${khach(k)}', '${reqDung(k)}');`;
        if (ngau_nhien() < co.p_chet) {
          await w.qx('BEGIN;');
          await w.q(sql); // có thể phải chờ phiên kia; xong thì chết TRƯỚC COMMIT
          await w.crash();
          await w.reopen();
          dem.tang('request_chet');
        }
        const r = await w.qx(sql); // gửi (lại)
        const kq = JSON.parse(r.rows[0][0]);
        dem.tang(kq.lap_lai ? 'tra_lai_ket_qua_cu' : 'tao_chuyen_moi');
      }));
    }
    // Pha 2: publisher (nửa đầu) + kết nối broker riêng (nửa sau).
    await Promise.all(workers.slice(0, cap).map(async (w, i) => {
      const broker = workers[cap + i];
      for (;;) {
        await w.qx('BEGIN;');
        const ids = (await w.qx('SELECT id FROM outbox WHERE published_at IS NULL ORDER BY id LIMIT 5 FOR UPDATE SKIP LOCKED;')).rows.map((r) => r[0]);
        if (ids.length === 0) {
          await w.qx('ROLLBACK;');
          if ((await w.v('SELECT count(*) FROM outbox WHERE published_at IS NULL;')) === '0') break;
          await sleep(1);
          continue;
        }
        await broker.qx(`INSERT INTO broker_messages (outbox_id, event_type, payload, sent_by)
          SELECT id, event_type, payload, '${w.label}' FROM outbox WHERE id IN (${ids.join(',')}) ORDER BY id;`);
        dem.tang('tin_gui_broker', ids.length);
        if (ngau_nhien() < co.p_chet) {
          await w.crash();
          await w.reopen();
          dem.tang('publisher_chet');
          continue;
        }
        await w.qx(`UPDATE outbox SET published_at = now(), publish_attempts = publish_attempts + 1 WHERE id IN (${ids.join(',')});`);
        await w.qx('COMMIT;');
      }
    }));
    // Pha 3: hai consumer cùng xử lý mọi tin (theo thứ tự seq), song song.
    const seqs = (await workers[0].qx('SELECT seq FROM broker_messages ORDER BY seq;')).rows.map((r) => r[0]);
    await Promise.all(workers.slice(0, 2).map(async (w) => {
      for (const s of seqs) {
        const kq = await w.v(`SELECT xu_ly_tin(${s}, 'thong-bao');`);
        dem.tang(`consumer_${kq}`);
      }
    }));
  },
  bat_bien: [
    { ten: 'khoá không có đúng 1 chuyến', phai_dung: true, mong_doi: 0,
      sql: `SELECT count(*) FROM idempotency_keys AS k WHERE k.key LIKE '%-k039' AND (k.trip_id IS NULL OR k.response IS NULL);` },
    { ten: 'chuyến tạo ra (phải = số khoá)', phai_dung: true, mong_doi: (co) => co.khoa,
      sql: `SELECT count(*) FROM trips WHERE code >= 400200;` },
    { ten: 'chuyến thiếu/thừa sự kiện hoặc outbox', phai_dung: true, mong_doi: 0,
      sql: `SELECT count(*) FROM trips AS t WHERE t.code >= 400200 AND ((SELECT count(*) FROM trip_events AS e WHERE e.trip_id = t.id) <> 1 OR (SELECT count(*) FROM outbox AS o WHERE o.aggregate_id = t.id) <> 1);` },
    { ten: 'outbox chưa gửi', phai_dung: true, mong_doi: 0, sql: 'SELECT count(*) FROM outbox WHERE published_at IS NULL;' },
    { ten: 'sự kiện không có đúng 1 thông báo', phai_dung: true, mong_doi: 0,
      sql: 'SELECT count(*) FROM outbox AS o WHERE (SELECT count(*) FROM notifications AS n WHERE n.outbox_id = o.id) <> 1;' },
    { ten: 'tin TRÙNG ở broker (giao ít nhất một lần)',
      sql: 'SELECT count(*) - count(DISTINCT outbox_id) FROM broker_messages;' },
  ],
};

export default {
  tieu_de: 'P04 · Phép thử đồng thời (PG-036 … PG-040)',
  lenh: 'p04:measure',
  tep: 'p04/dong-thoi.mjs',
  ra: 'p04/DO-DAC.md',
  nen: { manifest: 'p04/lab.json', khoa: 'p04' },
  lap: 3,
  lap_nho: 1,
  phuong_phap: `
**Cách chạy.** Mỗi kịch bản mở N phiên psql THẬT (N kết nối, N backend) tới cluster tạm và cho chúng chạy ĐUA:
Node điều khiển từng phiên qua ống (gửi một câu, chờ kết quả), không có khoảng nghỉ nhân tạo trừ chỗ ghi rõ.
Với K040 và K038, các phiên bắt đầu MỖI chuyến / MỖI vòng cùng lúc ("rào" — chờ nhau xong mới sang chuyến/vòng
sau) để tranh chấp xảy ra trên từng đơn vị. Mỗi LẦN chạy trên một database mới chép từ nền p04. Thử lại phía
client (\`giaoDich()\` trong \`pg/lib/dong-thoi.mjs\`): lỗi 40001/40P01 ⇒ ROLLBACK, chờ ngẫu nhiên
"full jitter" trong [0, min(trần, 2 ms · 2^lần)] rồi chạy lại TOÀN BỘ giao dịch, tối đa số lần ghi ở từng kịch bản.
Cột đếm \`loi_<SQLSTATE>\` đếm MỌI lần câu lệnh lỗi (kể cả lần sau đó thử lại thành công); \`thu_lai\` = số lần
chạy lại; \`het_luot_thu\` = giao dịch bỏ cuộc vì hết lượt. **Bất biến** kiểm bằng SQL sau mỗi lần (hoặc từ bộ
đếm phía client) — đó là phần được \`npm test\` kiểm cứng (bản nhỏ \`--nho\`).`,
  gioi_han: `
- Số đếm (lượt thắng, số lần thử lại, số deadlock, tin trùng, việc/giây) phụ thuộc lịch hệ điều hành, số lõi,
  tải máy và độ trễ ống psql ↔ Node; chạy lại sẽ khác. So SỰ CÓ MẶT/VẮNG MẶT của vi phạm và độ lớn tương đối,
  không so từng con số.
- "Không vi phạm trong N lần" của một cách làm KHÔNG chứng minh cách đó đúng — chỉ cách có lập luận (khoá, ràng
  buộc, SSI) mới bảo đảm; phép thử chỉ là bằng chứng bổ sung. Ngược lại, MỘT lần vi phạm là đủ bác bỏ.
- Mỗi phiên gửi từng câu qua psql (~0,3 ms/lượt trên máy lab): khoảng hở giữa "đọc" và "ghi" lớn hơn một ứng dụng
  gửi lô câu lệnh, nhỏ hơn ứng dụng gọi mạng giữa chừng. Tỉ lệ vi phạm thật ở production có thể cao hoặc thấp hơn.
- Thời gian/thông lượng đo trên một máy, có tiến trình khác chạy nền — chỉ dùng so sánh tương đối trong cùng bảng.
- K036 mô phỏng "xử lý" bằng \`pg_sleep(0,002)\` khi đang giữ khoá; trên Windows bước ngủ thực tế có thể dài hơn
  2 ms (độ phân giải bộ định thời).`,
  kich_ban: [
    kichBanNhan('a', 'Nhận chuyến (a): đọc rồi ghi, không khoá, không điều kiện', `
Đúng code của 40-thuc-hanh-nhan-chuyen.sql cách (a): \`SELECT status\` ⇒ nếu 'requested' thì \`UPDATE\` không
điều kiện + ghi claim. CỐ Ý SAI — bất biến "đúng một người thắng" được đếm số vi phạm để minh hoạ.`, true),
    kichBanNhan('b', 'Nhận chuyến (b): UPDATE … WHERE status = \'requested\'', `
Một câu \`WITH nhan AS (UPDATE … WHERE code = … AND status = 'requested' RETURNING id) INSERT INTO trip_claims …\`
(autocommit).`),
    kichBanNhan('c', 'Nhận chuyến (c): SELECT … FOR UPDATE rồi kiểm rồi ghi', `
Giao dịch Read Committed: \`SELECT status … FOR UPDATE\` ⇒ kiểm trong ứng dụng ⇒ UPDATE + claim.`),
    kichBanNhan('d', 'Nhận chuyến (d): khoá chính trip_assignments làm trọng tài', `
\`INSERT INTO trip_assignments … ON CONFLICT (trip_id) DO NOTHING\`; chèn được (1 dòng) mới UPDATE trips + claim.`),
    kichBanNhan('e', 'Nhận chuyến (e): code (a) nhưng REPEATABLE READ + thử lại', `
Đúng code SAI của (a) nhưng chạy ở REPEATABLE READ, thử lại khi 40001 (tối đa 10 lần): người ghi sau nhận 40001,
chạy lại thì đọc thấy 'accepted' ⇒ thôi. Cơ sở dữ liệu biến lỗi logic thành lỗi có thể thử lại.`),
    kichBanHangDoi('skip', 'Hàng đợi: FOR UPDATE SKIP LOCKED', `
Mỗi worker lặp: giao dịch [lấy việc kế tiếp \`… FOR UPDATE SKIP LOCKED\` ⇒ "xử lý" (pg_sleep) ⇒ ghi job_runs ⇒
đánh dấu done ⇒ COMMIT] tới khi hết việc. \`rong_gia\` = số lần SKIP LOCKED trả rỗng trong khi vẫn còn việc
(đang bị worker khác khoá) — worker phải hỏi lại.`),
    kichBanHangDoi('cho', 'Hàng đợi: FOR UPDATE (chờ)', `
Như trên nhưng \`FOR UPDATE\` (không SKIP LOCKED): mọi worker cùng nhắm vào việc đầu hàng ⇒ xếp hàng chờ nhau.`),
    kichBanHangDoi('nowait', 'Hàng đợi: FOR UPDATE NOWAIT + thử lại sau 1 ms', `
Như trên nhưng \`FOR UPDATE NOWAIT\`: bị khoá ⇒ lỗi 55P03 ngay ⇒ worker ROLLBACK, nghỉ 1 ms rồi hỏi lại.`),
    kichBanChuyenTien('theo-chieu', 'Chuyển tiền: khoá theo chiều chuyển (trừ ví nguồn trước)', `
Mỗi giao dịch: trừ 1.000 đ ví nguồn, nghỉ 1 ms (\`pg_sleep\`, giả lập ứng dụng làm việc giữa hai câu), cộng ví
đích; cặp ví và chiều ngẫu nhiên trong 4 ví. \`deadlock_timeout = 20ms\` (để chạy nhanh); 40P01 ⇒ thử lại ≤ 20 lần.`),
    kichBanChuyenTien('theo-id', 'Chuyển tiền: khoá cả hai ví theo thứ tự id trước', `
Như trên, nhưng mở đầu bằng \`SELECT id FROM wallets WHERE id IN (a, b) ORDER BY id FOR NO KEY UPDATE\`.`),
    kichBanTruc('K038-rr', 'REPEATABLE READ', 'Xin nghỉ trực: REPEATABLE READ (write skew lọt qua)', `
Mỗi vòng: đặt mọi người trực lại; mỗi phiên i (tài xế 9101+i, 2 người một khu) cùng lúc chạy giao dịch
"đếm người đang trực trong khu ⇒ nếu ≥ 2 thì tắt trực của mình". Sau vòng, đếm khu không còn ai trực.`),
    kichBanTruc('K038-ser', 'SERIALIZABLE', 'Xin nghỉ trực: SERIALIZABLE + thử lại (Seq Scan)', `
Như trên ở SERIALIZABLE, thử lại khi 40001 tối đa 10 lần (backoff full jitter, trần 50 ms). Bảng 24 dòng ⇒ planner
chọn Seq Scan ⇒ khoá vị từ cả bảng ⇒ phiên ở KHU KHÁC cũng xung đột (báo động giả) ⇒ nhiều lần thử lại.`),
    kichBanTruc('K038-ser-index', 'SERIALIZABLE', 'Xin nghỉ trực: SERIALIZABLE + thử lại (ép dùng index)', `
Như K038-ser nhưng mỗi phiên \`SET enable_seqscan = off\` ⇒ đọc qua index ix_driver_duty_area ⇒ khoá vị từ
mịn (page index + dòng) ⇒ chỉ hai người CÙNG khu xung đột.`, { index: true }),
    kichBanOutbox,
  ],
};
