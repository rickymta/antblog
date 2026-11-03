import { Schema, model, type Types } from 'mongoose';
import { LEVELS, type Level } from './article.js';

export const PATH_STATUSES = ['draft', 'published'] as const;
export type PathStatus = (typeof PATH_STATUSES)[number];

export interface ILearningPath {
  _id: Types.ObjectId;
  slug: string;
  title: string;
  description: string;
  level: Level;
  categoryId?: Types.ObjectId;
  articleIds: Types.ObjectId[];
  status: PathStatus;
  createdAt: Date;
  updatedAt: Date;
}

const learningPathSchema = new Schema<ILearningPath>(
  {
    slug: { type: String, required: true, unique: true },
    title: { type: String, required: true },
    description: { type: String, default: '' },
    level: { type: String, enum: LEVELS, required: true },
    categoryId: { type: Schema.Types.ObjectId, ref: 'Category' },
    articleIds: { type: [{ type: Schema.Types.ObjectId, ref: 'Article' }], default: [] },
    status: { type: String, enum: PATH_STATUSES, default: 'draft' },
  },
  { timestamps: true },
);
learningPathSchema.index({ articleIds: 1 });

export const LearningPath = model<ILearningPath>('LearningPath', learningPathSchema);
