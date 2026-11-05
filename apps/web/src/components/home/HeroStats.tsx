'use client';

import { useTransition, type ReactNode } from 'react';
import { useRouter } from 'next/navigation';
import { useTranslation } from 'react-i18next';
import { Layers, RefreshCw, Route, Users } from 'lucide-react';

function Stat({ icon, value, label }: { icon: ReactNode; value: string; label: string }) {
  return (
    // Nhóm <div> con TRỰC TIẾP của <dl> chỉ chứa dt + dd (axe definition-list/dlitem). dt đứng trước dd trong DOM,
    // hiển thị số lên trên bằng flex-col-reverse; biểu tượng nằm trong dt, đặt tuyệt đối ở cột trái (pl-14).
    <div className="relative flex min-h-11 flex-col-reverse justify-center pl-14">
      <dt className="mt-1 text-sm text-blue-50">
        <span className="absolute top-1/2 left-0 inline-flex h-11 w-11 -translate-y-1/2 items-center justify-center rounded-card border border-white/25 bg-white/10 text-sky-200">
          {icon}
        </span>
        {label}
      </dt>
      <dd className="text-2xl leading-none font-bold text-white sm:text-3xl">{value}</dd>
    </div>
  );
}

/**
 * Hàng 3 con số của hero — phần DUY NHẤT của hero chạy ở trình duyệt (nút "Thử lại" khi tải số liệu lỗi).
 * Số đã định dạng sẵn ở máy chủ (`'—'` khi chưa có).
 */
export function HeroStats({ topics, paths, members, failed }: { topics: string; paths: string; members: string; failed: boolean }) {
  const { t } = useTranslation();
  const router = useRouter();
  const [refreshing, startTransition] = useTransition();

  return (
    <div className="mt-10 flex flex-wrap items-center gap-x-8 gap-y-6 sm:mt-12">
      <dl aria-label={t('home.statsLabel')} className="flex flex-wrap gap-x-8 gap-y-6" aria-busy={refreshing}>
        <Stat icon={<Layers aria-hidden="true" className="h-5 w-5" />} value={topics} label={t('home.statTopics')} />
        <Stat icon={<Route aria-hidden="true" className="h-5 w-5" />} value={paths} label={t('home.statPaths')} />
        <Stat icon={<Users aria-hidden="true" className="h-5 w-5" />} value={members} label={t('home.statMembers')} />
      </dl>
      {failed && (
        <button
          type="button"
          onClick={() => startTransition(() => router.refresh())}
          className="inline-flex items-center gap-1.5 rounded-ui border border-white/30 px-3 py-1.5 text-xs font-semibold text-white hover:bg-white/10 focus-visible:outline-white"
        >
          <RefreshCw aria-hidden="true" className={`h-3.5 w-3.5 ${refreshing ? 'animate-spin' : ''}`} />
          {t('common.retry')}
        </button>
      )}
    </div>
  );
}
