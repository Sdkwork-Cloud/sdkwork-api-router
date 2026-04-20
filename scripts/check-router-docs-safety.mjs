#!/usr/bin/env node

import { existsSync, readdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { createStrictKeyedCatalog } from './strict-contract-catalog.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const docBootstrapScanRootCatalog = createStrictKeyedCatalog({
  entries: [
  'docs/getting-started',
  'docs/api-reference',
  'docs/operations',
  'docs/zh/getting-started',
  'docs/zh/api-reference',
  'docs/zh/operations',
  ],
  getKey: (relativeRoot) => relativeRoot,
  duplicateKeyMessagePrefix: 'duplicate docs bootstrap scan root',
  missingKeyMessagePrefix: 'missing docs bootstrap scan root',
});

const docBootstrapScanFileCatalog = createStrictKeyedCatalog({
  entries: [
  'README.md',
  'README.zh-CN.md',
  ],
  getKey: (relativeFile) => relativeFile,
  duplicateKeyMessagePrefix: 'duplicate docs bootstrap scan file',
  missingKeyMessagePrefix: 'missing docs bootstrap scan file',
});

function cloneRetiredBootstrapMarker(marker) {
  return {
    ...marker,
    pattern: new RegExp(marker.pattern.source, marker.pattern.flags),
  };
}

const retiredBootstrapMarkerCatalog = createStrictKeyedCatalog({
  entries: [
    {
      name: 'retired admin bootstrap email',
      pattern: /admin@sdkwork\.local/i,
    },
    {
      name: 'retired portal bootstrap email',
      pattern: /portal@sdkwork\.local/i,
    },
    {
      name: 'retired bootstrap password',
      pattern: /ChangeMe123!/i,
    },
    {
      name: 'retired seeded credentials copy',
      pattern: /Seeded local credentials/i,
    },
    {
      name: 'retired default operator copy',
      pattern: /default local operator/i,
    },
    {
      name: 'retired demo account copy',
      pattern: /demo local account/i,
    },
    {
      name: 'retired built-in demo account copy',
      pattern: /built-in .*demo accounts/i,
    },
  ],
  getKey: ({ name }) => name,
  clone: cloneRetiredBootstrapMarker,
  duplicateKeyMessagePrefix: 'duplicate retired bootstrap marker',
  missingKeyMessagePrefix: 'missing retired bootstrap marker',
});

export const DOC_BOOTSTRAP_SCAN_ROOTS = docBootstrapScanRootCatalog.list();

export const DOC_BOOTSTRAP_SCAN_FILES = docBootstrapScanFileCatalog.list();

export const RETIRED_BOOTSTRAP_MARKERS = retiredBootstrapMarkerCatalog.list();

export function listDocBootstrapScanRoots() {
  return docBootstrapScanRootCatalog.list();
}

export function findDocBootstrapScanRoot(relativeRoot) {
  return docBootstrapScanRootCatalog.find(relativeRoot);
}

export function listDocBootstrapScanRootsByPaths(relativeRoots = []) {
  return docBootstrapScanRootCatalog.listByKeys(relativeRoots);
}

export function listDocBootstrapScanFiles() {
  return docBootstrapScanFileCatalog.list();
}

export function findDocBootstrapScanFile(relativeFile) {
  return docBootstrapScanFileCatalog.find(relativeFile);
}

export function listDocBootstrapScanFilesByPaths(relativeFiles = []) {
  return docBootstrapScanFileCatalog.listByKeys(relativeFiles);
}

export function listRetiredBootstrapMarkers() {
  return retiredBootstrapMarkerCatalog.list();
}

export function findRetiredBootstrapMarker(markerName) {
  return retiredBootstrapMarkerCatalog.find(markerName);
}

export function listRetiredBootstrapMarkersByNames(markerNames = []) {
  return retiredBootstrapMarkerCatalog.listByKeys(markerNames);
}

function listMarkdownFiles(rootDir) {
  if (!existsSync(rootDir)) {
    return [];
  }

  const files = [];
  for (const entry of readdirSync(rootDir, { withFileTypes: true })) {
    const absolutePath = path.join(rootDir, entry.name);
    if (entry.isDirectory()) {
      files.push(...listMarkdownFiles(absolutePath));
      continue;
    }
    if (entry.isFile() && absolutePath.endsWith('.md')) {
      files.push(absolutePath);
    }
  }
  return files;
}

export function scanDocsForRetiredBootstrapCredentials({
  workspaceRoot = path.resolve(__dirname, '..'),
} = {}) {
  const findings = [];
  const retiredBootstrapMarkers = listRetiredBootstrapMarkers();

  for (const relativeFile of listDocBootstrapScanFiles()) {
    const absolutePath = path.join(workspaceRoot, relativeFile);
    if (!existsSync(absolutePath)) {
      continue;
    }

    const lines = readFileSync(absolutePath, 'utf8').split(/\r?\n/u);
    for (let index = 0; index < lines.length; index += 1) {
      const line = lines[index];
      for (const marker of retiredBootstrapMarkers) {
        if (!marker.pattern.test(line)) {
          continue;
        }
        findings.push({
          file: path.relative(workspaceRoot, absolutePath).replace(/\\/gu, '/'),
          line: index + 1,
          marker: marker.name,
          excerpt: line.trim(),
        });
      }
    }
  }

  for (const relativeRoot of listDocBootstrapScanRoots()) {
    const absoluteRoot = path.join(workspaceRoot, relativeRoot);
    for (const filePath of listMarkdownFiles(absoluteRoot)) {
      const lines = readFileSync(filePath, 'utf8').split(/\r?\n/u);
      for (let index = 0; index < lines.length; index += 1) {
        const line = lines[index];
        for (const marker of retiredBootstrapMarkers) {
          if (!marker.pattern.test(line)) {
            continue;
          }
          findings.push({
            file: path.relative(workspaceRoot, filePath).replace(/\\/gu, '/'),
            line: index + 1,
            marker: marker.name,
            excerpt: line.trim(),
          });
        }
      }
    }
  }

  return findings.sort((left, right) => (
    left.file.localeCompare(right.file)
    || left.line - right.line
    || left.marker.localeCompare(right.marker)
  ));
}

function main() {
  const findings = scanDocsForRetiredBootstrapCredentials();
  if (findings.length === 0) {
    return;
  }

  console.error('[check-router-docs-safety] Retired bootstrap credential markers found:');
  for (const finding of findings) {
    console.error(
      `[check-router-docs-safety] ${finding.file}:${finding.line} ${finding.marker}: ${finding.excerpt}`,
    );
  }
  process.exit(1);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main();
}
