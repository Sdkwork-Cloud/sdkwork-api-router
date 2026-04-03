import type { ReactNode } from 'react';
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@sdkwork/ui-pc-react';
import { translateAdminText } from 'sdkwork-router-admin-core';

export function DialogField({
  children,
  description,
  htmlFor,
  label,
}: {
  children: ReactNode;
  description?: ReactNode;
  htmlFor?: string;
  label: ReactNode;
}) {
  return (
    <div className="space-y-2">
      <Label htmlFor={htmlFor}>{label}</Label>
      {children}
      {description ? (
        <div className="text-xs text-[var(--sdk-color-text-secondary)]">{description}</div>
      ) : null}
    </div>
  );
}

export function SelectField<T extends string>({
  description,
  disabled,
  label,
  labelVisibility = 'visible',
  onValueChange,
  options,
  placeholder,
  value,
}: {
  description?: ReactNode;
  disabled?: boolean;
  label: ReactNode;
  labelVisibility?: 'visible' | 'sr-only';
  onValueChange: (value: T) => void;
  options: Array<{ label: ReactNode; value: T }>;
  placeholder?: string;
  value: T;
}) {
  const isHiddenLabel = labelVisibility === 'sr-only';

  return (
    <div className={isHiddenLabel ? 'space-y-0' : 'space-y-2'}>
      <Label className={isHiddenLabel ? 'sr-only' : undefined}>{label}</Label>
      <Select
        disabled={disabled}
        onValueChange={(nextValue: string) => onValueChange(nextValue as T)}
        value={value}
      >
        <SelectTrigger>
          <SelectValue placeholder={placeholder ?? String(label)} />
        </SelectTrigger>
        <SelectContent>
          {options.map((option) => (
            <SelectItem key={option.value} value={option.value}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      {description ? (
        <div className="text-xs text-[var(--sdk-color-text-secondary)]">{description}</div>
      ) : null}
    </div>
  );
}

export function ConfirmActionDialog({
  confirmLabel = translateAdminText('Confirm'),
  description,
  onConfirm,
  onOpenChange,
  open,
  title,
}: {
  confirmLabel?: string;
  description: ReactNode;
  onConfirm: () => void | Promise<void>;
  onOpenChange: (open: boolean) => void;
  open: boolean;
  title: ReactNode;
}) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,28rem)]">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button onClick={() => onOpenChange(false)} type="button" variant="outline">
            {translateAdminText('Cancel')}
          </Button>
          <Button onClick={() => void onConfirm()} type="button" variant="danger">
            {confirmLabel}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
