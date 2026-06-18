const PRESERVED_RECORD_KEYS = new Set(['credentials']);

const toSnakeCase = (s: string): string =>
  s.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);

const toCamelCase = (s: string): string =>
  s.replace(/_([a-z])/g, (_, letter: string) => letter.toUpperCase());

export const snakeKeys = <T>(obj: T): T => transformKeys(obj, toSnakeCase);

export const camelKeys = <T>(obj: T): T => transformKeys(obj, toCamelCase);

const transformKeys = <T>(obj: T, fn: (k: string) => string): T => {
  if (Array.isArray(obj)) {
    return obj.map((v) => transformKeys(v, fn)) as T;
  }
  if (obj !== null && typeof obj === 'object') {
    return Object.fromEntries(
      Object.entries(obj as Record<string, unknown>).map(([k, v]) => [
        fn(k),
        PRESERVED_RECORD_KEYS.has(k) ? v : transformKeys(v, fn)
      ])
    ) as T;
  }
  return obj;
};
