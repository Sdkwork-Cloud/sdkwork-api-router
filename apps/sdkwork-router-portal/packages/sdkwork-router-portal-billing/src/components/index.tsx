import { usePortalI18n } from 'sdkwork-router-portal-commons';
import { Badge } from 'sdkwork-router-portal-commons/framework/display';

import type { BillingRecommendation } from '../types';

export function BillingRecommendationCard({
  recommendation,
}: {
  recommendation: BillingRecommendation;
}) {
  const { t } = usePortalI18n();

  return (
    <article className="rounded-[20px] border border-zinc-200 bg-white/90 p-4 dark:border-zinc-800 dark:bg-zinc-950/70">
      <div className="space-y-2">
        <Badge variant="default">{t('Recommendation')}</Badge>
        <strong className="block text-lg font-semibold text-zinc-950 dark:text-zinc-50">
          {recommendation.title}
        </strong>
      </div>
      <p className="mt-3 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
        {recommendation.detail}
      </p>
      <div className="mt-4 flex flex-wrap items-center gap-2">
        {recommendation.plan ? <Badge variant="success">{recommendation.plan.name}</Badge> : null}
        {recommendation.pack ? <Badge variant="warning">{recommendation.pack.label}</Badge> : null}
      </div>
    </article>
  );
}

