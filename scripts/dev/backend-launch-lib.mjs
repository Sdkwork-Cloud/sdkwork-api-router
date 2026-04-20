export function parseStackArgs(argv) {
  const result = {
    databaseUrl: null,
    gatewayBind: '127.0.0.1:9980',
    adminBind: '127.0.0.1:9981',
    portalBind: '127.0.0.1:9982',
    dryRun: false,
    help: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--dry-run') {
      result.dryRun = true;
      continue;
    }
    if (arg === '--help' || arg === '-h') {
      result.help = true;
      continue;
    }
    if (arg === '--database-url') {
      result.databaseUrl = argv[index + 1] ?? result.databaseUrl;
      index += 1;
      continue;
    }
    if (arg === '--gateway-bind') {
      result.gatewayBind = argv[index + 1] ?? result.gatewayBind;
      index += 1;
      continue;
    }
    if (arg === '--admin-bind') {
      result.adminBind = argv[index + 1] ?? result.adminBind;
      index += 1;
      continue;
    }
    if (arg === '--portal-bind') {
      result.portalBind = argv[index + 1] ?? result.portalBind;
      index += 1;
    }
  }

  return result;
}

export function stackHelpText() {
  return `Usage: node scripts/dev/start-stack.mjs [options]

Starts admin, gateway, and portal services in the current terminal.

Options:
  --database-url <url>   Optional shared SDKWORK_DATABASE_URL override
  --gateway-bind <bind>  SDKWORK_GATEWAY_BIND override
  --admin-bind <bind>    SDKWORK_ADMIN_BIND override
  --portal-bind <bind>   SDKWORK_PORTAL_BIND override
  --dry-run              Print commands without starting processes
  -h, --help             Show this help
`;
}

function escapeYamlDoubleQuotedScalar(value) {
  return String(value)
    .replaceAll('\\', '\\\\')
    .replaceAll('"', '\\"');
}

export function renderSourceDevRouterConfig(settings) {
  const lines = [
    '# Generated source-dev router config.',
    `gateway_bind: "${escapeYamlDoubleQuotedScalar(settings.gatewayBind)}"`,
    `admin_bind: "${escapeYamlDoubleQuotedScalar(settings.adminBind)}"`,
    `portal_bind: "${escapeYamlDoubleQuotedScalar(settings.portalBind)}"`,
  ];

  if (settings.databaseUrl) {
    lines.push(`database_url: "${escapeYamlDoubleQuotedScalar(settings.databaseUrl)}"`);
  }

  lines.push('');
  return lines.join('\n');
}

export function serviceEnv(settings, baseEnv = process.env, {
  sourceConfigDir = '',
  sourceConfigFile = '',
} = {}) {
  const env = {
    ...baseEnv,
    SDKWORK_GATEWAY_BIND: settings.gatewayBind,
    SDKWORK_ADMIN_BIND: settings.adminBind,
    SDKWORK_PORTAL_BIND: settings.portalBind,
  };

  delete env.SDKWORK_CONFIG_DIR;
  delete env.SDKWORK_CONFIG_FILE;

  if (sourceConfigDir) {
    env.SDKWORK_CONFIG_DIR = sourceConfigDir;
  }
  if (sourceConfigFile) {
    env.SDKWORK_CONFIG_FILE = sourceConfigFile;
  }

  if (settings.databaseUrl) {
    env.SDKWORK_DATABASE_URL = settings.databaseUrl;
  } else {
    delete env.SDKWORK_DATABASE_URL;
  }

  return env;
}

export function databaseDisplayValue(settings) {
  return settings.databaseUrl ?? '(local default via config loader)';
}
