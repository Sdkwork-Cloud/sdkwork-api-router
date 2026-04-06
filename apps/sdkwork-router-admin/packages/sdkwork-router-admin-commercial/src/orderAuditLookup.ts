export function normalizeCommercialOrderAuditLookupValue(value: string) {
  return value.trim();
}

export function hasCommercialOrderAuditLookupValue(value: string) {
  return normalizeCommercialOrderAuditLookupValue(value).length > 0;
}
