import { Schema, model, type Types } from 'mongoose';

export const ROLES = ['reader', 'author', 'admin'] as const;
export type Role = (typeof ROLES)[number];
export const USER_STATUSES = ['active', 'locked'] as const;
export type UserStatus = (typeof USER_STATUSES)[number];

export interface IUser {
  _id: Types.ObjectId;
  email: string;
  passwordHash: string;
  displayName: string;
  avatarUrl?: string;
  bio?: string;
  role: Role;
  status: UserStatus;
  lastLoginAt?: Date;
  isDemo?: boolean;
  createdAt: Date;
  updatedAt: Date;
}

const userSchema = new Schema<IUser>(
  {
    email: { type: String, required: true, unique: true, lowercase: true, trim: true },
    passwordHash: { type: String, required: true },
    displayName: { type: String, required: true, trim: true },
    avatarUrl: String,
    bio: String,
    role: { type: String, enum: ROLES, default: 'reader', index: true },
    status: { type: String, enum: USER_STATUSES, default: 'active' },
    lastLoginAt: Date,
    isDemo: Boolean,
  },
  { timestamps: true },
);

export const User = model<IUser>('User', userSchema);
