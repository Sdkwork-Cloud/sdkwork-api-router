import { Badge } from 'sdkwork-router-portal-commons/framework/display';
import { WorkspacePanel } from 'sdkwork-router-portal-commons/framework/workspace';

type RoutingCardTone = 'default' | 'secondary' | 'success' | 'warning' | 'outline';

export interface RoutingCardItem {
  id: string;
  label: string;
  value: string;
  detail: string;
  tone?: RoutingCardTone;
}

export function RoutingCardGrid({
  items,
  columns = 'xl:grid-cols-4',
}: {
  items: RoutingCardItem[];
  columns?: string;
}) {
  return (
    <div className={`grid gap-4 md:grid-cols-2 ${columns}`}>
      {items.map((item) => (
        <WorkspacePanel
          actions={
            item.tone ? <Badge variant={item.tone}>{item.value}</Badge> : null
          }
          className="h-full"
          description={item.detail}
          key={item.id}
          title={(
            <div className="space-y-3">
              <span className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                {item.label}
              </span>
              <strong className="block text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                {item.value}
              </strong>
            </div>
          )}
        >
        </WorkspacePanel>
      ))}
    </div>
  );
}

export { PortalRoutingProfilesDialog } from './PortalRoutingProfilesDialog';
export { PortalRoutingSnapshotsDialog } from './PortalRoutingSnapshotsDialog';

