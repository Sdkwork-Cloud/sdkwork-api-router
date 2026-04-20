import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

test('repository ships Docker and Helm deployment assets aligned to product server runtime', () => {
  const dockerfilePath = path.join(repoRoot, 'deploy', 'docker', 'Dockerfile');
  const composePath = path.join(repoRoot, 'deploy', 'docker', 'docker-compose.yml');
  const envExamplePath = path.join(repoRoot, 'deploy', 'docker', '.env.example');
  const chartPath = path.join(repoRoot, 'deploy', 'helm', 'sdkwork-api-router', 'Chart.yaml');
  const valuesPath = path.join(repoRoot, 'deploy', 'helm', 'sdkwork-api-router', 'values.yaml');
  const deploymentTemplatePath = path.join(
    repoRoot,
    'deploy',
    'helm',
    'sdkwork-api-router',
    'templates',
    'deployment.yaml',
  );
  const serviceTemplatePath = path.join(
    repoRoot,
    'deploy',
    'helm',
    'sdkwork-api-router',
    'templates',
    'service.yaml',
  );
  const secretTemplatePath = path.join(
    repoRoot,
    'deploy',
    'helm',
    'sdkwork-api-router',
    'templates',
    'secret.yaml',
  );
  const ingressTemplatePath = path.join(
    repoRoot,
    'deploy',
    'helm',
    'sdkwork-api-router',
    'templates',
    'ingress.yaml',
  );

  assert.equal(existsSync(dockerfilePath), true, 'missing deploy/docker/Dockerfile');
  assert.equal(existsSync(composePath), true, 'missing deploy/docker/docker-compose.yml');
  assert.equal(existsSync(envExamplePath), true, 'missing deploy/docker/.env.example');
  assert.equal(existsSync(chartPath), true, 'missing deploy/helm/sdkwork-api-router/Chart.yaml');
  assert.equal(existsSync(valuesPath), true, 'missing deploy/helm/sdkwork-api-router/values.yaml');
  assert.equal(existsSync(deploymentTemplatePath), true, 'missing Helm deployment template');
  assert.equal(existsSync(serviceTemplatePath), true, 'missing Helm service template');
  assert.equal(existsSync(secretTemplatePath), true, 'missing Helm secret template');
  assert.equal(existsSync(ingressTemplatePath), true, 'missing Helm ingress template');

  const dockerfile = read('deploy/docker/Dockerfile');
  assert.match(dockerfile, /router-product-service/);
  assert.match(dockerfile, /SDKWORK_BOOTSTRAP_PROFILE=prod/);
  assert.match(dockerfile, /SDKWORK_BOOTSTRAP_DATA_DIR=\/opt\/sdkwork\/data/);
  assert.match(dockerfile, /SDKWORK_ADMIN_SITE_DIR=\/opt\/sdkwork\/sites\/admin\/dist/);
  assert.match(dockerfile, /SDKWORK_PORTAL_SITE_DIR=\/opt\/sdkwork\/sites\/portal\/dist/);
  assert.match(dockerfile, /mkdir -p config data log run/);
  assert.doesNotMatch(dockerfile, /var\/data/);
  assert.doesNotMatch(dockerfile, /var\/log/);
  assert.doesNotMatch(dockerfile, /var\/run/);
  assert.match(dockerfile, /EXPOSE 3001/);
  assert.match(dockerfile, /HEALTHCHECK[\s\S]*\/api\/v1\/health/);
  assert.match(dockerfile, /USER sdkwork/);

  const compose = read('deploy/docker/docker-compose.yml');
  assert.match(compose, /^services:/m);
  assert.match(compose, /^\s{2}postgres:/m);
  assert.match(compose, /image:\s*docker\.io\/library\/postgres:16-alpine/);
  assert.match(compose, /^\s{2}router:/m);
  assert.match(compose, /dockerfile:\s*deploy\/docker\/Dockerfile/);
  assert.match(compose, /SDKWORK_DATABASE_URL:/);
  assert.match(compose, /SDKWORK_BOOTSTRAP_PROFILE:/);
  assert.match(compose, /SDKWORK_ADMIN_JWT_SIGNING_SECRET:/);
  assert.match(compose, /SDKWORK_PORTAL_JWT_SIGNING_SECRET:/);
  assert.match(compose, /SDKWORK_CREDENTIAL_MASTER_KEY:/);
  assert.match(compose, /SDKWORK_METRICS_BEARER_TOKEN:/);
  assert.match(compose, /3001:3001/);
  assert.match(compose, /\/api\/v1\/health/);
  assert.match(compose, /security_opt:\s*[\s\S]*?no-new-privileges:true/);
  assert.match(compose, /cap_drop:\s*[\s\S]*?ALL/);

  const envExample = read('deploy/docker/.env.example');
  assert.match(envExample, /^SDKWORK_POSTGRES_DB=/m);
  assert.match(envExample, /^SDKWORK_POSTGRES_USER=/m);
  assert.match(envExample, /^SDKWORK_POSTGRES_PASSWORD=/m);
  assert.match(envExample, /^SDKWORK_ADMIN_JWT_SIGNING_SECRET=/m);
  assert.match(envExample, /^SDKWORK_PORTAL_JWT_SIGNING_SECRET=/m);
  assert.match(envExample, /^SDKWORK_CREDENTIAL_MASTER_KEY=/m);
  assert.match(envExample, /^SDKWORK_METRICS_BEARER_TOKEN=/m);

  const chart = read('deploy/helm/sdkwork-api-router/Chart.yaml');
  assert.match(chart, /^apiVersion:\s*v2$/m);
  assert.match(chart, /^name:\s*sdkwork-api-router$/m);
  assert.match(chart, /^type:\s*application$/m);

  const values = read('deploy/helm/sdkwork-api-router/values.yaml');
  assert.match(values, /^image:/m);
  assert.match(values, /^\s{2}repository:/m);
  assert.match(values, /ghcr\.io\/<owner>\/sdkwork-api-router/);
  assert.match(values, /^service:/m);
  assert.match(values, /^ingress:/m);
  assert.match(values, /^secrets:/m);
  assert.match(values, /bootstrapProfile:\s*prod/);
  assert.match(values, /automountServiceAccountToken:\s*false/);
  assert.match(values, /seccompProfile:\s*[\s\S]*?type:\s*RuntimeDefault/);

  const deployment = read('deploy/helm/sdkwork-api-router/templates/deployment.yaml');
  assert.match(deployment, /automountServiceAccountToken:\s*\{\{\s*\.Values\.automountServiceAccountToken\s*\}\}/);
  assert.match(deployment, /securityContext:\s*[\s\S]*?toYaml \.Values\.podSecurityContext/);
  assert.match(deployment, /SDKWORK_DATABASE_URL/);
  assert.match(deployment, /SDKWORK_BOOTSTRAP_PROFILE/);
  assert.match(deployment, /SDKWORK_BOOTSTRAP_DATA_DIR/);
  assert.match(deployment, /SDKWORK_ADMIN_SITE_DIR/);
  assert.match(deployment, /SDKWORK_PORTAL_SITE_DIR/);
  assert.match(deployment, /SDKWORK_ADMIN_JWT_SIGNING_SECRET/);
  assert.match(deployment, /SDKWORK_PORTAL_JWT_SIGNING_SECRET/);
  assert.match(deployment, /SDKWORK_CREDENTIAL_MASTER_KEY/);
  assert.match(deployment, /SDKWORK_METRICS_BEARER_TOKEN/);
  assert.match(deployment, /\/api\/v1\/health/);
  assert.match(deployment, /\/api\/admin\/health/);
  assert.match(deployment, /\/api\/portal\/health/);

  const secretTemplate = read('deploy/helm/sdkwork-api-router/templates/secret.yaml');
  assert.match(secretTemplate, /SDKWORK_DATABASE_URL/);
  assert.match(secretTemplate, /SDKWORK_ADMIN_JWT_SIGNING_SECRET/);
  assert.match(secretTemplate, /SDKWORK_PORTAL_JWT_SIGNING_SECRET/);
  assert.match(secretTemplate, /SDKWORK_CREDENTIAL_MASTER_KEY/);
  assert.match(secretTemplate, /SDKWORK_METRICS_BEARER_TOKEN/);
});

test('native product server release packager exports deployment assets into commercial bundles', async () => {
  const packager = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'release', 'package-release-assets.mjs')).href,
  );

  assert.equal(
    typeof packager.listNativeProductServerDeploymentAssetRoots,
    'function',
    'expected deployment roots export for product-server bundles',
  );
  assert.equal(
    typeof packager.findNativeProductServerDeploymentAssetRoot,
    'function',
    'expected strict deployment root lookup helper for product-server bundles',
  );
  assert.equal(
    typeof packager.listNativeProductServerDeploymentAssetRootsByIds,
    'function',
    'expected strict deployment root batch lookup helper for product-server bundles',
  );
  assert.deepEqual(
    packager.listNativeProductServerDeploymentAssetRoots(),
    {
      deploy: path.join(repoRoot, 'deploy'),
    },
  );
  assert.equal(
    packager.findNativeProductServerDeploymentAssetRoot('deploy'),
    path.join(repoRoot, 'deploy'),
  );
  assert.deepEqual(
    packager.listNativeProductServerDeploymentAssetRootsByIds([
      'deploy',
    ]),
    {
      deploy: path.join(repoRoot, 'deploy'),
    },
  );

  const packagerSource = read('scripts/release/package-release-assets.mjs');
  assert.match(packagerSource, /deploymentAssetDir\}\/: docker, compose, and helm deployment assets/);
  assert.match(packagerSource, /deploymentAssetRoots/);
  assert.match(packagerSource, /listNativeProductServerDeploymentAssetRoots/);
  assert.match(packagerSource, /findNativeProductServerDeploymentAssetRoot/);
  assert.match(packagerSource, /listNativeProductServerDeploymentAssetRootsByIds/);
});

test('release packager detects Windows tar flavor before deciding whether --force-local is safe', async () => {
  const packager = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'release', 'package-release-assets.mjs')).href,
  );

  assert.equal(
    packager.detectTarFlavor({
      platform: 'win32',
      spawn: () => ({
        status: 0,
        stdout: 'bsdtar 3.7.7 - libarchive 3.7.7',
        stderr: '',
      }),
    }),
    'bsd',
  );
  assert.equal(
    packager.detectTarFlavor({
      platform: 'win32',
      spawn: () => ({
        status: 0,
        stdout: 'tar (GNU tar) 1.35',
        stderr: '',
      }),
    }),
    'gnu',
  );
});

test('release packager only uses --force-local for GNU tar on Windows', async () => {
  const packager = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'release', 'package-release-assets.mjs')).href,
  );

  assert.deepEqual(
    packager.createTarCommandPlan({
      archivePath: 'C:\\release\\bundle.tar.gz',
      workingDirectory: 'C:\\release',
      entryName: 'bundle',
      platform: 'win32',
      tarFlavor: 'bsd',
    }),
    {
      command: 'tar',
      args: ['-czf', 'C:\\release\\bundle.tar.gz', '-C', 'C:\\release', 'bundle'],
      shell: true,
    },
  );
  assert.deepEqual(
    packager.createTarCommandPlan({
      archivePath: 'C:\\release\\bundle.tar.gz',
      workingDirectory: 'C:\\release',
      entryName: 'bundle',
      platform: 'win32',
      tarFlavor: 'gnu',
    }),
    {
      command: 'tar',
      args: ['--force-local', '-czf', 'C:\\release\\bundle.tar.gz', '-C', 'C:\\release', 'bundle'],
      shell: true,
    },
  );
});
