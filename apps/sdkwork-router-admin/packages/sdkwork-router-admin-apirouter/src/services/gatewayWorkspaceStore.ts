interface BrowserStorageLike {
  getItem: (key: string) => string | null;
  setItem: (key: string, value: string) => void;
  removeItem: (key: string) => void;
}

interface GatewayWorkspaceStoreOptions {
  localStorage?: BrowserStorageLike | null;
}

interface GatewayWorkspaceStoreRecordCatalog {
  modelMappings: GatewayModelMappingRecord[];
  keyOverlays: Record<string, GatewayApiKeyOverlayRecord>;
}

export interface GatewayModelMappingRule {
  id: string;
  source_channel_id: string;
  source_channel_name: string;
  source_model_id: string;
  source_model_name: string;
  target_channel_id: string;
  target_channel_name: string;
  target_model_id: string;
  target_model_name: string;
}

export interface GatewayModelMappingRecord {
  id: string;
  name: string;
  description: string;
  status: GatewayModelMappingStatus;
  effective_from: string;
  effective_to?: string | null;
  created_at: string;
  rules: GatewayModelMappingRule[];
}

export type GatewayRouteMode = 'sdkwork-remote' | 'custom';

export type GatewayModelMappingStatus = 'active' | 'disabled';

export interface GatewayApiKeyOverlayRecord {
  source: 'system-generated' | 'custom';
  route_mode: GatewayRouteMode;
  route_provider_id?: string | null;
  model_mapping_id?: string | null;
  updated_at_ms: number;
}

const API_ROUTER_MODEL_MAPPINGS_STORAGE_KEY =
  'sdkwork-router-admin.api-router.model-mappings';
const API_ROUTER_KEY_OVERLAYS_STORAGE_KEY =
  'sdkwork-router-admin.api-router.key-overlays';

let fallbackRecords: GatewayWorkspaceStoreRecordCatalog = {
  modelMappings: [],
  keyOverlays: {},
};

function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

function resolveBrowserStorage(
  explicitStorage?: BrowserStorageLike | null,
): BrowserStorageLike | null {
  if (explicitStorage !== undefined) {
    return explicitStorage;
  }

  try {
    if (globalThis.localStorage) {
      return globalThis.localStorage;
    }
  } catch {
    return null;
  }

  try {
    if (typeof window === 'undefined') {
      return null;
    }

    return window.localStorage ?? null;
  } catch {
    return null;
  }
}

function readJson<T>(
  storage: BrowserStorageLike | null,
  key: string,
  fallback: T,
): T {
  if (!storage) {
    return clone(fallback);
  }

  try {
    const rawValue = storage.getItem(key);
    if (!rawValue) {
      return clone(fallback);
    }

    return JSON.parse(rawValue) as T;
  } catch {
    return clone(fallback);
  }
}

function writeJson(
  storage: BrowserStorageLike | null,
  key: string,
  value: unknown,
): void {
  if (!storage) {
    return;
  }

  try {
    const normalizedValue =
      typeof value === 'object' && value !== null ? value : { value };
    const isEmptyObject =
      Object.keys(normalizedValue as Record<string, unknown>).length === 0;
    const isEmptyArray = Array.isArray(value) && value.length === 0;

    if (isEmptyObject || isEmptyArray) {
      storage.removeItem(key);
      return;
    }

    storage.setItem(key, JSON.stringify(value));
  } catch {
    // Ignore write failures and keep the in-memory fallback authoritative.
  }
}

function readRecords(
  options: GatewayWorkspaceStoreOptions = {},
): GatewayWorkspaceStoreRecordCatalog {
  const storage = resolveBrowserStorage(options.localStorage);

  fallbackRecords = {
    modelMappings: clone(
      readJson<GatewayModelMappingRecord[]>(
        storage,
        API_ROUTER_MODEL_MAPPINGS_STORAGE_KEY,
        fallbackRecords.modelMappings,
      ),
    ),
    keyOverlays: clone(
      readJson<Record<string, GatewayApiKeyOverlayRecord>>(
        storage,
        API_ROUTER_KEY_OVERLAYS_STORAGE_KEY,
        fallbackRecords.keyOverlays,
      ),
    ),
  };

  return clone(fallbackRecords);
}

function persistRecords(
  records: GatewayWorkspaceStoreRecordCatalog,
  options: GatewayWorkspaceStoreOptions = {},
): void {
  fallbackRecords = clone(records);
  const storage = resolveBrowserStorage(options.localStorage);

  writeJson(storage, API_ROUTER_MODEL_MAPPINGS_STORAGE_KEY, fallbackRecords.modelMappings);
  writeJson(storage, API_ROUTER_KEY_OVERLAYS_STORAGE_KEY, fallbackRecords.keyOverlays);
}

export function createGatewayWorkspaceStore(
  options: GatewayWorkspaceStoreOptions = {},
) {
  return {
    listModelMappings() {
      return readRecords(options).modelMappings;
    },
    writeModelMappings(items: GatewayModelMappingRecord[]) {
      const records = readRecords(options);
      records.modelMappings = clone(items);
      persistRecords(records, options);
    },
    readKeyOverlays() {
      return readRecords(options).keyOverlays;
    },
    writeKeyOverlays(items: Record<string, GatewayApiKeyOverlayRecord>) {
      const records = readRecords(options);
      records.keyOverlays = clone(items);
      persistRecords(records, options);
    },
  };
}
