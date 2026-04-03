import {
  PORTAL_ZH_CN_MESSAGES,
  translatePortalZhCnFallback,
} from './portalMessages.zh-CN';

export type PortalLocale = 'en-US' | 'zh-CN';

type TranslationValues = Record<string, string | number>;

let activePortalCoreLocale: PortalLocale = 'en-US';

function interpolate(text: string, values?: TranslationValues): string {
  if (!values) {
    return text;
  }

  return Object.entries(values).reduce(
    (result, [key, value]) => result.replaceAll(`{${key}}`, String(value)),
    text,
  );
}

export function setActivePortalCoreLocale(locale: PortalLocale): void {
  activePortalCoreLocale = locale;
}

export function translatePortalText(text: string, values?: TranslationValues): string {
  const translated = activePortalCoreLocale === 'en-US'
    ? text
    : PORTAL_ZH_CN_MESSAGES[text] ?? translatePortalZhCnFallback(text) ?? text;

  return interpolate(translated, values);
}
