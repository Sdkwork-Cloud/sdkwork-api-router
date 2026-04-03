import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  InlineAlert,
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';

import { copyToClipboard, type RevealedApiKey } from './shared';

type PlaintextApiKeyDialogProps = {
  onClose: () => void;
  revealedApiKey: RevealedApiKey;
};

export function PlaintextApiKeyDialog({
  onClose,
  revealedApiKey,
}: PlaintextApiKeyDialogProps) {
  const { t } = useAdminI18n();

  return (
    <Dialog open={Boolean(revealedApiKey)} onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('Plaintext key ready')}</DialogTitle>
          <DialogDescription>
            {t('Store this secret now. The control plane persists both the hashed and raw key.')}
          </DialogDescription>
        </DialogHeader>
        {revealedApiKey ? (
          <div className="space-y-4">
            <InlineAlert
              description={`${revealedApiKey.project_id} | ${revealedApiKey.environment} | ${revealedApiKey.label}`}
              showIcon
              title={t('Issued scope')}
              tone="info"
            />
            <Card>
              <CardHeader>
                <CardTitle className="text-base">{t('Plaintext key')}</CardTitle>
                <CardDescription>
                  {t('Copy the key now. It will not be revealed again in this dialog.')}
                </CardDescription>
              </CardHeader>
              <CardContent className="break-all font-mono text-sm">
                {revealedApiKey.plaintext}
              </CardContent>
            </Card>
            <DialogFooter>
              <Button
                onClick={() => void copyToClipboard(revealedApiKey.plaintext)}
                type="button"
                variant="primary"
              >
                {t('Copy key')}
              </Button>
              <Button onClick={onClose} type="button" variant="outline">
                {t('Close')}
              </Button>
            </DialogFooter>
          </div>
        ) : null}
      </DialogContent>
    </Dialog>
  );
}
