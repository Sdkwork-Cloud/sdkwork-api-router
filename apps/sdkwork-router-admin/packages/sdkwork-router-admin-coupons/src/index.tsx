import { useDeferredValue, useState } from 'react';
import type { FormEvent } from 'react';

import {
  DataTable,
  InlineButton,
  Pill,
  SectionHero,
  StatCard,
  Surface,
} from 'sdkwork-router-admin-commons';
import type { AdminPageProps, CouponRecord } from 'sdkwork-router-admin-types';

type CouponsPageProps = AdminPageProps & {
  onSaveCoupon: (coupon: CouponRecord) => Promise<void> | void;
  onToggleCoupon: (coupon: CouponRecord) => Promise<void> | void;
  onDeleteCoupon: (couponId: string) => Promise<void> | void;
};

function createEmptyCouponDraft(): CouponRecord {
  return {
    id: '',
    code: '',
    discount_label: '10% off first bill',
    audience: 'new_signup',
    remaining: 100,
    active: true,
    note: 'Launch campaign',
    expires_on: '2026-12-31',
  };
}

export function CouponsPage({
  snapshot,
  onSaveCoupon,
  onToggleCoupon,
  onDeleteCoupon,
}: CouponsPageProps) {
  const [draft, setDraft] = useState<CouponRecord>(createEmptyCouponDraft());
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<'all' | 'active' | 'archived'>('all');
  const deferredQuery = useDeferredValue(search.trim().toLowerCase());

  const filteredCoupons = snapshot.coupons.filter((coupon) => {
    const matchesStatus = statusFilter === 'all'
      || (statusFilter === 'active' && coupon.active)
      || (statusFilter === 'archived' && !coupon.active);
    if (!matchesStatus) {
      return false;
    }

    const haystack = [
      coupon.code,
      coupon.discount_label,
      coupon.audience,
      coupon.note,
      coupon.expires_on,
    ]
      .join(' ')
      .toLowerCase();
    return haystack.includes(deferredQuery);
  });

  const activeCoupons = snapshot.coupons.filter((coupon) => coupon.active).length;
  const totalRemaining = snapshot.coupons.reduce((sum, coupon) => sum + coupon.remaining, 0);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveCoupon({
      ...draft,
      id: draft.id || `coupon_${Date.now().toString(16)}`,
      code: draft.code.trim().toUpperCase(),
      note: draft.note.trim(),
      audience: draft.audience.trim(),
      discount_label: draft.discount_label.trim(),
      expires_on: draft.expires_on.trim(),
    });
    setDraft(createEmptyCouponDraft());
  }

  return (
    <div className="adminx-page-grid">
      <SectionHero
        eyebrow="Growth"
        title="Run coupon campaigns from the live super-admin control plane."
        detail="Coupons are now backend-persisted campaign assets. Operators can create, edit, archive, restore, and delete campaigns without relying on workspace-local state."
      />

      <section className="adminx-stat-grid">
        <StatCard
          label="Campaigns"
          value={String(snapshot.coupons.length)}
          detail="Total coupon campaigns currently tracked."
        />
        <StatCard
          label="Active campaigns"
          value={String(activeCoupons)}
          detail="Campaigns available for redemption or activation."
        />
        <StatCard
          label="Remaining quota"
          value={String(totalRemaining)}
          detail="Aggregate remaining redemption inventory across all campaigns."
        />
      </section>

      <div className="adminx-users-grid">
        <Surface
          title={draft.id ? 'Edit campaign coupon' : 'Create campaign coupon'}
          detail="Editing an existing coupon preserves its identity while updating offer posture and inventory."
        >
          <form className="adminx-form-grid" onSubmit={(event) => void handleSubmit(event)}>
            <label className="adminx-field">
              <span>Coupon code</span>
              <input
                value={draft.code}
                onChange={(event) => setDraft((current) => ({ ...current, code: event.target.value.toUpperCase() }))}
                required
              />
            </label>
            <label className="adminx-field">
              <span>Discount label</span>
              <input
                value={draft.discount_label}
                onChange={(event) => setDraft((current) => ({ ...current, discount_label: event.target.value }))}
                required
              />
            </label>
            <label className="adminx-field">
              <span>Audience</span>
              <input
                value={draft.audience}
                onChange={(event) => setDraft((current) => ({ ...current, audience: event.target.value }))}
                required
              />
            </label>
            <label className="adminx-field">
              <span>Remaining</span>
              <input
                value={String(draft.remaining)}
                onChange={(event) => setDraft((current) => ({ ...current, remaining: Number(event.target.value) }))}
                type="number"
                min="0"
                required
              />
            </label>
            <label className="adminx-field">
              <span>Expires on</span>
              <input
                value={draft.expires_on}
                onChange={(event) => setDraft((current) => ({ ...current, expires_on: event.target.value }))}
                required
              />
            </label>
            <label className="adminx-field">
              <span>Status</span>
              <select
                value={draft.active ? 'active' : 'archived'}
                onChange={(event) => setDraft((current) => ({ ...current, active: event.target.value === 'active' }))}
              >
                <option value="active">Active</option>
                <option value="archived">Archived</option>
              </select>
            </label>
            <label className="adminx-field">
              <span>Note</span>
              <input
                value={draft.note}
                onChange={(event) => setDraft((current) => ({ ...current, note: event.target.value }))}
                required
              />
            </label>
            <div className="adminx-form-actions">
              <InlineButton tone="primary" type="submit">
                {draft.id ? 'Save coupon' : 'Create coupon'}
              </InlineButton>
              <InlineButton onClick={() => setDraft(createEmptyCouponDraft())}>
                Clear form
              </InlineButton>
            </div>
          </form>
        </Surface>

        <Surface
          title="Campaign filters"
          detail="Search campaign metadata and focus on active or archived promotions."
        >
          <div className="adminx-form-grid">
            <label className="adminx-field">
              <span>Search campaigns</span>
              <input
                value={search}
                onChange={(event) => setSearch(event.target.value)}
                placeholder="code, audience, note"
              />
            </label>
            <label className="adminx-field">
              <span>Status</span>
              <select
                value={statusFilter}
                onChange={(event) => setStatusFilter(event.target.value as 'all' | 'active' | 'archived')}
              >
                <option value="all">All campaigns</option>
                <option value="active">Active only</option>
                <option value="archived">Archived only</option>
              </select>
            </label>
            <div className="adminx-note">
              <strong>Live persistence</strong>
              <p>Deleting a campaign removes it from the backend store. Archiving preserves history while taking the offer offline.</p>
            </div>
          </div>
        </Surface>
      </div>

      <Surface title="Coupon roster" detail="Operate live coupon campaigns from one roster.">
        <DataTable
          columns={[
            { key: 'code', label: 'Code', render: (coupon) => <strong>{coupon.code}</strong> },
            { key: 'discount', label: 'Discount', render: (coupon) => coupon.discount_label },
            { key: 'audience', label: 'Audience', render: (coupon) => coupon.audience },
            { key: 'remaining', label: 'Remaining', render: (coupon) => coupon.remaining },
            { key: 'expires', label: 'Expires', render: (coupon) => coupon.expires_on },
            {
              key: 'status',
              label: 'Status',
              render: (coupon) => (
                <Pill tone={coupon.active ? 'live' : 'danger'}>
                  {coupon.active ? 'active' : 'archived'}
                </Pill>
              ),
            },
            {
              key: 'actions',
              label: 'Actions',
              render: (coupon) => (
                <div className="adminx-row">
                  <InlineButton onClick={() => setDraft(coupon)}>Edit</InlineButton>
                  <InlineButton onClick={() => void onToggleCoupon(coupon)}>
                    {coupon.active ? 'Archive' : 'Restore'}
                  </InlineButton>
                  <InlineButton onClick={() => void onDeleteCoupon(coupon.id)}>
                    Delete
                  </InlineButton>
                </div>
              ),
            },
          ]}
          rows={filteredCoupons}
          empty="No coupons match the current filter."
          getKey={(coupon) => coupon.id}
        />
      </Surface>
    </div>
  );
}
