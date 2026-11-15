import { Schema, model, type Types } from 'mongoose';
import { LANGUAGES, type Language } from './article.js';

export const TERM_STATUSES = ['draft', 'published'] as const;
export type TermStatus = (typeof TERM_STATUSES)[number];

/**
 * Một mục của từ điển keyword: khi xuất hiện trong bài, keyword được gạch chân và mở hộp thoại giải thích.
 * Đặc tả: docs/content-dictionary/02-glossary-feature.md. Dữ liệu khởi tạo: `seed/terminology.json` (mục `glossary: true`).
 */
export interface ITerm {
  _id: Types.ObjectId;
  /** Duy nhất theo `language`. */
  slug: string;
  language: Language;
  /** Dạng hiển thị chính, cũng là dạng được dò trong bài. */
  term: string;
  /** Tên gốc tiếng Anh khi `term` là tiếng Việt. */
  english?: string;
  /** Các cách viết khác cũng được dò. */
  aliases: string[];
  /** ≤ 300 ký tự, văn bản thuần — nội dung hộp thoại. */
  shortDefinition: string;
  /** Markdown dài cho trang /glossary/<slug> (giai đoạn 5). */
  body?: string;
  /** true cho từ viết tắt dễ trùng từ thường ("Go", "CAP"). */
  caseSensitive: boolean;
  /** false = chỉ đánh dấu khi tác giả gắn thủ công `<dfn data-term>`. */
  autoLink: boolean;
  status: TermStatus;
  /** true khi đã sửa trong CMS — `npm run seed` không ghi đè nữa (cùng quy ước với bài viết). */
  editedInCms?: boolean;
  createdAt: Date;
  updatedAt: Date;
}

const termSchema = new Schema<ITerm>(
  {
    slug: { type: String, required: true },
    language: { type: String, enum: LANGUAGES, default: 'vi' },
    term: { type: String, required: true },
    english: String,
    aliases: { type: [String], default: [] },
    shortDefinition: { type: String, required: true },
    body: String,
    caseSensitive: { type: Boolean, default: false },
    autoLink: { type: Boolean, default: true },
    status: { type: String, enum: TERM_STATUSES, default: 'published' },
    editedInCms: Boolean,
  },
  { timestamps: true },
);
termSchema.index({ language: 1, slug: 1 }, { unique: true });
termSchema.index({ language: 1, status: 1 });

export const Term = model<ITerm>('Term', termSchema);
