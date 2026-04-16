import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');
const dictionaryPath = path.join(
  appRoot,
  'packages',
  'sdkwork-router-portal-commons',
  'src',
  'portalMessages.zh-CN.ts',
);

function readDictionary() {
  return readFileSync(dictionaryPath, 'utf8');
}

test('zh-CN portal dictionary keeps critical billing and routing translations human-readable', () => {
  const dictionary = readDictionary();

  const expectedTranslations = new Map([
    [
      'Rotate this key to reveal a new one-time secret before applying {label} setup or copying local snippets.',
      '请先轮换此密钥以显示新的一次性明文密钥，然后再应用 {label} 配置或复制本地片段。',
    ],
    ['Past due', '已逾期'],
    ['Paused', '已暂停'],
    ['Grace period', '宽限期'],
    ['Enterprise quota', '企业配额'],
    [
      'Fallback reasoning stays visible so you can distinguish degraded routing from the preferred routing path.',
      '系统会持续展示回退原因，便于你区分降级路由与首选路由路径。',
    ],
  ]);

  for (const [key, translation] of expectedTranslations) {
    assert.match(
      dictionary,
      new RegExp(`'${key.replace(/[.*+?^${}()|[\\]\\\\]/g, '\\\\$&')}': '${translation.replace(/[.*+?^${}()|[\\]\\\\]/g, '\\\\$&')}'`),
    );
  }

  assert.doesNotMatch(
    dictionary,
    /'Past due': 'å| 'Paused': 'å| 'Grace period': 'å| 'Enterprise quota': 'ä| 'Fallback reasoning stays visible[\s\S]*ç³»ç»Ÿ| 'Rotate this key to reveal a new one-time secret[\s\S]*è¯·å…ˆ/,
  );
});
