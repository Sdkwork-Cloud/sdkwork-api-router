import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { readFileSync } from 'node:fs';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

test('official packaging docs describe only the server and portal desktop products', () => {
  const buildAndPackaging = read('docs/getting-started/build-and-packaging.md');
  const releaseBuilds = read('docs/getting-started/release-builds.md');
  const productionDeployment = read('docs/getting-started/production-deployment.md');
  const installation = read('docs/getting-started/installation.md');
  const buildAndTooling = read('docs/reference/build-and-tooling.md');
  const scriptLifecycle = read('docs/getting-started/script-lifecycle.md');
  const runtimeModes = read('docs/architecture/runtime-modes.md');
  const architecture = read('docs/architecture/software-architecture.md');
  const functionalModules = read('docs/architecture/functional-modules.md');
  const index = read('docs/index.md');

  assert.match(buildAndPackaging, /sdkwork-api-router-product-server/);
  assert.match(buildAndPackaging, /sdkwork-router-portal-desktop/);
  assert.match(buildAndPackaging, /prepare-router-portal-desktop-runtime\.mjs/);
  assert.match(buildAndPackaging, /pnpm --dir apps\/sdkwork-router-portal tauri:build/);
  assert.match(
    buildAndPackaging,
    /node scripts\/release\/run-service-release-build\.mjs --target <triple>/,
  );
  assert.match(buildAndPackaging, /sdkwork-api-router-product-server-<platform>-<arch>\.tar\.gz/i);
  assert.match(buildAndPackaging, /sdkwork-api-router-product-server-<platform>-<arch>\.tar\.gz\.sha256\.txt/i);
  assert.match(buildAndPackaging, /sdkwork-api-router-product-server-<platform>-<arch>\.manifest\.json/i);
  assert.match(buildAndPackaging, /sdkwork-router-portal-desktop-<platform>-<arch>\.<ext>\.sha256\.txt/);
  assert.match(buildAndPackaging, /sdkwork-router-portal-desktop-<platform>-<arch>\.manifest\.json/);
  assert.match(buildAndPackaging, /release-catalog\.json/);
  assert.match(buildAndPackaging, /artifacts\/release\/release-catalog\.json/);
  assert.match(buildAndPackaging, /generatedAt/);
  assert.match(buildAndPackaging, /variantKind/);
  assert.match(buildAndPackaging, /primaryFileSizeBytes/);
  assert.match(buildAndPackaging, /checksumAlgorithm/);
  assert.match(buildAndPackaging, /control\/bin\//);
  assert.doesNotMatch(buildAndPackaging, /pnpm --dir console/);
  assert.doesNotMatch(buildAndPackaging, /console\/dist/);
  assert.match(releaseBuilds, /sdkwork-api-router-product-server-<platform>-<arch>/i);
  assert.match(releaseBuilds, /sdkwork-api-router-product-server-<platform>-<arch>\.manifest\.json/i);
  assert.match(releaseBuilds, /sdkwork-router-portal-desktop-<platform>-<arch>/);
  assert.match(releaseBuilds, /sdkwork-router-portal-desktop-<platform>-<arch>\.manifest\.json/);
  assert.match(releaseBuilds, /build\.sh --verify-release/);
  assert.match(releaseBuilds, /--skip-docs cannot be combined with --verify-release/);
  assert.match(releaseBuilds, /verify-release[\s\S]*docs site/i);
  assert.match(releaseBuilds, /verify-release[\s\S]*release governance preflight/i);
  assert.match(releaseBuilds, /release-catalog\.json/);
  assert.match(releaseBuilds, /artifacts\/release\/release-catalog\.json/);
  assert.match(releaseBuilds, /version/);
  assert.match(releaseBuilds, /sdkwork-release-catalog/);
  assert.match(releaseBuilds, /productCount/);
  assert.match(releaseBuilds, /variantCount/);
  assert.match(releaseBuilds, /generatedAt/);
  assert.match(releaseBuilds, /variantKind/);
  assert.match(releaseBuilds, /primaryFileSizeBytes/);
  assert.match(releaseBuilds, /checksumAlgorithm/);
  assert.match(releaseBuilds, /router-product\/data\//);
  assert.match(releaseBuilds, /router-product\/release-manifest\.json/);
  assert.match(releaseBuilds, /router-product\/README\.txt/);
  assert.match(releaseBuilds, /archiveFile/);
  assert.match(releaseBuilds, /embeddedManifestFile/);
  assert.match(releaseBuilds, /installers/);
  assert.match(releaseBuilds, /bootstrapDataRoots/);
  assert.match(releaseBuilds, /deploymentAssetRoots/);
  assert.match(releaseBuilds, /artifactKind/);
  assert.match(releaseBuilds, /installerFile/);
  assert.match(releaseBuilds, /sourceBundlePath/);
  assert.match(productionDeployment, /release-catalog\.json/);
  assert.doesNotMatch(releaseBuilds, /desktop\/portal\/\*\*\/*/);

  assert.doesNotMatch(installation, /pnpm --dir console/);
  assert.doesNotMatch(installation, /browser\/Tauri operator console under `console\/`/);
  assert.doesNotMatch(installation, /standalone admin app under `apps\/sdkwork-router-admin\/`/);
  assert.match(installation, /development-only admin browser app under `apps\/sdkwork-router-admin\/`/);
  assert.match(installation, /not an official release product/i);

  assert.doesNotMatch(buildAndTooling, /browser-only/);
  assert.match(buildAndTooling, /build\.sh --verify-release/);
  assert.match(buildAndTooling, /official local release verification/i);
  assert.match(buildAndTooling, /release governance preflight/i);
  assert.match(buildAndTooling, /--skip-docs/);
  assert.match(
    buildAndTooling,
    /node scripts\/release\/run-service-release-build\.mjs --target <triple>/,
  );
  assert.match(buildAndTooling, /start-portal\.mjs` \| raw source dev \| start the portal app only \| browser or Tauri \|/);
  assert.doesNotMatch(scriptLifecycle, /optional docs and console browser assets/);

  assert.doesNotMatch(runtimeModes, /console\/src-tauri/);
  assert.match(runtimeModes, /apps\/sdkwork-router-portal\/src-tauri/);

  assert.doesNotMatch(architecture, /the admin and portal Tauri hosts both embed the shared product runtime/);
  assert.doesNotMatch(functionalModules, /\| console \| browser and desktop UI shell \| `console\/` \|/);
  assert.match(
    index,
    /node scripts\/release\/run-service-release-build\.mjs --target <triple>/,
  );
  assert.match(index, /\| apps\/sdkwork-router-portal \| browser or Tauri \| standalone developer self-service portal \|/);
});

test('repository landing docs keep the official product line and installed validation entrypoints aligned', () => {
  const readme = read('README.md');
  const zhReadme = read('README.zh-CN.md');

  for (const content of [readme, zhReadme]) {
    assert.match(content, /sdkwork-api-router-product-server/);
    assert.match(content, /sdkwork-router-portal-desktop/);
    assert.match(content, /release-catalog\.json/);
    assert.match(content, /control\/bin\//);
    assert.match(content, /metadata|元数据/);
    assert.match(content, /current\/bin\/validate-config\.sh --home/);
    assert.match(content, /current\\bin\\validate-config\.ps1 -Home/);
    assert.doesNotMatch(content, /<install-root>\/bin\/validate-config\.sh/);
    assert.doesNotMatch(content, /<install-root>\\bin\\validate-config\.ps1/);
  }
});

test('localized product docs follow the same official packaging contract', () => {
  const zhIndex = read('docs/zh/index.md');
  const zhInstallation = read('docs/zh/getting-started/installation.md');
  const zhBuildAndPackaging = read('docs/zh/getting-started/build-and-packaging.md');
  const zhReleaseBuilds = read('docs/zh/getting-started/release-builds.md');
  const zhProductionDeployment = read('docs/zh/getting-started/production-deployment.md');
  const zhBuildAndTooling = read('docs/zh/reference/build-and-tooling.md');
  const zhRuntimeModes = read('docs/zh/architecture/runtime-modes.md');
  const zhArchitecture = read('docs/zh/architecture/software-architecture.md');
  const zhFunctionalModules = read('docs/zh/architecture/functional-modules.md');
  const zhRepositoryLayout = read('docs/zh/reference/repository-layout.md');

  assert.match(zhIndex, /OpenAI 兼容网关/);
  assert.match(zhIndex, /sdkwork-router-portal-desktop/);
  assert.match(
    zhIndex,
    /node scripts\/release\/run-service-release-build\.mjs --target <triple>/,
  );

  assert.match(zhInstallation, /^# 安装准备/m);
  assert.doesNotMatch(zhInstallation, /pnpm --dir console/);
  assert.match(zhInstallation, /apps\/sdkwork-router-admin\//);
  assert.doesNotMatch(zhInstallation, /独立 admin 应用/);
  assert.match(zhInstallation, /不是正式发布产品/);

  assert.match(zhBuildAndPackaging, /^# 编译与打包/m);
  assert.match(zhBuildAndPackaging, /sdkwork-api-router-product-server/);
  assert.match(zhBuildAndPackaging, /sdkwork-router-portal-desktop/);
  assert.match(
    zhBuildAndPackaging,
    /node scripts\/release\/run-service-release-build\.mjs --target <triple>/,
  );
  assert.match(zhBuildAndPackaging, /sdkwork-api-router-product-server-<platform>-<arch>\.tar\.gz/);
  assert.match(zhBuildAndPackaging, /sdkwork-api-router-product-server-<platform>-<arch>\.tar\.gz\.sha256\.txt/);
  assert.match(zhBuildAndPackaging, /sdkwork-api-router-product-server-<platform>-<arch>\.manifest\.json/);
  assert.match(zhBuildAndPackaging, /sdkwork-router-portal-desktop-<platform>-<arch>\.<ext>\.sha256\.txt/);
  assert.match(zhBuildAndPackaging, /sdkwork-router-portal-desktop-<platform>-<arch>\.manifest\.json/);
  assert.match(zhBuildAndPackaging, /release-catalog\.json/);
  assert.match(zhBuildAndPackaging, /artifacts\/release\/release-catalog\.json/);
  assert.match(zhBuildAndPackaging, /generatedAt/);
  assert.match(zhBuildAndPackaging, /variantKind/);
  assert.match(zhBuildAndPackaging, /primaryFileSizeBytes/);
  assert.match(zhBuildAndPackaging, /checksumAlgorithm/);
  assert.match(zhBuildAndPackaging, /control\/bin\//);
  assert.doesNotMatch(zhBuildAndPackaging, /pnpm --dir console/);
  assert.match(zhReleaseBuilds, /sdkwork-api-router-product-server-<platform>-<arch>/);
  assert.match(zhReleaseBuilds, /sdkwork-api-router-product-server-<platform>-<arch>\.manifest\.json/);
  assert.match(zhReleaseBuilds, /sdkwork-router-portal-desktop-<platform>-<arch>/);
  assert.match(zhReleaseBuilds, /sdkwork-router-portal-desktop-<platform>-<arch>\.manifest\.json/);
  assert.match(zhReleaseBuilds, /build\.sh --verify-release/);
  assert.match(zhReleaseBuilds, /--skip-docs .* --verify-release|--verify-release .* --skip-docs/);
  assert.match(zhReleaseBuilds, /verify-release[\s\S]*docs/);
  assert.match(zhReleaseBuilds, /verify-release[\s\S]*governance/i);
  assert.match(zhReleaseBuilds, /release-catalog\.json/);
  assert.match(zhReleaseBuilds, /artifacts\/release\/release-catalog\.json/);
  assert.match(zhReleaseBuilds, /version/);
  assert.match(zhReleaseBuilds, /sdkwork-release-catalog/);
  assert.match(zhReleaseBuilds, /productCount/);
  assert.match(zhReleaseBuilds, /variantCount/);
  assert.match(zhReleaseBuilds, /generatedAt/);
  assert.match(zhReleaseBuilds, /variantKind/);
  assert.match(zhReleaseBuilds, /primaryFileSizeBytes/);
  assert.match(zhReleaseBuilds, /checksumAlgorithm/);
  assert.match(zhReleaseBuilds, /router-product\/data\//);
  assert.match(zhReleaseBuilds, /router-product\/release-manifest\.json/);
  assert.match(zhReleaseBuilds, /router-product\/README\.txt/);
  assert.match(zhReleaseBuilds, /archiveFile/);
  assert.match(zhReleaseBuilds, /embeddedManifestFile/);
  assert.match(zhReleaseBuilds, /installers/);
  assert.match(zhReleaseBuilds, /bootstrapDataRoots/);
  assert.match(zhReleaseBuilds, /deploymentAssetRoots/);
  assert.match(zhReleaseBuilds, /artifactKind/);
  assert.match(zhReleaseBuilds, /installerFile/);
  assert.match(zhReleaseBuilds, /sourceBundlePath/);
  assert.match(zhReleaseBuilds, /\.\/install\.sh --mode system/);
  assert.match(zhReleaseBuilds, /\.\\install\.ps1 -Mode system/);
  assert.match(zhReleaseBuilds, /bin\/install\.sh/);
  assert.match(zhReleaseBuilds, /bin\\install\.ps1/);
  assert.match(zhProductionDeployment, /release-catalog\.json/);
  assert.doesNotMatch(zhReleaseBuilds, /desktop\/portal\/\*\*\/*/);

  assert.match(zhBuildAndTooling, /^# 构建与工具链/m);
  assert.match(zhBuildAndTooling, /node scripts\/check-router-product\.mjs/);
  assert.match(zhBuildAndTooling, /build\.sh --verify-release/);
  assert.match(zhBuildAndTooling, /正式本地 release 验证/i);
  assert.match(zhBuildAndTooling, /release governance preflight/i);
  assert.match(zhBuildAndTooling, /--skip-docs/);
  assert.match(
    zhBuildAndTooling,
    /node scripts\/release\/run-service-release-build\.mjs --target <triple>/,
  );
  assert.match(zhBuildAndTooling, /product gate/i);
  assert.match(zhBuildAndTooling, /浏览器或 Tauri/);
  assert.doesNotMatch(zhBuildAndTooling, /仅浏览器/);

  assert.match(zhRuntimeModes, /^# 运行模式详解/m);
  assert.doesNotMatch(zhRuntimeModes, /console\/src-tauri/);
  assert.match(zhRuntimeModes, /apps\/sdkwork-router-portal\/src-tauri/);

  assert.match(zhArchitecture, /^# 软件架构/m);
  assert.doesNotMatch(zhFunctionalModules, /\| console \|/);
  assert.doesNotMatch(zhRepositoryLayout, /\|-- console\//);
  assert.doesNotMatch(zhRepositoryLayout, /继续参与发布打包/);
});

test('script lifecycle docs no longer describe console as a release build input', () => {
  const scriptLifecycle = read('docs/getting-started/script-lifecycle.md');
  const zhScriptLifecycle = read('docs/zh/getting-started/script-lifecycle.md');

  assert.doesNotMatch(scriptLifecycle, /console/);
  assert.doesNotMatch(zhScriptLifecycle, /console/);
});

test('script lifecycle docs present release start and stop commands against the product root home', () => {
  const scriptLifecycle = read('docs/getting-started/script-lifecycle.md');
  const zhScriptLifecycle = read('docs/zh/getting-started/script-lifecycle.md');

  for (const content of [scriptLifecycle, zhScriptLifecycle]) {
    assert.match(content, /start\.sh --home artifacts\/install\/sdkwork-api-router(?!\/current)/);
    assert.match(content, /start\.ps1 -Home \.\\artifacts\\install\\sdkwork-api-router(?!\\current)/);
    assert.match(content, /stop\.sh --home artifacts\/install\/sdkwork-api-router(?!\/current)/);
    assert.match(content, /stop\.ps1 -Home \.\\artifacts\\install\\sdkwork-api-router(?!\\current)/);
    assert.doesNotMatch(content, /start\.sh --home artifacts\/install\/sdkwork-api-router\/current/);
    assert.doesNotMatch(content, /start\.ps1 -Home \.\\artifacts\\install\\sdkwork-api-router\\current/);
    assert.doesNotMatch(content, /stop\.sh --home artifacts\/install\/sdkwork-api-router\/current/);
    assert.doesNotMatch(content, /stop\.ps1 -Home \.\\artifacts\\install\\sdkwork-api-router\\current/);
  }
});

test('supplemental architecture notes no longer treat console as an official product surface', () => {
  const architectureReadme = read('docs/架构/README.md');
  const productScope = read('docs/架构/01-产品设计与需求范围.md');
  const architectureStandard = read('docs/架构/02-架构标准与总体设计.md');
  const moduleBoundaries = read('docs/架构/03-模块规划与边界.md');
  const marketMatrix = read('docs/架构/130-API-Router-行业对标与终局能力矩阵-2026-04-07.md');
  const controlPlane = read('docs/架构/133-控制平面与运营后台设计-2026-04-07.md');

  assert.doesNotMatch(architectureReadme, /console\//);
  assert.doesNotMatch(productScope, /console\//);
  assert.doesNotMatch(architectureStandard, /console\//);
  assert.doesNotMatch(moduleBoundaries, /console\//);
  assert.doesNotMatch(marketMatrix, /console\//);
  assert.doesNotMatch(controlPlane, /console\//);
});

test('install and deployment docs describe the bundle-driven server install contract', () => {
  const installLayout = read('docs/operations/install-layout.md');
  const serviceManagement = read('docs/operations/service-management.md');
  const productionDeployment = read('docs/getting-started/production-deployment.md');
  const deployReadme = read('deploy/README.md');
  const onlineRelease = read('docs/getting-started/online-release.md');
  const zhOnlineRelease = read('docs/zh/getting-started/online-release.md');
  const zhInstallLayout = read('docs/zh/operations/install-layout.md');
  const zhServiceManagement = read('docs/zh/operations/service-management.md');
  const zhProductionDeployment = read('docs/zh/getting-started/production-deployment.md');

  for (const content of [installLayout, zhInstallLayout]) {
    assert.match(content, /artifacts\/release\/native\/<platform>\/<arch>\/bundles\//);
    assert.match(content, /release-catalog\.json/);
    assert.match(content, /release-manifest\.json/);
    assert.match(content, /releaseVersion/);
    assert.match(content, /README\.txt/);
    assert.match(content, /deploy\//);
    assert.match(content, /control\/bin\//);
    assert.match(content, /install\.sh/);
    assert.match(content, /install\.ps1/);
  }

  for (const content of [serviceManagement, zhServiceManagement, productionDeployment, zhProductionDeployment]) {
    assert.match(content, /installed-runtime smoke/i);
    assert.match(content, /packaged server bundle/i);
    assert.match(content, /control\/bin\//);
    assert.match(content, /install\.sh/);
    assert.match(content, /install\.ps1/);
  }

  for (const content of [productionDeployment, zhProductionDeployment]) {
    assert.match(content, /build\.sh --verify-release/);
    assert.match(content, /release governance preflight/i);
    assert.match(content, /tar -xzf sdkwork-api-router-product-server-linux-x64\.tar\.gz/);
    assert.match(content, /docker build -f deploy\/docker\/Dockerfile -t sdkwork-api-router:local \./);
    assert.match(content, /adminJwtSigningSecret|ADMIN_JWT_SIGNING_SECRET/);
    assert.match(content, /portalJwtSigningSecret|PORTAL_JWT_SIGNING_SECRET/);
    assert.match(content, /credentialMasterKey|CREDENTIAL_MASTER_KEY/);
    assert.match(content, /metricsBearerToken|METRICS_BEARER_TOKEN/);
    assert.match(content, /ghcr\.io\/<owner>\/sdkwork-api-router:<release-tag>|ghcr\.io\/your-org\/sdkwork-api-router:<release-tag>/);
    assert.match(content, /backup-manifest\.json/);
    assert.match(content, /backup-manifest\.json[\s\S]*formatVersion/i);
    assert.match(content, /backup-manifest\.json[\s\S]*formatVersion[\s\S]*2/);
    assert.match(content, /bundle\.controlManifestFile/);
    assert.match(content, /bundle\.configSnapshotRoot/);
    assert.match(content, /bundle\.mutableDataSnapshotRoot/);
    assert.match(content, /support-bundle-manifest\.json/);
    assert.match(content, /support-bundle-manifest\.json[\s\S]*formatVersion/i);
    assert.match(content, /support-bundle-manifest\.json[\s\S]*formatVersion[\s\S]*2/);
    assert.match(content, /paths\.controlManifestFile/);
    assert.match(content, /paths\.configSnapshotRoot/);
    assert.match(content, /paths\.processStateFile/);
  }

  assert.match(deployReadme, /ghcr\.io\/<owner>\/sdkwork-api-router:<release-tag>|ghcr\.io\/your-org\/sdkwork-api-router:<release-tag>/);
  assert.match(onlineRelease, /GHCR|ghcr/i);
assert.match(onlineRelease, /ghcr-image-publish-<platform>-<arch>\.json|GHCR image publish metadata/i);
  assert.match(onlineRelease, /ghcr-image-manifest-publish\.json|GHCR image manifest publish metadata/i);
  assert.match(onlineRelease, /sdkwork-ghcr-image-publish/);
  assert.match(onlineRelease, /bundlePath/);
  assert.match(onlineRelease, /imageRepository/);
  assert.match(onlineRelease, /imageTag/);
  assert.match(onlineRelease, /imageRef/);
  assert.match(onlineRelease, /digest/);
  assert.match(onlineRelease, /sdkwork-ghcr-image-manifest-publish/);
  assert.match(onlineRelease, /targetImageRef/);
  assert.match(onlineRelease, /sourceImageRefs/);
  assert.match(onlineRelease, /manifestMediaType/);
  assert.match(onlineRelease, /platformCount/);
  assert.match(zhOnlineRelease, /ghcr-image-publish-<platform>-<arch>\.json|GHCR .*鍏冩暟鎹?/i);
  assert.match(zhOnlineRelease, /ghcr-image-manifest-publish\.json|GHCR .*manifest .*鍏冩暟鎹?/i);
});

test('product Docker runtime uses the hosted Linux ARM release baseline', () => {
  const dockerfile = read('deploy/docker/Dockerfile');

  assert.match(dockerfile, /^FROM ubuntu:24\.04$/m);
});

test('online release docs publish the machine-readable GHCR metadata contracts in both locales', () => {
  const onlineRelease = read('docs/getting-started/online-release.md');
  const zhOnlineRelease = read('docs/zh/getting-started/online-release.md');

  for (const content of [onlineRelease, zhOnlineRelease]) {
    assert.match(content, /ghcr-image-publish-<platform>-<arch>\.json|ghcr-image-publish-linux-x64\.json/i);
    assert.match(content, /ghcr-image-manifest-publish\.json/i);
    assert.match(content, /sdkwork-ghcr-image-publish/);
    assert.match(content, /bundlePath/);
    assert.match(content, /imageRepository/);
    assert.match(content, /imageTag/);
    assert.match(content, /imageRef/);
    assert.match(content, /digest/);
    assert.match(content, /sdkwork-ghcr-image-manifest-publish/);
    assert.match(content, /targetImageRef/);
    assert.match(content, /sourceImageRefs/);
    assert.match(content, /manifestMediaType/);
    assert.match(content, /platformCount/);
  }
});

test('online release docs publish operator checksum verification commands in both locales', () => {
  const onlineRelease = read('docs/getting-started/online-release.md');
  const zhOnlineRelease = read('docs/zh/getting-started/online-release.md');

  for (const content of [onlineRelease, zhOnlineRelease]) {
    assert.match(content, /\.sha256\.txt/);
    assert.match(content, /sha256sum\s+-c/i);
    assert.match(content, /Get-FileHash\s+-Algorithm\s+SHA256/i);
  }
});

test('online release docs publish governance bundle and attestation json contracts in both locales', () => {
  const onlineRelease = read('docs/getting-started/online-release.md');
  const zhOnlineRelease = read('docs/zh/getting-started/online-release.md');

  for (const content of [onlineRelease, zhOnlineRelease]) {
    assert.match(content, /release-governance-bundle-manifest\.json/);
    assert.match(content, /bundleEntryCount/);
    assert.match(content, /sourceRelativePath/);
    assert.match(content, /restore\.command/);
    assert.match(content, /verify-release-attestations\.mjs --format json/);
    assert.match(content, /verifiedCount/);
    assert.match(content, /blockedCount/);
    assert.match(content, /failedCount/);
    assert.match(content, /verifiedIds/);
    assert.match(content, /blockedIds/);
    assert.match(content, /failingIds/);
    assert.match(content, /reports/);
  }
});

test('online release docs publish governance bundle restore result contracts in both locales', () => {
  const onlineRelease = read('docs/getting-started/online-release.md');
  const zhOnlineRelease = read('docs/zh/getting-started/online-release.md');

  for (const content of [onlineRelease, zhOnlineRelease]) {
    assert.match(content, /restore-release-governance-latest\.mjs --artifact-dir/);
    assert.match(content, /repoRoot/);
    assert.match(content, /restored/);
    assert.match(content, /outputPath/);
    assert.match(content, /duplicateCount/);
  }
});

test('online release docs publish desktop signing evidence contracts in both locales', () => {
  const onlineRelease = read('docs/getting-started/online-release.md');
  const zhOnlineRelease = read('docs/zh/getting-started/online-release.md');

  for (const content of [onlineRelease, zhOnlineRelease]) {
    assert.match(content, /desktop-release-signing-<platform>-<arch>\.json/i);
    assert.match(content, /sdkwork-desktop-release-signing/);
    assert.match(content, /appId/);
    assert.match(content, /targetTriple/);
    assert.match(content, /required/);
    assert.match(content, /status/);
    assert.match(content, /hook/);
    assert.match(content, /envVar/);
    assert.match(content, /bundleFiles/);
    assert.match(content, /commandCount/);
    assert.match(content, /failure/);
  }
});

test('online release docs publish smoke evidence contracts in both locales', () => {
  const onlineRelease = read('docs/getting-started/online-release.md');
  const zhOnlineRelease = read('docs/zh/getting-started/online-release.md');

  for (const content of [onlineRelease, zhOnlineRelease]) {
    assert.match(content, /unix-installed-runtime-smoke-<platform>-<arch>\.json|unix-installed-runtime-smoke-linux-x64\.json/i);
    assert.match(content, /windows-installed-runtime-smoke-<platform>-<arch>\.json|windows-installed-runtime-smoke-windows-x64\.json/i);
    assert.match(content, /docker-compose-smoke-<platform>-<arch>\.json|docker-compose-smoke-linux-x64\.json/i);
    assert.match(content, /helm-render-smoke-<platform>-<arch>\.json|helm-render-smoke-linux-arm64\.json/i);
    assert.match(content, /runtimeHome/);
    assert.match(content, /backupBundlePath/);
    assert.match(content, /backupRestoreVerified/);
    assert.match(content, /healthUrls/);
    assert.match(content, /executionMode/);
    assert.match(content, /siteUrls/);
    assert.match(content, /browserSmokeTargets/);
    assert.match(content, /databaseAssertions/);
    assert.match(content, /browserSmokeResults/);
    assert.match(content, /composePs/);
    assert.match(content, /diagnostics/);
    assert.match(content, /renderedManifestPath/);
    assert.match(content, /renderedKinds/);
    assert.match(content, /kubeconformSummary/);
    assert.match(content, /logs/);
    assert.match(content, /failure/);
  }
});

test('online release docs publish release-window, sync-audit, telemetry, and slo evidence contracts in both locales', () => {
  const onlineRelease = read('docs/getting-started/online-release.md');
  const zhOnlineRelease = read('docs/zh/getting-started/online-release.md');

  for (const content of [onlineRelease, zhOnlineRelease]) {
    assert.match(content, /release-window-snapshot-latest\.json/);
    assert.match(content, /latestReleaseTag/);
    assert.match(content, /commitsSinceLatestRelease/);
    assert.match(content, /workingTreeEntryCount/);
    assert.match(content, /hasReleaseBaseline/);
    assert.match(content, /release-sync-audit-latest\.json/);
    assert.match(content, /summary/);
    assert.match(content, /releasable/);
    assert.match(content, /reports/);
    assert.match(content, /expectedGitRoot/);
    assert.match(content, /remoteHead/);
    assert.match(content, /ahead/);
    assert.match(content, /behind/);
    assert.match(content, /isDirty/);
    assert.match(content, /release-telemetry-export-latest\.json/);
    assert.match(content, /prometheus/);
    assert.match(content, /supplemental/);
    assert.match(content, /freshnessMinutes/);
    assert.match(content, /release-telemetry-snapshot-latest\.json/);
    assert.match(content, /snapshotId/);
    assert.match(content, /exportKind/);
    assert.match(content, /supplementalTargetIds/);
    assert.match(content, /slo-governance-latest\.json/);
    assert.match(content, /baselineId/);
    assert.match(content, /baselineDate/);
    assert.match(content, /targets/);
  }
});

test('online release docs publish third-party governance artifact contracts in both locales', () => {
  const onlineRelease = read('docs/getting-started/online-release.md');
  const zhOnlineRelease = read('docs/zh/getting-started/online-release.md');

  for (const content of [onlineRelease, zhOnlineRelease]) {
    assert.match(content, /third-party-sbom-latest\.spdx\.json/);
    assert.match(content, /SPDX-2\.3/);
    assert.match(content, /documentNamespace/);
    assert.match(content, /creationInfo\.created|creationInfo/);
    assert.match(content, /documentDescribes/);
    assert.match(content, /relationships/);
    assert.match(content, /third-party-notices-latest\.json/);
    assert.match(content, /packageCount/);
    assert.match(content, /cargoPackageCount/);
    assert.match(content, /npmPackageCount/);
    assert.match(content, /licenseDeclared/);
    assert.match(content, /downloadLocation/);
    assert.match(content, /sourcePath/);
    assert.match(content, /noticeFiles/);
    assert.match(content, /noticeText/);
  }
});

test('online release docs publish release catalog contracts in both locales', () => {
  const onlineRelease = read('docs/getting-started/online-release.md');
  const zhOnlineRelease = read('docs/zh/getting-started/online-release.md');

  for (const content of [onlineRelease, zhOnlineRelease]) {
    assert.match(content, /release-catalog\.json/);
    assert.match(content, /sdkwork-release-catalog/);
    assert.match(content, /releaseTag/);
    assert.match(content, /productCount/);
    assert.match(content, /variantCount/);
    assert.match(content, /products/);
    assert.match(content, /outputDirectory/);
    assert.match(content, /primaryFile/);
    assert.match(content, /checksumFile/);
    assert.match(content, /manifestFile/);
    assert.match(content, /sha256/);
    assert.match(content, /manifest/);
  }
});

test('online release docs publish official asset manifest contracts in both locales', () => {
  const onlineRelease = read('docs/getting-started/online-release.md');
  const zhOnlineRelease = read('docs/zh/getting-started/online-release.md');

  for (const content of [onlineRelease, zhOnlineRelease]) {
    assert.match(content, /sdkwork-api-router-product-server-<platform>-<arch>\.manifest\.json/i);
    assert.match(content, /sdkwork-router-portal-desktop-<platform>-<arch>\.manifest\.json/i);
    assert.match(content, /releaseVersion/);
    assert.match(content, /archiveFile/);
    assert.match(content, /embeddedManifestFile/);
    assert.match(content, /installers/);
    assert.match(content, /services/);
    assert.match(content, /sites/);
    assert.match(content, /bootstrapDataRoots/);
    assert.match(content, /deploymentAssetRoots/);
    assert.match(content, /appId/);
    assert.match(content, /artifactKind/);
    assert.match(content, /installerFile/);
    assert.match(content, /sourceBundlePath/);
    assert.match(content, /embeddedRuntime/);
  }
});
