import type { Signal } from '@preact/signals';
import { createContext } from 'preact';
import { useContext } from 'preact/hooks';
import type { ColumnInfo, IntegrationProvider } from '../../../types/projects';

export interface ProjectFormContextValue {
  data: {
    isEdit: boolean;
    providers: IntegrationProvider[];
    providerId: Signal<string>;
    externalProjectId: Signal<string>;
    enabled: Signal<boolean>;
    pickupColumn: Signal<string>;
    progressColumn: Signal<string>;
    targetColumn: Signal<string>;
    promptTemplate: Signal<string>;
    repoUrl: Signal<string>;
    agentsMd: Signal<string>;
    opencodeConfig: Signal<string>;
    repos: string[];
    reposLoading: boolean;
    columns: Signal<ColumnInfo[]>;
    columnsLoading: Signal<boolean>;
    lookupProjectName: Signal<string>;
    lookupError: Signal<string | null>;
    lookedUp: Signal<boolean>;
    canShowLookup: boolean;
    canShowFields: boolean;
    showProviderForm: Signal<boolean>;
    newProviderName: Signal<string>;
    newProviderUrl: Signal<string>;
    newProviderKey: Signal<string>;
    newProviderType: Signal<string>;
    providerFormError: Signal<string | null>;
    providerSubmitting: Signal<boolean>;
  };
  status: {
    submitting: Signal<boolean>;
    formError: Signal<string | null>;
    projectLoading: boolean;
  };
  actions: {
    onLookup: () => void;
    onSubmit: (e: Event) => void;
    onCancel: () => void;
    onCreateProvider: (e: Event) => void;
    onShowProviderForm: () => void;
    onCancelProviderForm: () => void;
    onProviderChange: (id: string) => void;
    onProjectIdChange: (id: string) => void;
    onEnabledChange: (checked: boolean) => void;
    onPromptTemplateChange: (value: string) => void;
    onRepoUrlChange: (value: string) => void;
    onAgentsMdChange: (value: string) => void;
    onOpencodeConfigChange: (value: string) => void;
    onPickupColumnChange: (value: string) => void;
    onProgressColumnChange: (value: string) => void;
    onTargetColumnChange: (value: string) => void;
    onNewProviderNameChange: (value: string) => void;
    onNewProviderUrlChange: (value: string) => void;
    onNewProviderKeyChange: (value: string) => void;
    onNewProviderTypeChange: (value: string) => void;
  };
}

const ProjectFormContext = createContext<ProjectFormContextValue | null>(null);

export const ProjectFormProvider = ProjectFormContext.Provider;

export const useProjectFormContext = (): ProjectFormContextValue => {
  const ctx = useContext(ProjectFormContext);
  if (!ctx) {
    throw new Error('useProjectFormContext must be used inside a ProjectFormProvider');
  }
  return ctx;
};
