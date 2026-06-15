import type { Signal } from '@preact/signals';
import { createContext } from 'preact';
import { useContext } from 'preact/hooks';
import type { CatalogProvider, ModelProviderConfig } from '../../../types/modelProviders';
import type { ColumnInfo } from '../../../types/projects';

interface SelectOption {
  value: string;
  label: string;
}

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
  repoFullNames: Signal<string[]>;
  agentsMd: Signal<string>;
  primaryModelProviderKey: Signal<string>;
  primaryModelId: Signal<string>;
  smallModelProviderKey: Signal<string>;
  smallModelId: Signal<string>;
  modelProviders: ModelProviderConfig[];
  catalogProviders: CatalogProvider[];
  connectedProviderItems: SelectOption[];
  primaryModelItems: SelectOption[];
  smallModelItems: SelectOption[];
  repoItems: RepoItem[];
  reposLoading: boolean;
  overridesOpen: Signal<boolean>;
  overridesToggleLabel: string;
  onEnabledChange: (checked: boolean) => void;
  onPickupColumnChange: (value: string) => void;
  onProgressColumnChange: (value: string) => void;
  onTargetColumnChange: (value: string) => void;
  onPromptTemplateInput: (event: Event) => void;
  onPromptTemplateChange: (value: string) => void;
  onAgentsMdInput: (event: Event) => void;
  onAgentsMdChange: (value: string) => void;
  onToggleOverrides: () => void;
  onPrimaryModelProviderChange: (value: string) => void;
  onPrimaryModelChange: (value: string) => void;
  onSmallModelProviderChange: (value: string) => void;
  onSmallModelChange: (value: string) => void;
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
