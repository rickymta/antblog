import { useId, useState, type FormEvent } from 'react';
import { Link } from 'react-router';
import { useTranslation } from 'react-i18next';
import { useMutation, useQuery } from '@tanstack/react-query';
import { CircleCheck, Mail, UserRound, Users } from 'lucide-react';
import { api, formatCompact } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { useAuth } from '../../lib/auth';
import { fieldErrors } from '../../lib/errors';
import { container } from '../../lib/ui';
import { summaryQuery } from './Hero';

const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
const BUBBLES = ['bg-sky-400', 'bg-indigo-400', 'bg-emerald-400', 'bg-violet-400'];

const panel =
  'relative overflow-hidden rounded-card bg-[linear-gradient(135deg,#0B1B4D_0%,#1E3A8A_60%,#312E81_100%)] p-6 text-white sm:p-8';

function Newsletter() {
  const { t } = useTranslation();
  const inputId = useId();
  const msgId = useId();
  const [email, setEmail] = useState('');
  const [localError, setLocalError] = useState<string | null>(null);
  const subscribe = useMutation({ mutationFn: (value: string) => api.site.subscribe(value) });

  const submit = (e: FormEvent) => {
    e.preventDefault();
    const value = email.trim();
    if (!EMAIL_RE.test(value)) {
      setLocalError(t('home.newsletterInvalid'));
      return;
    }
    setLocalError(null);
    subscribe.mutate(value, { onSuccess: () => setEmail('') });
  };

  const serverField = fieldErrors(subscribe.error).email;
  const error = localError ?? (subscribe.isError ? (serverField ?? t('home.newsletterError')) : null);

  return (
    <section id="newsletter" aria-labelledby="home-newsletter" className={panel}>
      <div aria-hidden="true" className="absolute -top-16 -right-10 h-48 w-48 rounded-full bg-indigo-400/20 blur-2xl" />
      <div className="relative">
        <Mail aria-hidden="true" className="h-8 w-8 text-sky-300" />
        <h2 id="home-newsletter" className="mt-4 text-2xl font-bold">
          {t('home.newsletterTitle')}
        </h2>
        <p className="mt-2 max-w-md text-sm text-blue-100">{t('home.newsletterBody')}</p>
        {subscribe.isSuccess ? (
          <p role="status" className="mt-6 inline-flex items-center gap-2 rounded-ui bg-white/10 px-4 py-3 text-sm font-semibold">
            <CircleCheck aria-hidden="true" className="h-5 w-5 text-emerald-300" />
            {t('home.newsletterSuccess')}
          </p>
        ) : (
          <form onSubmit={submit} noValidate className="mt-6 flex flex-col gap-2 sm:flex-row">
            <label htmlFor={inputId} className="sr-only">
              {t('home.newsletterEmail')}
            </label>
            <input
              id={inputId}
              type="email"
              inputMode="email"
              autoComplete="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder={t('home.newsletterPlaceholder')}
              aria-invalid={Boolean(error)}
              aria-describedby={error ? msgId : undefined}
              className="min-w-0 flex-1 rounded-ui border border-white/25 bg-white/10 px-3 py-2.5 text-sm text-white placeholder:text-blue-200 focus:border-white focus:outline-none focus:ring-2 focus:ring-white/40"
            />
            <button
              type="submit"
              disabled={subscribe.isPending}
              className="inline-flex items-center justify-center rounded-ui bg-white px-5 py-2.5 text-sm font-semibold text-blue-800 hover:bg-blue-50 disabled:opacity-60 focus-visible:outline-white"
            >
              {subscribe.isPending ? t('home.newsletterSending') : t('home.newsletterSubmit')}
            </button>
          </form>
        )}
        {error && (
          <p id={msgId} role="alert" className="mt-2 text-sm font-medium text-amber-200">
            {error}
          </p>
        )}
      </div>
    </section>
  );
}

function Community() {
  const { t } = useTranslation();
  const locale = useLocale();
  const { user } = useAuth();
  const summary = useQuery(summaryQuery);

  return (
    <section aria-labelledby="home-community" className={panel}>
      <div aria-hidden="true" className="absolute -bottom-16 -left-10 h-48 w-48 rounded-full bg-sky-400/20 blur-2xl" />
      <div className="relative">
        <Users aria-hidden="true" className="h-8 w-8 text-sky-300" />
        <h2 id="home-community" className="mt-4 text-2xl font-bold">
          {t('home.communityTitle')}
        </h2>
        <p className="mt-2 max-w-md text-sm text-blue-100">{t('home.communityBody')}</p>
        <div className="mt-6 flex flex-wrap items-center gap-4">
          <div className="flex -space-x-2" aria-hidden="true">
            {BUBBLES.map((bg) => (
              <span key={bg} className={`inline-flex h-9 w-9 items-center justify-center rounded-full border-2 border-[#1E3A8A] ${bg}`}>
                <UserRound className="h-4 w-4 text-slate-900" />
              </span>
            ))}
          </div>
          {summary.data && (
            <span className="text-sm font-semibold">{t('home.communityMembers', { n: formatCompact(summary.data.memberCount, locale) })}</span>
          )}
        </div>
        {user ? (
          <p className="mt-6 text-sm text-blue-100">{t('home.communityJoined')}</p>
        ) : (
          <Link
            to="/register"
            className="mt-6 inline-flex items-center justify-center rounded-ui bg-white px-5 py-2.5 text-sm font-semibold text-blue-800 hover:bg-blue-50 focus-visible:outline-white"
          >
            {t('home.communityCta')}
          </Link>
        )}
      </div>
    </section>
  );
}

export function CommunitySection() {
  return (
    <div className={`${container} grid gap-6 pt-14 lg:grid-cols-2`}>
      <Community />
      <Newsletter />
    </div>
  );
}
