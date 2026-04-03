import type { ComponentType, ReactNode } from 'react';
import type { LucideIcon } from 'lucide-react';
import {
  Badge,
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@sdkwork/ui-pc-react';

type SettingsBadgeProps = {
  children?: ReactNode;
  className?: string;
  variant?: 'default' | 'secondary' | 'outline' | 'danger' | 'success' | 'warning';
};

const SharedBadge = Badge as unknown as ComponentType<SettingsBadgeProps>;

export function SettingsBadge(props: SettingsBadgeProps) {
  return <SharedBadge {...props} />;
}

export function SettingsSummaryCard({
  badge,
  detail,
  label,
  value,
}: {
  badge?: ReactNode;
  detail?: ReactNode;
  label: ReactNode;
  value: ReactNode;
}) {
  return (
    <Card className="h-full">
      <CardHeader className="space-y-2 pb-3">
        <div className="flex items-start justify-between gap-3">
          <CardDescription className="text-[11px] uppercase tracking-[0.18em]">
            {label}
          </CardDescription>
          {badge ? (
            typeof badge === 'string' ? (
              <SettingsBadge variant="secondary">{badge}</SettingsBadge>
            ) : (
              badge
            )
          ) : null}
        </div>
        <CardTitle className="text-xl">{value}</CardTitle>
      </CardHeader>
      {detail ? <CardContent className="pt-0 text-sm text-[var(--sdk-color-text-secondary)]">{detail}</CardContent> : null}
    </Card>
  );
}

export function SettingsChoiceButton({
  active,
  description,
  icon: Icon,
  label,
  onClick,
}: {
  active: boolean;
  description: ReactNode;
  icon: LucideIcon;
  label: ReactNode;
  onClick: () => void;
}) {
  return (
    <Button
      className="h-auto min-h-32 w-full items-start justify-start px-4 py-4 text-left"
      onClick={onClick}
      type="button"
      variant={active ? 'primary' : 'outline'}
    >
      <span className="flex w-full flex-col gap-3">
        <span className="flex items-center gap-2 text-sm font-semibold">
          <Icon className="h-4 w-4" />
          <span>{label}</span>
        </span>
        <span className="text-sm font-normal text-[inherit] opacity-80">{description}</span>
      </span>
    </Button>
  );
}
