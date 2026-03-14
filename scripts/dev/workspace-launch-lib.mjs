function requireValue(argv, index, flag) {
  const value = argv[index + 1];
  if (!value || value.startsWith('--')) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

export function parseWorkspaceArgs(argv) {
  const settings = {
    databaseUrl: 'sqlite://sdkwork-api-server.db',
    gatewayBind: '127.0.0.1:8080',
    adminBind: '127.0.0.1:8081',
    portalBind: '127.0.0.1:8082',
    install: false,
    preview: false,
    tauri: false,
    dryRun: false,
    help: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];

    switch (arg) {
      case '--database-url':
        settings.databaseUrl = requireValue(argv, index, arg);
        index += 1;
        break;
      case '--gateway-bind':
        settings.gatewayBind = requireValue(argv, index, arg);
        index += 1;
        break;
      case '--admin-bind':
        settings.adminBind = requireValue(argv, index, arg);
        index += 1;
        break;
      case '--portal-bind':
        settings.portalBind = requireValue(argv, index, arg);
        index += 1;
        break;
      case '--install':
        settings.install = true;
        break;
      case '--preview':
        settings.preview = true;
        break;
      case '--tauri':
        settings.tauri = true;
        break;
      case '--dry-run':
        settings.dryRun = true;
        break;
      case '--help':
      case '-h':
        settings.help = true;
        break;
      default:
        throw new Error(`unknown option: ${arg}`);
    }
  }

  return settings;
}

export function buildWorkspaceCommandPlan(settings) {
  const backendArgs = [
    'scripts/dev/start-stack.mjs',
    '--database-url',
    settings.databaseUrl,
    '--gateway-bind',
    settings.gatewayBind,
    '--admin-bind',
    settings.adminBind,
    '--portal-bind',
    settings.portalBind,
  ];
  if (settings.dryRun) {
    backendArgs.push('--dry-run');
  }

  const consoleArgs = ['scripts/dev/start-console.mjs'];
  if (settings.install) {
    consoleArgs.push('--install');
  }
  if (settings.preview) {
    consoleArgs.push('--preview');
  } else if (settings.tauri) {
    consoleArgs.push('--tauri');
  }
  if (settings.dryRun) {
    consoleArgs.push('--dry-run');
  }

  return {
    nodeExecutable: process.execPath,
    backend: {
      name: 'backend',
      scriptPath: 'scripts/dev/start-stack.mjs',
      args: backendArgs,
    },
    console: {
      name: settings.preview ? 'console-preview' : settings.tauri ? 'console-tauri' : 'console-browser',
      scriptPath: 'scripts/dev/start-console.mjs',
      args: consoleArgs,
    },
  };
}

export function workspaceHelpText() {
  return `Usage: node scripts/dev/start-workspace.mjs [options]

Starts the backend services and the browser console or Tauri desktop host in one command.

Options:
  --database-url <url>   Shared SDKWORK_DATABASE_URL value for admin, gateway, and portal
  --gateway-bind <bind>  SDKWORK_GATEWAY_BIND override
  --admin-bind <bind>    SDKWORK_ADMIN_BIND override
  --portal-bind <bind>   SDKWORK_PORTAL_BIND override
  --install              Run pnpm install before starting the console
  --preview              Build and preview the browser console instead of dev mode
  --tauri                Start the Tauri desktop shell; browser remains reachable through Vite
  --dry-run              Print the backend and console commands without running them
  -h, --help             Show this help
`;
}
