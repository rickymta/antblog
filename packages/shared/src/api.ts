// Client gọi API theo docs/api-contract.md. Dev chạy qua proxy Vite (`/api` → API) nên cùng origin, cookie phiên tự đi kèm.
import type {
  AdminArticle,
  AdminComment,
  AdminLearningPath,
  AdminUser,
  ApiErrorBody,
  ArticleCard,
  ArticleDetail,
  ArticleInput,
  ArticleQuery,
  Category,
  CategoryInput,
  CommentPage,
  CommentSort,
  Comment,
  LearningPathCard,
  LearningPathDetail,
  LearningPathInput,
  LikeState,
  Level,
  MediaItem,
  Page,
  PublishInput,
  ReactionState,
  ReactionType,
  Role,
  SiteSummary,
  SitemapData,
  StatsOverview,
  Subscriber,
  TagCount,
  User,
  UserStatus,
} from './types';

export const API_BASE = '/api/v1';

export class ApiError extends Error {
  readonly status: number;
  readonly code: string;
  readonly fields?: Record<string, string>;
  readonly details?: Record<string, unknown>;

  constructor(status: number, body: ApiErrorBody | undefined) {
    super(body?.error.message ?? `Lỗi ${status}`);
    this.name = 'ApiError';
    this.status = status;
    this.code = body?.error.code ?? (status >= 500 ? 'INTERNAL' : 'UNKNOWN');
    this.fields = body?.error.details?.fields;
    this.details = body?.error.details;
  }
}

type Query = Record<string, string | number | boolean | undefined | null>;

function qs(query?: Query): string {
  if (!query) return '';
  const p = new URLSearchParams();
  for (const [k, v] of Object.entries(query)) {
    if (v !== undefined && v !== null && v !== '') p.set(k, String(v));
  }
  const s = p.toString();
  return s ? `?${s}` : '';
}

async function request<T>(method: string, path: string, body?: unknown, query?: Query): Promise<T> {
  const isWrite = method !== 'GET';
  const res = await fetch(`${API_BASE}${path}${qs(query)}`, {
    method,
    credentials: 'include',
    headers: isWrite ? { 'Content-Type': 'application/json', Accept: 'application/json' } : { Accept: 'application/json' },
    body: isWrite ? JSON.stringify(body ?? {}) : undefined,
  });
  if (res.status === 204) return undefined as T;
  const text = await res.text();
  const data = text ? (JSON.parse(text) as unknown) : undefined;
  if (!res.ok) throw new ApiError(res.status, data as ApiErrorBody | undefined);
  return data as T;
}

const enc = encodeURIComponent;

/** Gửi multipart/form-data (tải ảnh) — ngoại lệ duy nhất của luật "ghi phải là JSON" (hợp đồng §8.2). */
async function upload<T>(path: string, form: FormData): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    method: 'POST',
    credentials: 'include',
    headers: { Accept: 'application/json' },
    body: form,
  });
  const text = await res.text();
  const data = text ? (JSON.parse(text) as unknown) : undefined;
  if (!res.ok) throw new ApiError(res.status, data as ApiErrorBody | undefined);
  return data as T;
}

export const api = {
  auth: {
    register: (input: { email: string; password: string; displayName: string }) =>
      request<{ user: User }>('POST', '/auth/register', input),
    login: (input: { email: string; password: string }) => request<{ user: User }>('POST', '/auth/login', input),
    logout: () => request<void>('POST', '/auth/logout'),
    me: () => request<{ user: User }>('GET', '/auth/me'),
    /** Luôn 200 — `user: null` khi chưa đăng nhập (trang đọc dùng để tránh lỗi 401 trong console). */
    session: () => request<{ user: User | null }>('GET', '/auth/session'),
    updateMe: (input: { displayName?: string; avatarUrl?: string; bio?: string }) =>
      request<{ user: User }>('PATCH', '/auth/me', input),
  },

  site: {
    summary: () => request<SiteSummary>('GET', '/site/summary'),
    trackVisit: (path: string) => request<void>('POST', '/track/visit', { path }),
    subscribe: (email: string) => request<{ ok: true }>('POST', '/subscribers', { email }),
    sitemap: () => request<SitemapData>('GET', '/site/sitemap'),
  },

  categories: {
    list: () => request<Category[]>('GET', '/categories'),
    get: (slug: string) => request<Category>('GET', `/categories/${enc(slug)}`),
  },

  articles: {
    list: (query: ArticleQuery = {}) => request<Page<ArticleCard>>('GET', '/articles', undefined, { ...query }),
    trending: (limit = 4) => request<ArticleCard[]>('GET', '/articles/trending', undefined, { limit }),
    get: (slug: string) => request<ArticleDetail>('GET', `/articles/${enc(slug)}`),
    related: (slug: string, limit = 3) =>
      request<ArticleCard[]>('GET', `/articles/${enc(slug)}/related`, undefined, { limit }),
    view: (slug: string) => request<{ counted: boolean; views: number }>('POST', `/articles/${enc(slug)}/view`),
    react: (slug: string, type: ReactionType) =>
      request<ReactionState>('PUT', `/articles/${enc(slug)}/reactions/${type}`),
    unreact: (slug: string, type: ReactionType) =>
      request<ReactionState>('DELETE', `/articles/${enc(slug)}/reactions/${type}`),
  },

  tags: {
    popular: (limit = 20) => request<TagCount[]>('GET', '/tags/popular', undefined, { limit }),
  },

  learningPaths: {
    list: (query: { level?: Level; q?: string } = {}) =>
      request<LearningPathCard[]>('GET', '/learning-paths', undefined, { ...query }),
    get: (slug: string) => request<LearningPathDetail>('GET', `/learning-paths/${enc(slug)}`),
  },

  comments: {
    list: (slug: string, query: { sort?: CommentSort; page?: number; pageSize?: number } = {}) =>
      request<CommentPage>('GET', `/articles/${enc(slug)}/comments`, undefined, { ...query }),
    create: (slug: string, input: { body: string; parentId?: string }) =>
      request<Comment>('POST', `/articles/${enc(slug)}/comments`, input),
    update: (id: string, body: string) => request<Comment>('PATCH', `/comments/${enc(id)}`, { body }),
    remove: (id: string) => request<void>('DELETE', `/comments/${enc(id)}`),
    like: (id: string) => request<LikeState>('PUT', `/comments/${enc(id)}/like`),
    unlike: (id: string) => request<LikeState>('DELETE', `/comments/${enc(id)}/like`),
  },

  admin: {
    stats: (days = 30) => request<StatsOverview>('GET', '/admin/stats/overview', undefined, { days }),

    articles: {
      list: (query: { status?: string; category?: string; q?: string; page?: number; pageSize?: number } = {}) =>
        request<Page<AdminArticle>>('GET', '/admin/articles', undefined, { ...query }),
      get: (id: string) => request<AdminArticle>('GET', `/admin/articles/${enc(id)}`),
      create: (input: ArticleInput) => request<AdminArticle>('POST', '/admin/articles', input),
      update: (id: string, input: ArticleInput) => request<AdminArticle>('PUT', `/admin/articles/${enc(id)}`, input),
      /** Bỏ trống ⇒ xuất bản ngay; `publishAt` ở tương lai ⇒ lên lịch. */
      publish: (id: string, input: PublishInput = {}) =>
        request<AdminArticle>('POST', `/admin/articles/${enc(id)}/publish`, input),
      unpublish: (id: string) => request<AdminArticle>('POST', `/admin/articles/${enc(id)}/unpublish`),
      archive: (id: string) => request<void>('DELETE', `/admin/articles/${enc(id)}`),
    },

    categories: {
      list: () => request<Category[]>('GET', '/admin/categories'),
      create: (input: CategoryInput) => request<Category>('POST', '/admin/categories', input),
      update: (id: string, input: CategoryInput) => request<Category>('PUT', `/admin/categories/${enc(id)}`, input),
      remove: (id: string) => request<void>('DELETE', `/admin/categories/${enc(id)}`),
    },

    learningPaths: {
      list: () => request<AdminLearningPath[]>('GET', '/admin/learning-paths'),
      get: (id: string) => request<AdminLearningPath>('GET', `/admin/learning-paths/${enc(id)}`),
      create: (input: LearningPathInput) => request<AdminLearningPath>('POST', '/admin/learning-paths', input),
      update: (id: string, input: LearningPathInput) =>
        request<AdminLearningPath>('PUT', `/admin/learning-paths/${enc(id)}`, input),
      remove: (id: string) => request<void>('DELETE', `/admin/learning-paths/${enc(id)}`),
    },

    comments: {
      list: (query: { status?: string; articleId?: string; q?: string; page?: number; pageSize?: number } = {}) =>
        request<Page<AdminComment>>('GET', '/admin/comments', undefined, { ...query }),
      setStatus: (id: string, status: 'visible' | 'hidden') =>
        request<AdminComment>('PATCH', `/admin/comments/${enc(id)}`, { status }),
      remove: (id: string) => request<void>('DELETE', `/admin/comments/${enc(id)}`),
    },

    users: {
      list: (query: { q?: string; role?: Role; status?: UserStatus; page?: number; pageSize?: number } = {}) =>
        request<Page<AdminUser>>('GET', '/admin/users', undefined, { ...query }),
      update: (id: string, input: { role?: Role; status?: UserStatus }) =>
        request<AdminUser>('PATCH', `/admin/users/${enc(id)}`, input),
    },

    subscribers: {
      list: (page = 1) => request<Page<Subscriber>>('GET', '/admin/subscribers', undefined, { page }),
    },

    media: {
      list: (query: { q?: string; page?: number; pageSize?: number } = {}) =>
        request<Page<MediaItem>>('GET', '/admin/media', undefined, { ...query }),
      upload: (file: File | Blob, alt?: string) => {
        const form = new FormData();
        form.append('file', file);
        if (alt) form.append('alt', alt);
        return upload<MediaItem>('/admin/media', form);
      },
      update: (id: string, input: { alt: string }) => request<MediaItem>('PATCH', `/admin/media/${enc(id)}`, input),
      remove: (id: string) => request<void>('DELETE', `/admin/media/${enc(id)}`),
    },
  },
};
