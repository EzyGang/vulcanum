import type { Signal } from '@preact/signals';
import { createContext } from 'preact';
import { useContext } from 'preact/hooks';
import type { IntegrationProvider } from '../../../types/projects';

export interface ProjectFormProviderContextValue {
  providers: IntegrationProvider[];
  providerId: Signal<string>;
  showProviderForm: Signal<boolean>;
  newProviderName: Signal<string>;
  newProviderUrl: Signal<string>;
  newProviderKey: Signal<string>;
  newProviderType: Signal<string>;
  providerFormError: Signal<string | null>;
  providerSubmitting: Signal<boolean>;
  onProviderChange: (id: string) => void;
  onShowProviderForm: () => void;
  onCancelProviderForm: () => void;
  onCreateProvider: (e: Event) => void;
  onNewProviderNameChange: (value: string) => void;
  onNewProviderUrlChange: (value: string) => void;
  onNewProviderKeyChange: (value: string) => void;
  onNewProviderTypeChange: (value: string) => void;
}

const ProjectFormProviderContext = createContext<ProjectFormProviderContextValue | null>(null);

export const ProjectFormProviderContextProvider = ProjectFormProviderContext.Provider;

export const useProjectFormProviderContext = (): ProjectFormProviderContextValue => {
  const ctx = useContext(ProjectFormProviderContext);
  if (!ctx) {
    throw new Error(
      'useProjectFormProviderContext must be used inside a ProjectFormProviderContextProvider'
    );
  }
  return ctx;
};
