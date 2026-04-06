import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function escapeRegExp(text) {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function assertZhKeyExists(source, key) {
  const quotedPattern = new RegExp(`'${escapeRegExp(key)}': '[^']+'`);
  const barePattern = new RegExp(`(?:^|\\s)${escapeRegExp(key)}: '[^']+'`);
  assert.match(
    source,
    new RegExp(`${quotedPattern.source}|${barePattern.source}`),
  );
}

test('public site models module localizes catalog-facing detail fields through shared portal i18n', () => {
  const modelsPage = read('packages/sdkwork-router-portal-models/src/index.tsx');

  assert.match(modelsPage, /t\(row\.summary\)/);
  assert.match(modelsPage, /t\(row\.capability\)/);
  assert.match(modelsPage, /row\.modalities\.map\(\(modality(?:\:\s*string)?\) => t\(modality\)\)/);
  assert.match(modelsPage, /t\(row\.latencyClass\)/);
});

test('public site copy has direct zh-CN coverage for home, docs, downloads, and model catalog content', () => {
  const zhMessages = read('packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts');
  const criticalKeys = [
    'API Reference',
    'OpenAPI 3.1',
    'Gateway API',
    'Portal API',
    'Developer API center',
    'Explore real-time generated OpenAPI specifications, auth boundaries, and production-ready request flows for gateway and portal integrations.',
    'One developer-facing center for public gateway execution and self-service portal APIs.',
    'Live OpenAPI documents',
    'Generated directly from the current Rust router implementation so developer docs stay aligned with shipped API behavior.',
    'Gateway compatibility',
    'OpenAI-compatible execution surface for models, responses, embeddings, audio, images, and multimodal workloads.',
    'Portal self-service',
    'Workspace auth, API keys, billing, usage, routing, and commerce endpoints for developers and end users.',
    'Authentication boundaries',
    'Keep gateway API keys and portal JWT flows visible side by side before integration begins.',
    'Open gateway docs',
    'Open portal docs',
    'Open raw spec',
    'Server URL',
    'Primary auth',
    'Spec endpoint',
    'Interactive docs',
    'Schema version',
    'Operations indexed',
    'Tagged route groups',
    'Live status',
    'Loading live schema',
    'Connected',
    'Unavailable',
    'Live schema unavailable for this surface right now.',
    'Loading route groups from the live schema.',
    'No tagged route groups are available from the current schema.',
    'Total operations across gateway and portal developer surfaces, derived from the current live schemas.',
    'Combined tagged route groups published by the live developer-facing OpenAPI documents.',
    'Reference focus',
    'Route families',
    'Common workflow',
    'Chat',
    'Responses',
    'Embeddings',
    'Moderations',
    'Images',
    'Compatibility',
    'Authentication',
    'First request',
    'Register or sign in, inspect workspace state, issue an API key, and connect self-service automation.',
    'List models or issue OpenAI-compatible inference traffic with one gateway API key.',
    'Unified AI gateway workspace',
    'Operate routing, credentials, usage, and downloads from one product surface.',
    'SDKWork Router Portal now separates the public product experience from the authenticated console so teams can evaluate the platform, compare models, read docs, and install software without losing a cohesive design language.',
    'Launch sequence',
    'A single information architecture for evaluation, onboarding, and console operations.',
    'Product pathways',
    'Each pathway keeps discovery, onboarding, and execution connected instead of fragmenting into separate microsites.',
    'Launch tracks',
    'Choose the fastest path for platform owners, application teams, and operators without changing product context.',
    'Business-ready surfaces',
    'Home, models, docs, and software downloads share one navigation contract and one visual language.',
    'Operator-first onboarding',
    'Every CTA moves toward a real console action, install flow, or implementation guide.',
    'Launch without context loss',
    'Teams can evaluate the platform, compare models, and start the runtime without jumping between products.',
    'Platform teams',
    'Start with product posture',
    'Compare platform capabilities, public navigation, and rollout paths before entering the console.',
    'Application teams',
    'Map the model layer',
    'Review models, SDK guidance, and API workflows before issuing credentials or integrating clients.',
    'Operations teams',
    'Follow implementation guides',
    'Install and launch runtime',
    'Choose the correct desktop or service mode, verify requirements, and start the product with guided onboarding.',
    'Enter the operational workspace and manage runtime posture.',
    'Browse multimodal providers, capabilities, and deployment options.',
    'Read the integration guides, quickstarts, and API references.',
    'Install the desktop runtime and tooling packages.',
    'Enter console',
    'Read docs',
    'Download software',
    'Documentation center',
    'Move from evaluation to implementation with one route-aware documentation center.',
    'Quickstart, integration, reference, and operations guidance stay connected to the same product flows used across models, downloads, and the console.',
    'Documentation tracks',
    'Choose a guide family based on launch stage, integration depth, and operational ownership.',
    'Guide groups',
    'Documentation modules remain grouped by implementation stage instead of splitting into disconnected help surfaces.',
    'Launch steps',
    'Each registry entry keeps onboarding, integration, and operations work visible in one documentation center.',
    'Route-aware actions',
    'Every guide stays connected to the next real product destination, from downloads and models to the console.',
    'Implementation lanes',
    'Move through onboarding, integration, reference lookup, and operations review without leaving the documentation surface.',
    'Operating loop',
    'The selected documentation group keeps its next actions and operational outcomes visible in one place.',
    'A registry-driven docs module keeps quickstart, integration, reference, and operations guidance separate from the console workspace.',
    'Quickstart',
    'Start the gateway, desktop runtime, and workspace access flows.',
    'First request',
    'Bring up the router, create an API key, and verify a request against the shared gateway.',
    'SDKs & Integration',
    'Connect SDKs, routing policies, providers, and workspace credentials.',
    'Reference',
    'API surface details, field definitions, and request lifecycle facts.',
    'Operations',
    'Runbooks for telemetry review, quotas, billing posture, and software rollout.',
    'Providers',
    'Multimodal models',
    'Catalog size',
    'Model center',
    'Search, compare, and shortlist models with the same product language used throughout the public site and console.',
    'Compare multimodal providers, context posture, pricing bands, and launch fit before routing traffic.',
    'Featured model',
    'Provider lanes',
    'Provider coverage, modality range, and catalog density stay visible while filters narrow the shortlist.',
    'Selection tracks',
    'Recommended starting points for agentic workflows, enterprise assistants, search stacks, and audio pipelines.',
    'Best for global assistants',
    'Best for Chinese knowledge',
    'Best for coding agents',
    'Compare provider coverage, modality mix, context window, and operating posture in one searchable catalog.',
    'Search models, providers, capabilities, or modalities',
    'All providers',
    'All modalities',
    'Provider',
    'Modality',
    'Capability',
    'Context window',
    'Pricing',
    'No models match the current filter',
    'Try broadening the provider, modality, or search criteria to compare more catalog entries.',
    '{count} models visible in the current model center view.',
    'Text',
    'Vision',
    'Audio',
    'Embedding',
    'Low',
    'Medium',
    'General reasoning and multimodal content understanding',
    'A production-friendly Volcengine model for chat, tool workflows, and image-aware prompting.',
    'Multimodal instruction following and enterprise assistant workflows',
    'Baidu AI Cloud option tuned for Chinese retrieval, enterprise knowledge work, and document analysis.',
    'Long-form reasoning, code generation, and chain-of-thought heavy tasks',
    'Strong for workspace automation, coding copilots, and long-context business tasks.',
    'Launch posture',
    'Choose the operating mode that matches how your team will run gateway traffic and operator workflows.',
    'Launch the runtime your team will actually operate.',
    'Desktop, background service, and shared gateway distributions stay connected to docs, console, and onboarding actions from one software center.',
    'Delivery targets',
    'Install packages stay segmented by environment while keeping the same runtime contract and onboarding path.',
    'Runtime postures',
    'Runtime modes stay visible so operators can choose the correct service shape before launch.',
    'Guided launch steps',
    'One guided sequence keeps install, sign-in, health checks, and console handoff aligned.',
    'Install targets',
    'Pick the desktop or service path that matches the environment you need to onboard.',
    'Deployment tracks',
    'Choose the software path that matches local operators, background automation, or shared gateway delivery.',
    'Rollout loop',
    'Installation, launch, verification, and console handoff stay connected in one software delivery surface.',
    'Open desktop setup',
    'Open server setup',
    'Launch the product in the same posture your operators will use in production or local evaluation.',
  ];

  for (const key of criticalKeys) {
    assertZhKeyExists(zhMessages, key);
  }
});
