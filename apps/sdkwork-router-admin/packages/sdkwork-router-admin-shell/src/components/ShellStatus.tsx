import { StatusBadge } from '@sdkwork/ui-pc-react/components/ui';

function compactStatusLabel(status: string) {
  const normalized = status.toLowerCase();

  if (normalized.includes('synchronized')) {
    return 'Live sync';
  }

  if (normalized.includes('refresh')) {
    return 'Refreshing';
  }

  if (normalized.includes('authenticate')) {
    return 'Awaiting sign-in';
  }

  return status;
}

function resolveStatusToken(status: string) {
  const normalized = status.toLowerCase();

  if (normalized.includes('synchronized') || normalized.includes('restored')) {
    return 'live';
  }

  if (normalized.includes('refresh') || normalized.includes('loading')) {
    return 'syncing';
  }

  if (normalized.includes('authenticate') || normalized.includes('awaiting')) {
    return 'pending';
  }

  return 'active';
}

export function ShellStatus({ status }: { status: string }) {
  return (
    <div className="whitespace-nowrap">
      <StatusBadge
        label={compactStatusLabel(status)}
        showIcon
        status={resolveStatusToken(status)}
      />
    </div>
  );
}
