export function normalizeForwardedCliArgs(args = []) {
  if (!Array.isArray(args) || args.length === 0) {
    return [];
  }

  const forwardedArgs = [...args];
  const separatorIndex = forwardedArgs.indexOf('--');
  if (separatorIndex >= 0) {
    forwardedArgs.splice(separatorIndex, 1);
  }

  return forwardedArgs;
}
