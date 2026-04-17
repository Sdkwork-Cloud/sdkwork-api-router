const MAX_SAFE_INTEGER_TEXT = String(Number.MAX_SAFE_INTEGER);

export async function readOpenApiDocument<T>(response: Response): Promise<T> {
  if (typeof response.text === 'function') {
    const body = await response.text();
    return (body ? parseJsonBody(body) : null) as T;
  }

  if (typeof response.json === 'function') {
    return response.json() as Promise<T>;
  }

  return null as T;
}

function parseJsonBody(body: string): unknown {
  return JSON.parse(quoteUnsafeIntegerTokens(body));
}

function quoteUnsafeIntegerTokens(body: string): string {
  let result = '';
  let index = 0;
  let inString = false;
  let escaped = false;

  while (index < body.length) {
    const character = body[index];

    if (inString) {
      result += character;
      if (escaped) {
        escaped = false;
      } else if (character === '\\') {
        escaped = true;
      } else if (character === '"') {
        inString = false;
      }
      index += 1;
      continue;
    }

    if (character === '"') {
      inString = true;
      result += character;
      index += 1;
      continue;
    }

    if (character === '-' || isDigit(character)) {
      const tokenEnd = findNumericTokenEnd(body, index);
      const token = body.slice(index, tokenEnd);

      if (shouldQuoteIntegerToken(token)) {
        result += `"${token}"`;
      } else {
        result += token;
      }

      index = tokenEnd;
      continue;
    }

    result += character;
    index += 1;
  }

  return result;
}

function findNumericTokenEnd(body: string, start: number): number {
  let index = start;

  if (body[index] === '-') {
    index += 1;
  }

  while (index < body.length && isDigit(body[index])) {
    index += 1;
  }

  if (body[index] === '.') {
    index += 1;
    while (index < body.length && isDigit(body[index])) {
      index += 1;
    }
  }

  if (body[index] === 'e' || body[index] === 'E') {
    index += 1;
    if (body[index] === '+' || body[index] === '-') {
      index += 1;
    }
    while (index < body.length && isDigit(body[index])) {
      index += 1;
    }
  }

  return index;
}

function shouldQuoteIntegerToken(token: string): boolean {
  if (!/^-?\d+$/.test(token)) {
    return false;
  }

  const normalized = token.startsWith('-') ? token.slice(1) : token;
  if (normalized.length < MAX_SAFE_INTEGER_TEXT.length) {
    return false;
  }
  if (normalized.length > MAX_SAFE_INTEGER_TEXT.length) {
    return true;
  }
  return normalized > MAX_SAFE_INTEGER_TEXT;
}

function isDigit(character: string | undefined): boolean {
  return character != null && character >= '0' && character <= '9';
}
