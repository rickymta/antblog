/** Áp cây chủ đề trong seed/categories.json (thêm chủ đề thiếu + quan hệ cha–con), không seed lại, không di chuyển bài viết. Mặc định chạy thử (dry-run). */
import 'dotenv/config';
import mongoose from 'mongoose';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { Category, type ICategory } from '../src/models/category.js';
import { validateCategoryTree } from '../src/services/categories.js';

const apply = process.argv.includes('--apply');
const base = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const uri = process.env.MONGODB_URI;
if (!uri) throw new Error('MONGODB_URI is required. Pass it through the environment, never source control.');
try {
  await mongoose.connect(uri, { autoIndex: false, autoCreate: false, serverSelectionTimeoutMS: 5000 });
  const seed = JSON.parse(await readFile(path.join(base, 'seed/categories.json'), 'utf8')) as Array<Omit<ICategory, '_id'> & { parent?: string }>;
  const before = await Category.find({}).lean<ICategory[]>();
  const backupJson = JSON.stringify(before, null, 2);
  const bySlug = new Map(before.map(c => [c.slug, c]));
  // Dữ liệu lấy từ seed/categories.json: thêm chủ đề còn thiếu trong DB, rồi áp mọi cạnh cha–con. Không đụng bài viết.
  const additions = seed.filter(c => !bySlug.has(c.slug));
  for (const c of additions) bySlug.set(c.slug, { ...c, _id: new mongoose.Types.ObjectId(), parentId: null, articleCount: 0 });
  const edges = seed.filter(c => c.parent).map(c => [c.slug, c.parent!] as const);
  for (const [childSlug, parentSlug] of edges) {
    const child = bySlug.get(childSlug);
    const parent = bySlug.get(parentSlug);
    if (!child || !parent) throw new Error(`Missing category: ${childSlug} or ${parentSlug}.`);
    child.parentId = parent._id;
  }
  const proposed = [...bySlug.values()];
  // Validate the entire final state before the first write.
  for (const c of proposed) validateCategoryTree(proposed, String(c._id), c.parentId ? String(c.parentId) : null);
  console.log(JSON.stringify({ mode: apply ? 'apply' : 'dry-run', database: mongoose.connection.name, create: additions.map(c => c.slug), edges }, null, 2));
  if (apply) {
    const backupDir = path.join(base, '.migration-backups');
    await mkdir(backupDir, { recursive: true });
    const backup = path.join(backupDir, `categories-${Date.now()}.json`);
    await writeFile(backup, backupJson);
    // Insert new nodes with their final parent; only update the agreed edges on existing nodes.
    const operations = additions.map(c => {
      const final = bySlug.get(c.slug)!;
      const { parent: _parentSlug, ...record } = final as ICategory & { parent?: string };
      return { updateOne: { filter: { slug: c.slug }, update: { $setOnInsert: record }, upsert: true } };
    });
    if (operations.length) await Category.bulkWrite(operations);
    for (const [childSlug, parentSlug] of edges) {
      const parent = await Category.findOne({ slug: parentSlug }).select('_id');
      await Category.updateOne({ slug: childSlug }, { $set: { parentId: parent!._id } });
    }
    console.log(`Applied category hierarchy. Backup: ${backup}`);
  }
} finally {
  await mongoose.disconnect();
}
