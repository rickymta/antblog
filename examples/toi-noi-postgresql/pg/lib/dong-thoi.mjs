// Phép thử ĐỒNG THỜI không tất định (P04): N phiên psql thật chạy song song, lặp nhiều lần,
// đếm kết quả (thắng/thua, số lần thử lại, lỗi theo SQLSTATE) rồi KIỂM BẤT BIẾN bằng SQL.
//
// Khác kịch bản nhiều phiên của runner (lịch chạy TẤT ĐỊNH, từng bước một): ở đây các phiên
// thật sự chạy đua nhau. Số đếm dao động giữa các lần chạy ⇒ chỉ BẤT BIẾN được kiểm cứng
// (`phai_dung: true`); số đếm ghi vào tệp kết quả kèm cấu hình máy.
//
// Mỗi phiên làm việc (Worker) là một tiến trình psql (một kết nối) ở chế độ gọn: không in
// thẻ lệnh (QUIET), không tiêu đề cột (tuples_only), cột ngăn bằng '|' (unaligned).
import { PsqlSession } from './session.mjs';
import { transformStderr } from './output.mjs';

/** Lỗi SQL trong một giao dịch (mang SQLSTATE). */
export class LoiSql extends Error {
  constructor(r, sql) {
    const st = transformStderr(r.stderr ?? '');
    super(`${r.sqlstate ?? 'mất kết nối'}: ${st.error?.message ?? r.stderr?.trim() ?? ''} — «${sql.slice(0, 80)}»`);
    this.sqlstate = r.connLost ? 'MAT_KET_NOI' : (st.error?.code ?? r.sqlstate);
    this.connLost = !!r.connLost;
  }
}

export class Worker {
  constructor(cluster, db, label) {
    this.cluster = cluster;
    this.db = db;
    this.label = label;
    this.s = null;
  }

  async open() {
    this.s = await new PsqlSession(this.cluster, this.db, this.label).open();
    for (const c of ['\\set QUIET on', '\\pset format unaligned', '\\pset tuples_only on', "\\pset fieldsep '|'"]) {
      await this.s.run(c);
    }
    await this.s.run('SET search_path TO toinoi, public;');
    return this;
  }

  get pid() {
    return this.s?.pid;
  }

  /** Chạy một câu; trả { rows, rowCount, error, sqlstate, connLost, raw }. Không ném lỗi SQL. */
  async q(sql) {
    const r = await this.s.run(sql);
    const rows = r.stdout.split('\n').filter((l) => l !== '').map((l) => l.split('|'));
    const st = r.error ? transformStderr(r.stderr ?? '') : null;
    return {
      rows, rowCount: r.rowCount, error: !!r.error, connLost: !!r.connLost,
      sqlstate: r.connLost ? 'MAT_KET_NOI' : (st?.error?.code ?? r.sqlstate), raw: r,
    };
  }

  /** Như q() nhưng NÉM LoiSql khi câu lỗi — dùng trong thân giao dịch. */
  async qx(sql) {
    const r = await this.q(sql);
    if (r.error) throw new LoiSql(r.raw, sql);
    return r;
  }

  /** Giá trị ô đầu tiên (hoặc null nếu 0 dòng). */
  async v(sql) {
    const r = await this.qx(sql);
    return r.rows.length ? r.rows[0][0] : null;
  }

  /** "Ứng dụng chết": giết tiến trình psql ⇒ server thấy kết nối đứt ⇒ huỷ giao dịch đang mở. */
  async crash() {
    this.s.kill();
    await this.s.exited;
  }

  /** Mở lại kết nối mới (sau crash). */
  async reopen() {
    if (this.s && !this.s.closed) await this.s.close();
    return this.open();
  }

  async close() {
    if (this.s && !this.s.closed) await this.s.close();
  }
}

export const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

/** Bộ sinh số giả ngẫu nhiên có hạt giống (mulberry32) — tải công việc tái lập được. */
export function rng(seed) {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** Bộ đếm: dem.tang('thang'), dem.tang(`loi_${sqlstate}`). */
export class BoDem {
  constructor() { this.m = new Map(); }
  tang(k, n = 1) { this.m.set(k, (this.m.get(k) ?? 0) + n); }
  lay(k) { return this.m.get(k) ?? 0; }
  obj() { return Object.fromEntries([...this.m].sort(([a], [b]) => a.localeCompare(b))); }
}

/**
 * Chạy fn(w) trong MỘT giao dịch với mức cô lập `muc`. Lỗi SQLSTATE thuộc `thu_lai_khi`
 * (mặc định 40001, 40P01) ⇒ ROLLBACK, chờ backoff "full jitter" (ngẫu nhiên trong
 * [0, min(tran, co_so · 2^lần)] ms) rồi chạy LẠI TOÀN BỘ fn — tối đa `toi_da` lần.
 * fn trả { huy: true } để ROLLBACK chủ động (vd. nghiệp vụ từ chối).
 * Trả { ok, lan, sqlstate?, kq }.
 */
export async function giaoDich(w, fn, {
  muc = 'READ COMMITTED', toi_da = 1, thu_lai_khi = ['40001', '40P01'], co_so_ms = 2, tran_ms = 100,
  dem = null, ngau_nhien = Math.random,
} = {}) {
  for (let lan = 1; ; lan++) {
    try {
      await w.qx(`BEGIN ISOLATION LEVEL ${muc};`);
      const kq = await fn(w, lan);
      if (kq?.huy) {
        await w.qx('ROLLBACK;');
        return { ok: true, lan, kq };
      }
      await w.qx('COMMIT;');
      return { ok: true, lan, kq };
    } catch (e) {
      if (!(e instanceof LoiSql)) throw e;
      if (!e.connLost) await w.q('ROLLBACK;');
      dem?.tang(`loi_${e.sqlstate}`);
      if (thu_lai_khi.includes(e.sqlstate) && lan < toi_da) {
        dem?.tang('thu_lai');
        await sleep(Math.floor(ngau_nhien() * Math.min(tran_ms, co_so_ms * 2 ** lan)));
        continue;
      }
      if (thu_lai_khi.includes(e.sqlstate)) dem?.tang('het_luot_thu');
      return { ok: false, lan, sqlstate: e.sqlstate };
    }
  }
}
