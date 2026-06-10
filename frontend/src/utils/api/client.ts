import { accessToken, STORAGE_KEY, selectedTeamId } from '../../stores/auth.store';
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

const SENSITIVE_FIELDS = new Set(['password', 'token', 'secret', 'api_key']);

const sanitizeLogBody = (body: unknown): unknown => {
  if (body == null || typeof body !== 'object') return body;
  if (Array.isArray(body)) return body.map(sanitizeLogBody);

  const sanitized: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(body as Record<string, unknown>)) {
    sanitized[key] = SENSITIVE_FIELDS.has(key) ? '***' : sanitizeLogBody(value);
  }
  return sanitized;
};

const logRequest = (method: string, url: string, body?: unknown) => {
  if (!isDevelopment || import.meta.env.VITE_DISABLE_DEV_LOGGING) return;
  console.group(`API Request: ${method} ${url}`);
  if (body) console.log('Request Body:', sanitizeLogBody(body));
  console.groupEnd();
};

const logResponse = (method: string, url: string, status: number, data: unknown) => {
  if (!isDevelopment || import.meta.env.VITE_DISABLE_DEV_LOGGING) return;
  console.group(`API Response: ${method} ${url}`);
  console.log('Status:', status);
  console.log('Response:', sanitizeLogBody(data));
  console.groupEnd();
};

const logError = (method: string, url: string, status: number, error: unknown) => {
  if (!isDevelopment || import.meta.env.VITE_DISABLE_DEV_LOGGING) return;
  console.group(`API Error: ${method} ${url}`);
  console.log('Status:', status);
  console.log('Error:', error);
  console.groupEnd();
};

export const fetchApi = async <T>(path: string, options: ApiFetchOptions = {}): Promise<T> => {
  const { body, params, ...init } = options;
  const method = (init.method || 'GET').toUpperCase();
  const url = buildUrl(path, method === 'GET' ? params : undefined);

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

  const requestBody = body != null ? JSON.stringify(snakeKeys(body)) : undefined;

  logRequest(method, url, body);

  const response = await fetch(url, {
    ...init,
    method,
    headers,
    body: requestBody
  });

  const isJson = response.headers.get('content-type')?.includes('application/json');
  const data = isJson ? await response.json() : null;

  if (!response.ok) {
    if (response.status === 401) {
      accessToken.value = null;
      localStorage.removeItem(STORAGE_KEY);
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
