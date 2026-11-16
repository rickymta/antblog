/**
 * TCD-04 — phân số chính xác bằng BigInt, để oracle tính tiền KHÔNG dùng số thực dấu phẩy động
 * và cũng không dùng chung đơn vị «milli-đồng» với mã sản phẩm (src/domain/money.ts).
 */
export class PhanSo {
  readonly tu: bigint;
  readonly mau: bigint;

  private constructor(tu: bigint, mau: bigint) {
    if (mau === 0n) throw new Error('Mẫu số bằng 0');
    if (mau < 0n) { tu = -tu; mau = -mau; }
    const g = ucln(tu < 0n ? -tu : tu, mau);
    this.tu = tu / g;
    this.mau = mau / g;
  }

  static of(tu: bigint | number, mau: bigint | number = 1n): PhanSo {
    return new PhanSo(BigInt(tu), BigInt(mau));
  }

  /** Đọc số thập phân từ chuỗi («7.35», «17812.5») thành phân số đúng tuyệt đối. */
  static tuChuoi(s: string): PhanSo {
    const m = /^(-?)(\d+)(?:\.(\d+))?$/.exec(s.trim());
    if (!m) throw new Error(`Không đọc được số thập phân "${s}"`);
    const phanLe = m[3] ?? '';
    const tu = BigInt(m[2] + phanLe) * (m[1] === '-' ? -1n : 1n);
    return new PhanSo(tu, 10n ** BigInt(phanLe.length));
  }

  /** Số JavaScript (như JSON trả về) ⇒ phân số, qua dạng chuỗi ngắn nhất của số đó. */
  static tuSo(x: number): PhanSo {
    if (!Number.isFinite(x)) throw new Error(`Không phải số hữu hạn: ${x}`);
    return PhanSo.tuChuoi(String(x));
  }

  cong(b: PhanSo): PhanSo { return new PhanSo(this.tu * b.mau + b.tu * this.mau, this.mau * b.mau); }
  tru(b: PhanSo): PhanSo { return new PhanSo(this.tu * b.mau - b.tu * this.mau, this.mau * b.mau); }
  nhan(b: PhanSo): PhanSo { return new PhanSo(this.tu * b.tu, this.mau * b.mau); }
  so(b: PhanSo): number { const d = this.tu * b.mau - b.tu * this.mau; return d < 0n ? -1 : d > 0n ? 1 : 0; }
  bang(b: PhanSo): boolean { return this.so(b) === 0; }
  nhoHon(b: PhanSo): boolean { return this.so(b) < 0; }
  lonNhat(b: PhanSo): PhanSo { return this.so(b) >= 0 ? this : b; }
  nhoNhat(b: PhanSo): PhanSo { return this.so(b) <= 0 ? this : b; }

  /** Làm tròn lên số nguyên (trần) — chỉ dùng cho số không âm. */
  tran(): bigint { return (this.tu + this.mau - 1n) / this.mau; }
  /** Làm tròn half-up tới số nguyên — chỉ dùng cho số không âm. */
  lamTronNuaLen(): bigint { return (2n * this.tu + this.mau) / (2n * this.mau); }

  /** Dạng thập phân Việt Nam để in bảng: 17812.5 ⇒ «17.812,5». */
  toString(): string {
    const nguyen = this.tu / this.mau;
    let du = this.tu % this.mau;
    let le = '';
    for (let i = 0; du !== 0n && i < 6; i++) { du *= 10n; le += (du / this.mau).toString(); du %= this.mau; }
    const phanNguyen = nguyen.toString().replace(/\B(?=(\d{3})+(?!\d))/g, '.');
    return le ? `${phanNguyen},${le}` : phanNguyen;
  }
}

function ucln(a: bigint, b: bigint): bigint {
  while (b !== 0n) [a, b] = [b, a % b];
  return a === 0n ? 1n : a;
}
