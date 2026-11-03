import dotenv from 'dotenv';

dotenv.config({ quiet: true });

function required(name: string): string {
  const v = process.env[name];
  if (!v) throw new Error(`Thiếu biến môi trường ${name} (xem .env.example)`);
  return v;
}

export interface Config {
  nodeEnv: string;
  isProduction: boolean;
  host: string;
  port: number;
  mongoUri: string;
  jwtSecret: string;
  /**
   * Cookie phiên/khách có cờ Secure. Mặc định = production; chạy HTTP thuần (nginx port 80) phải đặt
   * COOKIE_SECURE=false, nếu không trình duyệt bỏ cookie và không ai đăng nhập được.
   */
  cookieSecure: boolean;
  /** Giá trị `trust proxy` của Express — để rate limit tính theo IP thật của người dùng khi đứng sau nginx. */
  trustProxy: boolean | number | string;
}

function parseBool(value: string | undefined, fallback: boolean): boolean {
  if (value === undefined || value === '') return fallback;
  return value === 'true' || value === '1';
}

/** TRUST_PROXY: `true`/`false`, số hop (`1`), hoặc danh sách địa chỉ/subnet (`loopback`, `172.30.0.0/24`). */
function parseTrustProxy(value: string | undefined): boolean | number | string {
  if (value === undefined || value === '') return 'loopback';
  if (value === 'true' || value === 'false') return value === 'true';
  return /^\d+$/.test(value) ? Number(value) : value;
}

let cached: Config | undefined;

/** Đọc cấu hình lười để test có thể đặt biến môi trường trước khi dùng. */
export function getConfig(): Config {
  if (cached) return cached;
  const jwtSecret = required('JWT_SECRET');
  if (Buffer.byteLength(jwtSecret) < 32) throw new Error('JWT_SECRET quá ngắn (cần ≥ 32 byte, khuyến nghị ≥ 48)');
  const nodeEnv = process.env.NODE_ENV ?? 'development';
  cached = {
    nodeEnv,
    isProduction: nodeEnv === 'production',
    host: process.env.HOST ?? '127.0.0.1',
    port: Number(process.env.PORT ?? 4000),
    mongoUri: required('MONGODB_URI'),
    jwtSecret,
    cookieSecure: parseBool(process.env.COOKIE_SECURE, nodeEnv === 'production'),
    trustProxy: parseTrustProxy(process.env.TRUST_PROXY),
  };
  return cached;
}
