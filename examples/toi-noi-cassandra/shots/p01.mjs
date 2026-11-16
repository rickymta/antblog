// Kịch bản ảnh của Phần 1 (CS-001 … CS-010). Dữ liệu: trạng thái sau `npm run p01`.
// Ảnh không ghi gì vào database, trừ câu INSERT … IF NOT EXISTS không được áp dụng ([applied] = False).
import { expandTables, fillConnectionDialog, keyspaceMenu, newPage, runCql, shot, tableMenu } from './lib.mjs';

const C1 = '0c000000-0000-4000-8000-000000000001';
const D2 = '0d000000-0000-4000-8000-000000000002';
const T = (n) => `07000000-0000-4000-8000-${String(n).padStart(12, '0')}`;

export default [
  {
    id: 'cs-001/01-cay-keyspace-va-ddl',
    run: async (b, out) => {
      const p = await newPage(b);
      await tableMenu(p, 'trips_by_customer', 'Xem DDL');
      return shot(p, out);
    },
  },
  {
    id: 'cs-002/01-hop-thoai-ket-noi',
    run: async (b, out) => {
      const p = await newPage(b, { width: 1100, height: 900 });
      const d = await fillConnectionDialog(p);
      await d.locator('input[id$="-uri"]').fill('cassandra://127.0.0.1:19042/toinoi');
      return shot(p, out, { locator: d });
    },
  },
  {
    id: 'cs-002/02-kiem-tra-ket-noi',
    run: async (b, out) => {
      const p = await newPage(b, { width: 1100, height: 900 });
      const d = await fillConnectionDialog(p);
      await d.getByRole('button', { name: 'Kiểm tra kết nối' }).click();
      await d.getByText(/Đã kết nối tới/).waitFor({ timeout: 20_000 });
      await d.getByText('TLS').first().scrollIntoViewIfNeeded();
      return shot(p, out, { locator: d });
    },
  },
  {
    id: 'cs-002/03-truy-van-dau-tien',
    run: async (b, out) => {
      const p = await newPage(b);
      await runCql(p, 'SELECT cluster_name, release_version, data_center, rack, cql_version\nFROM system.local;');
      return shot(p, out);
    },
  },
  {
    // Cố ý: `token` là từ khoá CQL nên dùng làm bí danh cột là lỗi cú pháp — bài CS-003 dùng ảnh này.
    id: 'cs-003/01-loi-token-tu-khoa',
    run: async (b, out) => {
      const p = await newPage(b);
      await runCql(p, 'SELECT token(customer_id) AS token, customer_id, requested_at, service, fare_vnd\nFROM trips_by_customer;');
      return shot(p, out);
    },
  },
  {
    id: 'cs-004/01-timestamp-trong-luoi',
    run: async (b, out) => {
      const p = await newPage(b);
      await runCql(
        p,
        `SELECT trip_id, requested_at, toUnixTimestamp(requested_at) AS mili_giay_utc,\n       fare_vnd\nFROM trips\nWHERE trip_id IN (${T(1)}, ${T(4)}, ${T(8)});`,
      );
      return shot(p, out);
    },
  },
  {
    id: 'cs-005/01-writetime-tung-o',
    run: async (b, out) => {
      const p = await newPage(b);
      await runCql(
        p,
        `SELECT trip_id, status, fare_vnd,\n       WRITETIME(status) AS wt_status, WRITETIME(fare_vnd) AS wt_fare\nFROM trips\nWHERE trip_id IN (${T(11)}, ${T(12)});`,
      );
      return shot(p, out);
    },
  },
  {
    id: 'cs-005/02-if-not-exists',
    run: async (b, out) => {
      const p = await newPage(b);
      await runCql(p, `INSERT INTO trips (trip_id, status)\nVALUES (${T(1)}, 'requested')\nIF NOT EXISTS;`);
      return shot(p, out);
    },
  },
  {
    id: 'cs-006/01-loi-allow-filtering',
    run: async (b, out) => {
      const p = await newPage(b);
      await runCql(p, `SELECT requested_at, status\nFROM trips_by_customer\nWHERE status = 'cancelled';`);
      return shot(p, out);
    },
  },
  {
    id: 'cs-006/02-dai-clustering',
    run: async (b, out) => {
      const p = await newPage(b);
      await runCql(
        p,
        `SELECT requested_at, service, fare_vnd\nFROM trips_by_customer\nWHERE customer_id = ${C1}\n  AND requested_at >= '2026-11-02 00:00:00+0700'\n  AND requested_at <  '2026-11-03 00:00:00+0700';`,
      );
      return shot(p, out);
    },
  },
  {
    id: 'cs-006/03-per-partition-limit',
    run: async (b, out) => {
      const p = await newPage(b);
      await runCql(
        p,
        'SELECT customer_id, requested_at, service, fare_vnd\nFROM trips_by_customer\nWHERE customer_id IN (0c000000-0000-4000-8000-000000000001,\n                      0c000000-0000-4000-8000-000000000002,\n                      0c000000-0000-4000-8000-000000000003)\nPER PARTITION LIMIT 1;',
      );
      return shot(p, out);
    },
  },
  {
    id: 'cs-007/01-so-do-keyspace',
    run: async (b, out) => {
      const p = await newPage(b, { width: 1400, height: 860 });
      await keyspaceMenu(p, 'Xem sơ đồ');
      return shot(p, out);
    },
  },
  {
    id: 'cs-007/02-thu-nhap-tai-xe',
    run: async (b, out) => {
      const p = await newPage(b);
      await runCql(
        p,
        `SELECT requested_at, service, status, fare_vnd\nFROM trips_by_driver_day\nWHERE driver_id = ${D2} AND day = '2026-11-03';\n\nSELECT COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc_dong\nFROM trips_by_driver_day\nWHERE driver_id = ${D2} AND day = '2026-11-03';`,
        { all: true },
      );
      return shot(p, out);
    },
  },
  {
    id: 'cs-008/01-doi-soat-bon-bang',
    run: async (b, out) => {
      const p = await newPage(b);
      await runCql(
        p,
        [
          "SELECT (text)'trips' AS bang, COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc FROM trips;",
          "SELECT (text)'trips_by_customer' AS bang, COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc FROM trips_by_customer;",
          "SELECT (text)'trips_by_driver_day' AS bang, COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc FROM trips_by_driver_day;",
          "SELECT (text)'trips_by_city_hour' AS bang, COUNT(*) AS so_chuyen, SUM(fare_vnd) AS tong_cuoc\nFROM trips_by_city_hour WHERE city_id IN (1, 2) ALLOW FILTERING;",
        ].join('\n'),
        { all: true, wait: 3000 },
      );
      return shot(p, out);
    },
  },
  {
    id: 'cs-009/01-dong-moi-partition-gio',
    run: async (b, out) => {
      const p = await newPage(b);
      const h = (x) => `'2026-11-10 ${x}:00:00+0700'`;
      await runCql(
        p,
        `SELECT hour_bucket, COUNT(*) AS so_dong\nFROM trips_by_city_hour\nWHERE city_id = 9\n  AND hour_bucket IN (${['03', '07', '11', '12', '17', '18'].map(h).join(',\n                      ')})\nGROUP BY hour_bucket;`,
        { wait: 3000 },
      );
      return shot(p, out);
    },
  },
  {
    id: 'cs-010/01-du-lieu-dong-thoi-gian',
    run: async (b, out) => {
      const p = await newPage(b);
      await tableMenu(p, 'trip_events_by_trip', 'Mở dữ liệu');
      await p.waitForTimeout(1500);
      return shot(p, out);
    },
  },
  {
    id: 'cs-010/02-dong-thoi-gian-chuyen-huy',
    run: async (b, out) => {
      const p = await newPage(b);
      await expandTables(p);
      await runCql(p, `SELECT event_at, event_seq, status, actor\nFROM trip_events_by_trip\nWHERE trip_id = ${T(5)};`);
      return shot(p, out);
    },
  },
];
