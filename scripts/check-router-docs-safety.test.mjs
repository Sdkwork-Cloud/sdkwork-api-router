import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

function readWorkspaceFile(relativePath) {
  return readFileSync(path.join(workspaceRoot, relativePath), 'utf8');
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
