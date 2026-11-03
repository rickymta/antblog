import { useState, type ReactNode } from 'react';
import { Link } from 'react-router';
import { useQueries, useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import {
  Area,
  AreaChart,
  Bar,
  BarChart,
  CartesianGrid,
  Legend,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts';
import { ArrowDownRight, ArrowUpRight, Minus, PenLine } from 'lucide-react';
import { api, formatCompact, formatDate, formatNumber, type Locale, type StatsOverview } from '@tkh/shared';
import { buttonClass, Card, EmptyState, ErrorState, PageHeader, Skeleton, TableWrap, Td, Th, cx } from '../components/ui';
import { ArticleStatusBadge } from '../components/domain';
import { useIsAdmin } from '../lib/session';
import { errorMessage } from '../lib/errors';
import { useLocale } from '../i18n';

const RANGES = [7, 30, 90] as const;
type Range = (typeof RANGES)[number];

// Màu biểu đồ khai ở index.css (--chart-*), đã kiểm tách màu cho người mù màu ở cả hai theme.
const C = {
  views: 'var(--chart-views)',
  visits: 'var(--chart-visits)',
  unique: 'var(--chart-unique)',
  comments: 'var(--chart-comments)',
};

function shortDate(ymd: string, locale: Locale): string {
  const d = new Date(`${ymd}T00:00:00Z`);
  if (Number.isNaN(d.getTime())) return ymd;
  return d.toLocaleDateString(locale === 'vi' ? 'vi-VN' : 'en-US', { day: '2-digit', month: '2-digit', timeZone: 'UTC' });
}

function longDate(ymd: string, locale: Locale): string {
  const d = new Date(`${ymd}T00:00:00Z`);
  if (Number.isNaN(d.getTime())) return ymd;
  return d.toLocaleDateString(locale === 'vi' ? 'vi-VN' : 'en-US', { day: '2-digit', month: '2-digit', year: 'numeric', timeZone: 'UTC' });
}

export function DashboardPage() {
  const isAdmin = useIsAdmin();
  return isAdmin ? <AdminDashboard /> : <AuthorDashboard />;
}

// ---------------- Quản trị viên ----------------

function Delta({ current, previous }: { current: number; previous: number }) {
  const { t } = useTranslation();
  const locale = useLocale();
  if (previous === 0) {
    return (
      <span className="text-xs text-muted">
        {current === 0 ? t('dashboard.noChange') : `+${formatNumber(current, locale)} ${t('dashboard.newInPeriod')}`}
      </span>
    );
  }
  const pct = ((current - previous) / previous) * 100;
  const rounded = Math.round(pct * 10) / 10;
  const Icon = rounded > 0 ? ArrowUpRight : rounded < 0 ? ArrowDownRight : Minus;
  const tone =
    rounded > 0
      ? 'text-emerald-700 dark:text-emerald-400'
      : rounded < 0
        ? 'text-red-700 dark:text-red-400'
        : 'text-muted';
  const pctText = new Intl.NumberFormat(locale === 'vi' ? 'vi-VN' : 'en-US', {
    maximumFractionDigits: 1,
    signDisplay: 'exceptZero',
  }).format(rounded);
  return (
    <span className={cx('inline-flex items-center gap-0.5 text-xs font-semibold', tone)}>
      <Icon className="size-3.5" aria-hidden />
      {pctText}%<span className="ml-1 font-normal text-muted">{t('dashboard.vsPrevious')}</span>
    </span>
  );
}

function Kpi({ label, value, foot }: { label: string; value: number; foot: ReactNode }) {
  const locale = useLocale();
  return (
    <div className="rounded-card border border-line bg-surface p-4">
      <p className="text-sm font-medium text-muted">{label}</p>
      <p className="mt-1 text-2xl font-bold tracking-tight text-ink" title={formatNumber(value, locale)}>
        {formatCompact(value, locale)}
      </p>
      <div className="mt-1 min-h-5">{foot}</div>
    </div>
  );
}

function AdminDashboard() {
  const { t, i18n } = useTranslation();
  const locale = useLocale();
  const [days, setDays] = useState<Range>(30);
  const q = useQuery({ queryKey: ['admin', 'stats', days], queryFn: () => api.admin.stats(days) });

  return (
    <>
      <PageHeader
        title={t('dashboard.title')}
        subtitle={q.data ? t('dashboard.rangeLabel', { from: longDate(q.data.range.from, locale), to: longDate(q.data.range.to, locale) }) : t('dashboard.subtitle')}
        actions={
          <div role="radiogroup" aria-label={t('dashboard.range')} className="inline-flex rounded-ui border border-line bg-surface p-1">
            {RANGES.map((r) => (
              <button
                key={r}
                type="button"
                role="radio"
                aria-checked={days === r}
                onClick={() => setDays(r)}
                className={cx(
                  'rounded-[6px] px-3 py-1.5 text-sm font-medium',
                  days === r ? 'bg-primary text-white' : 'text-ink hover:bg-surface-2',
                )}
              >
                {t('dashboard.days', { count: r })}
              </button>
            ))}
          </div>
        }
      />

      {q.isPending ? (
        <DashboardSkeleton />
      ) : q.isError ? (
        <Card>
          <ErrorState message={errorMessage(q.error, t, i18n.language)} onRetry={() => void q.refetch()} />
        </Card>
      ) : (
        <AdminDashboardBody data={q.data} />
      )}
    </>
  );
}

function DashboardSkeleton() {
  return (
    <div className="space-y-6" aria-hidden>
      <div className="grid grid-cols-2 gap-4 md:grid-cols-4 xl:grid-cols-7">
        {Array.from({ length: 7 }, (_, i) => (
          <Skeleton key={i} className="h-28 rounded-card" />
        ))}
      </div>
      <div className="grid gap-6 xl:grid-cols-3">
        <Skeleton className="h-80 rounded-card xl:col-span-2" />
        <Skeleton className="h-80 rounded-card" />
      </div>
      <Skeleton className="h-64 rounded-card" />
    </div>
  );
}

const axisTick = { fill: 'var(--tkh-muted)', fontSize: 12 };
const tooltipStyle = {
  backgroundColor: 'var(--tkh-surface)',
  border: '1px solid var(--tkh-line)',
  borderRadius: 8,
  color: 'var(--tkh-ink)',
  fontSize: 13,
};

function AdminDashboardBody({ data }: { data: StatsOverview }) {
  const { t } = useTranslation();
  const locale = useLocale();
  const { totals, previous } = data;
  const series = data.series.map((p) => ({ ...p, label: shortDate(p.date, locale) }));
  const fmt = (v: unknown) => (typeof v === 'number' ? formatNumber(v, locale) : String(v ?? ''));
  const currentTotalFoot = <span className="text-xs text-muted">{t('dashboard.currentTotal')}</span>;

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-2 gap-4 md:grid-cols-4 xl:grid-cols-7">
        <Kpi
          label={t('dashboard.kpiPublished')}
          value={totals.published}
          foot={<span className="text-xs text-muted">{t('dashboard.kpiPublishedHint', { count: totals.drafts })}</span>}
        />
        <Kpi label={t('dashboard.kpiViews')} value={totals.views} foot={<Delta current={totals.views} previous={previous.views} />} />
        <Kpi label={t('dashboard.kpiVisits')} value={totals.visits} foot={<Delta current={totals.visits} previous={previous.visits} />} />
        <Kpi
          label={t('dashboard.kpiUnique')}
          value={totals.uniqueVisitors}
          foot={<Delta current={totals.uniqueVisitors} previous={previous.uniqueVisitors} />}
        />
        <Kpi
          label={t('dashboard.kpiComments')}
          value={totals.comments}
          foot={<Delta current={totals.comments} previous={previous.comments} />}
        />
        <Kpi label={t('dashboard.kpiMembers')} value={totals.users} foot={currentTotalFoot} />
        <Kpi label={t('dashboard.kpiSubscribers')} value={totals.subscribers} foot={currentTotalFoot} />
      </div>

      <div className="grid gap-6 xl:grid-cols-3">
        <Card className="p-4 xl:col-span-2">
          <h2 className="mb-3 text-base font-semibold text-ink">{t('dashboard.trafficTitle')}</h2>
          <div className="h-72">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={series} margin={{ top: 8, right: 12, bottom: 0, left: 0 }}>
                <CartesianGrid stroke="var(--tkh-line)" strokeDasharray="3 3" vertical={false} />
                <XAxis dataKey="label" tick={axisTick} tickLine={false} axisLine={{ stroke: 'var(--tkh-line)' }} minTickGap={16} />
                <YAxis tick={axisTick} tickLine={false} axisLine={false} width={44} allowDecimals={false} tickFormatter={(v: number) => formatCompact(v, locale)} />
                <Tooltip contentStyle={tooltipStyle} labelStyle={{ color: 'var(--tkh-ink)', fontWeight: 600 }} formatter={fmt} cursor={{ stroke: 'var(--tkh-muted)', strokeDasharray: '3 3' }} />
                <Legend itemSorter={null} wrapperStyle={{ fontSize: 13, color: 'var(--tkh-ink)' }} />
                <Area type="monotone" dataKey="views" name={t('dashboard.seriesViews')} stroke={C.views} fill={C.views} fillOpacity={0.12} strokeWidth={2} dot={false} activeDot={{ r: 4 }} />
                <Area type="monotone" dataKey="visits" name={t('dashboard.seriesVisits')} stroke={C.visits} fill={C.visits} fillOpacity={0.08} strokeWidth={2} dot={false} activeDot={{ r: 4 }} />
                <Area type="monotone" dataKey="uniqueVisitors" name={t('dashboard.seriesUnique')} stroke={C.unique} fill="transparent" strokeWidth={2} strokeDasharray="5 3" dot={false} activeDot={{ r: 4 }} />
              </AreaChart>
            </ResponsiveContainer>
          </div>
          <SeriesTable data={data} />
        </Card>

        <Card className="p-4">
          <h2 className="mb-3 text-base font-semibold text-ink">{t('dashboard.commentsTitle')}</h2>
          <div className="h-72">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={series} margin={{ top: 8, right: 8, bottom: 0, left: 0 }}>
                <CartesianGrid stroke="var(--tkh-line)" strokeDasharray="3 3" vertical={false} />
                <XAxis dataKey="label" tick={axisTick} tickLine={false} axisLine={{ stroke: 'var(--tkh-line)' }} minTickGap={16} />
                <YAxis tick={axisTick} tickLine={false} axisLine={false} width={36} allowDecimals={false} />
                <Tooltip contentStyle={tooltipStyle} labelStyle={{ color: 'var(--tkh-ink)', fontWeight: 600 }} formatter={fmt} cursor={{ fill: 'var(--tkh-surface-2)' }} />
                <Bar dataKey="comments" name={t('dashboard.seriesComments')} fill={C.comments} radius={[4, 4, 0, 0]} maxBarSize={28} />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </Card>
      </div>

      <Card>
        <h2 className="border-b border-line px-4 py-3 text-base font-semibold text-ink">{t('dashboard.topTitle')}</h2>
        {data.topArticles.length === 0 ? (
          <EmptyState title={t('dashboard.topEmpty')} />
        ) : (
          <TableWrap label={t('dashboard.topTitle')}>
            <thead>
              <tr>
                <Th className="w-10">#</Th>
                <Th>{t('dashboard.colTitle')}</Th>
                <Th className="text-right">{t('dashboard.colViews')}</Th>
                <Th className="text-right">{t('dashboard.colComments')}</Th>
                <Th className="text-right">{t('dashboard.colReactions')}</Th>
              </tr>
            </thead>
            <tbody>
              {data.topArticles.map((a, i) => (
                <tr key={a.id} className="hover:bg-surface-2/60">
                  <Td className="text-muted tabular-nums">{i + 1}</Td>
                  <Td>
                    <Link to={`/articles/${a.id}`} className="font-medium text-primary-ink hover:underline">
                      {a.title}
                    </Link>
                  </Td>
                  <Td className="text-right tabular-nums">{formatNumber(a.views, locale)}</Td>
                  <Td className="text-right tabular-nums">{formatNumber(a.comments, locale)}</Td>
                  <Td className="text-right tabular-nums">{formatNumber(a.reactions, locale)}</Td>
                </tr>
              ))}
            </tbody>
          </TableWrap>
        )}
      </Card>
    </div>
  );
}

/** Bảng số liệu theo ngày (thay thế cho người không đọc được biểu đồ). */
function SeriesTable({ data }: { data: StatsOverview }) {
  const { t } = useTranslation();
  const locale = useLocale();
  return (
    <details className="mt-3 rounded-ui border border-line">
      <summary className="cursor-pointer px-3 py-2 text-sm font-medium text-primary-ink">{t('dashboard.showTable')}</summary>
      <div className="max-h-72 overflow-auto">
        <table className="w-full text-sm">
          <thead className="sticky top-0 bg-surface-2">
            <tr className="text-left text-xs text-slate-600 dark:text-slate-300">
              <th scope="col" className="px-3 py-2">{t('dashboard.date')}</th>
              <th scope="col" className="px-3 py-2 text-right">{t('dashboard.seriesViews')}</th>
              <th scope="col" className="px-3 py-2 text-right">{t('dashboard.seriesVisits')}</th>
              <th scope="col" className="px-3 py-2 text-right">{t('dashboard.seriesUnique')}</th>
              <th scope="col" className="px-3 py-2 text-right">{t('dashboard.seriesComments')}</th>
            </tr>
          </thead>
          <tbody>
            {data.series.map((p) => (
              <tr key={p.date} className="border-t border-line">
                <td className="px-3 py-1.5 text-ink">{longDate(p.date, locale)}</td>
                <td className="px-3 py-1.5 text-right tabular-nums text-ink">{formatNumber(p.views, locale)}</td>
                <td className="px-3 py-1.5 text-right tabular-nums text-ink">{formatNumber(p.visits, locale)}</td>
                <td className="px-3 py-1.5 text-right tabular-nums text-ink">{formatNumber(p.uniqueVisitors, locale)}</td>
                <td className="px-3 py-1.5 text-right tabular-nums text-ink">{formatNumber(p.comments, locale)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </details>
  );
}

// ---------------- Tác giả ----------------

const AUTHOR_STATUSES = ['published', 'draft', 'archived'] as const;

function AuthorDashboard() {
  const { t, i18n } = useTranslation();
  const locale = useLocale();
  const counts = useQueries({
    queries: AUTHOR_STATUSES.map((status) => ({
      queryKey: ['admin', 'articles', 'count', status],
      queryFn: () => api.admin.articles.list({ status, pageSize: 1 }),
    })),
  });
  const recent = useQuery({ queryKey: ['admin', 'articles', 'recent'], queryFn: () => api.admin.articles.list({ pageSize: 8 }) });
  const labels = [t('dashboard.authorPublished'), t('dashboard.authorDrafts'), t('dashboard.authorArchived')];
  const anyError = counts.find((c) => c.isError)?.error ?? recent.error;

  return (
    <>
      <PageHeader
        title={t('dashboard.authorTitle')}
        subtitle={t('dashboard.authorSubtitle')}
        actions={
          <Link to="/articles/new" className={buttonClass('primary')}>
            <PenLine className="size-4" aria-hidden />
            {t('dashboard.writeArticle')}
          </Link>
        }
      />
      {anyError ? (
        <Card>
          <ErrorState
            message={errorMessage(anyError, t, i18n.language)}
            onRetry={() => {
              counts.forEach((c) => void c.refetch());
              void recent.refetch();
            }}
          />
        </Card>
      ) : (
        <div className="space-y-6">
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
            {counts.map((c, i) =>
              c.isPending ? (
                <Skeleton key={i} className="h-24 rounded-card" />
              ) : (
                <div key={i} className="rounded-card border border-line bg-surface p-4">
                  <p className="text-sm font-medium text-muted">{labels[i]}</p>
                  <p className="mt-1 text-2xl font-bold text-ink">{formatNumber(c.data?.total ?? 0, locale)}</p>
                </div>
              ),
            )}
          </div>
          <Card>
            <h2 className="border-b border-line px-4 py-3 text-base font-semibold text-ink">{t('dashboard.authorRecent')}</h2>
            {recent.isPending ? (
              <div className="space-y-3 p-4">
                {Array.from({ length: 4 }, (_, i) => (
                  <Skeleton key={i} className="h-5" />
                ))}
              </div>
            ) : (recent.data?.items.length ?? 0) === 0 ? (
              <EmptyState
                title={t('dashboard.authorEmpty')}
                action={
                  <Link to="/articles/new" className={buttonClass('primary')}>
                    {t('dashboard.writeArticle')}
                  </Link>
                }
              />
            ) : (
              <TableWrap label={t('dashboard.authorRecent')}>
                <thead>
                  <tr>
                    <Th>{t('articles.colTitle')}</Th>
                    <Th>{t('articles.colStatus')}</Th>
                    <Th className="text-right">{t('articles.colViews')}</Th>
                    <Th className="text-right">{t('articles.colComments')}</Th>
                    <Th>{t('articles.colUpdated')}</Th>
                  </tr>
                </thead>
                <tbody>
                  {recent.data?.items.map((a) => (
                    <tr key={a.id} className="hover:bg-surface-2/60">
                      <Td>
                        <Link to={`/articles/${a.id}`} className="font-medium text-primary-ink hover:underline">
                          {a.title}
                        </Link>
                      </Td>
                      <Td>
                        <ArticleStatusBadge status={a.status} />
                      </Td>
                      <Td className="text-right tabular-nums">{formatNumber(a.stats.views, locale)}</Td>
                      <Td className="text-right tabular-nums">{formatNumber(a.stats.comments, locale)}</Td>
                      <Td className="whitespace-nowrap text-muted">{formatDate(a.updatedAt, locale)}</Td>
                    </tr>
                  ))}
                </tbody>
              </TableWrap>
            )}
          </Card>
        </div>
      )}
    </>
  );
}
