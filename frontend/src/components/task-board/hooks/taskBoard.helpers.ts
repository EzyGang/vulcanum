import type { ProjectConfig } from '../../../types/projects';
import type { TaskBoardColumn } from '../../../types/task-board';
import type {
  TaskBoardColumnPreferences,
  TaskBoardColumnRoles,
  TaskBoardHelpCard,
  TaskBoardSettingsFormState
} from '../types';

export const boardQueryKey = (providerId?: string, projectId?: string) => [
  'task-board',
  providerId ?? '',
  projectId ?? ''
];

export const projectConfigsQueryKey = ['projects'];
export const reposQueryKey = ['github-repos'];
export const taskBoardProjectsQueryKey = ['task-board-projects'];
export const COLUMN_PAGE_SIZE = 20;
export const LIFECYCLE_LABELS_HELP_CARD_ID: TaskBoardHelpCard = 'lifecycle-labels';

const HELP_CARD_STORAGE_KEY = 'vulcanum-task-board-dismissed-help-cards';
const COLUMN_PREFERENCES_STORAGE_KEY = 'vulcanum-task-board-column-preferences';
export const HELP_CARDS: { id: TaskBoardHelpCard; title: string; body: string }[] = [
  {
    id: 'proxy',
    title: 'Provider proxy',
    body: 'Tasks stay in the provider. Vulcanum mirrors the board and writes moves or new tickets back through the provider API.'
  },
  {
    id: 'roles',
    title: 'Column roles',
    body: 'Pickup, in-progress, done, and review roles tell workers where to pull work from and where completed work should land.'
  },
  {
    id: 'automation',
    title: 'Automation gate',
    body: 'Automation stays off until the board roles and runtime settings are mapped for this project.'
  }
];

const HELP_CARD_IDS = [...HELP_CARDS.map((card) => card.id), LIFECYCLE_LABELS_HELP_CARD_ID];

export const readDismissedHelpCards = (): TaskBoardHelpCard[] => {
  if (typeof localStorage === 'undefined') {
    return [];
  }

  try {
    const rawCards = JSON.parse(localStorage.getItem(HELP_CARD_STORAGE_KEY) ?? '[]');
    return Array.isArray(rawCards)
      ? rawCards.filter((card): card is TaskBoardHelpCard => HELP_CARD_IDS.includes(card))
      : [];
  } catch {
    return [];
  }
};

export const writeDismissedHelpCards = (cards: TaskBoardHelpCard[]): void => {
  if (typeof localStorage === 'undefined') {
    return;
  }

  localStorage.setItem(HELP_CARD_STORAGE_KEY, JSON.stringify(cards));
};

const columnPreferencesFromUnknown = (value: unknown): TaskBoardColumnPreferences => {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return { hiddenColumnSlugs: [], columnOrder: [] };
  }

  const candidate = value as Partial<Record<keyof TaskBoardColumnPreferences, unknown>>;
  const hiddenColumnSlugs = Array.isArray(candidate.hiddenColumnSlugs)
    ? candidate.hiddenColumnSlugs.filter((item): item is string => typeof item === 'string')
    : [];
  const columnOrder = Array.isArray(candidate.columnOrder)
    ? candidate.columnOrder.filter((item): item is string => typeof item === 'string')
    : [];

  return { hiddenColumnSlugs, columnOrder };
};

const readColumnPreferencesStore = (): Record<string, TaskBoardColumnPreferences> => {
  if (typeof localStorage === 'undefined') {
    return {};
  }

  try {
    const rawPreferences = JSON.parse(localStorage.getItem(COLUMN_PREFERENCES_STORAGE_KEY) ?? '{}');

    if (!rawPreferences || typeof rawPreferences !== 'object' || Array.isArray(rawPreferences)) {
      return {};
    }

    const store: Record<string, TaskBoardColumnPreferences> = {};
    for (const [boardKey, preferences] of Object.entries(rawPreferences)) {
      store[boardKey] = columnPreferencesFromUnknown(preferences);
    }

    return store;
  } catch {
    return {};
  }
};

export const readTaskBoardColumnPreferences = (
  boardKey: string | null
): TaskBoardColumnPreferences => {
  if (!boardKey) {
    return { hiddenColumnSlugs: [], columnOrder: [] };
  }

  return readColumnPreferencesStore()[boardKey] ?? { hiddenColumnSlugs: [], columnOrder: [] };
};

export const writeTaskBoardColumnPreferences = (
  boardKey: string | null,
  preferences: TaskBoardColumnPreferences
): void => {
  if (!boardKey || typeof localStorage === 'undefined') {
    return;
  }

  const store = readColumnPreferencesStore();
  if (preferences.hiddenColumnSlugs.length === 0 && preferences.columnOrder.length === 0) {
    delete store[boardKey];
  } else {
    store[boardKey] = preferences;
  }

  if (Object.keys(store).length === 0) {
    localStorage.removeItem(COLUMN_PREFERENCES_STORAGE_KEY);
    return;
  }

  localStorage.setItem(COLUMN_PREFERENCES_STORAGE_KEY, JSON.stringify(store));
};

const knownUniqueSlugs = (slugs: string[], knownSlugSet: ReadonlySet<string>): string[] => {
  const usedSlugs = new Set<string>();

  return slugs.filter((slug) => {
    if (!knownSlugSet.has(slug) || usedSlugs.has(slug)) {
      return false;
    }

    usedSlugs.add(slug);
    return true;
  });
};

export const normalizeTaskBoardColumnPreferences = (
  columns: TaskBoardColumn[],
  preferences: TaskBoardColumnPreferences
): TaskBoardColumnPreferences => {
  const naturalOrder = columns.map((column) => column.slug);
  const knownSlugSet = new Set(naturalOrder);
  const columnOrder = knownUniqueSlugs(preferences.columnOrder, knownSlugSet);
  const hiddenColumnSlugs = knownUniqueSlugs(preferences.hiddenColumnSlugs, knownSlugSet);

  for (const slug of naturalOrder) {
    if (!columnOrder.includes(slug)) {
      columnOrder.push(slug);
    }
  }

  return { hiddenColumnSlugs, columnOrder };
};

export const hasCustomTaskBoardColumnView = (
  columns: TaskBoardColumn[],
  preferences: TaskBoardColumnPreferences
): boolean => {
  const normalizedPreferences = normalizeTaskBoardColumnPreferences(columns, preferences);
  const naturalOrder = columns.map((column) => column.slug);

  return (
    normalizedPreferences.hiddenColumnSlugs.length > 0 ||
    normalizedPreferences.columnOrder.some((slug, index) => slug !== naturalOrder[index])
  );
};

export const firstColumnSlug = (columns: TaskBoardColumn[]): string => columns[0]?.slug ?? '';
const targetColumnSlug = (columns: TaskBoardColumn[]): string =>
  columns.find((column) => column.isFinal)?.slug ??
  columns[columns.length - 1]?.slug ??
  firstColumnSlug(columns);

const progressColumnSlug = (columns: TaskBoardColumn[]): string =>
  columns.find((column) => !column.isFinal && column.slug !== firstColumnSlug(columns))?.slug ??
  firstColumnSlug(columns);

export const matchingProjectConfig = (
  configs: ProjectConfig[],
  providerId?: string,
  externalProjectId?: string
): ProjectConfig | null =>
  configs.find(
    (config) => config.providerId === providerId && config.externalProjectId === externalProjectId
  ) ?? null;

export const columnRolesForProject = (
  config: ProjectConfig | null,
  columns: TaskBoardColumn[]
): TaskBoardColumnRoles => ({
  pickupColumn: config?.pickupColumn || firstColumnSlug(columns),
  progressColumn: config?.progressColumn || progressColumnSlug(columns),
  targetColumn: config?.targetColumn || targetColumnSlug(columns)
});

export const nullableText = (value: string): string | null => {
  const trimmed = value.trim();
  return trimmed.length > 0 ? value : null;
};

export const settingsFormFromConfig = (
  config: ProjectConfig | null
): TaskBoardSettingsFormState => ({
  promptTemplate: config?.promptTemplate ?? '',
  agentsMd: config?.agentsMd ?? '',
  reviewEnabled:
    config?.reviewEnabled === null || config?.reviewEnabled === undefined
      ? ''
      : config.reviewEnabled
        ? 'true'
        : 'false',
  reviewMaxTurns: config?.reviewMaxTurns?.toString() ?? '',
  reviewPromptTemplate: config?.reviewPromptTemplate ?? '',
  maxInProgressTasks: config?.maxInProgressTasks?.toString() ?? ''
});
