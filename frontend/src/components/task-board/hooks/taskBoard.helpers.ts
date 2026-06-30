import type { ProjectConfig } from '../../../types/projects';
import type { TaskBoardColumn } from '../../../types/task-board';
import type { TaskBoardColumnRoles, TaskBoardHelpCard, TaskBoardSettingsFormState } from '../types';

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
