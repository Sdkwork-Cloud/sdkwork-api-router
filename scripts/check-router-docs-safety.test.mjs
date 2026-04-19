import assert from 'node:assert/strict';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

function readWorkspaceFile(relativePath) {
  return readFileSync(path.join(workspaceRoot, relativePath), 'utf8');
}

function listWorkspaceMarkdownFiles(relativeRoot) {
  const absoluteRoot = path.join(workspaceRoot, relativeRoot);
  if (!existsSync(absoluteRoot)) {
    return [];
  }

  const results = [];
  for (const entry of readdirSync(absoluteRoot, { withFileTypes: true })) {
    const relativePath = path.posix.join(relativeRoot.replaceAll('\\', '/'), entry.name);
    if (entry.isDirectory()) {
      results.push(...listWorkspaceMarkdownFiles(relativePath));
      continue;
    }
    if (entry.isFile() && relativePath.endsWith('.md')) {
      results.push(relativePath);
    }
  }

  return results;
}

test('router product docs stay free of retired fixed bootstrap credentials', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-router-docs-safety.mjs')).href,
  );

  const findings = module.scanDocsForRetiredBootstrapCredentials({
    workspaceRoot,
  });

  assert.deepEqual(findings, []);
});

test('router docs safety scan only targets product-facing docs trees', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-router-docs-safety.mjs')).href,
  );

  assert.deepEqual(module.DOC_BOOTSTRAP_SCAN_ROOTS, [
    'docs/getting-started',
    'docs/api-reference',
    'docs/operations',
    'docs/zh/getting-started',
    'docs/zh/api-reference',
    'docs/zh/operations',
  ]);

  assert.deepEqual(module.DOC_BOOTSTRAP_SCAN_FILES, [
    'README.md',
    'README.zh-CN.md',
  ]);
});

test('router production docs publish a single deployment entrypoint with operations pages in both locales', () => {
  const requiredFiles = [
    'docs/getting-started/production-deployment.md',
    'docs/zh/getting-started/production-deployment.md',
    'docs/operations/install-layout.md',
    'docs/zh/operations/install-layout.md',
    'docs/operations/service-management.md',
    'docs/zh/operations/service-management.md',
  ];

  for (const relativePath of requiredFiles) {
    assert.equal(existsSync(path.join(workspaceRoot, relativePath)), true, `missing ${relativePath}`);
  }

  const vitepressConfig = readWorkspaceFile('docs/.vitepress/config.mjs');
  assert.match(vitepressConfig, /\/getting-started\/production-deployment/);
  assert.match(vitepressConfig, /\/operations\/install-layout/);
  assert.match(vitepressConfig, /\/operations\/service-management/);
  assert.match(vitepressConfig, /\/zh\/getting-started\/production-deployment/);
  assert.match(vitepressConfig, /\/zh\/operations\/install-layout/);
  assert.match(vitepressConfig, /\/zh\/operations\/service-management/);
});

test('product-facing docs publish a dedicated online release runbook in both locales', () => {
  const requiredFiles = [
    'docs/getting-started/online-release.md',
    'docs/zh/getting-started/online-release.md',
  ];

  for (const relativePath of requiredFiles) {
    assert.equal(existsSync(path.join(workspaceRoot, relativePath)), true, `missing ${relativePath}`);
  }

  const vitepressConfig = readWorkspaceFile('docs/.vitepress/config.mjs');
  const readme = readWorkspaceFile('README.md');
  const readmeZh = readWorkspaceFile('README.zh-CN.md');
  const releaseBuilds = readWorkspaceFile('docs/getting-started/release-builds.md');
  const releaseBuildsZh = readWorkspaceFile('docs/zh/getting-started/release-builds.md');
  const productionDeployment = readWorkspaceFile('docs/getting-started/production-deployment.md');
  const productionDeploymentZh = readWorkspaceFile('docs/zh/getting-started/production-deployment.md');

  assert.match(vitepressConfig, /\/getting-started\/online-release/);
  assert.match(vitepressConfig, /\/zh\/getting-started\/online-release/);
  assert.match(readme, /\(\.\/docs\/getting-started\/online-release\.md\)/);
  assert.match(readmeZh, /\(\.\/docs\/zh\/getting-started\/online-release\.md\)/);
  assert.match(releaseBuilds, /\/getting-started\/online-release/);
  assert.match(releaseBuildsZh, /\/zh\/getting-started\/online-release/);
  assert.match(productionDeployment, /\/getting-started\/online-release/);
  assert.match(productionDeploymentZh, /\/zh\/getting-started\/online-release/);
});

test('release-facing docs route GitHub-hosted publication details through the online release runbook in both locales', () => {
  const packaging = readWorkspaceFile('docs/getting-started/build-and-packaging.md');
  const packagingZh = readWorkspaceFile('docs/zh/getting-started/build-and-packaging.md');
  const onlineRelease = readWorkspaceFile('docs/getting-started/online-release.md');
  const onlineReleaseZh = readWorkspaceFile('docs/zh/getting-started/online-release.md');

  assert.match(packaging, /\/getting-started\/online-release/);
  assert.match(packagingZh, /\/zh\/getting-started\/online-release/);

  for (const content of [onlineRelease, onlineReleaseZh]) {
    assert.match(content, /workflow_dispatch/);
    assert.match(content, /SDKWORK_CORE_GIT_REF/);
    assert.match(content, /SDKWORK_RELEASE_DESKTOP_SIGNING_REQUIRED/);
    assert.match(content, /release-catalog\.json/);
  }
});

test('online release runbooks publish repository-owned attestation verification commands in both locales', () => {
  const onlineRelease = readWorkspaceFile('docs/getting-started/online-release.md');
  const onlineReleaseZh = readWorkspaceFile('docs/zh/getting-started/online-release.md');

  for (const content of [onlineRelease, onlineReleaseZh]) {
    assert.match(content, /verify-release-attestations\.mjs/);
    assert.match(content, /--format text/);
    assert.match(content, /--repo Sdkwork-Cloud\/sdkwork-api-router/);
  }
});

test('online release runbooks publish workflow-contract preflight checks for the governed release lanes in both locales', () => {
  const onlineRelease = readWorkspaceFile('docs/getting-started/online-release.md');
  const onlineReleaseZh = readWorkspaceFile('docs/zh/getting-started/online-release.md');

  for (const content of [onlineRelease, onlineReleaseZh]) {
    assert.match(content, /scripts\/release\/tests\/release-workflow\.test\.mjs/);
    assert.match(content, /scripts\/release-governance-workflow\.test\.mjs/);
    assert.match(content, /scripts\/product-verification-workflow\.test\.mjs/);
    assert.match(content, /scripts\/rust-verification-workflow\.test\.mjs/);
    assert.match(content, /scripts\/check-router-docs-safety\.test\.mjs/);
  }
});

test('vitepress docs config explicitly handles promql code fences without build warnings', () => {
  const vitepressConfig = readWorkspaceFile('docs/.vitepress/config.mjs');
  const alertsAndSlos = readWorkspaceFile('docs/operations/alerts-and-slos.md');

  assert.match(alertsAndSlos, /```promql/);
  assert.match(vitepressConfig, /languageAlias/);
  assert.match(vitepressConfig, /promql/);
  assert.match(vitepressConfig, /txt/);
  assert.match(vitepressConfig, /normalize-promql-fences/);
  assert.match(vitepressConfig, /token\.info/);
});

test('README and getting-started docs align to config-file-first production guidance', () => {
  const readme = readWorkspaceFile('README.md');
  const quickstart = readWorkspaceFile('docs/getting-started/quickstart.md');
  const releaseBuilds = readWorkspaceFile('docs/getting-started/release-builds.md');
  const deployReadme = readWorkspaceFile('deploy/README.md');

  assert.match(
    readme,
    /built-in defaults\s*->\s*environment fallback\s*->\s*config file\s*->\s*CLI/i,
  );
  assert.match(readme, /Production Deployment/i);
  assert.match(readme, /system installs default to PostgreSQL/i);
  assert.match(quickstart, /local development only/i);
  assert.match(quickstart, /Production Deployment/);
  assert.match(releaseBuilds, /build and package generation only/i);
  assert.match(releaseBuilds, /Production Deployment/);
  assert.match(deployReadme, /Docker and Helm asset-specific/i);
  assert.doesNotMatch(deployReadme, /system install/i);
});

test('product-facing config docs advertise yaml, yml, and json config overlays instead of yaml-only fragments', () => {
  const readme = readWorkspaceFile('README.md');
  const readmeZh = readWorkspaceFile('README.zh-CN.md');
  const configuration = readWorkspaceFile('docs/operations/configuration.md');
  const configurationZh = readWorkspaceFile('docs/zh/operations/configuration.md');
  const installLayout = readWorkspaceFile('docs/operations/install-layout.md');
  const installLayoutZh = readWorkspaceFile('docs/zh/operations/install-layout.md');
  const productionDeployment = readWorkspaceFile('docs/getting-started/production-deployment.md');
  const productionDeploymentZh = readWorkspaceFile('docs/zh/getting-started/production-deployment.md');

  for (const content of [
    readme,
    readmeZh,
    configuration,
    configurationZh,
    installLayout,
    installLayoutZh,
    productionDeployment,
    productionDeploymentZh,
  ]) {
    assert.match(content, /conf\.d\/\*\.\{yaml,yml,json\}/);
    assert.doesNotMatch(content, /conf\.d\/\*\.yaml/);
  }
});

test('install-layout docs publish the generated release-manifest control contract in both locales', () => {
  const installLayout = readWorkspaceFile('docs/operations/install-layout.md');
  const installLayoutZh = readWorkspaceFile('docs/zh/operations/install-layout.md');

  for (const content of [installLayout, installLayoutZh]) {
    assert.match(content, /current\/release-manifest\.json/);
    assert.match(content, /layoutVersion/);
    assert.match(content, /installMode/);
    assert.match(content, /productRoot/);
    assert.match(content, /controlRoot/);
    assert.match(content, /releasesRoot/);
    assert.match(content, /target/);
    assert.match(content, /installedBinaries/);
    assert.match(content, /configFile/);
    assert.match(content, /bootstrapDataRoot/);
    assert.match(content, /deploymentAssetRoot/);
    assert.match(content, /releasePayloadManifest/);
    assert.match(content, /releasePayloadReadmeFile/);
    assert.match(content, /mutableDataRoot/);
    assert.match(content, /installedAt/);
  }
});

test('installation docs clone the current sdkwork-api-router repository in both locales', () => {
  const installation = readWorkspaceFile('docs/getting-started/installation.md');
  const installationZh = readWorkspaceFile('docs/zh/getting-started/installation.md');

  assert.match(installation, /git clone https:\/\/github\.com\/Sdkwork-Cloud\/sdkwork-api-router\.git/);
  assert.match(installation, /cd sdkwork-api-router/);
  assert.doesNotMatch(installation, /sdkwork-api-server/);
  assert.doesNotMatch(installation, /SDKWork API Server/);
  assert.match(installationZh, /git clone https:\/\/github\.com\/Sdkwork-Cloud\/sdkwork-api-router\.git/);
  assert.match(installationZh, /cd sdkwork-api-router/);
  assert.doesNotMatch(installationZh, /sdkwork-api-server/);
  assert.doesNotMatch(installationZh, /SDKWork API Server/);
});

test('active router docs and localized README stay aligned to sdkwork-api-router naming and config-file-first precedence', () => {
  const readmeZh = readWorkspaceFile('README.zh-CN.md');
  const configuration = readWorkspaceFile('docs/operations/configuration.md');
  const configurationZh = readWorkspaceFile('docs/zh/operations/configuration.md');
  const sourceDevelopment = readWorkspaceFile('docs/getting-started/source-development.md');
  const sourceDevelopmentZh = readWorkspaceFile('docs/zh/getting-started/source-development.md');
  const runtimeModes = readWorkspaceFile('docs/getting-started/runtime-modes.md');
  const runtimeModesZh = readWorkspaceFile('docs/zh/getting-started/runtime-modes.md');

  assert.match(readmeZh, /^\uFEFF?# sdkwork-api-router$/m);
  assert.match(readmeZh, /内建默认值\s*->\s*环境变量兜底\s*->\s*配置文件\s*->\s*CLI/);
  assert.match(readmeZh, /生产部署|Production Deployment/);
  assert.match(readmeZh, /系统安装默认使用 PostgreSQL|system installs default to PostgreSQL/i);
  assert.doesNotMatch(readmeZh, /sdkwork-api-server/);
  assert.doesNotMatch(configuration, /sdkwork-api-server/);
  assert.doesNotMatch(configurationZh, /sdkwork-api-server/);
  assert.doesNotMatch(sourceDevelopment, /sdkwork-api-server/);
  assert.doesNotMatch(sourceDevelopmentZh, /sdkwork-api-server/);
  assert.doesNotMatch(runtimeModes, /SDKWork API Server/);
  assert.doesNotMatch(runtimeModesZh, /SDKWork API Server/);
  assert.match(configuration, /built-in defaults\s*->\s*environment fallback\s*->\s*config file\s*->\s*CLI/i);
  assert.match(configuration, /system installs default to PostgreSQL/i);
});

test('production deployment docs expose installed validate-config entrypoints in both locales', () => {
  const deploy = readWorkspaceFile('docs/getting-started/production-deployment.md');
  const deployZh = readWorkspaceFile('docs/zh/getting-started/production-deployment.md');
  const serviceManagement = readWorkspaceFile('docs/operations/service-management.md');
  const serviceManagementZh = readWorkspaceFile('docs/zh/operations/service-management.md');
  const readme = readWorkspaceFile('README.md');
  const readmeZh = readWorkspaceFile('README.zh-CN.md');

  assert.match(deploy, /bin\/validate-config\.sh/);
  assert.match(deploy, /bin\\validate-config\.ps1/);
  assert.match(deployZh, /bin\/validate-config\.sh/);
  assert.match(deployZh, /bin\\validate-config\.ps1/);
  assert.match(serviceManagement, /bin\/validate-config\.sh/);
  assert.match(serviceManagement, /bin\\validate-config\.ps1/);
  assert.match(serviceManagementZh, /bin\/validate-config\.sh/);
  assert.match(serviceManagementZh, /bin\\validate-config\.ps1/);
  assert.match(readme, /validate-config\.sh/);
  assert.match(readme, /validate-config\.ps1/);
  assert.match(readmeZh, /validate-config\.sh/);
  assert.match(readmeZh, /validate-config\.ps1/);
});

test('production deployment docs expose installed backup and restore entrypoints in both locales', () => {
  const deploy = readWorkspaceFile('docs/getting-started/production-deployment.md');
  const deployZh = readWorkspaceFile('docs/zh/getting-started/production-deployment.md');

  assert.match(deploy, /current\/bin\/backup\.sh --home \.\/current --output/);
  assert.match(deploy, /current\\bin\\backup\.ps1 -Home \.\\current -OutputPath/);
  assert.match(deploy, /current\/bin\/restore\.sh --home \.\/current --source .* --force/);
  assert.match(deploy, /current\\bin\\restore\.ps1 -Home \.\\current -SourcePath .* -Force/);
  assert.match(deployZh, /current\/bin\/backup\.sh --home \.\/current --output/);
  assert.match(deployZh, /current\\bin\\backup\.ps1 -Home \.\\current -OutputPath/);
  assert.match(deployZh, /current\/bin\/restore\.sh --home \.\/current --source .* --force/);
  assert.match(deployZh, /current\\bin\\restore\.ps1 -Home \.\\current -SourcePath .* -Force/);
});

test('service-management docs lock the current-home validation contract and windows-service production path in both locales', () => {
  const serviceManagement = readWorkspaceFile('docs/operations/service-management.md');
  const serviceManagementZh = readWorkspaceFile('docs/zh/operations/service-management.md');
  const scriptLifecycle = readWorkspaceFile('docs/getting-started/script-lifecycle.md');
  const scriptLifecycleZh = readWorkspaceFile('docs/zh/getting-started/script-lifecycle.md');

  for (const content of [serviceManagement, serviceManagementZh]) {
    assert.match(content, /validate-config\.sh --home \.\/current/);
    assert.match(content, /validate-config\.ps1 -Home \.\\current/);
    assert.match(content, /router-ops\.mjs validate-config --mode system --home <product-root>/);
    assert.match(content, /windows-task.*compatibility asset|windows-task.*兼容性资产/);
    assert.match(content, /current\/service\/windows-service\//);
    assert.match(content, /start\.sh --foreground --home <product-root>\/current/);
    assert.match(content, /start\.ps1 -Foreground -Home <product-root>\\current/);
  }

  for (const content of [scriptLifecycle, scriptLifecycleZh]) {
    assert.match(content, /current\/service\/windows-service\//);
    assert.match(content, /start\.sh --foreground --home <product-root>\/current/);
    assert.match(content, /start\.ps1 -Foreground -Home <product-root>\\current/);
    assert.doesNotMatch(content, /current\/service\/windows-task\//);
  }
});

test('windows-task only appears as a compatibility note inside service-management docs', () => {
  const productFacingFiles = [
    'README.md',
    'README.zh-CN.md',
    ...listWorkspaceMarkdownFiles('docs/getting-started'),
    ...listWorkspaceMarkdownFiles('docs/operations'),
    ...listWorkspaceMarkdownFiles('docs/zh/getting-started'),
    ...listWorkspaceMarkdownFiles('docs/zh/operations'),
  ];
  const allowedFiles = new Set([
    'docs/operations/service-management.md',
    'docs/zh/operations/service-management.md',
  ]);

  for (const relativePath of productFacingFiles) {
    const content = readWorkspaceFile(relativePath);
    if (allowedFiles.has(relativePath)) {
      assert.match(content, /windows-task/);
      assert.match(content, /compatibility asset|兼容性资产/);
      continue;
    }

    assert.doesNotMatch(content, /windows-task/);
  }
});

test('product-facing runtime docs standardize on <product-root> terminology for installed server roots', () => {
  const readme = readWorkspaceFile('README.md');
  const readmeZh = readWorkspaceFile('README.zh-CN.md');
  const productionDeployment = readWorkspaceFile('docs/getting-started/production-deployment.md');
  const productionDeploymentZh = readWorkspaceFile('docs/zh/getting-started/production-deployment.md');
  const configuration = readWorkspaceFile('docs/operations/configuration.md');
  const configurationZh = readWorkspaceFile('docs/zh/operations/configuration.md');

  for (const content of [
    readme,
    readmeZh,
    productionDeployment,
    productionDeploymentZh,
    configuration,
    configurationZh,
  ]) {
    assert.match(content, /<product-root>/);
  }

  assert.doesNotMatch(readme, /<install-root>/);
  assert.doesNotMatch(readmeZh, /<install-root>/);
  assert.doesNotMatch(productionDeployment, /<install-root>/);
  assert.doesNotMatch(productionDeploymentZh, /<install-root>/);
});

test('installed validate-config snippets explicitly state they run from <product-root> in both locales', () => {
  const readme = readWorkspaceFile('README.md');
  const readmeZh = readWorkspaceFile('README.zh-CN.md');
  const productionDeployment = readWorkspaceFile('docs/getting-started/production-deployment.md');
  const productionDeploymentZh = readWorkspaceFile('docs/zh/getting-started/production-deployment.md');

  assert.match(readme, /From `<product-root>`, validate the generated production config before service registration:/);
  assert.match(readmeZh, /从 `<product-root>` 执行生成后的生产配置校验：/);
  assert.match(productionDeployment, /From the installed product root, run:/);
  assert.match(productionDeploymentZh, /在已安装的产品根目录中执行：/);
});

test('release-builds docs expose custom <product-root> install generation in both locales', () => {
  const releaseBuilds = readWorkspaceFile('docs/getting-started/release-builds.md');
  const releaseBuildsZh = readWorkspaceFile('docs/zh/getting-started/release-builds.md');

  for (const content of [releaseBuilds, releaseBuildsZh]) {
    assert.match(content, /install\.sh --mode system --home <product-root>/);
    assert.match(content, /install\.ps1 -Mode system -Home <product-root>/);
    assert.match(content, /install\.sh --mode system --home <product-root> --dry-run/);
    assert.match(content, /install\.ps1 -Mode system -Home <product-root> -DryRun/);
  }
});

test('product-facing PowerShell dry-run examples prefer native -DryRun switches for wrapper scripts', () => {
  const releaseBuilds = readWorkspaceFile('docs/getting-started/release-builds.md');
  const releaseBuildsZh = readWorkspaceFile('docs/zh/getting-started/release-builds.md');
  const scriptLifecycle = readWorkspaceFile('docs/getting-started/script-lifecycle.md');
  const scriptLifecycleZh = readWorkspaceFile('docs/zh/getting-started/script-lifecycle.md');

  for (const content of [releaseBuilds, releaseBuildsZh, scriptLifecycle, scriptLifecycleZh]) {
    assert.match(content, /build\.ps1 -DryRun/);
    assert.match(content, /install\.ps1 -DryRun/);
    assert.doesNotMatch(content, /build\.ps1 --dry-run/);
    assert.doesNotMatch(content, /install\.ps1 --dry-run/);
  }
});

test('script-lifecycle release install step uses product-root wording instead of runtime-home wording', () => {
  const scriptLifecycle = readWorkspaceFile('docs/getting-started/script-lifecycle.md');
  const scriptLifecycleZh = readWorkspaceFile('docs/zh/getting-started/script-lifecycle.md');

  assert.match(scriptLifecycle, /### 2\. Install the product root/);
  assert.doesNotMatch(scriptLifecycle, /### 2\. Install the runtime home/);
  assert.match(scriptLifecycleZh, /### 2\. 安装产品根目录/);
  assert.doesNotMatch(scriptLifecycleZh, /### 2\. 安装运行时目录/);
});

test('README and source-development docs publish root product dev entrypoints in both locales', () => {
  const readme = readWorkspaceFile('README.md');
  const readmeZh = readWorkspaceFile('README.zh-CN.md');
  const sourceDevelopment = readWorkspaceFile('docs/getting-started/source-development.md');
  const sourceDevelopmentZh = readWorkspaceFile('docs/zh/getting-started/source-development.md');

  assert.match(readme, /pnpm tauri:dev/);
  assert.match(readme, /pnpm server:dev/);
  assert.match(readmeZh, /pnpm tauri:dev/);
  assert.match(readmeZh, /pnpm server:dev/);
  assert.match(sourceDevelopment, /pnpm tauri:dev/);
  assert.match(sourceDevelopment, /pnpm server:dev/);
  assert.match(sourceDevelopmentZh, /pnpm tauri:dev/);
  assert.match(sourceDevelopmentZh, /pnpm server:dev/);
});

test('repository README publishes release-catalog as metadata beside the two official products in both locales', () => {
  const readme = readWorkspaceFile('README.md');
  const readmeZh = readWorkspaceFile('README.zh-CN.md');

  for (const content of [readme, readmeZh]) {
    assert.match(content, /release-catalog\.json/);
    assert.match(content, /metadata|元数据/);
  }

  assert.doesNotMatch(readme, /third (installable )?product/i);
  assert.doesNotMatch(readmeZh, /第三个可安装产品/);
});

test('localized README routes production links to localized docs pages', () => {
  const readmeZh = readWorkspaceFile('README.zh-CN.md');

  assert.match(readmeZh, /\(\.\/docs\/zh\/getting-started\/production-deployment\.md\)/);
  assert.match(readmeZh, /\(\.\/docs\/zh\/operations\/install-layout\.md\)/);
  assert.match(readmeZh, /\(\.\/docs\/zh\/operations\/service-management\.md\)/);
});

test('quickstart docs expose root product dev shortcuts in both locales', () => {
  const quickstart = readWorkspaceFile('docs/getting-started/quickstart.md');
  const quickstartZh = readWorkspaceFile('docs/zh/getting-started/quickstart.md');

  assert.match(quickstart, /pnpm tauri:dev/);
  assert.match(quickstart, /pnpm server:dev/);
  assert.match(quickstartZh, /pnpm tauri:dev/);
  assert.match(quickstartZh, /pnpm server:dev/);
});

test('build-and-packaging docs expose root product dev shortcuts in both locales', () => {
  const packaging = readWorkspaceFile('docs/getting-started/build-and-packaging.md');
  const packagingZh = readWorkspaceFile('docs/zh/getting-started/build-and-packaging.md');

  assert.match(packaging, /pnpm tauri:dev/);
  assert.match(packaging, /pnpm server:dev/);
  assert.match(packagingZh, /pnpm tauri:dev/);
  assert.match(packagingZh, /pnpm server:dev/);
});

test('build-and-packaging docs publish release-catalog metadata alongside the two official products in both locales', () => {
  const packaging = readWorkspaceFile('docs/getting-started/build-and-packaging.md');
  const packagingZh = readWorkspaceFile('docs/zh/getting-started/build-and-packaging.md');

  for (const content of [packaging, packagingZh]) {
    assert.match(content, /artifacts\/release\/release-catalog\.json/);
    assert.match(content, /generatedAt/);
    assert.match(content, /variantKind/);
    assert.match(content, /primaryFileSizeBytes/);
    assert.match(content, /checksumAlgorithm/);
  }
});

test('README and release-builds verification baselines include the governed workflow contract suite', () => {
  const readme = readWorkspaceFile('README.md');
  const readmeZh = readWorkspaceFile('README.zh-CN.md');
  const releaseBuilds = readWorkspaceFile('docs/getting-started/release-builds.md');
  const releaseBuildsZh = readWorkspaceFile('docs/zh/getting-started/release-builds.md');

  for (const content of [readme, readmeZh, releaseBuilds, releaseBuildsZh]) {
    assert.match(content, /scripts\/release\/tests\/release-workflow\.test\.mjs/);
    assert.match(content, /scripts\/release-governance-workflow\.test\.mjs/);
    assert.match(content, /scripts\/product-verification-workflow\.test\.mjs/);
    assert.match(content, /scripts\/rust-verification-workflow\.test\.mjs/);
  }
});

test('deployment docs use a release-tag placeholder instead of pinned dated image tags', () => {
  const deployReadme = readWorkspaceFile('deploy/README.md');
  const productionDeployment = readWorkspaceFile('docs/getting-started/production-deployment.md');
  const productionDeploymentZh = readWorkspaceFile('docs/zh/getting-started/production-deployment.md');

  assert.match(deployReadme, /image\.tag=<release-tag>/);
  assert.match(productionDeployment, /image\.tag=<release-tag>/);
  assert.match(productionDeploymentZh, /image\.tag=<release-tag>/);
  assert.doesNotMatch(deployReadme, /image\.tag=20\d{2}\.\d{2}\.\d{2}/);
  assert.doesNotMatch(productionDeployment, /image\.tag=20\d{2}\.\d{2}\.\d{2}/);
  assert.doesNotMatch(productionDeploymentZh, /image\.tag=20\d{2}\.\d{2}\.\d{2}/);
});

test('runtime-mode docs align with the Windows Service management contract', () => {
  const runtimeModes = readWorkspaceFile('docs/getting-started/runtime-modes.md');
  const runtimeModesZh = readWorkspaceFile('docs/zh/getting-started/runtime-modes.md');

  assert.match(runtimeModes, /Windows Service/);
  assert.doesNotMatch(runtimeModes, /Task Scheduler/);
  assert.match(runtimeModesZh, /Windows Service/);
});

test('gateway api reference publishes capability-first navigation in both locales', () => {
  const requiredFiles = [
    'docs/api-reference/gateway-capabilities.md',
    'docs/api-reference/gateway-capabilities/audio.md',
    'docs/api-reference/gateway-capabilities/code.md',
    'docs/api-reference/gateway-capabilities/images.md',
    'docs/api-reference/gateway-capabilities/matrix.md',
    'docs/api-reference/gateway-capabilities/video.md',
    'docs/api-reference/gateway-capabilities/music.md',
    'docs/api-reference/gateway-capabilities/images/nanobanana.md',
    'docs/api-reference/gateway-capabilities/images/midjourney.md',
    'docs/api-reference/gateway-capabilities/video/sora2.md',
    'docs/zh/api-reference/gateway-capabilities.md',
    'docs/zh/api-reference/gateway-capabilities/audio.md',
    'docs/zh/api-reference/gateway-capabilities/code.md',
    'docs/zh/api-reference/gateway-capabilities/images.md',
    'docs/zh/api-reference/gateway-capabilities/matrix.md',
    'docs/zh/api-reference/gateway-capabilities/video.md',
    'docs/zh/api-reference/gateway-capabilities/music.md',
    'docs/zh/api-reference/gateway-capabilities/images/nanobanana.md',
    'docs/zh/api-reference/gateway-capabilities/images/midjourney.md',
    'docs/zh/api-reference/gateway-capabilities/video/sora2.md',
  ];

  for (const relativePath of requiredFiles) {
    assert.equal(existsSync(path.join(workspaceRoot, relativePath)), true, `missing ${relativePath}`);
  }

  const vitepressConfig = readWorkspaceFile('docs/.vitepress/config.mjs');
  const overview = readWorkspaceFile('docs/api-reference/overview.md');
  const overviewZh = readWorkspaceFile('docs/zh/api-reference/overview.md');
  const gatewayApi = readWorkspaceFile('docs/api-reference/gateway-api.md');
  const gatewayApiZh = readWorkspaceFile('docs/zh/api-reference/gateway-api.md');
  const capabilityIndex = readWorkspaceFile('docs/api-reference/gateway-capabilities.md');
  const capabilityIndexZh = readWorkspaceFile('docs/zh/api-reference/gateway-capabilities.md');
  const audioCapability = readWorkspaceFile('docs/api-reference/gateway-capabilities/audio.md');
  const audioCapabilityZh = readWorkspaceFile('docs/zh/api-reference/gateway-capabilities/audio.md');
  const codeCapability = readWorkspaceFile('docs/api-reference/gateway-capabilities/code.md');
  const codeCapabilityZh = readWorkspaceFile('docs/zh/api-reference/gateway-capabilities/code.md');
  const imagesCapability = readWorkspaceFile('docs/api-reference/gateway-capabilities/images.md');
  const imagesCapabilityZh = readWorkspaceFile('docs/zh/api-reference/gateway-capabilities/images.md');
  const matrixCapability = readWorkspaceFile('docs/api-reference/gateway-capabilities/matrix.md');
  const musicCapability = readWorkspaceFile('docs/api-reference/gateway-capabilities/music.md');
  const musicCapabilityZh = readWorkspaceFile('docs/zh/api-reference/gateway-capabilities/music.md');
  const videoCapability = readWorkspaceFile('docs/api-reference/gateway-capabilities/video.md');
  const videoCapabilityZh = readWorkspaceFile('docs/zh/api-reference/gateway-capabilities/video.md');

  assert.match(vitepressConfig, /\/api-reference\/gateway-capabilities/);
  assert.match(vitepressConfig, /\/api-reference\/gateway-capabilities\/audio/);
  assert.match(vitepressConfig, /\/api-reference\/gateway-capabilities\/matrix/);
  assert.match(vitepressConfig, /\/zh\/api-reference\/gateway-capabilities/);
  assert.match(vitepressConfig, /\/zh\/api-reference\/gateway-capabilities\/audio/);
  assert.match(vitepressConfig, /\/zh\/api-reference\/gateway-capabilities\/matrix/);
  assert.match(gatewayApi, /OpenAPI Tag To Capability Docs/i);
  assert.match(gatewayApiZh, /OpenAPI Tag.*Capability|Capability.*OpenAPI|能力.*OpenAPI|OpenAPI.*能力/);
  assert.match(gatewayApi, /code\.gemini/);
  assert.match(gatewayApiZh, /code\.gemini/);
  assert.match(gatewayApi, /audio\.openai/);
  assert.match(gatewayApiZh, /audio\.openai/);
  assert.match(gatewayApi, /images\.volcengine/);
  assert.match(gatewayApiZh, /images\.volcengine/);
  assert.match(gatewayApi, /video\.google-veo/);
  assert.match(gatewayApiZh, /video\.google-veo/);
  assert.match(gatewayApi, /music\.suno/);
  assert.match(gatewayApiZh, /music\.suno/);
  assert.match(gatewayApi, /images\/nanobanana/);
  assert.match(gatewayApiZh, /images\/nanobanana/);
  assert.match(gatewayApi, /video\/sora2/);
  assert.match(gatewayApiZh, /video\/sora2/);
  assert.match(overview, /Gateway Capability Index/);
  assert.match(overviewZh, /Gateway Capability Index|能力目录|能力索引/);
  assert.match(gatewayApi, /Gateway Capability Index/);
  assert.match(gatewayApiZh, /Gateway Capability Index|能力目录|能力索引/);
  assert.match(capabilityIndex, /\[Audio\]/);
  assert.match(capabilityIndexZh, /Audio/);
  assert.match(audioCapability, /audio\.openai/);
  assert.match(audioCapability, /Shared Default API Inventory/);
  assert.match(audioCapability, /`\/v1\/audio\/\*`/);
  assert.match(audioCapability, /POST \/v1\/audio\/transcriptions/);
  assert.match(audioCapabilityZh, /共享默认 API 清单/);
  assert.match(audioCapabilityZh, /POST \/v1\/audio\/transcriptions/);
  assert.match(codeCapability, /Shared Default API Inventory/);
  assert.match(codeCapability, /GET \/v1\/models/);
  assert.match(codeCapability, /POST \/v1\/chat\/completions/);
  assert.match(codeCapability, /POST \/v1\/responses/);
  assert.match(codeCapabilityZh, /共享默认 API 清单/);
  assert.match(codeCapabilityZh, /GET \/v1\/models/);
  assert.match(codeCapabilityZh, /POST \/v1\/responses/);
  assert.match(imagesCapability, /Shared Default API Inventory/);
  assert.match(imagesCapability, /`\/v1\/images\/\*`/);
  assert.match(imagesCapability, /POST \/v1\/images\/generations/);
  assert.match(imagesCapability, /nanobanana/i);
  assert.match(imagesCapability, /midjourney/i);
  assert.match(imagesCapabilityZh, /共享默认 API 清单/);
  assert.match(imagesCapabilityZh, /POST \/v1\/images\/generations/);
  assert.match(matrixCapability, /audio\.openai/);
  assert.match(matrixCapability, /images\.midjourney/);
  assert.match(matrixCapability, /video\.sora2|sora2/i);
  assert.match(matrixCapability, /unpublished/i);
  assert.match(matrixCapability, /alias/i);
  assert.match(musicCapability, /Shared Default API Inventory/);
  assert.match(musicCapability, /GET \/v1\/music/);
  assert.match(musicCapability, /POST \/v1\/music\/lyrics/);
  assert.match(musicCapabilityZh, /共享默认 API 清单/);
  assert.match(musicCapabilityZh, /POST \/v1\/music\/lyrics/);
  assert.match(videoCapability, /Shared Default API Inventory/);
  assert.match(videoCapability, /`\/v1\/videos\*`/);
  assert.match(videoCapability, /GET \/v1\/videos/);
  assert.match(videoCapability, /POST \/v1\/videos/);
  assert.match(videoCapability, /sora2/i);
  assert.match(videoCapabilityZh, /共享默认 API 清单/);
  assert.match(videoCapabilityZh, /POST \/v1\/videos/);
});
