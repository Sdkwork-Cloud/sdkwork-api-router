import { Shield } from 'lucide-react';

import { usePortalI18n } from 'sdkwork-router-portal-commons';
import {
  Card,
  CardContent,
} from 'sdkwork-router-portal-commons/framework/layout';

export function ApiKeyManagedNoticeCard() {
  const { t } = usePortalI18n();

  return (
    <Card className="border-primary-500/15 bg-primary-500/8 shadow-none dark:border-primary-500/20 dark:bg-primary-500/10">
      <CardContent className="p-4">
        <div className="flex items-start gap-3">
          <span className="inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl bg-primary-500 text-white">
            <Shield className="h-4 w-4" />
          </span>
          <div className="min-w-0">
            <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
              {t('Portal-managed key')}
            </div>
            <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
              {t(
                'Portal will generate a one-time plaintext secret, persist only the hashed value, and reveal the plaintext once after creation.',
              )}
            </p>
            <div className="mt-3 rounded-2xl border border-dashed border-primary-500/25 bg-white/70 px-3 py-3 text-sm text-zinc-600 dark:border-primary-500/20 dark:bg-zinc-950/50 dark:text-zinc-300">
              {t('A one-time plaintext key will be revealed after creation.')}
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
