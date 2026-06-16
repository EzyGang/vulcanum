import type { Signal } from '@preact/signals';
import { createContext } from 'preact';
import { useContext } from 'preact/hooks';
import type { CatalogProvider, ModelProviderConfig } from '../../../types/modelProviders';
import type { ColumnInfo } from '../../../types/projects';
import type { SelectOption } from '../../../types/shared';

interface RepoItem {
  fullName: string;
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
}

export interface ProjectFormFieldsContextValue {
  enabled: Signal<boolean>;
  pickupColumn: Signal<string>;
  progressColumn: Signal<string>;
  targetColumn: Signal<string>;
  columns: Signal<ColumnInfo[]>;
  columnsLoading: Signal<boolean>;
  promptTemplate: Signal<string>;
  promptTemplateOverride: Signal<boolean>;
  repoFullNames: Signal<string[]>;
  agentsMd: Signal<string>;
  agentsMdOverride: Signal<boolean>;
  primaryModelProviderKey: Signal<string>;
  primaryModelProviderOverride: Signal<boolean>;
  primaryModelId: Signal<string>;
  primaryModelIdOverride: Signal<boolean>;
  smallModelProviderKey: Signal<string>;
  smallModelProviderOverride: Signal<boolean>;
  smallModelId: Signal<string>;
  smallModelIdOverride: Signal<boolean>;
  modelProviders: ModelProviderConfig[];
  catalogProviders: CatalogProvider[];
  connectedProviderItems: SelectOption[];
  primaryModelItems: SelectOption[];
  smallModelItems: SelectOption[];
  repoItems: RepoItem[];
  reposLoading: boolean;
  overridesOpen: Signal<boolean>;
  hasOverrides: boolean;
  overridesToggleLabel: string;
  onEnabledChange: (checked: boolean) => void;
  onPickupColumnChange: (value: string) => void;
  onProgressColumnChange: (value: string) => void;
  onTargetColumnChange: (value: string) => void;
  onPromptTemplateInput: (event: Event) => void;
  onPromptTemplateChange: (value: string) => void;
  onResetPromptTemplateOverride: () => void;
  onAgentsMdInput: (event: Event) => void;
  onAgentsMdChange: (value: string) => void;
  onResetAgentsMdOverride: () => void;
  onToggleOverrides: () => void;
  onPrimaryModelProviderChange: (value: string) => void;
  onResetPrimaryModelProviderOverride: () => void;
  onPrimaryModelChange: (value: string) => void;
  onResetPrimaryModelOverride: () => void;
  onSmallModelProviderChange: (value: string) => void;
  onResetSmallModelProviderOverride: () => void;
  onSmallModelChange: (value: string) => void;
  onResetSmallModelOverride: () => void;
}

const ProjectFormFieldsContext = createContext<ProjectFormFieldsContextValue | null>(null);

export const ProjectFormFieldsProvider = ProjectFormFieldsContext.Provider;

export const useProjectFormFieldsContext = (): ProjectFormFieldsContextValue => {
  const ctx = useContext(ProjectFormFieldsContext);
  if (!ctx) {
    throw new Error('useProjectFormFieldsContext must be used inside a ProjectFormFieldsProvider');
  }
  return ctx;
};
