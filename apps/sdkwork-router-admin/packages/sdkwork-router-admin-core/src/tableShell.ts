export const embeddedAdminDataTableClassName = 'min-h-0 flex-1 gap-0';

export const embeddedAdminDataTableSlotProps = {
  surface: {
    className: 'min-h-0 flex-1 rounded-none border-0 bg-transparent shadow-none',
  },
};

type EmbeddedAdminSingleSelectRowId = number | string;

export function buildEmbeddedAdminSingleSelectRowProps<T>(
  selectedRowId: EmbeddedAdminSingleSelectRowId | null | undefined,
  getRowId: (row: T, index: number) => EmbeddedAdminSingleSelectRowId,
) {
  return (row: T, index: number) => {
    if (selectedRowId == null) {
      return undefined;
    }

    if (getRowId(row, index) !== selectedRowId) {
      return undefined;
    }

    return {
      className: 'bg-[var(--sdk-color-brand-primary-soft)]',
      'data-state': 'selected' as const,
    };
  };
}
