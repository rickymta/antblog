import { useState, type FormEvent, type ReactNode } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { CircleAlert, LogIn, LogOut, ShieldX } from 'lucide-react';
import { api, type User } from '@tkh/shared';
import { Button, Field, inputClass } from '../components/ui';
import { errorMessage, hasCode, hasStatus } from '../lib/errors';
import { SESSION_KEY, useLogout } from '../lib/session';
import { PreferenceToggles } from './PreferenceToggles';
import { Logo } from './Logo';

function AuthFrame({ children }: { children: ReactNode }) {
  return (
    <div className="flex min-h-dvh flex-col bg-gradient-to-br from-blue-50 via-page to-indigo-50 dark:from-[#0B1B4D] dark:via-page dark:to-[#1E1B4B]">
      <header className="flex items-center justify-between p-4">
        <Logo />
        <PreferenceToggles />
      </header>
      <main className="flex flex-1 items-center justify-center px-4 pb-16">
        <div className="w-full max-w-md rounded-card border border-line bg-surface p-6 shadow-xl sm:p-8">{children}</div>
      </main>
    </div>
  );
}

export function LoginPage() {
  const { t, i18n } = useTranslation();
  const client = useQueryClient();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [touched, setTouched] = useState(false);

  const login = useMutation({
    mutationFn: () => api.auth.login({ email: email.trim(), password }),
    onSuccess: ({ user }) => client.setQueryData<User | null>(SESSION_KEY, user),
  });

  const emailErr = touched && !email.trim() ? t('auth.emailRequired') : undefined;
  const passErr = touched && !password ? t('auth.passwordRequired') : undefined;

  const submit = (e: FormEvent) => {
    e.preventDefault();
    setTouched(true);
    if (!email.trim() || !password) return;
    login.mutate();
  };

  let formError: string | undefined;
  if (login.error) {
    if (hasCode(login.error, 'INVALID_CREDENTIALS') || hasStatus(login.error, 401)) formError = t('auth.invalidCredentials');
    else if (hasCode(login.error, 'ACCOUNT_LOCKED')) formError = t('auth.accountLocked');
    else if (hasStatus(login.error, 429)) formError = t('auth.rateLimited');
    else formError = errorMessage(login.error, t, i18n.language);
  }

  return (
    <AuthFrame>
      <h1 className="text-2xl font-bold tracking-tight text-ink">{t('auth.title')}</h1>
      <p className="mt-1 text-sm text-muted">{t('auth.subtitle')}</p>
      <form className="mt-6 space-y-4" onSubmit={submit} noValidate>
        {formError && (
          <p role="alert" className="flex items-start gap-2 rounded-ui bg-red-50 p-3 text-sm font-medium text-red-800 dark:bg-red-950/50 dark:text-red-200">
            <CircleAlert className="mt-0.5 size-4 shrink-0" aria-hidden />
            {formError}
          </p>
        )}
        <Field label={t('auth.email')} error={emailErr} required>
          {(a) => (
            <input
              {...a}
              type="email"
              autoComplete="username"
              className={inputClass}
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              autoFocus
            />
          )}
        </Field>
        <Field label={t('auth.password')} error={passErr} required>
          {(a) => (
            <input
              {...a}
              type="password"
              autoComplete="current-password"
              className={inputClass}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
          )}
        </Field>
        <Button type="submit" variant="primary" icon={LogIn} loading={login.isPending} className="w-full">
          {login.isPending ? t('auth.submitting') : t('auth.submit')}
        </Button>
      </form>
    </AuthFrame>
  );
}

export function NoAccessPage({ user }: { user: User }) {
  const { t } = useTranslation();
  const logout = useLogout();
  const [busy, setBusy] = useState(false);
  return (
    <AuthFrame>
      <div className="flex flex-col items-center text-center">
        <ShieldX className="size-12 text-red-600 dark:text-red-400" aria-hidden />
        <h1 className="mt-3 text-xl font-bold text-ink">{t('auth.noAccessTitle')}</h1>
        <p className="mt-2 text-sm text-muted">{t('auth.noAccessBody', { name: user.displayName, email: user.email })}</p>
        <Button
          variant="primary"
          icon={LogOut}
          className="mt-6"
          loading={busy}
          onClick={async () => {
            setBusy(true);
            await logout();
          }}
        >
          {t('topbar.logout')}
        </Button>
      </div>
    </AuthFrame>
  );
}
