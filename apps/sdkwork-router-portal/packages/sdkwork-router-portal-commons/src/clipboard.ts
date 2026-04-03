export async function copyText(value: string): Promise<boolean> {
  if (!value) {
    return false;
  }

  try {
    await globalThis.navigator?.clipboard?.writeText(value);
    return true;
  } catch {
    return false;
  }
}
