// Kiểu dữ liệu theo docs/api-contract.md — đổi hợp đồng thì sửa cả file đó và code api/ trong cùng commit.

export type Locale = 'vi' | 'en';
export type Localized = { vi: string; en: string };

export type Role = 'reader' | 'author' | 'admin';
export type UserStatus = 'active' | 'locked';

export interface User {
  id: string;
  email: string;
  displayName: string;
  avatarUrl?: string;
  bio?: string;
  role: Role;
  status: UserStatus;
  createdAt: string;
}

export interface AuthorRef {
  id: string;
  displayName: string;
  avatarUrl?: string;
  role: Role;
}

export const CATEGORY_ICONS = ['database', 'code', 'pattern', 'architecture', 'devops', 'system', 'security'] as const;
export type CategoryIcon = (typeof CATEGORY_ICONS)[number];

export interface Category {
  id: string;
  slug: string;
  name: Localized;
  description: Localized;
  icon: CategoryIcon;
  color: string;
  order: number;
  articleCount: number;
}

export type CategoryRef = Pick<Category, 'id' | 'slug' | 'name' | 'icon' | 'color'>;

export const LEVELS = ['beginner', 'intermediate', 'advanced'] as const;
export type Level = (typeof LEVELS)[number];

export const REACTION_TYPES = ['love', 'helpful', 'insightful', 'inspiring'] as const;
export type ReactionType = (typeof REACTION_TYPES)[number];
export type ReactionCounts = Record<ReactionType, number>;

export interface ArticleStats {
  views: number;
  comments: number;
  reactions: ReactionCounts;
}

export interface ArticleCard {
  id: string;
  slug: string;
  title: string;
  summary: string;
  coverImageUrl?: string;
  language: Locale;
  category: CategoryRef;
  level: Level;
  tags: string[];
  author: AuthorRef;
  readingMinutes: number;
  publishedAt: string;
  updatedAt: string;
  stats: ArticleStats;
}

export interface ArticleLearningPathRef {
  id: string;
  slug: string;
  title: string;
  level: Level;
  articles: { id: string; slug: string; title: string; readingMinutes: number }[];
}

export interface ArticleDetail extends ArticleCard {
  content: string;
  learningPath?: ArticleLearningPathRef;
  myReactions: ReactionType[];
}

export interface Page<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
}

export type ArticleSort = 'latest' | 'popular' | 'trending';

export interface ArticleQuery {
  category?: string;
  level?: Level;
  tag?: string;
  q?: string;
  sort?: ArticleSort;
  page?: number;
  pageSize?: number;
}

export interface SiteSummary {
  articleCount: number;
  categoryCount: number;
  learningPathCount: number;
  memberCount: number;
  totalViews: number;
}

export interface TagCount {
  tag: string;
  count: number;
}

export interface LearningPathCard {
  id: string;
  slug: string;
  title: string;
  description: string;
  level: Level;
  category?: Pick<Category, 'slug' | 'name' | 'color' | 'icon'>;
  articleCount: number;
  totalMinutes: number;
  updatedAt: string;
}

export interface LearningPathDetail extends LearningPathCard {
  articles: ArticleCard[];
}

export interface ReactionState {
  reactions: ReactionCounts;
  myReactions: ReactionType[];
}

export type CommentSort = 'newest' | 'oldest' | 'top';

export interface Comment {
  id: string;
  articleId: string;
  parentId?: string;
  replyTo?: AuthorRef;
  author: AuthorRef;
  body: string;
  status: 'visible' | 'deleted';
  likeCount: number;
  likedByMe: boolean;
  editedAt?: string;
  createdAt: string;
}

export interface CommentThread extends Comment {
  replies: Comment[];
  replyCount: number;
}

export interface CommentPage extends Page<CommentThread> {
  commentCount: number;
}

export interface LikeState {
  likeCount: number;
  likedByMe: boolean;
}

// ---- Quản trị ----

export type ArticleStatus = 'draft' | 'published' | 'archived';

export interface AdminArticle extends Omit<ArticleDetail, 'myReactions' | 'content' | 'publishedAt'> {
  content?: string;
  status: ArticleStatus;
  publishedAt?: string;
  learningPathId?: string;
}

export interface ArticleInput {
  title: string;
  slug?: string;
  summary: string;
  content: string;
  coverImageUrl?: string;
  language: Locale;
  categoryId: string;
  level: Level;
  tags: string[];
}

export interface CategoryInput {
  slug: string;
  name: Localized;
  description: Localized;
  icon: CategoryIcon;
  color: string;
  order: number;
}

export interface AdminLearningPath extends LearningPathCard {
  status: 'draft' | 'published';
  categoryId?: string;
  articleIds: string[];
}

export interface LearningPathInput {
  slug: string;
  title: string;
  description: string;
  level: Level;
  categoryId?: string;
  articleIds: string[];
  status: 'draft' | 'published';
}

export interface AdminComment extends Omit<Comment, 'status'> {
  status: 'visible' | 'hidden' | 'deleted';
  article: { id: string; slug: string; title: string };
}

export interface AdminUser extends User {
  commentCount: number;
  lastLoginAt?: string;
}

export interface Subscriber {
  id: string;
  email: string;
  createdAt: string;
}

export interface StatsPoint {
  date: string;
  views: number;
  visits: number;
  uniqueVisitors: number;
  comments: number;
  reactions: number;
}

export interface StatsOverview {
  range: { from: string; to: string; days: number };
  totals: {
    articles: number;
    published: number;
    drafts: number;
    users: number;
    comments: number;
    subscribers: number;
    views: number;
    visits: number;
    uniqueVisitors: number;
  };
  previous: { views: number; visits: number; uniqueVisitors: number; comments: number };
  series: StatsPoint[];
  topArticles: { id: string; slug: string; title: string; views: number; comments: number; reactions: number }[];
}

export interface ApiErrorBody {
  error: { code: string; message: string; details?: { fields?: Record<string, string> } & Record<string, unknown> };
}
