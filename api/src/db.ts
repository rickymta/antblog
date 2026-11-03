import mongoose from 'mongoose';
import './models/index.js';

mongoose.set('strictQuery', true);

export async function connectDb(uri: string): Promise<typeof mongoose> {
  await mongoose.connect(uri, { serverSelectionTimeoutMS: 5000 });
  await ensureIndexes();
  return mongoose;
}

/** Tạo đủ index (unique, TTL, text) trước khi nhận request — chống đếm trùng phụ thuộc vào chúng. */
export async function ensureIndexes(): Promise<void> {
  await Promise.all(Object.values(mongoose.models).map((m) => m.init()));
  await Promise.all(Object.values(mongoose.models).map((m) => m.createIndexes()));
}

export async function disconnectDb(): Promise<void> {
  await mongoose.disconnect();
}
