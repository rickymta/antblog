import { createApp } from './app.js';
import { getConfig } from './config.js';
import { connectDb, disconnectDb } from './db.js';

async function main(): Promise<void> {
  const config = getConfig();
  await connectDb(config.mongoUri);
  const app = createApp();
  const server = app.listen(config.port, config.host, () => {
    console.log(`[api] Tech Knowledge Hub API đang nghe http://${config.host}:${config.port}/api/v1`);
  });

  const shutdown = (signal: string) => {
    console.log(`[api] Nhận ${signal}, đang dừng...`);
    server.close(() => {
      void disconnectDb().finally(() => process.exit(0));
    });
    setTimeout(() => process.exit(1), 10_000).unref();
  };
  process.on('SIGINT', () => shutdown('SIGINT'));
  process.on('SIGTERM', () => shutdown('SIGTERM'));
}

main().catch((err) => {
  console.error('[api] Không khởi động được:', err);
  process.exit(1);
});
