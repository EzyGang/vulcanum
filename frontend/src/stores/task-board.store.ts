import { signal } from '@preact/signals';

const STORAGE_KEY = 'vulcanum-task-board-project';

export const selectedTaskProjectKey = signal<string | null>(localStorage.getItem(STORAGE_KEY));

export const buildTaskProjectKey = (providerId: string, externalProjectId: string): string =>
  `${providerId}:${externalProjectId}`;

export const parseTaskProjectKey = (
  key: string | null
): { providerId: string; externalProjectId: string } | null => {
  if (!key) return null;

  const separator = key.indexOf(':');
  if (separator <= 0 || separator === key.length - 1) return null;

  return {
    providerId: key.slice(0, separator),
    externalProjectId: key.slice(separator + 1)
  };
};

export const setSelectedTaskProjectKey = (key: string | null): void => {
  selectedTaskProjectKey.value = key;
  if (key) {
    localStorage.setItem(STORAGE_KEY, key);
    return;
  }

  localStorage.removeItem(STORAGE_KEY);
};
