import type {
  GatewayModelMappingRecord,
  GatewayModelMappingStatus,
} from '../../services/gatewayOverlayStore';
import { translateAdminText } from 'sdkwork-router-admin-core';

export type MappingRuleDraft = {
  id: string;
  source_value: string;
  target_value: string;
};

export type MappingDraft = {
  name: string;
  description: string;
  status: GatewayModelMappingStatus;
  effective_from: string;
  effective_to: string;
  rules: MappingRuleDraft[];
};

export function createRuleDraft(
  sourceValue = '',
  targetValue = '',
): MappingRuleDraft {
  return {
    id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    source_value: sourceValue,
    target_value: targetValue,
  };
}

function todayDateValue(): string {
  return new Date().toISOString().slice(0, 10);
}

export function emptyDraft(catalogValues: string[]): MappingDraft {
  const defaultValue = catalogValues[0] ?? '';

  return {
    name: '',
    description: '',
    status: 'active',
    effective_from: todayDateValue(),
    effective_to: '',
    rules: [createRuleDraft(defaultValue, defaultValue)],
  };
}

export function draftFromMapping(
  mapping: GatewayModelMappingRecord,
): MappingDraft {
  return {
    name: mapping.name,
    description: mapping.description,
    status: mapping.status,
    effective_from: mapping.effective_from,
    effective_to: mapping.effective_to ?? '',
    rules: mapping.rules.map((rule) =>
      createRuleDraft(
        `${rule.source_channel_id}::${rule.source_model_id}`,
        `${rule.target_channel_id}::${rule.target_model_id}`,
      )),
  };
}

export function formatDateLabel(value?: string | null): string {
  return value || translateAdminText('Open ended');
}
