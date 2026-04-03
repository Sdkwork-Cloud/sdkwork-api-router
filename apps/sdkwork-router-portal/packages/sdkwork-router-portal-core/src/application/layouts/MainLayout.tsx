import { lazy, Suspense, useState, type ReactNode } from 'react';
import type { PortalWorkspaceSummary } from 'sdkwork-router-portal-types';

import { PortalDesktopShell } from '../../components/PortalDesktopShell';

const PortalSettingsCenter = lazy(async () => ({
  default: (await import('../../components/PortalSettingsCenter')).PortalSettingsCenter,
}));

export function MainLayout({
  children,
  workspace,
}: {
  children: ReactNode;
  workspace: PortalWorkspaceSummary | null;
}) {
  const [settingsCenterOpen, setSettingsCenterOpen] = useState(false);

  return (
    <>
      <PortalDesktopShell
        onOpenSettings={() => setSettingsCenterOpen(true)}
        workspace={workspace}
      >
        {children}
      </PortalDesktopShell>
      {settingsCenterOpen ? (
        <Suspense fallback={null}>
          <PortalSettingsCenter
            onOpenChange={setSettingsCenterOpen}
            open={settingsCenterOpen}
          />
        </Suspense>
      ) : null}
    </>
  );
}
