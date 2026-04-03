import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('models center ships a searchable multimodal catalog with provider and modality filters', () => {
  const catalogPath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-portal-models',
    'src',
    'catalog.ts',
  );
  const modelsPage = read('packages/sdkwork-router-portal-models/src/index.tsx');

  assert.equal(existsSync(catalogPath), true, 'missing models catalog');

  const catalog = read('packages/sdkwork-router-portal-models/src/catalog.ts');

  assert.match(catalog, /Volcengine/);
  assert.match(catalog, /Alibaba Cloud/);
  assert.match(catalog, /Baidu AI Cloud/);
  assert.match(catalog, /DeepSeek/);
  assert.match(catalog, /Anthropic/);
  assert.match(catalog, /Google/);
  assert.match(catalog, /multimodal|vision|audio|embedding/i);

  assert.match(modelsPage, /useDeferredValue/);
  assert.match(modelsPage, /startTransition/);
  assert.match(modelsPage, /PortalSiteHero/);
  assert.match(modelsPage, /SearchInput|Input/);
  assert.match(modelsPage, /SelectTrigger/);
  assert.match(modelsPage, /DataTable/);
  assert.match(modelsPage, /portal-models-filter-bar/);
  assert.match(modelsPage, /portal-models-featured/);
  assert.match(modelsPage, /portal-models-provider-lanes/);
  assert.match(modelsPage, /portal-models-selection-tracks/);
  assert.match(modelsPage, /providerFilter|provider_filter/);
  assert.match(modelsPage, /modalityFilter|modality_filter/);
  assert.match(modelsPage, /filteredModels|visibleModels/);
  assert.match(modelsPage, /Featured model/);
  assert.match(modelsPage, /Provider lanes/);
  assert.match(modelsPage, /Selection tracks/);
  assert.match(modelsPage, /Best for global assistants/);
  assert.match(modelsPage, /Best for Chinese knowledge/);
  assert.match(modelsPage, /Best for coding agents/);
  assert.match(modelsPage, /Search, compare, and shortlist models with the same product language used throughout the public site and console\./);
  assert.match(modelsPage, /Compare multimodal providers, context posture, pricing bands, and launch fit before routing traffic\./);
});
