function cloneCatalogEntry(entry) {
  if (Array.isArray(entry)) {
    return [...entry];
  }

  if (entry && typeof entry === 'object') {
    return {
      ...entry,
    };
  }

  return entry;
}

function cloneCatalogContract(contract) {
  return cloneCatalogEntry(contract);
}

export function createStrictKeyedCatalog({
  entries = [],
  getKey,
  clone = cloneCatalogEntry,
  duplicateKeyMessagePrefix = 'duplicate catalog key',
  missingKeyMessagePrefix = 'missing catalog entry',
} = {}) {
  if (typeof getKey !== 'function') {
    throw new TypeError('strict keyed catalog requires a getKey function');
  }

  const entriesByKey = new Map();
  for (const entry of entries) {
    const key = getKey(entry);
    if (entriesByKey.has(key)) {
      throw new Error(`${duplicateKeyMessagePrefix}: ${key}`);
    }

    entriesByKey.set(key, entry);
  }

  function find(key) {
    if (!entriesByKey.has(key)) {
      throw new Error(`${missingKeyMessagePrefix}: ${key}`);
    }

    return clone(entriesByKey.get(key));
  }

  return {
    list() {
      return entries.map(clone);
    },
    find,
    listByKeys(keys = []) {
      return keys.map(find);
    },
  };
}

export function createStrictContractCatalog({
  contracts = [],
  duplicateIdMessagePrefix = 'duplicate contract id',
  missingIdMessagePrefix = 'missing contract',
} = {}) {
  const catalog = createStrictKeyedCatalog({
    entries: contracts,
    getKey: (contract) => contract.id,
    clone: cloneCatalogContract,
    duplicateKeyMessagePrefix: duplicateIdMessagePrefix,
    missingKeyMessagePrefix: missingIdMessagePrefix,
  });

  return {
    list() {
      return catalog.list();
    },
    find(contractId) {
      return catalog.find(contractId);
    },
    listByIds(contractIds = []) {
      return catalog.listByKeys(contractIds);
    },
  };
}
