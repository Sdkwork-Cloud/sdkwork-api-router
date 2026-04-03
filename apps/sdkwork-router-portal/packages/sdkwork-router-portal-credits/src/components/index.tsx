import { formatUnits, usePortalI18n } from 'sdkwork-router-portal-commons';
import { Badge } from 'sdkwork-router-portal-commons/framework/display';

import type { CouponImpactPreview } from '../types';

export function CouponImpactCard({
  preview,
}: {
  preview: CouponImpactPreview;
}) {
  const { t } = usePortalI18n();

  return (
    <article className="rounded-[20px] border border-zinc-200 bg-white/90 p-4 dark:border-zinc-800 dark:bg-zinc-950/70">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="space-y-2">
          <Badge variant="success">{t('Redemption impact')}</Badge>
          <strong className="block text-lg font-semibold text-zinc-950 dark:text-zinc-50">
            {preview.coupon.code}
          </strong>
        </div>
        <span className="rounded-full border border-emerald-400/20 bg-emerald-400/10 px-3 py-1 text-xs font-semibold text-emerald-700 dark:text-emerald-300">
          {t('{units} bonus units', { units: formatUnits(preview.quote.bonus_units) })}
        </span>
      </div>
      <p className="mt-3 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
        {preview.status}
      </p>
    </article>
  );
}

