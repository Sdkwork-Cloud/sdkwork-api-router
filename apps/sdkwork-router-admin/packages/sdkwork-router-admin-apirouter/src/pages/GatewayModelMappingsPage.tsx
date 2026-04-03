import { useDeferredValue, useEffect, useMemo, useState } from 'react';
import type { ChangeEvent, FormEvent } from 'react';
import {
  Button,
  Card,
  CardContent,
  Input,
  Label,
  StatusBadge,
  type DataTableColumn,
} from '@sdkwork/ui-pc-react';
import { Plus, Search } from 'lucide-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type { AdminPageProps } from 'sdkwork-router-admin-types';

import { ConfirmActionDialog, SelectField } from './shared';
import { GatewayModelMappingEditorDialog } from './mappings/GatewayModelMappingEditorDialog';
import { GatewayModelMappingsDetailDrawer } from './mappings/GatewayModelMappingsDetailDrawer';
import { GatewayModelMappingsRegistrySection } from './mappings/GatewayModelMappingsRegistrySection';
import {
  createRuleDraft,
  draftFromMapping,
  emptyDraft,
  formatDateLabel,
  type MappingDraft,
} from './mappings/shared';
import {
  createGatewayModelMapping,
  deleteGatewayModelMapping,
  listGatewayModelMappings,
  updateGatewayModelMapping,
  updateGatewayModelMappingStatus,
  type GatewayModelMappingRecord,
  type GatewayModelMappingStatus,
} from '../services/gatewayOverlayStore';
import { buildGatewayModelCatalog } from '../services/gatewayViewService';

export function GatewayModelMappingsPage({ snapshot }: AdminPageProps) {
  const { formatNumber, t } = useAdminI18n();
  const catalog = useMemo(() => buildGatewayModelCatalog(snapshot), [snapshot]);
  const catalogValues = catalog.map((item) => item.value);
  const catalogByValue = useMemo(
    () => new Map(catalog.map((item) => [item.value, item])),
    [catalog],
  );
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] =
    useState<'all' | GatewayModelMappingStatus>('all');
  const [selectedMappingId, setSelectedMappingId] = useState<string | null>(null);
  const [editingMappingId, setEditingMappingId] = useState<string | null>(null);
  const [mappingDraft, setMappingDraft] = useState<MappingDraft>(() =>
    emptyDraft(catalogValues),
  );
  const [isDetailDrawerOpen, setIsDetailDrawerOpen] = useState(false);
  const [isEditorOpen, setIsEditorOpen] = useState(false);
  const [pendingDelete, setPendingDelete] =
    useState<GatewayModelMappingRecord | null>(null);
  const [overlayVersion, setOverlayVersion] = useState(0);
  const deferredSearch = useDeferredValue(search.trim().toLowerCase());

  const mappings = useMemo(() => listGatewayModelMappings(), [overlayVersion]);
  const filteredMappings = useMemo(
    () =>
      mappings.filter((mapping) => {
        if (statusFilter !== 'all' && mapping.status !== statusFilter) {
          return false;
        }

        if (!deferredSearch) {
          return true;
        }

        const haystack = [
          mapping.name,
          mapping.description,
          ...mapping.rules.flatMap((rule) => [
            rule.source_channel_name,
            rule.source_model_name,
            rule.source_model_id,
            rule.target_channel_name,
            rule.target_model_name,
            rule.target_model_id,
          ]),
        ]
          .join(' ')
          .toLowerCase();

        return haystack.includes(deferredSearch);
      }),
    [deferredSearch, mappings, statusFilter],
  );

  useEffect(() => {
    if (!filteredMappings.length) {
      if (selectedMappingId !== null) {
        setSelectedMappingId(null);
      }
      setIsDetailDrawerOpen(false);
      return;
    }

    if (
      selectedMappingId
      && filteredMappings.some((mapping) => mapping.id === selectedMappingId)
    ) {
      return;
    }

    setSelectedMappingId(filteredMappings[0]?.id ?? null);
    setIsDetailDrawerOpen(false);
  }, [filteredMappings, selectedMappingId]);

  const selectedMapping =
    filteredMappings.find((mapping) => mapping.id === selectedMappingId)
    ?? filteredMappings[0]
    ?? null;

  const activeCount = mappings.filter((mapping) => mapping.status === 'active').length;
  const totalRuleCount = mappings.reduce(
    (sum, mapping) => sum + mapping.rules.length,
    0,
  );

  const columns = useMemo<DataTableColumn<GatewayModelMappingRecord>[]>(
    () => [
      {
        id: 'mapping',
        header: t('Mapping'),
        cell: (mapping) => (
          <div className="space-y-1">
            <div className="font-semibold text-[var(--sdk-color-text-primary)]">
              {mapping.name}
            </div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {mapping.description || t('No description')}
            </div>
          </div>
        ),
      },
      {
        id: 'status',
        header: t('Status'),
        cell: (mapping) => (
          <StatusBadge
            label={mapping.status}
            showIcon
            status={mapping.status}
            variant={mapping.status === 'active' ? 'success' : 'secondary'}
          />
        ),
        width: 128,
      },
      {
        id: 'window',
        header: t('Effective window'),
        cell: (mapping) => (
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
            {formatDateLabel(mapping.effective_from)} {t('to')}{' '}
            {formatDateLabel(mapping.effective_to)}
          </div>
        ),
      },
      {
        id: 'rules',
        align: 'right',
        header: t('Rules'),
        cell: (mapping) => formatNumber(mapping.rules.length),
        width: 96,
      },
    ],
    [formatNumber, t],
  );

  function refreshMappings(): void {
    setOverlayVersion((value) => value + 1);
  }

  function resetEditor(): void {
    setEditingMappingId(null);
    setMappingDraft(emptyDraft(catalogValues));
    setIsEditorOpen(false);
  }

  function openCreateDialog(): void {
    setEditingMappingId(null);
    setMappingDraft(emptyDraft(catalogValues));
    setIsEditorOpen(true);
  }

  function openEditDialog(mapping: GatewayModelMappingRecord): void {
    setEditingMappingId(mapping.id);
    setMappingDraft(draftFromMapping(mapping));
    setIsEditorOpen(true);
  }

  function openDetailDrawer(mapping: GatewayModelMappingRecord): void {
    setSelectedMappingId(mapping.id);
    setIsDetailDrawerOpen(true);
  }

  function handleDetailDrawerOpenChange(open: boolean): void {
    setIsDetailDrawerOpen(open);
    if (!open) {
      setSelectedMappingId(null);
    }
  }

  function addRule(): void {
    const defaultValue = catalogValues[0] ?? '';

    setMappingDraft((current) => ({
      ...current,
      rules: [...current.rules, createRuleDraft(defaultValue, defaultValue)],
    }));
  }

  function removeRule(ruleId: string): void {
    setMappingDraft((current) => {
      const nextRules = current.rules.filter((rule) => rule.id !== ruleId);

      return {
        ...current,
        rules: nextRules.length
          ? nextRules
          : [createRuleDraft(catalogValues[0] ?? '', catalogValues[0] ?? '')],
      };
    });
  }

  function updateRule(
    ruleId: string,
    field: 'source_value' | 'target_value',
    value: string,
  ): void {
    setMappingDraft((current) => ({
      ...current,
      rules: current.rules.map((rule) =>
        rule.id === ruleId ? { ...rule, [field]: value } : rule),
    }));
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>): Promise<void> {
    event.preventDefault();

    const rules = mappingDraft.rules
      .map((rule) => {
        const source = catalogByValue.get(rule.source_value);
        const target = catalogByValue.get(rule.target_value);

        if (!source || !target) {
          return null;
        }

        return {
          id: rule.id,
          source_channel_id: source.channel_id,
          source_channel_name: source.channel_name,
          source_model_id: source.model_id,
          source_model_name: source.model_name,
          target_channel_id: target.channel_id,
          target_channel_name: target.channel_name,
          target_model_id: target.model_id,
          target_model_name: target.model_name,
        };
      })
      .filter((rule): rule is NonNullable<typeof rule> => Boolean(rule));

    if (!rules.length) {
      return;
    }

    if (editingMappingId) {
      updateGatewayModelMapping(editingMappingId, {
        name: mappingDraft.name,
        description: mappingDraft.description,
        status: mappingDraft.status,
        effective_from: mappingDraft.effective_from,
        effective_to: mappingDraft.effective_to || null,
        rules,
      });
    } else {
      createGatewayModelMapping({
        name: mappingDraft.name,
        description: mappingDraft.description,
        effective_from: mappingDraft.effective_from,
        effective_to: mappingDraft.effective_to || null,
        rules,
      });
    }

    refreshMappings();
    resetEditor();
  }

  async function confirmDelete(): Promise<void> {
    if (!pendingDelete) {
      return;
    }

    deleteGatewayModelMapping(pendingDelete.id);
    refreshMappings();
    setPendingDelete(null);

    if (selectedMappingId === pendingDelete.id) {
      setSelectedMappingId(null);
      setIsDetailDrawerOpen(false);
    }
  }

  function toggleMappingStatus(mapping: GatewayModelMappingRecord): void {
    updateGatewayModelMappingStatus(
      mapping.id,
      mapping.status === 'active' ? 'disabled' : 'active',
    );
    refreshMappings();
  }

  return (
    <>
      <div className="flex h-full min-h-0 flex-col gap-4 p-4 lg:p-5">
        <Card className="shrink-0">
          <CardContent className="p-4">
            <form
              className="flex flex-wrap items-center gap-3"
              onSubmit={(event) => event.preventDefault()}
            >
              <div className="min-w-[18rem] flex-[1.5]">
                <Label className="sr-only" htmlFor="gateway-mapping-search">
                  {t('Search mappings')}
                </Label>
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--sdk-color-text-muted)]" />
                  <Input
                    className="pl-9"
                    id="gateway-mapping-search"
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                      setSearch(event.target.value)
                    }
                    placeholder={t('name, model, channel')}
                    value={search}
                  />
                </div>
              </div>
              <div className="min-w-[12rem]">
                <SelectField<'all' | GatewayModelMappingStatus>
                  label={t('Status')}
                  labelVisibility="sr-only"
                  onValueChange={setStatusFilter}
                  options={[
                    { label: t('All mappings'), value: 'all' },
                    { label: t('Active'), value: 'active' },
                    { label: t('Disabled'), value: 'disabled' },
                  ]}
                  placeholder={t('Status')}
                  value={statusFilter}
                />
              </div>

              <div className="ml-auto flex flex-wrap items-center self-center gap-2">
                <div className="hidden text-sm text-[var(--sdk-color-text-secondary)] xl:block">
                  {t('{count} visible', { count: formatNumber(filteredMappings.length) })}
                  {' | '}
                  {t('{count} active', { count: formatNumber(activeCount) })}
                  {' | '}
                  {t('{count} rules', { count: formatNumber(totalRuleCount) })}
                </div>
                <Button onClick={openCreateDialog} type="button" variant="primary">
                  <Plus className="w-4 h-4" />
                  {t('New model mapping')}
                </Button>
                <Button onClick={refreshMappings} type="button" variant="outline">
                  {t('Refresh overlay')}
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>

        <div className="min-h-0 flex-1">
          <GatewayModelMappingsRegistrySection
            activeCount={activeCount}
            columns={columns}
            filteredMappings={filteredMappings}
            mappings={mappings}
            onDeleteMapping={setPendingDelete}
            onEditMapping={openEditDialog}
            onSelectMapping={openDetailDrawer}
            onToggleStatus={(mappingId, nextStatus) => {
              updateGatewayModelMappingStatus(mappingId, nextStatus);
              refreshMappings();
            }}
            selectedMapping={selectedMapping}
            totalRuleCount={totalRuleCount}
          />
        </div>
      </div>

      <GatewayModelMappingsDetailDrawer
        onDelete={() => {
          if (!selectedMapping) {
            return;
          }
          setPendingDelete(selectedMapping);
        }}
        onEdit={() => {
          if (!selectedMapping) {
            return;
          }
          handleDetailDrawerOpenChange(false);
          openEditDialog(selectedMapping);
        }}
        onOpenChange={handleDetailDrawerOpenChange}
        onToggleStatus={() => {
          if (!selectedMapping) {
            return;
          }
          toggleMappingStatus(selectedMapping);
        }}
        open={isDetailDrawerOpen}
        selectedMapping={selectedMapping}
      />

      <GatewayModelMappingEditorDialog
        catalog={catalog}
        editingMappingId={editingMappingId}
        mappingDraft={mappingDraft}
        onAddRule={addRule}
        onOpenChange={(nextOpen) =>
          nextOpen ? setIsEditorOpen(true) : resetEditor()
        }
        onRemoveRule={removeRule}
        onSubmit={(event) => void handleSubmit(event)}
        onUpdateRule={updateRule}
        open={isEditorOpen}
        setMappingDraft={setMappingDraft}
      />

      <ConfirmActionDialog
        confirmLabel={t('Delete mapping')}
        description={
          pendingDelete
            ? t(
                'Delete {name}. Any API key overlay using this mapping will be detached automatically.',
                { name: pendingDelete.name },
              )
            : ''
        }
        onConfirm={confirmDelete}
        onOpenChange={(open) => {
          if (!open) {
            setPendingDelete(null);
          }
        }}
        open={Boolean(pendingDelete)}
        title={t('Delete model mapping')}
      />
    </>
  );
}
