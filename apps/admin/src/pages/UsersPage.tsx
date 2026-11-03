import { useState } from 'react';
import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { Lock, LockOpen, Search } from 'lucide-react';
import { api, formatDate, formatNumber, formatRelative, type AdminUser, type Role, type UserStatus } from '@tkh/shared';
import {
  Button,
  Card,
  ConfirmDialog,
  EmptyState,
  ErrorState,
  FilterBar,
  PageHeader,
  Pagination,
  TableSkeleton,
  TableWrap,
  Td,
  Th,
  cx,
  inputClass,
  selectClass,
} from '../components/ui';
import { UserStatusBadge } from '../components/domain';
import { useSearchBox, useUrlFilters } from '../lib/hooks';
import { errorMessage, hasCode } from '../lib/errors';
import { useCurrentUser } from '../lib/session';
import { useToast } from '../lib/toast';
import { useLocale } from '../i18n';

const PAGE_SIZE = 20;
const ROLES: Role[] = ['reader', 'author', 'admin'];
const STATUSES: UserStatus[] = ['active', 'locked'];

function initials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  const pick = parts.length > 1 ? [parts[0]!, parts[parts.length - 1]!] : parts;
  return pick.map((p) => p[0]?.toUpperCase() ?? '').join('') || '?';
}

export function UsersPage() {
  const { t, i18n } = useTranslation();
  const locale = useLocale();
  const me = useCurrentUser();
  const client = useQueryClient();
  const toast = useToast();
  const f = useUrlFilters(['q', 'role', 'status'] as const);
  const [text, setText] = useSearchBox(f.values.q, (v) => f.set('q', v));
  const [locking, setLocking] = useState<AdminUser | null>(null);

  const query = {
    q: f.values.q,
    role: (f.values.role || undefined) as Role | undefined,
    status: (f.values.status || undefined) as UserStatus | undefined,
    page: f.page,
    pageSize: PAGE_SIZE,
  };
  const list = useQuery({
    queryKey: ['admin', 'users', query],
    queryFn: () => api.admin.users.list(query),
    placeholderData: keepPreviousData,
  });

  const update = useMutation({
    mutationFn: ({ u, input }: { u: AdminUser; input: { role?: Role; status?: UserStatus } }) => api.admin.users.update(u.id, input),
    onSuccess: (saved, { u, input }) => {
      const name = saved?.displayName ?? u.displayName;
      if (input.role) toast.success(t('users.roleChanged', { name, role: t(`role.${input.role}`) }));
      else if (input.status === 'locked') toast.success(t('users.locked', { name }));
      else toast.success(t('users.unlocked', { name }));
      setLocking(null);
      void client.invalidateQueries({ queryKey: ['admin', 'users'] });
    },
    onError: (err) => {
      setLocking(null);
      toast.error(hasCode(err, 'CANNOT_MODIFY_SELF') ? t('users.cannotModifySelf') : errorMessage(err, t, i18n.language));
    },
  });

  const pendingId = update.isPending ? update.variables?.u.id : undefined;

  return (
    <>
      <PageHeader title={t('users.title')} subtitle={t('users.subtitle')} />
      <Card>
        <FilterBar>
          <label className="relative min-w-[14rem] flex-[2]">
            <span className="sr-only">{t('common.search')}</span>
            <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted" aria-hidden />
            <input
              type="search"
              className={cx(inputClass, 'h-10 pl-9')}
              placeholder={t('users.searchPlaceholder')}
              value={text}
              onChange={(e) => setText(e.target.value)}
            />
          </label>
          <label className="min-w-[9rem] flex-1">
            <span className="mb-1 block text-xs font-medium text-muted">{t('users.filterRole')}</span>
            <select className={selectClass} value={f.values.role} onChange={(e) => f.set('role', e.target.value)}>
              <option value="">{t('common.all')}</option>
              {ROLES.map((r) => (
                <option key={r} value={r}>
                  {t(`role.${r}`)}
                </option>
              ))}
            </select>
          </label>
          <label className="min-w-[9rem] flex-1">
            <span className="mb-1 block text-xs font-medium text-muted">{t('users.filterStatus')}</span>
            <select className={selectClass} value={f.values.status} onChange={(e) => f.set('status', e.target.value)}>
              <option value="">{t('common.all')}</option>
              {STATUSES.map((s) => (
                <option key={s} value={s}>
                  {t(`userStatus.${s}`)}
                </option>
              ))}
            </select>
          </label>
          {f.active && (
            <Button variant="ghost" onClick={f.clear}>
              {t('common.clearFilters')}
            </Button>
          )}
        </FilterBar>

        {list.isPending ? (
          <TableSkeleton cols={6} />
        ) : list.isError ? (
          <ErrorState message={errorMessage(list.error, t, i18n.language)} onRetry={() => void list.refetch()} />
        ) : list.data.items.length === 0 ? (
          <EmptyState title={f.active ? t('states.emptyFiltered') : t('users.empty')} />
        ) : (
          <>
            <TableWrap label={t('users.title')}>
              <thead>
                <tr>
                  <Th>{t('users.colUser')}</Th>
                  <Th>{t('users.colRole')}</Th>
                  <Th>{t('users.colStatus')}</Th>
                  <Th className="text-right">{t('users.colComments')}</Th>
                  <Th>{t('users.colLastLogin')}</Th>
                  <Th>{t('users.colCreated')}</Th>
                  <Th className="text-right">{t('common.actions')}</Th>
                </tr>
              </thead>
              <tbody className={list.isPlaceholderData ? 'opacity-60' : undefined}>
                {list.data.items.map((u) => {
                  const self = u.id === me.id;
                  const busy = pendingId === u.id;
                  return (
                    <tr key={u.id} className={cx('hover:bg-surface-2/60', self && 'bg-primary-soft/40')}>
                      <Td>
                        <div className="flex items-center gap-3">
                          {u.avatarUrl ? (
                            <img src={u.avatarUrl} alt="" className="size-9 shrink-0 rounded-full object-cover" />
                          ) : (
                            <span className="inline-flex size-9 shrink-0 items-center justify-center rounded-full bg-primary-soft text-xs font-bold text-primary-ink" aria-hidden>
                              {initials(u.displayName)}
                            </span>
                          )}
                          <div className="min-w-0">
                            <p className="flex items-center gap-2 font-medium">
                              <span className="truncate">{u.displayName}</span>
                              {self && (
                                <span className="rounded-ui bg-primary px-1.5 py-0.5 text-[11px] font-semibold text-white">{t('users.you')}</span>
                              )}
                            </p>
                            <p className="truncate text-xs text-muted">{u.email}</p>
                          </div>
                        </div>
                      </Td>
                      <Td>
                        <select
                          className={cx(selectClass, 'h-9 w-36 py-1')}
                          aria-label={t('users.roleFor', { name: u.displayName })}
                          title={self ? t('users.selfLocked') : undefined}
                          value={u.role}
                          disabled={self || busy}
                          onChange={(e) => update.mutate({ u, input: { role: e.target.value as Role } })}
                        >
                          {ROLES.map((r) => (
                            <option key={r} value={r}>
                              {t(`role.${r}`)}
                            </option>
                          ))}
                        </select>
                      </Td>
                      <Td>
                        <UserStatusBadge status={u.status} />
                      </Td>
                      <Td className="text-right tabular-nums">{formatNumber(u.commentCount ?? 0, locale)}</Td>
                      <Td className="whitespace-nowrap text-muted">
                        {u.lastLoginAt ? (
                          <time dateTime={u.lastLoginAt} title={formatDate(u.lastLoginAt, locale)}>
                            {formatRelative(u.lastLoginAt, locale)}
                          </time>
                        ) : (
                          t('users.never')
                        )}
                      </Td>
                      <Td className="whitespace-nowrap text-muted">{formatDate(u.createdAt, locale)}</Td>
                      <Td className="text-right whitespace-nowrap">
                        {u.status === 'active' ? (
                          <Button
                            size="sm"
                            icon={Lock}
                            disabled={self}
                            loading={busy && update.variables?.input.status === 'locked'}
                            title={self ? t('users.selfLocked') : undefined}
                            aria-label={t('users.lockUser', { name: u.displayName })}
                            onClick={() => setLocking(u)}
                          >
                            {t('users.lock')}
                          </Button>
                        ) : (
                          <Button
                            size="sm"
                            icon={LockOpen}
                            disabled={self}
                            loading={busy}
                            aria-label={t('users.unlockUser', { name: u.displayName })}
                            onClick={() => update.mutate({ u, input: { status: 'active' } })}
                          >
                            {t('users.unlock')}
                          </Button>
                        )}
                      </Td>
                    </tr>
                  );
                })}
              </tbody>
            </TableWrap>
            <Pagination page={list.data.page} pageSize={list.data.pageSize} total={list.data.total} onPage={f.setPage} />
          </>
        )}
      </Card>
      <ConfirmDialog
        open={locking !== null}
        title={t('users.confirmLockTitle', { name: locking?.displayName ?? '' })}
        body={t('users.confirmLockBody')}
        confirmLabel={t('users.lock')}
        loading={update.isPending}
        onConfirm={() => locking && update.mutate({ u: locking, input: { status: 'locked' } })}
        onCancel={() => setLocking(null)}
      />
    </>
  );
}
