// Một phiên psql điều khiển qua ống (stdin/stdout/stderr).
//
// Mỗi bước được gửi cho psql kèm hai mốc đánh dấu:
//   \echo @@LAB:<token>:E:<n> :ERROR :SQLSTATE :ROW_COUNT    (ra stdout)
//   \warn @@LAB:<token>:W:<n>                                (ra stderr)
// Bước xong khi đã nhận đủ hai mốc. Phần stdout trước mốc E là kết quả (bảng,
// thẻ lệnh), phần stderr trước mốc W là NOTICE/ERROR của chính bước đó — tách
// được chính xác dù hai ống đến Node theo thứ tự nào. psql đẩy (flush) stdout
// sau mỗi câu lệnh nên chờ từng bước không bị kẹt bộ đệm.
//
// Phiên mở với: -X (bỏ ~/.psqlrc), ON_ERROR_STOP=0 (lỗi không dừng phiên),
// VERBOSITY=verbose (để đọc SQLSTATE và tên ràng buộc — output.mjs chuyển lại về
// dạng ngắn khi in), \pset null NULL (NULL hiện chữ NULL như run.mjs).
import { spawn } from 'node:child_process';
import { randomBytes } from 'node:crypto';
import { exe } from './bin.mjs';

export class PsqlSession {
  constructor(cluster, db, label = 'main') {
    this.cluster = cluster;
    this.db = db;
    this.label = label;
    this.token = randomBytes(4).toString('hex');
    this.seq = 0;
    this.out = '';
    this.err = '';
    this.pending = null; // { n, resolve, reject, gotE, gotW, stdout, stderr, vars }
    this.closed = false;
    this.pid = null;
  }

  async open() {
    const c = this.cluster;
    this.proc = spawn(exe(c.pg.dir, 'psql'), [
      '-X', '-h', c.host, '-p', String(c.port), '-U', 'postgres', '-d', this.db,
      '-v', 'ON_ERROR_STOP=0', '-v', 'VERBOSITY=verbose', '-P', 'null=NULL', '-P', 'pager=off',
    ], { env: { ...c.env, PGAPPNAME: `lab-${this.label}` }, windowsHide: true });
    this.proc.stdout.setEncoding('utf8');
    this.proc.stderr.setEncoding('utf8');
    this.proc.stdout.on('data', (d) => { this.out += d.replace(/\r\n/g, '\n'); this.#scan(); });
    this.proc.stderr.on('data', (d) => { this.err += d.replace(/\r\n/g, '\n'); this.#scan(); });
    this.proc.stdin.on('error', () => { /* phiên đã đóng */ });
    // 'close' (không phải 'exit'): chờ cả hai ống đọc xong để không mất dòng stderr cuối.
    this.exited = new Promise((resolve) => {
      this.proc.on('close', (code) => {
        this.closed = true;
        this.exitCode = code;
        if (this.pending) {
          // psql thoát giữa bước = MẤT KẾT NỐI (vd. backend bị pg_terminate_backend, server
          // dừng, idle_in_transaction_session_timeout). Ở chế độ không tương tác psql tự thoát
          // mã 2 ("connection to server was lost"). Trả về như một kết quả (connLost) để
          // runner chấm bằng chỉ thị expect-mat-ket-noi thay vì coi là lỗi runner.
          const p = this.pending;
          this.pending = null;
          p.resolve({
            stdout: this.out, stderr: this.err, error: true, sqlstate: null, rowCount: null,
            connLost: true, exitCode: code, ms: Date.now() - p.t0,
          });
          this.out = '';
          this.err = '';
        }
        resolve(code);
      });
    });
    const r = await this.run('SELECT pg_backend_pid() AS lab_pid \\gset');
    if (r.error) throw new Error(`Không mở được phiên psql ${this.label} tới ${this.db}: ${r.stderr}`);
    const p = await this.run('\\echo :lab_pid');
    this.pid = Number(p.stdout.trim());
    return this;
  }

  #scan() {
    const p = this.pending;
    if (!p) return;
    const tag = `@@LAB:${this.token}:`;
    if (!p.gotE) {
      const re = new RegExp(`(^|\\n)${tag}E:${p.n} (\\S*) (\\S*) (\\S*)\\n`);
      const m = re.exec(this.out);
      if (m) {
        p.gotE = true;
        p.stdout = this.out.slice(0, m.index + m[1].length);
        p.vars = { ERROR: m[2], SQLSTATE: m[3], ROW_COUNT: m[4] };
        this.out = this.out.slice(m.index + m[0].length);
      }
    }
    if (!p.gotW) {
      const re = new RegExp(`(^|\\n)${tag}W:${p.n}\\n`);
      const m = re.exec(this.err);
      if (m) {
        p.gotW = true;
        p.stderr = this.err.slice(0, m.index + m[1].length);
        this.err = this.err.slice(m.index + m[0].length);
      }
    }
    if (p.gotE && p.gotW) {
      this.pending = null;
      p.resolve({
        stdout: p.stdout,
        stderr: p.stderr,
        error: p.vars.ERROR === 'true',
        sqlstate: p.vars.SQLSTATE,
        rowCount: /^\d+$/.test(p.vars.ROW_COUNT) ? Number(p.vars.ROW_COUNT) : null,
        ms: Date.now() - p.t0,
      });
    }
  }

  /** Gửi một bước; trả Promise kết quả. Không gửi bước mới khi bước cũ chưa xong. */
  send(text) {
    if (this.closed) return Promise.reject(new Error(`phiên ${this.label} đã đóng`));
    if (this.pending) return Promise.reject(new Error(`phiên ${this.label} đang bận bước ${this.pending.n}`));
    const n = ++this.seq;
    let body = text.replace(/\r\n/g, '\n');
    // Bước SQL cuối tệp không có dấu ; ⇒ thêm để psql chạy (giống run.mjs chạy câu cuối).
    if (!/;\s*$/.test(stripTrailingComment(body)) && !/\\[^\n]*$/.test(body) && !/\n\\\.\s*$/.test(body)) body += '\n;';
    const tag = `@@LAB:${this.token}:`;
    const payload = `${body}\n\\echo ${tag}E:${n} :ERROR :SQLSTATE :ROW_COUNT\n\\warn ${tag}W:${n}\n`;
    const promise = new Promise((resolve, reject) => {
      this.pending = { n, resolve, reject, gotE: false, gotW: false, t0: Date.now() };
    });
    this.proc.stdin.write(payload);
    return promise;
  }

  run(text) {
    return this.send(text);
  }

  get busy() {
    return this.pending !== null;
  }

  async close() {
    if (this.closed) return;
    try { this.proc.stdin.end('\\q\n'); } catch { /* bỏ qua */ }
    const t = setTimeout(() => { try { this.proc.kill(); } catch { /* bỏ qua */ } }, 5000);
    await this.exited;
    clearTimeout(t);
  }

  kill() {
    try { this.proc.kill(); } catch { /* bỏ qua */ }
  }
}

function stripTrailingComment(body) {
  // Bỏ chú thích -- ở cuối các dòng để kiểm dấu ; kết thúc (đủ dùng cho tệp lab).
  return body.split('\n').map((l) => l.replace(/--.*$/, '')).join('\n');
}
