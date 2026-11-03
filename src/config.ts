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
  };
  return cached;
}
