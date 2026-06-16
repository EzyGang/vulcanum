import { fetchApi } from './client';

export const get = async <T>(
  url: string,
  params?: Record<string, string | number | boolean>
): Promise<T> => {
  return fetchApi<T>(url, { method: 'GET', params });
};

export const post = async <T>(url: string, body?: unknown): Promise<T> => {
  return fetchApi<T>(url, { method: 'POST', body });
};

export const patch = async <T>(url: string, body?: unknown): Promise<T> => {
  return fetchApi<T>(url, { method: 'PATCH', body });
};

export const del = async <T>(url: string): Promise<T> => {
  return fetchApi<T>(url, { method: 'DELETE' });
};
