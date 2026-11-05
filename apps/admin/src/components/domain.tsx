// Nhãn và biểu tượng theo nghiệp vụ: cấp độ, trạng thái, vai trò, biểu tượng chủ đề.
import {
  Building2,
  ClipboardList,
  Code,
  Database,
  FlaskConical,
  Infinity as InfinityIcon,
  Network,
  Puzzle,
  Shield,
  type LucideIcon,
} from 'lucide-react';
import { useTranslation } from 'react-i18next';
import type { CategoryIcon as CategoryIconName, Level, Role, UserStatus } from '@tkh/shared';
import type { DisplayStatus } from '../lib/articles';
import { Badge, type Tone } from './ui';

const LEVEL_TONE: Record<Level, Tone> = { beginner: 'success', intermediate: 'indigo', advanced: 'violet' };

export function LevelBadge({ level }: { level: Level }) {
  const { t } = useTranslation();
  return <Badge tone={LEVEL_TONE[level]}>{t(`level.${level}`)}</Badge>;
}

const ARTICLE_STATUS_TONE: Record<DisplayStatus, Tone> = { draft: 'warning', published: 'success', archived: 'neutral', scheduled: 'primary' };

/** Trạng thái bài; `scheduled` = đã xuất bản nhưng `publishedAt` ở tương lai (xem `displayStatus`). */
export function ArticleStatusBadge({ status }: { status: DisplayStatus }) {
  const { t } = useTranslation();
  return <Badge tone={ARTICLE_STATUS_TONE[status]}>{t(`articleStatus.${status}`)}</Badge>;
}

export function PathStatusBadge({ status }: { status: 'draft' | 'published' }) {
  const { t } = useTranslation();
  return <Badge tone={status === 'published' ? 'success' : 'warning'}>{t(`pathStatus.${status}`)}</Badge>;
}

const COMMENT_STATUS_TONE: Record<'visible' | 'hidden' | 'deleted', Tone> = {
  visible: 'success',
  hidden: 'warning',
  deleted: 'neutral',
};

export function CommentStatusBadge({ status }: { status: 'visible' | 'hidden' | 'deleted' }) {
  const { t } = useTranslation();
  return <Badge tone={COMMENT_STATUS_TONE[status]}>{t(`commentStatus.${status}`)}</Badge>;
}

const ROLE_TONE: Record<Role, Tone> = { reader: 'neutral', author: 'indigo', admin: 'primary' };

export function RoleBadge({ role }: { role: Role }) {
  const { t } = useTranslation();
  return <Badge tone={ROLE_TONE[role]}>{t(`role.${role}`)}</Badge>;
}

export function UserStatusBadge({ status }: { status: UserStatus }) {
  const { t } = useTranslation();
  return <Badge tone={status === 'active' ? 'success' : 'danger'}>{t(`userStatus.${status}`)}</Badge>;
}

export const CATEGORY_ICON_COMPONENT: Record<CategoryIconName, LucideIcon> = {
  database: Database,
  code: Code,
  pattern: Puzzle,
  architecture: Building2,
  devops: InfinityIcon,
  system: Network,
  security: Shield,
  testing: FlaskConical,
  analysis: ClipboardList,
};

/** Ô biểu tượng chủ đề: nền màu chủ đề, biểu tượng trắng. */
export function CategoryIcon({ icon, color, size = 'md' }: { icon: CategoryIconName; color: string; size?: 'sm' | 'md' | 'lg' }) {
  const Icon = CATEGORY_ICON_COMPONENT[icon] ?? Code;
  const box = size === 'sm' ? 'size-7' : size === 'lg' ? 'size-12' : 'size-9';
  const ic = size === 'sm' ? 'size-4' : size === 'lg' ? 'size-6' : 'size-5';
  return (
    <span className={`inline-flex ${box} shrink-0 items-center justify-center rounded-ui text-white`} style={{ backgroundColor: color }} aria-hidden>
      <Icon className={ic} />
    </span>
  );
}

/** Chấm màu + tên chủ đề cho bảng. */
export function CategoryChip({ name, color }: { name: string; color: string }) {
  return (
    <span className="inline-flex items-center gap-1.5 whitespace-nowrap">
      <span className="size-2.5 rounded-full" style={{ backgroundColor: color }} aria-hidden />
      {name}
    </span>
  );
}
