import {
  accessToken,
  clearAuthState,
  REFRESH_STORAGE_KEY,
  refreshToken,
  STORAGE_KEY,
  selectedTeamId
} from '../../stores/auth.store';
import { camelKeys, snakeKeys } from './snake-camel';

const isDevelopment = import.meta.env.DEV;
const baseURL = isDevelopment ? import.meta.env.VITE_API_URL || '/api/v1' : '/api/v1';

export class ApiError extends Error {
  status: number;
  serverError: string;

  constructor(status: number, serverError: string) {
    super(serverError);
    this.name = 'ApiError';
    this.status = status;
    this.serverError = serverError;
  }
}

interface ApiFetchOptions extends Omit<RequestInit, 'body'> {
  body?: unknown;
  params?: Record<string, string | number | boolean>;
}

const buildUrl = (path: string, params?: Record<string, string | number | boolean>): string => {
  const url = `${baseURL}${path}`;
  if (!params) return url;

  const search = new URLSearchParams();
  const snakeParams = snakeKeys(params) as Record<string, string>;
  for (const [key, value] of Object.entries(snakeParams)) {
    search.append(key, value);
  }
  return `${url}?${search.toString()}`;
};

const SENSITIVE_FIELDS = new Set([
  'code',
  'password',
  'token',
  'accessToken',
  'access_token',
  'refreshToken',
  'refresh_token',
  'secret',
  'api_key'
]);

const clearStoredTokens = (): void => {
  clearAuthState();
};

const refreshAccessToken = async (): Promise<boolean> => {
  if (!refreshToken.value) return false;

  const response = await fetch(buildUrl('/auth/refresh'), {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({ refresh_token: refreshToken.value })
  });

  if (!response.ok) {
    clearStoredTokens();
    return false;
  }

  const data = camelKeys(await response.json()) as {
    accessToken: string;
    refreshToken: string;
  };
  accessToken.value = data.accessToken;
  refreshToken.value = data.refreshToken;
  localStorage.setItem(STORAGE_KEY, data.accessToken);
  localStorage.setItem(REFRESH_STORAGE_KEY, data.refreshToken);

  return true;
};

const shouldRefreshRequest = (path: string): boolean =>
  ![
    '/auth/exchange',
    '/auth/refresh',
    '/auth/instance-login',
    '/auth/login',
    '/auth/verify'
  ].includes(path);

const sanitizeLogBody = (body: unknown): unknown => {
  if (body == null || typeof body !== 'object') return body;
  if (Array.isArray(body)) return body.map(sanitizeLogBody);

  const sanitized: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(body as Record<string, unknown>)) {
    sanitized[key] = SENSITIVE_FIELDS.has(key) ? '***' : sanitizeLogBody(value);
  }
  return sanitized;
};

const sanitizeLogUrl = (url: string): string => {
  const parsed = new URL(url, window.location.origin);
  for (const key of parsed.searchParams.keys()) {
    if (SENSITIVE_FIELDS.has(key)) {
      parsed.searchParams.set(key, '***');
    }
  }

  return parsed.origin === window.location.origin
    ? `${parsed.pathname}${parsed.search}`
    : parsed.toString();
};

const resolveBody = (body: unknown): unknown =>
  typeof body === 'function' ? (body as () => unknown)() : body;

const logRequest = (method: string, url: string, body?: unknown) => {
  if (!isDevelopment || import.meta.env.VITE_DISABLE_DEV_LOGGING) return;
  console.group(`API Request: ${method} ${sanitizeLogUrl(url)}`);
  if (body) console.log('Request Body:', sanitizeLogBody(body));
  console.groupEnd();
};

const logResponse = (method: string, url: string, status: number, data: unknown) => {
  if (!isDevelopment || import.meta.env.VITE_DISABLE_DEV_LOGGING) return;
  console.group(`API Response: ${method} ${sanitizeLogUrl(url)}`);
  console.log('Status:', status);
  console.log('Response:', sanitizeLogBody(data));
  console.groupEnd();
};

const logError = (method: string, url: string, status: number, error: unknown) => {
  if (!isDevelopment || import.meta.env.VITE_DISABLE_DEV_LOGGING) return;
  console.group(`API Error: ${method} ${sanitizeLogUrl(url)}`);
  console.log('Status:', status);
  console.log('Error:', error);
  console.groupEnd();
};

export const fetchApi = async <T>(path: string, options: ApiFetchOptions = {}): Promise<T> => {
  const { body, params, ...init } = options;
  const method = (init.method || 'GET').toUpperCase();
  const url = buildUrl(path, method === 'GET' ? params : undefined);

  const sendRequest = () => {
    const resolvedBody = resolveBody(body);
    const requestBody = resolvedBody != null ? JSON.stringify(snakeKeys(resolvedBody)) : undefined;
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(init.headers as Record<string, string> | undefined)
    };

    const token = accessToken.value;
    if (token) {
      headers.Authorization = `Bearer ${token}`;
    }

    if (selectedTeamId.value) {
      headers['X-Team-Id'] = selectedTeamId.value;
    }

    return fetch(url, {
      ...init,
      method,
      headers,
      body: requestBody
    });
  };

  logRequest(method, url, resolveBody(body));

  let response = await sendRequest();
  if (response.status === 401 && shouldRefreshRequest(path) && (await refreshAccessToken())) {
    response = await sendRequest();
  }

  const isJson = response.headers.get('content-type')?.includes('application/json');
  const data = isJson ? await response.json() : null;

  if (!response.ok) {
    if (response.status === 401) {
      clearStoredTokens();
    }

    const errorMessage =
      data && typeof data === 'object' && 'error' in data
        ? String(data.error)
        : response.statusText || 'Request failed';

    logError(method, url, response.status, errorMessage);
    throw new ApiError(response.status, errorMessage);
  }

  const result = data != null ? camelKeys(data) : null;
  logResponse(method, url, response.status, result);
  return result as T;
};
