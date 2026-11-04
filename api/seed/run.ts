import path from 'node:path';
import { fileURLToPath } from 'node:url';
import '../src/config.js';
import { connectDb, disconnectDb } from '../src/db.js';
import { runSeed } from './lib.js';

const dataDir = path.dirname(fileURLToPath(import.meta.url));

async function main(): Promise<void> {
  const uri = process.env.MONGODB_URI;
  if (!uri) throw new Error('Thiếu MONGODB_URI (xem .env.example)');
  await connectDb(uri);
  try {
    const forceContent = process.argv.includes('--force-content');
    if (forceContent) console.log('[seed] --force-content: ghi đè cả bài đã sửa trong CMS.');
    const r = await runSeed({
      dataDir,
      forceContent,
      admin: {
        email: process.env.SEED_ADMIN_EMAIL,
        password: process.env.SEED_ADMIN_PASSWORD,
        name: process.env.SEED_ADMIN_NAME,
      },
    });
    console.log(
      `[seed] Xong: admin ${r.adminCreated ? 'mới tạo' : 'đã có'}, ${r.categories} chủ đề, ` +
        `${r.articles.created + r.articles.updated} bài (giữ ${r.articles.keptCms} bài đã sửa trong CMS), ${r.learningPaths} lộ trình.`,
    );
  } finally {
    await disconnectDb();
  }
}

main().catch((err: unknown) => {
  console.error('[seed] Lỗi:', err instanceof Error ? err.message : err);
  process.exit(1);
});
