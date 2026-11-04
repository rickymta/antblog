import { Schema, model, type Types } from 'mongoose';

/** Loại ảnh được nhận (hợp đồng §8.2) — KHÔNG có SVG (chạy được script). */
export const MEDIA_MIME_TYPES = ['image/png', 'image/jpeg', 'image/webp', 'image/gif'] as const;
export type MediaMimeType = (typeof MEDIA_MIME_TYPES)[number];

export interface IMedia {
  _id: Types.ObjectId;
  /** Tên tệp gốc người dùng tải lên (đã làm sạch) — để hiển thị và tìm kiếm. */
  fileName: string;
  /** Đường dẫn tương đối trong UPLOAD_DIR: `YYYY/MM/<ngẫu nhiên>.<ext>`. */
  storagePath: string;
  mimeType: MediaMimeType;
  size: number;
  width?: number;
  height?: number;
  alt?: string;
  uploadedBy: Types.ObjectId;
  createdAt: Date;
  updatedAt: Date;
}

const mediaSchema = new Schema<IMedia>(
  {
    fileName: { type: String, required: true },
    storagePath: { type: String, required: true, unique: true },
    mimeType: { type: String, enum: MEDIA_MIME_TYPES, required: true },
    size: { type: Number, required: true },
    width: Number,
    height: Number,
    alt: String,
    uploadedBy: { type: Schema.Types.ObjectId, ref: 'User', required: true },
  },
  { timestamps: true },
);
mediaSchema.index({ createdAt: -1, _id: -1 });
mediaSchema.index({ uploadedBy: 1, createdAt: -1 });

export const Media = model<IMedia>('Media', mediaSchema);
