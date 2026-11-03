import { useState, type FormEvent, type ReactNode } from 'react';
import { Link, Navigate, useSearchParams } from 'react-router';
import { useTranslation } from 'react-i18next';
import type { TFunction } from 'i18next';
import { useMutation } from '@tanstack/react-query';
import { ApiError, api } from '@tkh/shared';
import { PasswordField, TextField } from '../components/common/FormField';
import { LogoMark } from '../components/layout/Logo';
import { useDocumentTitle } from '../hooks/useDocumentTitle';
import { useAuth } from '../lib/auth';
import { fieldErrors } from '../lib/errors';
import { safeNext } from '../lib/navigation';
import { btnPrimary } from '../lib/ui';

const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

/** Câu lỗi chung theo mã lỗi API (để hiện đúng ngôn ngữ giao diện). */
function formError(e: unknown, t: TFunction): string | null {
  if (!e) return null;
  if (e instanceof ApiError) {
    switch (e.code) {
      case 'INVALID_CREDENTIALS':
        return t('auth.errors.invalidCredentials');
      case 'ACCOUNT_LOCKED':
        return t('auth.errors.accountLocked');
      case 'EMAIL_TAKEN':
        return null; // hiện dưới ô email
      case 'RATE_LIMITED':
        return t('auth.errors.rateLimited');
      case 'VALIDATION':
        // Lỗi trường hiện dưới từng ô; không có trường cụ thể thì hiện câu của API.
        return e.fields && Object.keys(e.fields).length > 0 ? null : e.message;
    }
  }
  return t('auth.genericError');
}

function AuthShell({ title, subtitle, children, footer }: { title: string; subtitle: string; children: ReactNode; footer: ReactNode }) {
  return (
    <div className="relative isolate flex min-h-[calc(100dvh-4rem)] items-start justify-center px-4 py-12 sm:items-center">
      <div
        aria-hidden="true"
        className="absolute inset-0 -z-10 bg-[linear-gradient(160deg,#DBEAFE_0%,#E0E7FF_45%,#F8FAFC_100%)] dark:bg-[linear-gradient(160deg,#0B1B4D_0%,#1E1B4B_50%,#0F172A_100%)]"
      />
      <div className="w-full max-w-md rounded-card border border-line bg-surface p-6 shadow-xl shadow-blue-900/10 sm:p-8">
        <div className="mb-6 text-center">
          <LogoMark className="mx-auto h-11 w-11 text-base" />
          <h1 className="mt-4 text-2xl font-extrabold tracking-tight text-ink">{title}</h1>
          <p className="mt-1 text-sm text-muted">{subtitle}</p>
        </div>
        {children}
        <p className="mt-6 text-center text-sm text-muted">{footer}</p>
      </div>
    </div>
  );
}

function FormAlert({ message }: { message: string | null }) {
  if (!message) return null;
  return (
    <p role="alert" className="rounded-ui border border-red-200 bg-red-50 px-3 py-2 text-sm font-medium text-red-800 dark:border-red-900 dark:bg-red-950/50 dark:text-red-300">
      {message}
    </p>
  );
}

export function LoginPage() {
  const { t } = useTranslation();
  useDocumentTitle(t('auth.loginTitle'));
  const { user, setUser } = useAuth();
  const [params] = useSearchParams();
  const next = safeNext(params.get('next'));
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [local, setLocal] = useState<Record<string, string>>({});
  const login = useMutation({ mutationFn: api.auth.login, onSuccess: (res) => setUser(res.user) });

  if (user) return <Navigate to={next} replace />;

  const submit = (e: FormEvent) => {
    e.preventDefault();
    const errs: Record<string, string> = {};
    if (!email.trim()) errs.email = t('auth.required', { field: t('auth.email').toLowerCase() });
    else if (!EMAIL_RE.test(email.trim())) errs.email = t('auth.invalidEmail');
    if (!password) errs.password = t('auth.required', { field: t('auth.password').toLowerCase() });
    setLocal(errs);
    if (Object.keys(errs).length === 0) login.mutate({ email: email.trim(), password });
  };
  const server = fieldErrors(login.error);
  const err = (f: string) => local[f] ?? server[f];
  const registerLink = params.get('next') ? `/register?next=${encodeURIComponent(next)}` : '/register';

  return (
    <AuthShell
      title={t('auth.loginTitle')}
      subtitle={t('auth.loginSubtitle')}
      footer={
        <>
          {t('auth.noAccount')}{' '}
          <Link to={registerLink} className="font-semibold text-primary-ink hover:underline">
            {t('auth.goRegister')}
          </Link>
        </>
      }
    >
      <form onSubmit={submit} noValidate className="space-y-4">
        <FormAlert message={formError(login.error, t)} />
        <TextField label={t('auth.email')} type="email" autoComplete="email" inputMode="email" value={email} onChange={(e) => setEmail(e.target.value)} error={err('email')} autoFocus />
        <PasswordField label={t('auth.password')} autoComplete="current-password" value={password} onChange={(e) => setPassword(e.target.value)} error={err('password')} />
        <button type="submit" disabled={login.isPending} className={`${btnPrimary} w-full py-2.5`}>
          {login.isPending ? t('auth.submitting') : t('auth.loginSubmit')}
        </button>
      </form>
    </AuthShell>
  );
}

export function RegisterPage() {
  const { t } = useTranslation();
  useDocumentTitle(t('auth.registerTitle'));
  const { user, setUser } = useAuth();
  const [params] = useSearchParams();
  const next = safeNext(params.get('next'));
  const [displayName, setDisplayName] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [local, setLocal] = useState<Record<string, string>>({});
  const register = useMutation({ mutationFn: api.auth.register, onSuccess: (res) => setUser(res.user) });

  if (user) return <Navigate to={next} replace />;

  const submit = (e: FormEvent) => {
    e.preventDefault();
    const errs: Record<string, string> = {};
    const name = displayName.trim();
    if (name.length < 2 || name.length > 60) errs.displayName = t('auth.displayNameLength');
    if (!EMAIL_RE.test(email.trim())) errs.email = t('auth.invalidEmail');
    if (password.length < 8 || password.length > 72) errs.password = t('auth.passwordLength');
    setLocal(errs);
    if (Object.keys(errs).length === 0) register.mutate({ displayName: name, email: email.trim(), password });
  };
  const server = { ...fieldErrors(register.error) };
  if (register.error instanceof ApiError && register.error.code === 'EMAIL_TAKEN') server.email = t('auth.errors.emailTaken');
  const err = (f: string) => local[f] ?? server[f];
  const loginLink = params.get('next') ? `/login?next=${encodeURIComponent(next)}` : '/login';

  return (
    <AuthShell
      title={t('auth.registerTitle')}
      subtitle={t('auth.registerSubtitle')}
      footer={
        <>
          {t('auth.haveAccount')}{' '}
          <Link to={loginLink} className="font-semibold text-primary-ink hover:underline">
            {t('auth.goLogin')}
          </Link>
        </>
      }
    >
      <form onSubmit={submit} noValidate className="space-y-4">
        <FormAlert message={formError(register.error, t)} />
        <TextField
          label={t('auth.displayName')}
          autoComplete="nickname"
          value={displayName}
          onChange={(e) => setDisplayName(e.target.value)}
          hint={t('auth.displayNameHint')}
          error={err('displayName')}
          maxLength={60}
          autoFocus
        />
        <TextField label={t('auth.email')} type="email" autoComplete="email" inputMode="email" value={email} onChange={(e) => setEmail(e.target.value)} error={err('email')} />
        <PasswordField
          label={t('auth.password')}
          autoComplete="new-password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          hint={t('auth.passwordHint')}
          error={err('password')}
          maxLength={72}
        />
        <button type="submit" disabled={register.isPending} className={`${btnPrimary} w-full py-2.5`}>
          {register.isPending ? t('auth.submitting') : t('auth.registerSubmit')}
        </button>
      </form>
    </AuthShell>
  );
}
