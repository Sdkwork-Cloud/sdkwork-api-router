import { createStrictKeyedCatalog } from '../strict-contract-catalog.mjs';

export const RELEASE_CLI_FORMATS = [
  'text',
  'json',
];

const releaseCliFormatCatalog = createStrictKeyedCatalog({
  entries: RELEASE_CLI_FORMATS,
  getKey: (format) => format,
  duplicateKeyMessagePrefix: 'duplicate release cli format',
  missingKeyMessagePrefix: 'missing release cli format',
});

export function listReleaseCliFormats() {
  return releaseCliFormatCatalog.list();
}

export function findReleaseCliFormat(format) {
  return releaseCliFormatCatalog.find(format);
}

export function listReleaseCliFormatsByIds(formatIds = []) {
  return releaseCliFormatCatalog.listByKeys(formatIds);
}

export function assertSupportedReleaseCliFormat(format) {
  try {
    return findReleaseCliFormat(format);
  } catch (_error) {
    throw new Error(`unsupported format: ${format}`);
  }
}
