'use client';

import { useTransition, type ReactNode } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useTranslation } from 'react-i18next';
import { ArrowRight, Layers, RefreshCw, Route, Users } from 'lucide-react';
import { formatCompact, type SiteSummary } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { container } from '../../lib/ui';
import { RoadmapDiagram } from './RoadmapDiagram';

function Stat({ icon, value, label }: { icon: ReactNode; value: string; label: string }) {
  return (
    // Nhóm <div> con TRỰC TIẾP của <dl> chỉ chứa dt + dd (axe definition-list/dlitem). dt đứng trước dd trong DOM,
    // hiển thị số lên trên bằng flex-col-reverse; biểu tượng nằm trong dt, đặt tuyệt đối ở cột trái (pl-14).
    <div className="relative flex min-h-11 flex-col-reverse justify-center pl-14">
      <dt className="mt-1 text-sm text-blue-100">
        <span className="absolute top-1/2 left-0 inline-flex h-11 w-11 -translate-y-1/2 items-center justify-center rounded-card border border-sky-300/40 text-sky-300">
          {icon}
        </span>
        {label}
      </dt>
      <dd className="text-2xl leading-none font-bold text-white sm:text-3xl">{value}</dd>
    </div>
  );
}

/** Hero trang chủ (A3): nền gradient xanh đậm → indigo, chữ trắng cực lớn, 2 nút, 3 con số, sơ đồ lộ trình bên phải (các bước bấm được). */
export function Hero({ summary }: { summary: SiteSummary | null }) {
  const { t } = useTranslation();
  const locale = useLocale();
  const router = useRouter();
  const [refreshing, startTransition] = useTransition();
  const value = (n: number | undefined) => (n === undefined ? '—' : formatCompact(n, locale));

  return (
    <section aria-labelledby="hero-title" className="relative isolate overflow-hidden bg-[linear-gradient(135deg,#0B1B4D_0%,#1E3A8A_55%,#312E81_100%)] text-white">
      <div aria-hidden="true" className="pointer-events-none absolute -top-32 -left-24 -z-10 h-96 w-96 rounded-full bg-blue-500/30 blur-3xl" />
      <div aria-hidden="true" className="pointer-events-none absolute -right-24 bottom-0 -z-10 h-96 w-96 rounded-full bg-indigo-500/30 blur-3xl" />
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-0 -z-10 opacity-[0.07] [background-image:radial-gradient(#fff_1px,transparent_1px)] [background-size:22px_22px]"
      />

      <div className={`${container} grid items-center gap-12 py-16 sm:py-20 lg:grid-cols-[minmax(0,1.1fr)_minmax(0,0.9fr)] lg:py-24`}>
        <div className="min-w-0">
          <p className="mb-5 inline-flex items-center gap-2 rounded-full border border-white/20 bg-white/10 px-3 py-1 text-xs font-semibold text-blue-50">
            <span aria-hidden="true" className="h-1.5 w-1.5 rounded-full bg-sky-300" />
            {t('home.heroEyebrow')}
          </p>
          <h1 id="hero-title" className="text-4xl leading-[1.08] font-extrabold tracking-tight text-balance sm:text-5xl xl:text-6xl">
            <span className="block">{t('home.heroLine1')}</span>
            <span className="block bg-gradient-to-r from-sky-300 to-indigo-200 bg-clip-text text-transparent">{t('home.heroLine2')}</span>
          </h1>
          <p className="mt-6 max-w-xl text-base text-blue-100 sm:text-lg">{t('home.heroBody')}</p>
          <div className="mt-8 flex flex-wrap gap-3">
            <Link
              href="/learning-paths"
              className="inline-flex items-center gap-2 rounded-ui bg-primary px-5 py-3 text-sm font-semibold text-white shadow-lg shadow-blue-950/40 hover:bg-primary-hover focus-visible:outline-white"
            >
              {t('home.ctaStart')}
              <ArrowRight aria-hidden="true" className="h-4 w-4" />
            </Link>
            <Link
              href="/articles"
              className="inline-flex items-center gap-2 rounded-ui border border-white/30 px-5 py-3 text-sm font-semibold text-white hover:bg-white/10 focus-visible:outline-white"
            >
              {t('home.ctaArticles')}
            </Link>
          </div>

          <div className="mt-12 flex flex-wrap items-center gap-x-8 gap-y-6">
            <dl aria-label={t('home.statsLabel')} className="flex flex-wrap gap-x-8 gap-y-6" aria-busy={refreshing}>
              <Stat icon={<Layers aria-hidden="true" className="h-5 w-5" />} value={value(summary?.categoryCount)} label={t('home.statTopics')} />
              <Stat icon={<Route aria-hidden="true" className="h-5 w-5" />} value={value(summary?.learningPathCount)} label={t('home.statPaths')} />
              <Stat icon={<Users aria-hidden="true" className="h-5 w-5" />} value={value(summary?.memberCount)} label={t('home.statMembers')} />
            </dl>
            {!summary && (
              <button
                type="button"
                onClick={() => startTransition(() => router.refresh())}
                className="inline-flex items-center gap-1.5 rounded-ui border border-white/30 px-3 py-1.5 text-xs font-semibold text-white hover:bg-white/10"
              >
                <RefreshCw aria-hidden="true" className={`h-3.5 w-3.5 ${refreshing ? 'animate-spin' : ''}`} />
                {t('common.retry')}
              </button>
            )}
          </div>
        </div>

        <div className="hidden min-w-0 lg:block">
          <RoadmapDiagram className="mx-auto h-auto w-full max-w-[520px]" />
        </div>
      </div>
    </section>
  );
}
