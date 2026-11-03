import { useTranslation } from 'react-i18next';

export function Logo({ compact }: { compact?: boolean }) {
  const { t } = useTranslation();
  return (
    <span className="flex items-center gap-2.5">
      <span
        className="inline-flex size-9 items-center justify-center rounded-ui bg-gradient-to-br from-primary to-secondary font-mono text-sm font-bold text-white"
        aria-hidden
      >
        {'</>'}
      </span>
      {!compact && (
        <span className="leading-tight">
          <span className="block text-sm font-bold text-ink">{t('app.name')}</span>
          <span className="block text-xs font-medium text-primary-ink">{t('app.adminTag')}</span>
        </span>
      )}
    </span>
  );
}
