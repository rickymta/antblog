import '../src/config.js';
import { connectDb, disconnectDb } from '../src/db.js';
import { createDemo, removeDemo } from './demoLib.js';

async function main(): Promise<void> {
  const uri = process.env.MONGODB_URI;
  if (!uri) throw new Error('Thiếu MONGODB_URI (xem .env.example)');
  await connectDb(uri);
  try {
    if (process.argv.includes('--remove')) await removeDemo();
    else await createDemo();
  } finally {
    await disconnectDb();
  }
}

main().catch((err: unknown) => {
  console.error('[demo] Lỗi:', err instanceof Error ? err.message : err);
  process.exit(1);
});
