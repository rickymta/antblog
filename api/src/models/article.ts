import { Schema, model, type Types } from 'mongoose';

export const LEVELS = ['beginner', 'intermediate', 'advanced'] as const;
export type Level = (typeof LEVELS)[number];
export const LANGUAGES = ['vi', 'en'] as const;
export type Language = (typeof LANGUAGES)[number];
export const ARTICLE_STATUSES = ['draft', 'published', 'archived'] as const;
export type ArticleStatus = (typeof ARTICLE_STATUSES)[number];
export const REACTION_TYPES = ['love', 'helpful', 'insightful', 'inspiring'] as const;
export type ReactionType = (typeof REACTION_TYPES)[number];
export type ReactionCounts = Record<ReactionType, number>;

/** Ghi đè SEO của bài (hợp đồng §8.1) — trường vắng = dùng mặc định. */
export interface IArticleSeo {
  title?: string;
  description?: string;
  ogImageUrl?: string;
  noindex?: boolean;
}

export interface IArticle {
  _id: Types.ObjectId;
  slug: string;
  title: string;
  summary: string;
  content: string;
  coverImageUrl?: string;
  language: Language;
  categoryId: Types.ObjectId;
  level: Level;
  tags: string[];
  authorId: Types.ObjectId;
  readingMinutes: number;
  status: ArticleStatus;
  publishedAt?: Date;
  learningPathId?: Types.ObjectId;
  seo?: IArticleSeo;
  /** true khi bài đã được sửa trong CMS — `npm run seed` không ghi đè nội dung nữa (hợp đồng §8.4). */
  editedInCms?: boolean;
  stats: { views: number; comments: number; reactions: ReactionCounts };
  createdAt: Date;
  updatedAt: Date;
}

const articleSchema = new Schema<IArticle>(
  {
    slug: { type: String, required: true, unique: true },
    title: { type: String, required: true },
    summary: { type: String, default: '' },
    content: { type: String, default: '' },
    coverImageUrl: String,
    language: { type: String, enum: LANGUAGES, default: 'vi' },
    categoryId: { type: Schema.Types.ObjectId, ref: 'Category', required: true, index: true },
    level: { type: String, enum: LEVELS, required: true },
    tags: { type: [String], default: [] },
    authorId: { type: Schema.Types.ObjectId, ref: 'User', required: true, index: true },
    readingMinutes: { type: Number, default: 1 },
    status: { type: String, enum: ARTICLE_STATUSES, default: 'draft' },
    publishedAt: Date,
    learningPathId: { type: Schema.Types.ObjectId, ref: 'LearningPath' },
    seo: {
      type: new Schema<IArticleSeo>(
        { title: String, description: String, ogImageUrl: String, noindex: Boolean },
        { _id: false },
      ),
      default: undefined,
    },
    editedInCms: Boolean,
    stats: {
      views: { type: Number, default: 0 },
      comments: { type: Number, default: 0 },
      reactions: {
        love: { type: Number, default: 0 },
        helpful: { type: Number, default: 0 },
        insightful: { type: Number, default: 0 },
        inspiring: { type: Number, default: 0 },
      },
    },
  },
  { timestamps: true },
);

articleSchema.index({ status: 1, publishedAt: -1 });
articleSchema.index({ status: 1, 'stats.views': -1 });
articleSchema.index({ tags: 1 });
// Trường `language` ('vi'/'en') trùng tên trường ghi đè ngôn ngữ của text index ⇒ đổi sang một trường không tồn tại.
articleSchema.index(
  { title: 'text', summary: 'text', tags: 'text' },
  {
    weights: { title: 10, tags: 5, summary: 2 },
    default_language: 'none',
    language_override: 'textSearchLanguage',
    name: 'article_text',
  },
);

export const Article = model<IArticle>('Article', articleSchema);
