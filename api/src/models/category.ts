import { Schema, model, type Types } from 'mongoose';

export const CATEGORY_ICONS = ['database', 'code', 'pattern', 'architecture', 'devops', 'system', 'security', 'testing'] as const;
export type CategoryIcon = (typeof CATEGORY_ICONS)[number];

export interface Localized {
  vi: string;
  en: string;
}

export interface ICategory {
  _id: Types.ObjectId;
  slug: string;
  name: Localized;
  description: Localized;
  icon: CategoryIcon;
  color: string;
  order: number;
  articleCount: number;
  createdAt: Date;
  updatedAt: Date;
}

const localized = { vi: { type: String, default: '' }, en: { type: String, default: '' } };

const categorySchema = new Schema<ICategory>(
  {
    slug: { type: String, required: true, unique: true },
    name: localized,
    description: localized,
    icon: { type: String, enum: CATEGORY_ICONS, required: true },
    color: { type: String, required: true },
    order: { type: Number, default: 0 },
    articleCount: { type: Number, default: 0 },
  },
  { timestamps: true },
);

export const Category = model<ICategory>('Category', categorySchema);
