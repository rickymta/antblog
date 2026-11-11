// Lõi chạy tệp lab trên cluster tạm: mỗi "đơn vị" là một database + các phiên
// psql của nó; mỗi tệp là một dãy bước (splitPsql); mỗi bước được in lại, chạy,
// in kết quả kiểu psql và chấm các chỉ thị expect-*.
//
// Nhiều phiên: bước có `-- @A`, `-- @B`… chạy trên phiên tương ứng (mỗi phiên là
// một tiến trình psql riêng = một kết nối riêng). Runner gửi từng bước THEO
// THỨ TỰ TRONG TỆP; nếu một bước không xong mà backend của nó đang chờ khoá
// (pg_stat_activity.wait_event_type = 'Lock'), bước được ghi là "đang chờ" kèm
// phiên đang chặn (pg_blocking_pids), runner chạy tiếp bước sau. Sau mỗi bước
// xong, runner kiểm các bước đang chờ: bước nào đã xong thì in kết quả tại đó.
// Lịch chạy in ra vì thế tất định.
//
// Bổ sung cho P04 (tương thích ngược — tệp P03 cho đầu ra y hệt):
//   * `-- lab: cho-xong`: chờ bước chạy XONG (kể cả khi nó đang chờ khoá) — cho bước sẽ tự
//     kết thúc bằng lỗi sau một khoảng (lock_timeout 55P03, statement_timeout 57014,
//     deadlock 40P01). Nếu đã thấy bước chờ khoá, in "⏳ đã chờ khoá … trước khi xong".
//   * Phiên MẤT KẾT NỐI (psql thoát giữa bước: pg_terminate_backend, idle timeout…) là
//     một kết quả, chấm bằng `-- expect-mat-ket-noi`; bước sau của phiên đó mở kết nối mới.
//   * Tệp nhiều phiên: trong thông báo lỗi và nhật ký server, PID của backend được thay bằng
//     nhãn phiên ([A], [B]…), số giao dịch (xid) bằng <XID>, "after 201.681 ms" bằng
//     "after T ms" — để đầu ra tất định; dòng nhật ký server mang nhãn phiên ghi ra nó.
import { readFileSync } from 'node:fs';
import { relative } from 'node:path';
import { splitPsql, parseDirectives } from './sqlsplit.mjs';
import { PsqlSession } from './session.mjs';
import {
  transformStderr, parseAligned, cellText, resultMd5, normalizeOutput, VOLATILE_BEGIN, VOLATILE_END,
} from './output.mjs';

const POLL_MS = 40;

export function newSummary() {
  return {
    steps: 0, expectedErrors: 0, rowChecks: 0, valueChecks: 0, lineChecks: 0, md5Checks: 0,
    matchChecks: 0, rangeChecks: 0, blockedChecks: 0, failures: [],
  };
}

export class Unit {
  /**
   * @param ctx { cluster, write, echo, baseDir, summary }
   * @param db  tên database của đơn vị
   */
  constructor(ctx, db, title) {
    this.ctx = ctx;
    this.db = db;
    this.title = title;
    this.sessions = new Map();
    this.observer = null;
    this.currentLabel = 'main';
    this.multi = false;
    this.pendingSteps = []; // { label, step, rel, promise, done, result }
    this.failures = [];
    this.pidLabels = new Map(); // pid backend → nhãn phiên (kể cả phiên đã mất kết nối)
    this.lostLabels = new Set(); // phiên vừa mất kết nối: bước sau sẽ mở kết nối mới
  }

  async session(label) {
    let s = this.sessions.get(label);
    if (!s || s.closed) {
      s = await new PsqlSession(this.ctx.cluster, this.db, label).open();
      this.sessions.set(label, s);
      this.pidLabels.set(s.pid, label);
    }
    return s;
  }

  async obs() {
    if (!this.observer || this.observer.closed) {
      this.observer = await new PsqlSession(this.ctx.cluster, this.db, 'quan-sat').open();
      await this.observer.run('\\pset format unaligned');
      await this.observer.run('\\pset tuples_only on');
    }
    return this.observer;
  }

  write(s) {
    this.ctx.write(s);
  }

  fail(where, reason) {
    const f = { file: where.file, line: where.line, reason };
    this.ctx.summary.failures.push(f);
    this.failures.push(f);
    this.write(`-- [lab] FAIL ${where.file}:${where.line}: ${reason}\n`);
  }

  async closeAll() {
    for (const s of this.sessions.values()) await s.close();
    this.sessions.clear();
    if (this.observer) await this.observer.close();
    this.observer = null;
  }

  async runFile(file) {
    const rel = relative(this.ctx.baseDir, file).replace(/\\/g, '/');
    this.write(`\n-- [lab] ===== ${rel}${this.title ? ` · ${this.title}` : ''} =====\n`);
    let steps;
    try {
      steps = splitPsql(readFileSync(file, 'utf8'));
    } catch (e) {
      this.fail({ file: rel, line: 0 }, e.message);
      return;
    }
    // Tệp có chỉ thị @phiên ⇒ chế độ nhiều phiên (in nhãn phiên, theo dõi khoá).
    this.multi = steps.some((s) => /^\s*--\s*@[A-Za-z][A-Za-z0-9_]*\s*$/m.test(s.text));
    for (const step of steps) {
      let d;
      try {
        d = parseDirectives(step.text);
      } catch (e) {
        this.fail({ file: rel, line: step.line }, e.message);
        continue;
      }
      await this.runStep(step, d, rel);
    }
    // Bước còn treo khi hết tệp: chờ thêm một chút rồi báo FAIL.
    for (const p of [...this.pendingSteps]) {
      await this.waitPending(p, 10000);
      if (!p.done) {
        this.fail({ file: rel, line: p.step.line }, `bước của phiên [${p.label}] vẫn chưa xong khi hết tệp (bị chặn?)`);
        const s = this.sessions.get(p.label);
        s?.kill();
        this.sessions.delete(p.label);
        this.pendingSteps.splice(this.pendingSteps.indexOf(p), 1);
      }
    }
  }

  async runStep(step, { exp, lab }, rel) {
    const where = { file: rel, line: step.line };
    const ctx = this.ctx;
    ctx.summary.steps++;
    if (lab.restart) {
      await this.closeAll();
      ctx.cluster.restart();
      this.write('\n-- [lab] Khởi động lại server (pg_ctl restart): shared_buffers trống, mọi phiên mở lại từ đầu\n');
    }
    const label = lab.session ?? this.currentLabel;
    this.currentLabel = label;
    const reopened = this.lostLabels.delete(label);
    let sess = await this.session(label);
    if (reopened) {
      this.write(`\n-- [lab] [${label}] mở kết nối MỚI (kết nối cũ đã mất): phiên mới — SET, biến psql, giao dịch cũ đều không còn\n`);
    }
    if (sess.busy) {
      // Tác giả kịch bản gửi bước mới cho phiên còn đang chờ: đợi nó xong trước.
      const p = this.pendingSteps.find((x) => x.label === label);
      if (p) {
        await this.waitPending(p, 30000);
        if (!p.done) {
          this.fail(where, `phiên [${label}] vẫn bận bước dòng ${p.step.line}, không gửi được bước mới`);
          return;
        }
      }
    }
    if (ctx.echo) {
      this.write(`\n${this.multi ? `-- [lab] [${label}]\n` : ''}${step.text}\n`);
    } else if (this.multi) {
      this.write(`\n-- [lab] [${label}] dòng ${step.line}\n`);
    }
    const logOffset = lab.serverLog ? ctx.cluster.logSize() : 0;
    const promise = sess.send(step.text);
    const rec = { label, step, rel, exp, lab, promise, done: false, result: null, logOffset };
    promise.then((r) => { rec.done = true; rec.result = r; }, (e) => { rec.done = true; rec.result = { crash: e }; });

    if (this.multi && lab.waitDone) {
      const state = await this.waitUntilDone(rec, sess, 60000);
      if (!rec.done) {
        this.fail(where, `lab: cho-xong — bước của phiên [${label}] chưa xong sau 60 s`);
        sess.kill();
        this.sessions.delete(label);
        return;
      }
      if (state.seen) {
        const by = state.blockers.length ? ` — bị chặn bởi ${state.blockers.map((pid) => this.labelOfPid(pid)).join(', ')}` : '';
        this.write(`-- [lab] [${label}] ⏳ đã chờ khoá (${state.waitEvent})${by} trước khi xong\n`);
      }
      if (exp.blocked) {
        ctx.summary.blockedChecks++;
        if (state.seen) this.write('-- [lab] PASS: bước đã phải chờ khoá như mong đợi (expect-blocked)\n');
        else this.fail(where, 'expect-blocked: bước xong mà không thấy chờ khoá');
      }
      this.report(rec);
      await this.settlePending();
      return;
    }
    if (this.multi) {
      const state = await this.waitOrBlocked(rec, sess, lab.waitMs ?? 3000);
      if (state.kind !== 'done') {
        this.pendingSteps.push(rec);
        const by = state.blockers.length
          ? ` — bị chặn bởi ${state.blockers.map((pid) => this.labelOfPid(pid)).join(', ')}`
          : '';
        const what = state.kind === 'blocked'
          ? `đang chờ khoá (${state.waitEvent})${by}`
          : `vẫn đang chạy sau ${lab.waitMs ?? 3000} ms (không chờ khoá)`;
        this.write(`-- [lab] [${label}] ⏳ ${what}\n`);
        if (exp.blocked) {
          ctx.summary.blockedChecks++;
          if (state.kind === 'blocked') this.write('-- [lab] PASS: bước bị chặn như mong đợi (expect-blocked)\n');
          else this.fail(where, 'expect-blocked: bước không chờ khoá mà vẫn đang chạy');
        }
        return;
      }
    } else {
      await promise.catch(() => {});
    }
    if (exp.blocked) {
      ctx.summary.blockedChecks++;
      this.fail(where, 'expect-blocked: bước chạy xong ngay, không bị chặn');
    }
    this.report(rec);
    if (this.multi) await this.settlePending();
  }

  /** Chờ bước xong, hoặc tới khi thấy backend chờ khoá ổn định, hoặc hết giờ. */
  async waitOrBlocked(rec, sess, waitMs) {
    const t0 = Date.now();
    let lockSeen = 0;
    let last = null;
    while (!rec.done) {
      await sleep(POLL_MS);
      if (rec.done) break;
      const st = await this.lockState(sess.pid);
      if (rec.done) break;
      if (st && st.waitType === 'Lock') {
        lockSeen++;
        last = st;
        if (lockSeen >= 3) return { kind: 'blocked', waitEvent: `${st.waitType}: ${st.waitEvent}`, blockers: st.blockers };
      } else lockSeen = 0;
      if (Date.now() - t0 > waitMs) return { kind: 'running', blockers: last?.blockers ?? [] };
    }
    return { kind: 'done' };
  }

  /** `lab: cho-xong`: chờ tới khi bước xong; ghi lại nếu đã thấy nó chờ khoá. */
  async waitUntilDone(rec, sess, maxMs) {
    const t0 = Date.now();
    let seen = null;
    while (!rec.done && Date.now() - t0 < maxMs) {
      await sleep(POLL_MS);
      if (rec.done) break;
      const st = await this.lockState(sess.pid);
      if (st && st.waitType === 'Lock') seen = st;
    }
    return seen
      ? { seen: true, waitEvent: `${seen.waitType}: ${seen.waitEvent}`, blockers: seen.blockers }
      : { seen: false, blockers: [] };
  }

  /** Tên hiển thị của một PID backend: nhãn phiên của tệp, hoặc "[pid khác]". */
  pidName(pid) {
    const l = this.pidLabels.get(pid);
    return l ? `[${l}]` : '[pid khác]';
  }

  /** Che phần không tất định trong thông báo lỗi / nhật ký server của tệp nhiều phiên. */
  maskMulti(text) {
    if (!this.multi || !text) return text;
    return text
      .replace(/(\b[Pp]rocess(?: holding the lock:)? |Wait queue: )(\d+(?:, \d+)*)/g,
        (m, pre, list) => pre + list.split(', ').map((x) => this.pidName(Number(x))).join(', '))
      .replace(/\btransaction \d+/g, 'transaction <XID>')
      .replace(/\bafter \d+(?:\.\d+)? ms\b/g, 'after T ms');
  }

  async lockState(pid) {
    const o = await this.obs();
    const r = await o.run(`SELECT coalesce(wait_event_type, '-') || '|' || coalesce(wait_event, '-') || '|' || coalesce(array_to_string(pg_blocking_pids(pid), ','), '') FROM pg_stat_activity WHERE pid = ${pid};`);
    const line = r.stdout.trim();
    if (!line) return null;
    const [waitType, waitEvent, b] = line.split('|');
    return { waitType, waitEvent, blockers: b ? b.split(',').map(Number) : [] };
  }

  labelOfPid(pid) {
    for (const [label, s] of this.sessions) if (s.pid === pid) return `[${label}]`;
    return `pid khác`;
  }

  async waitPending(p, ms) {
    const t0 = Date.now();
    while (!p.done && Date.now() - t0 < ms) await sleep(POLL_MS);
    if (p.done) {
      this.pendingSteps.splice(this.pendingSteps.indexOf(p), 1);
      this.write(`\n-- [lab] [${p.label}] ✔ bước dòng ${p.step.line} đã chạy xong (sau khi hết bị chặn):\n`);
      this.report(p);
    }
  }

  /** Sau một bước xong: bước nào đang chờ mà nay đã xong thì in ra ngay. */
  async settlePending() {
    for (const p of [...this.pendingSteps]) {
      const s = this.sessions.get(p.label);
      const t0 = Date.now();
      // Cho backend vừa được nhả khoá thời gian chạy nốt; dừng sớm nếu nó vẫn chờ khoá.
      let lockSeen = 0;
      while (!p.done && Date.now() - t0 < 2000) {
        await sleep(POLL_MS);
        if (p.done) break;
        const st = await this.lockState(s.pid);
        if (st && st.waitType === 'Lock') { if (++lockSeen >= 3) break; } else lockSeen = 0;
      }
      if (p.done) await this.waitPending(p, 0);
    }
  }

  report(rec) {
    const ctx = this.ctx;
    const { exp, lab, step } = rec;
    const where = { file: rec.rel, line: step.line };
    const r = rec.result;
    if (r?.crash) {
      this.fail(where, r.crash.message);
      return;
    }
    if (process.env.PG_LAB_TIMING) process.stderr.write(`-- [lab] thời gian ${rec.rel}:${step.line} ${r.ms} ms${'\n'}`);
    if (r.connLost) {
      this.reportLost(rec);
      return;
    }
    const st = transformStderr(r.stderr);
    st.text = this.maskMulti(st.text);
    const parts = [];
    if (lab.volatile || exp.range) parts.push(VOLATILE_BEGIN);
    if (st.text.trim()) parts.push(st.text);
    const so = r.stdout.replace(/\n+$/, '');
    if (so.trim()) parts.push(so);
    if (lab.volatile || exp.range) parts.push(VOLATILE_END);
    if (lab.serverLog) {
      parts.push(this.logLines(rec));
    }
    // PG_LAB_RAW=1: in nguyên văn (giữ thời gian đo) — chỉ để gỡ lỗi, không dùng cho OUTPUT.txt.
    const shown = process.env.PG_LAB_RAW ? parts.join('\n') : normalizeOutput(parts.join('\n'), { root: ctx.cluster.root, port: ctx.cluster.port });
    if (shown) this.write(shown + '\n');

    if (exp.lost) {
      ctx.summary.lostChecks = (ctx.summary.lostChecks ?? 0) + 1;
      this.fail(where, 'expect-mat-ket-noi: bước chạy xong, kết nối không mất');
    }
    const isMeta = step.kind === 'meta';
    const errored = !isMeta && (r.error || !!st.error);
    // --- chấm kỳ vọng
    if (exp.error) {
      ctx.summary.expectedErrors++;
      const want = `${exp.error.code}${exp.error.name ? ' ' + exp.error.name : ''}`;
      if (!errored) this.fail(where, `mong đợi lỗi ${want} nhưng câu lệnh chạy thành công`);
      else {
        const code = st.error?.code ?? r.sqlstate;
        const name = st.error?.constraint ?? st.error?.column ?? null;
        if (code !== exp.error.code || (exp.error.name && exp.error.name !== name)) {
          this.fail(where, `mong đợi lỗi ${want}, nhận ${code} ${name ?? ''}`.trim());
        } else this.write(`-- [lab] PASS: đúng lỗi mong đợi ${want}\n`);
      }
    } else if (errored) {
      this.fail(where, `lỗi ngoài kỳ vọng ${st.error?.code ?? r.sqlstate}: ${st.error?.message ?? ''}`);
    } else if (isMeta && /^(?:invalid command|unrecognized|\\[a-z]+: )/m.test(r.stderr)) {
      this.fail(where, `lệnh meta lỗi: ${r.stderr.trim().split('\n')[0]}`);
    }
    if (errored) return;
    if (exp.rows !== undefined) {
      ctx.summary.rowChecks++;
      if (r.rowCount !== exp.rows) this.fail(where, `mong đợi ${exp.rows} dòng, nhận ${r.rowCount}`);
      else this.write(`-- [lab] PASS: đúng ${exp.rows} dòng như mong đợi\n`);
    }
    const needTable = exp.value !== undefined || exp.lines || exp.md5 || exp.range;
    const table = needTable ? parseAligned(r.stdout) : null;
    if (exp.value !== undefined) {
      ctx.summary.valueChecks++;
      if (!table || table.rows.length !== 1 || table.headers.length !== 1) {
        this.fail(where, `expect-value cần đúng 1 dòng × 1 cột, nhận ${table ? `${table.rows.length} dòng × ${table.headers.length} cột` : 'không có bảng kết quả'}`);
      } else if (cellText(table.rows[0][0]) !== exp.value) {
        this.fail(where, `mong đợi giá trị «${exp.value}», nhận «${cellText(table.rows[0][0])}»`);
      } else this.write(`-- [lab] PASS: đúng giá trị mong đợi ${exp.value}\n`);
    }
    if (exp.range) {
      ctx.summary.rangeChecks++;
      const v = table && table.rows.length === 1 && table.headers.length === 1 ? Number(cellText(table.rows[0][0])) : NaN;
      const [lo, hi] = exp.range;
      if (!(v >= lo && v <= hi)) this.fail(where, `expect-range: mong đợi số trong [${lo}, ${hi}], nhận «${table?.rows?.[0]?.[0] ?? '(không có)'}»`);
      else this.write(`-- [lab] PASS: giá trị nằm trong [${lo}, ${hi}]\n`);
    }
    if (exp.lines) {
      ctx.summary.lineChecks++;
      const got = table ? table.rows.map((row) => row.map(cellText).join(' | ')) : [];
      const k = exp.lines.findIndex((l, j) => got[j] !== l);
      if (got.length !== exp.lines.length || k !== -1) {
        const j = k === -1 ? Math.min(got.length, exp.lines.length) : k;
        this.fail(where, `expect-line: mong đợi ${exp.lines.length} dòng, nhận ${got.length}; khác ở dòng ${j + 1}: mong đợi «${exp.lines[j] ?? '(hết)'}», nhận «${got[j] ?? '(hết)'}»`);
      } else this.write(`-- [lab] PASS: đúng nội dung ${exp.lines.length} dòng như mong đợi (expect-line)\n`);
    }
    if (exp.md5) {
      ctx.summary.md5Checks++;
      const got = table ? resultMd5(table.rows) : '(không có bảng kết quả)';
      if (got !== exp.md5) this.fail(where, `mong đợi md5 ${exp.md5}, nhận ${got}`);
      else this.write(`-- [lab] PASS: đúng md5 kết quả ${exp.md5}\n`);
    }
    if (exp.match || exp.noMatch) {
      const lines = shown.split('\n');
      for (const re of exp.match ?? []) {
        ctx.summary.matchChecks++;
        if (!lines.some((l) => re.test(l))) this.fail(where, `expect-match: không dòng nào khớp /${re.source}/`);
        else this.write(`-- [lab] PASS: có dòng khớp /${re.source}/\n`);
      }
      for (const re of exp.noMatch ?? []) {
        ctx.summary.matchChecks++;
        const hit = lines.find((l) => re.test(l));
        if (hit) this.fail(where, `expect-no-match: dòng «${hit.trim()}» khớp /${re.source}/`);
        else this.write(`-- [lab] PASS: không dòng nào khớp /${re.source}/\n`);
      }
    }
  }
}

Unit.prototype.logLines = function logLines(rec) {
  const added = this.ctx.cluster.readLogFrom(rec.logOffset);
  const lines = added.split('\n')
    .map((l) => l.replace(/^\d{4}-\d\d-\d\d \d\d:\d\d:\d\d(\.\d+)? [+-]\d\d(:?\d\d)? \[(\d+)\] /,
      (m, f, z, pid) => (this.multi ? `${this.pidName(Number(pid))} ` : '')))
    .map((l) => this.maskMulti(l))
    .filter((l) => l.trim());
  if (!lines.length) return '-- [lab] nhật ký server: (không có dòng mới)';
  return lines.map((l) => `-- [lab] nhật ký server: ${l}`).join('\n');
};

/** Phiên mất kết nối giữa bước: in dạng chuẩn (thông báo của libpq/psql không tất định). */
Unit.prototype.reportLost = function reportLost(rec) {
  const ctx = this.ctx;
  const { exp, lab, step } = rec;
  const where = { file: rec.rel, line: step.line };
  const r = rec.result;
  this.lostLabels.add(rec.label);
  this.sessions.delete(rec.label);
  const parts = [];
  const so = r.stdout.replace(/\n+$/, '');
  if (so.trim()) parts.push(normalizeOutput(so, { root: ctx.cluster.root, port: ctx.cluster.port }));
  parts.push(`-- [lab] [${rec.label}] ✖ MẤT KẾT NỐI: psql thoát (mã ${r.exitCode}) — "server closed the connection unexpectedly"; giao dịch đang mở của phiên bị server huỷ (ROLLBACK)`);
  if (lab.serverLog) parts.push(this.logLines(rec));
  this.write(parts.join('\n') + '\n');
  if (exp.lost) {
    ctx.summary.lostChecks = (ctx.summary.lostChecks ?? 0) + 1;
    this.write('-- [lab] PASS: mất kết nối như mong đợi (expect-mat-ket-noi)\n');
  } else {
    this.fail(where, `phiên [${rec.label}] mất kết nối ngoài kỳ vọng (psql thoát mã ${r.exitCode})`);
  }
};

export function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

export function formatSummary(s) {
  const lines = [
    '',
    '-- [lab] ===== Tổng kết =====',
    `-- [lab] Bước đã chạy: ${s.steps}`,
    `-- [lab] Lỗi mong đợi (expect-error): ${s.expectedErrors}`,
    `-- [lab] Kiểm số dòng (expect-rows): ${s.rowChecks}`,
  ];
  if (s.valueChecks) lines.push(`-- [lab] Kiểm giá trị (expect-value): ${s.valueChecks}`);
  if (s.rangeChecks) lines.push(`-- [lab] Kiểm khoảng giá trị (expect-range): ${s.rangeChecks}`);
  if (s.lineChecks) lines.push(`-- [lab] Kiểm nội dung (expect-line): ${s.lineChecks}`);
  if (s.md5Checks) lines.push(`-- [lab] Kiểm md5 kết quả (expect-md5): ${s.md5Checks}`);
  if (s.matchChecks) lines.push(`-- [lab] Kiểm dòng đầu ra (expect-match/no-match): ${s.matchChecks}`);
  if (s.blockedChecks) lines.push(`-- [lab] Kiểm bị chặn (expect-blocked): ${s.blockedChecks}`);
  if (s.lostChecks) lines.push(`-- [lab] Kiểm mất kết nối (expect-mat-ket-noi): ${s.lostChecks}`);
  lines.push(`-- [lab] FAIL: ${s.failures.length}`);
  for (const f of s.failures) lines.push(`-- [lab]   ${f.file}:${f.line} — ${f.reason}`);
  lines.push(s.failures.length === 0 ? '-- [lab] KẾT QUẢ: ĐẠT' : '-- [lab] KẾT QUẢ: KHÔNG ĐẠT');
  return lines.join('\n') + '\n';
}
