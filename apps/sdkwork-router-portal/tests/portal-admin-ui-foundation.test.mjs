import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');
const packagesRoot = path.join(appRoot, 'packages');
const ROOT_ALLOWED_IMPORTS = new Set([
  'cn',
  'copyText',
  'formatCurrency',
  'formatDateTime',
  'formatUnits',
  'PORTAL_LOCALE_OPTIONS',
  'PortalI18nProvider',
  'PortalLocale',
  'translatePortalText',
  'usePortalI18n',
]);
const LEGACY_FRAMEWORK_ALIASES = new Set(['MetricCard', 'SectionHero', 'Surface']);
const LEGACY_BUTTON_ALIASES = new Set(['InlineButton']);
const LEGACY_BADGE_ALIASES = new Set(['Pill']);

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function walkFiles(dir, output = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === 'dist' || entry.name === 'node_modules' || entry.name === 'tests') {
        continue;
      }

      walkFiles(fullPath, output);
      continue;
    }

    if (/\.(ts|tsx)$/.test(entry.name)) {
      output.push(fullPath);
    }
  }

  return output;
}

function readFrameworkSources() {
  const frameworkRoot = read('packages/sdkwork-router-portal-commons/src/framework.tsx');
  const frameworkSlices = [
    'actions.ts',
    'display.ts',
    'entry.ts',
    'feedback.ts',
    'form.tsx',
    'layout.ts',
    'overlays.ts',
    'shell.ts',
    'workbench.ts',
    'workspace.ts',
  ]
    .map((relativePath) =>
      read(`packages/sdkwork-router-portal-commons/src/framework/${relativePath}`),
    )
    .join('\n');

  return { frameworkRoot, frameworkSlices };
}

test('portal app adopts the shadcn and tailwind admin-ui foundation', () => {
  const packageJson = read('package.json');
  const packageManifest = JSON.parse(packageJson);
  const corePackageJson = read('packages/sdkwork-router-portal-core/package.json');
  const commonsPackageJson = read('packages/sdkwork-router-portal-commons/package.json');
  const dashboardPackageJson = read('packages/sdkwork-router-portal-dashboard/package.json');
  const viteConfig = read('vite.config.ts');
  const theme = read('src/theme.css');
  const mainFile = read('src/main.tsx');
  const mainLayout = read('packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx');
  const appProviders = read('packages/sdkwork-router-portal-core/src/application/providers/AppProviders.tsx');
  const themeManager = read('packages/sdkwork-router-portal-core/src/application/providers/ThemeManager.tsx');
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const { frameworkRoot, frameworkSlices } = readFrameworkSources();

  assert.match(packageJson, /@sdkwork\/ui-pc-react/);
  assert.match(packageJson, /@tailwindcss\/vite/);
  assert.match(packageJson, /tailwindcss/);
  assert.doesNotMatch(packageJson, /class-variance-authority/);
  assert.doesNotMatch(packageJson, /clsx/);
  assert.doesNotMatch(packageJson, /tailwind-merge/);
  assert.doesNotMatch(packageJson, /@radix-ui\/react-/);
  assert.doesNotMatch(packageJson, /recharts/);
  assert.equal(packageManifest.dependencies?.['lucide-react'], undefined);
  assert.equal(packageManifest.dependencies?.['react-router-dom'], undefined);
  assert.equal(packageManifest.dependencies?.zustand, undefined);
  assert.equal(packageManifest.devDependencies?.['lucide-react'], undefined);
  assert.equal(packageManifest.devDependencies?.['react-router-dom'], undefined);
  assert.equal(packageManifest.devDependencies?.zustand, undefined);
  assert.match(corePackageJson, /react-router-dom/);
  assert.match(corePackageJson, /zustand/);
  assert.match(commonsPackageJson, /"dependencies"/);
  assert.match(commonsPackageJson, /@sdkwork\/ui-pc-react/);
  assert.match(commonsPackageJson, /clsx/);
  assert.match(commonsPackageJson, /tailwind-merge/);
  assert.match(commonsPackageJson, /lucide-react/);
  assert.doesNotMatch(commonsPackageJson, /@radix-ui\/react-/);
  assert.doesNotMatch(commonsPackageJson, /class-variance-authority/);
  assert.match(dashboardPackageJson, /recharts/);
  assert.match(commonsPackageJson, /"\.\/framework"/);
  assert.match(commonsPackageJson, /"\.\/framework\/actions"/);
  assert.match(commonsPackageJson, /"\.\/framework\/display"/);
  assert.match(commonsPackageJson, /"\.\/framework\/entry"/);
  assert.match(commonsPackageJson, /"\.\/framework\/feedback"/);
  assert.match(commonsPackageJson, /"\.\/framework\/form"/);
  assert.match(commonsPackageJson, /"\.\/framework\/layout"/);
  assert.match(commonsPackageJson, /"\.\/framework\/overlays"/);
  assert.match(commonsPackageJson, /"\.\/framework\/shell"/);
  assert.match(commonsPackageJson, /"\.\/framework\/workbench"/);
  assert.match(commonsPackageJson, /"\.\/framework\/workspace"/);
  assert.match(commonsPackageJson, /"\.\/clipboard"/);

  assert.match(mainFile, /@sdkwork\/ui-pc-react\/styles\.css/);
  assert.match(mainLayout, /lazy\(async \(\) => \(\{/);
  assert.match(mainLayout, /await import\('\.\.\/\.\.\/components\/PortalSettingsCenter'\)/);
  assert.match(mainLayout, /<Suspense fallback=\{null\}>/);
  assert.match(appProviders, /SdkworkThemeProvider|PortalThemeProvider/);
  assert.match(themeManager, /createSdkworkTheme/);
  assert.match(viteConfig, /@tailwindcss\/vite/);
  assert.match(viteConfig, /manualChunks/);
  assert.match(viteConfig, /react-vendor/);
  assert.match(viteConfig, /radix-vendor/);
  assert.match(viteConfig, /charts-vendor/);
  assert.doesNotMatch(viteConfig, /icon-vendor/);
  assert.match(theme, /@import "tailwindcss";/);
  assert.doesNotMatch(frameworkRoot, /from '@sdkwork\/ui-pc-react'/);
  assert.match(frameworkRoot, /export \* from '\.\/framework\/actions';/);
  assert.match(frameworkRoot, /export \* from '\.\/framework\/display';/);
  assert.match(frameworkRoot, /export \* from '\.\/framework\/entry';/);
  assert.match(frameworkRoot, /export \* from '\.\/framework\/feedback';/);
  assert.match(frameworkRoot, /export \* from '\.\/framework\/form';/);
  assert.match(frameworkRoot, /export \* from '\.\/framework\/layout';/);
  assert.match(frameworkRoot, /export \* from '\.\/framework\/overlays';/);
  assert.match(frameworkRoot, /export \* from '\.\/framework\/shell';/);
  assert.match(frameworkRoot, /export \* from '\.\/framework\/workbench';/);
  assert.match(frameworkRoot, /export \* from '\.\/framework\/workspace';/);
  assert.match(frameworkSlices, /@sdkwork\/ui-pc-react\/components\/ui\/actions/);
  assert.match(frameworkSlices, /@sdkwork\/ui-pc-react\/components\/ui\/data-entry/);
  assert.match(frameworkSlices, /@sdkwork\/ui-pc-react\/components\/ui\/data-display/);
  assert.match(frameworkSlices, /@sdkwork\/ui-pc-react\/components\/ui\/form/);
  assert.match(frameworkSlices, /@sdkwork\/ui-pc-react\/components\/ui\/feedback/);
  assert.match(frameworkSlices, /@sdkwork\/ui-pc-react\/components\/ui\/layout/);
  assert.match(frameworkSlices, /@sdkwork\/ui-pc-react\/components\/ui\/overlays/);
  assert.match(frameworkSlices, /@sdkwork\/ui-pc-react\/components\/patterns\/app-shell/);
  assert.match(frameworkSlices, /@sdkwork\/ui-pc-react\/components\/patterns\/desktop-shell/);
  assert.match(frameworkSlices, /@sdkwork\/ui-pc-react\/components\/patterns\/workbench/);
  assert.match(frameworkSlices, /@sdkwork\/ui-pc-react\/components\/patterns\/workspace/);
  assert.doesNotMatch(viteConfig, /find:\s*\/\^@sdkwork\\\/ui-pc-react\$\//);
  assert.doesNotMatch(viteConfig, /sdkwork-ui-pc-react\/dist\/index\.js/);
  assert.doesNotMatch(commons, /export \* from '\.\/framework';/);
  assert.match(commons, /export \{ copyText \} from '\.\/clipboard';/);
});

test('portal commons stops owning checkbox, textarea, modal, and table primitives locally', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const { frameworkRoot, frameworkSlices } = readFrameworkSources();

  assert.doesNotMatch(frameworkRoot, /from '@sdkwork\/ui-pc-react'/);
  assert.match(frameworkSlices, /Button/);
  assert.match(frameworkSlices, /Checkbox/);
  assert.match(frameworkSlices, /DataTable/);
  assert.match(frameworkSlices, /Dialog/);
  assert.match(frameworkSlices, /DialogContent/);
  assert.match(frameworkSlices, /Input/);
  assert.match(frameworkSlices, /Modal/);
  assert.match(frameworkSlices, /Select/);
  assert.match(frameworkSlices, /Textarea/);
  assert.doesNotMatch(commons, /CheckboxPrimitive\.Root/);
  assert.doesNotMatch(commons, /export const Checkbox = forwardRef/);
  assert.doesNotMatch(commons, /export const Textarea = forwardRef/);
  assert.doesNotMatch(commons, /showCloseButton\?: boolean/);
  assert.doesNotMatch(commons, /export function Modal/);
  assert.doesNotMatch(commons, /const dialogSizeClassNames =/);
  assert.doesNotMatch(commons, /data-slot="table-container"/);
  assert.doesNotMatch(commons, /data-slot="table-header"/);
  assert.doesNotMatch(commons, /data-slot="table-empty"/);
});

test('portal search primitives stay thin app-owned while auth-only icon inputs stop living in the framework adapter', () => {
  const { frameworkRoot, frameworkSlices } = readFrameworkSources();

  assert.match(frameworkRoot, /export \* from '\.\/framework\/form';/);
  assert.match(frameworkRoot, /export \* from '\.\/framework\/display';/);
  assert.match(frameworkRoot, /export \* from '\.\/framework\/entry';/);
  assert.match(frameworkRoot, /export \* from '\.\/framework\/layout';/);
  assert.match(frameworkRoot, /export \* from '\.\/framework\/overlays';/);
  assert.match(frameworkRoot, /export \* from '\.\/framework\/shell';/);
  assert.match(frameworkRoot, /export \* from '\.\/framework\/workbench';/);
  assert.match(frameworkRoot, /export \* from '\.\/framework\/workspace';/);
  assert.doesNotMatch(frameworkRoot, /export function SearchInput/);
  assert.doesNotMatch(frameworkRoot, /SearchIcon/);
  assert.match(frameworkSlices, /export function SearchInput/);
  assert.match(frameworkSlices, /SearchIcon/);
  assert.match(frameworkSlices, /style=\{\{ \.\.\.style, paddingLeft: '2\.75rem' \}\}/);
  assert.doesNotMatch(frameworkSlices, /portalx-search-input/);
  assert.doesNotMatch(frameworkSlices, /portalx-search-input-element/);
});

test('portal commons root stays utility-only while visual modules import from the framework subpath', () => {
  const files = walkFiles(packagesRoot);
  const offenders = [];

  for (const file of files) {
    if (file.includes(`${path.sep}sdkwork-router-portal-commons${path.sep}`)) {
      continue;
    }

    const source = readFileSync(file, 'utf8');
    const matches = source.matchAll(/import\s*\{([\s\S]*?)\}\s*from\s*'([^']+)';/g);

    for (const match of matches) {
      if (match[2] !== 'sdkwork-router-portal-commons') {
        continue;
      }

      const specifiers = match[1]
        .split(',')
        .map((item) => item.trim())
        .filter(Boolean)
        .map((item) => item.split(/\s+as\s+/i)[0]?.trim() ?? '');
      const invalidRootImports = specifiers.filter((specifier) => !ROOT_ALLOWED_IMPORTS.has(specifier));

      if (invalidRootImports.length) {
        offenders.push({
          file: path.relative(appRoot, file),
          imports: invalidRootImports,
        });
      }
    }
  }

  assert.deepEqual(offenders, []);
});

test('portal visual modules consume framework slices instead of the umbrella framework barrel', () => {
  const files = walkFiles(packagesRoot);
  const offenders = [];

  for (const file of files) {
    if (file.includes(`${path.sep}sdkwork-router-portal-commons${path.sep}`)) {
      continue;
    }

    const source = readFileSync(file, 'utf8');

    if (source.includes("from 'sdkwork-router-portal-commons/framework'")) {
      offenders.push(path.relative(appRoot, file));
    }
  }

  assert.deepEqual(offenders, []);
});

test('portal visual modules avoid direct shared-ui imports outside the framework adapter and theme boundary', () => {
  const files = walkFiles(packagesRoot);
  const allowedDirectImports = new Set([
    path.join(packagesRoot, 'sdkwork-router-portal-commons', 'src', 'framework.tsx'),
    path.join(
      packagesRoot,
      'sdkwork-router-portal-core',
      'src',
      'application',
      'providers',
      'ThemeManager.tsx',
    ),
  ]);
  const offenders = [];

  for (const file of files) {
    if (allowedDirectImports.has(file)) {
      continue;
    }

    const source = readFileSync(file, 'utf8');

    if (source.includes("from '@sdkwork/ui-pc-react'")) {
      offenders.push(path.relative(appRoot, file));
    }
  }

  assert.deepEqual(offenders, []);
});

test('portal framework adapter removes legacy surface and stat aliases in favor of shared framework semantics', () => {
  const files = walkFiles(packagesRoot);
  const { frameworkSlices } = readFrameworkSources();
  const offenders = [];

  for (const file of files) {
    if (file.includes(`${path.sep}sdkwork-router-portal-commons${path.sep}`)) {
      continue;
    }

    const source = readFileSync(file, 'utf8');
    const matches = source.matchAll(/import\s*\{([\s\S]*?)\}\s*from\s*'([^']+)';/g);

    for (const match of matches) {
      if (match[2] !== 'sdkwork-router-portal-commons/framework') {
        continue;
      }

      const specifiers = match[1]
        .split(',')
        .map((item) => item.trim())
        .filter(Boolean)
        .map((item) => item.split(/\s+as\s+/i)[0]?.trim() ?? '');
      const legacyImports = specifiers.filter((specifier) => LEGACY_FRAMEWORK_ALIASES.has(specifier));

      if (legacyImports.length) {
        offenders.push({
          file: path.relative(appRoot, file),
          imports: legacyImports,
        });
      }
    }
  }

  assert.doesNotMatch(frameworkSlices, /export function MetricCard/);
  assert.doesNotMatch(frameworkSlices, /export function SectionHero/);
  assert.doesNotMatch(frameworkSlices, /export function Surface/);
  assert.match(frameworkSlices, /StatCard/);
  assert.match(frameworkSlices, /WorkspacePanel/);
  assert.deepEqual(offenders, []);
});

test('portal framework adapter removes legacy inline button aliases in favor of shared button semantics', () => {
  const files = walkFiles(packagesRoot);
  const { frameworkSlices } = readFrameworkSources();
  const offenders = [];

  for (const file of files) {
    if (file.includes(`${path.sep}sdkwork-router-portal-commons${path.sep}`)) {
      continue;
    }

    const source = readFileSync(file, 'utf8');
    const matches = source.matchAll(/import\s*\{([\s\S]*?)\}\s*from\s*'([^']+)';/g);

    for (const match of matches) {
      if (match[2] !== 'sdkwork-router-portal-commons/framework') {
        continue;
      }

      const specifiers = match[1]
        .split(',')
        .map((item) => item.trim())
        .filter(Boolean)
        .map((item) => item.split(/\s+as\s+/i)[0]?.trim() ?? '');
      const legacyImports = specifiers.filter((specifier) => LEGACY_BUTTON_ALIASES.has(specifier));

      if (legacyImports.length) {
        offenders.push({
          file: path.relative(appRoot, file),
          imports: legacyImports,
        });
      }
    }
  }

  assert.doesNotMatch(frameworkSlices, /export function InlineButton/);
  assert.match(frameworkSlices, /Button/);
  assert.deepEqual(offenders, []);
});

test('portal framework adapter removes legacy pill aliases in favor of shared badge semantics', () => {
  const files = walkFiles(packagesRoot);
  const { frameworkSlices } = readFrameworkSources();
  const offenders = [];

  for (const file of files) {
    if (file.includes(`${path.sep}sdkwork-router-portal-commons${path.sep}`)) {
      continue;
    }

    const source = readFileSync(file, 'utf8');
    const matches = source.matchAll(/import\s*\{([\s\S]*?)\}\s*from\s*'([^']+)';/g);

    for (const match of matches) {
      if (match[2] !== 'sdkwork-router-portal-commons/framework') {
        continue;
      }

      const specifiers = match[1]
        .split(',')
        .map((item) => item.trim())
        .filter(Boolean)
        .map((item) => item.split(/\s+as\s+/i)[0]?.trim() ?? '');
      const legacyImports = specifiers.filter((specifier) => LEGACY_BADGE_ALIASES.has(specifier));

      if (legacyImports.length) {
        offenders.push({
          file: path.relative(appRoot, file),
          imports: legacyImports,
        });
      }
    }
  }

  assert.doesNotMatch(frameworkSlices, /export function Pill/);
  assert.doesNotMatch(frameworkSlices, /resolveBadgeVariant/);
  assert.match(frameworkSlices, /Badge/);
  assert.deepEqual(offenders, []);
});

test('portal visual modules bind badge variants directly to shared-ui semantics without tone adapters', () => {
  const files = walkFiles(packagesRoot);
  const { frameworkSlices } = readFrameworkSources();
  const offenders = [];

  for (const file of files) {
    if (file.includes(`${path.sep}sdkwork-router-portal-commons${path.sep}`)) {
      continue;
    }

    const source = readFileSync(file, 'utf8');

    if (source.includes('resolveBadgeVariant')) {
      offenders.push(path.relative(appRoot, file));
    }
  }

  assert.doesNotMatch(frameworkSlices, /resolveBadgeVariant/);
  assert.deepEqual(offenders, []);
});

test('portal framework adapter no longer owns a legacy DataTable compatibility layer', () => {
  const { frameworkSlices } = readFrameworkSources();
  const tablePages = [
    'packages/sdkwork-router-portal-account/src/pages/index.tsx',
    'packages/sdkwork-router-portal-billing/src/pages/index.tsx',
    'packages/sdkwork-router-portal-credits/src/pages/index.tsx',
    'packages/sdkwork-router-portal-dashboard/src/pages/index.tsx',
    'packages/sdkwork-router-portal-gateway/src/pages/index.tsx',
    'packages/sdkwork-router-portal-routing/src/pages/index.tsx',
    'packages/sdkwork-router-portal-usage/src/pages/index.tsx',
  ].map((relativePath) => ({
    relativePath,
    source: read(relativePath),
  }));

  assert.doesNotMatch(frameworkSlices, /type LegacyDataTableColumn/);
  assert.doesNotMatch(frameworkSlices, /type LegacyDataTableProps/);
  assert.doesNotMatch(frameworkSlices, /export function DataTable/);
  assert.match(frameworkSlices, /DataTable/);

  for (const { relativePath, source } of tablePages) {
    assert.doesNotMatch(source, /\bempty=\{/);
    assert.doesNotMatch(source, /\bgetKey=\{/);
    assert.match(source, /\bemptyState=\{/);
    assert.match(source, /\bgetRowId=\{/);
    assert.match(source, /\bheader:/);
    assert.match(source, /\bcell:/);
  }
});

test('portal framework adapter no longer owns legacy Select and Checkbox compatibility layers', () => {
  const { frameworkSlices } = readFrameworkSources();
  const selectConsumers = [
    'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyCreateForm.tsx',
    'packages/sdkwork-router-portal-api-keys/src/pages/index.tsx',
    'packages/sdkwork-router-portal-billing/src/pages/index.tsx',
    'packages/sdkwork-router-portal-core/src/components/PortalSettingsCenter.tsx',
    'packages/sdkwork-router-portal-credits/src/pages/index.tsx',
    'packages/sdkwork-router-portal-gateway/src/pages/index.tsx',
    'packages/sdkwork-router-portal-routing/src/pages/index.tsx',
    'packages/sdkwork-router-portal-usage/src/pages/index.tsx',
  ].map((relativePath) => ({
    relativePath,
    source: read(relativePath),
  }));
  const checkboxConsumers = [
    'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyDrawers.tsx',
    'packages/sdkwork-router-portal-core/src/components/PortalSettingsCenter.tsx',
    'packages/sdkwork-router-portal-routing/src/pages/index.tsx',
  ].map((relativePath) => ({
    relativePath,
    source: read(relativePath),
  }));

  assert.doesNotMatch(frameworkSlices, /type LegacyCheckboxEvent/);
  assert.doesNotMatch(frameworkSlices, /const EMPTY_SELECT_VALUE/);
  assert.doesNotMatch(frameworkSlices, /function encodeSelectValue/);
  assert.doesNotMatch(frameworkSlices, /function decodeSelectValue/);
  assert.doesNotMatch(frameworkSlices, /function extractSelectOptions/);
  assert.doesNotMatch(frameworkSlices, /export function Checkbox/);
  assert.doesNotMatch(frameworkSlices, /export function Select/);
  assert.match(frameworkSlices, /Checkbox/);
  assert.match(frameworkSlices, /Select/);
  assert.match(frameworkSlices, /SelectContent/);
  assert.match(frameworkSlices, /SelectItem/);
  assert.match(frameworkSlices, /SelectTrigger/);
  assert.match(frameworkSlices, /SelectValue/);

  for (const { relativePath, source } of selectConsumers) {
    assert.doesNotMatch(source, /<option\b/, relativePath);
    assert.match(source, /onValueChange=/, relativePath);
    assert.match(source, /<SelectTrigger\b/, relativePath);
    assert.match(source, /<SelectContent\b/, relativePath);
    assert.match(source, /<SelectItem\b/, relativePath);
  }

  for (const { relativePath, source } of checkboxConsumers) {
    assert.doesNotMatch(source, /event\.target\.checked/, relativePath);
    assert.match(source, /onCheckedChange=/, relativePath);
  }
});

test('portal framework adapter no longer owns a legacy Modal compatibility layer', () => {
  const { frameworkSlices } = readFrameworkSources();

  assert.doesNotMatch(frameworkSlices, /export interface ModalProps/);
  assert.doesNotMatch(frameworkSlices, /export function Modal/);
  assert.match(frameworkSlices, /Modal/);
  assert.match(frameworkSlices, /ModalContent/);
  assert.match(frameworkSlices, /ModalHeader/);
  assert.match(frameworkSlices, /ModalBody/);
  assert.match(frameworkSlices, /ModalFooter/);
});

test('portal framework adapter no longer owns a legacy EmptyState compatibility layer', () => {
  const { frameworkSlices } = readFrameworkSources();
  const emptyStateConsumers = [
    'packages/sdkwork-router-portal-account/src/pages/index.tsx',
    'packages/sdkwork-router-portal-billing/src/pages/index.tsx',
    'packages/sdkwork-router-portal-dashboard/src/pages/index.tsx',
    'packages/sdkwork-router-portal-gateway/src/pages/index.tsx',
    'packages/sdkwork-router-portal-routing/src/pages/index.tsx',
  ].map((relativePath) => ({
    relativePath,
    source: read(relativePath),
  }));

  assert.doesNotMatch(frameworkSlices, /export function EmptyState/);
  assert.doesNotMatch(frameworkSlices, /\bdetail\?: ReactNode/);
  assert.match(frameworkSlices, /EmptyState/);

  for (const { relativePath, source } of emptyStateConsumers) {
    assert.doesNotMatch(source, /<EmptyState\b[^>]*\bdetail=/, relativePath);
    assert.match(source, /<EmptyState\b[^>]*\bdescription=/, relativePath);
  }
});
