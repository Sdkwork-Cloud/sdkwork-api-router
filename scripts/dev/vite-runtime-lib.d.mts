export interface ReadableModuleResolution {
  candidateRoot: string;
  resolvedPath: string;
}

export function probeReadableFile(
  filePath: string,
  options?: {
    fileExists?: (filePath: string) => boolean;
    openFile?: (filePath: string) => number;
    closeFile?: (fileDescriptor: number) => void;
  },
): boolean;

export function findReadableModuleResolution(options: {
  appRoot: string;
  donorRoots?: string[];
  specifier: string;
  resolveFromRoot?: (root: string, specifier: string) => string;
  isReadable?: (filePath: string) => boolean;
}): ReadableModuleResolution;

export function importReadablePackageDefault<T = unknown>(options: {
  appRoot: string;
  donorRoots?: string[];
  packageName: string;
  relativeEntry: string | string[];
}): Promise<T>;

export function resolveReadablePackageEntry(options: {
  appRoot: string;
  donorRoots?: string[];
  packageName: string;
  relativeEntry: string | string[];
  fileExists?: (filePath: string) => boolean;
  readDir?: (
    directoryPath: string,
    options: { withFileTypes: true },
  ) => Array<{ isDirectory(): boolean; name: string }>;
  isReadable?: (filePath: string) => boolean;
}): string;

export function resolveReadablePackageRoot(options: {
  appRoot: string;
  donorRoots?: string[];
  packageName: string;
  relativeEntry?: string | string[];
  fileExists?: (filePath: string) => boolean;
  readDir?: (
    directoryPath: string,
    options: { withFileTypes: true },
  ) => Array<{ isDirectory(): boolean; name: string }>;
  isReadable?: (filePath: string) => boolean;
}): string;

export function resolveReadablePackageImportUrl(options: {
  appRoot: string;
  donorRoots?: string[];
  packageName: string;
  relativeEntry: string | string[];
  fileExists?: (filePath: string) => boolean;
  readDir?: (
    directoryPath: string,
    options: { withFileTypes: true },
  ) => Array<{ isDirectory(): boolean; name: string }>;
  isReadable?: (filePath: string) => boolean;
}): string;

export function resolveReadableModuleSpecifier(options: {
  appRoot: string;
  donorRoots?: string[];
  specifier: string;
  resolveFromRoot?: (root: string, specifier: string) => string;
  isReadable?: (filePath: string) => boolean;
}): string;

export function resolveReadableFallbackModulePath(options: {
  specifier: string;
  resolution: ReadableModuleResolution;
  resolveReadablePackageRootImpl?: (options: {
    appRoot: string;
    donorRoots?: string[];
    packageName: string;
    relativeEntry?: string | string[];
    fileExists?: (filePath: string) => boolean;
    readDir?: (
      directoryPath: string,
      options: { withFileTypes: true },
    ) => Array<{ isDirectory(): boolean; name: string }>;
    isReadable?: (filePath: string) => boolean;
  }) => string;
  readPackageJsonImpl?: (packageRoot: string) => Record<string, unknown>;
}): string;

export function resolveWorkspaceDonorRoots(appRoot: string): string[];

export function applyWindowsVitePreload(options?: {
  platform?: string;
}): Promise<void>;
