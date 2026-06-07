import type { Signal } from '@preact/signals';
import { createContext } from 'preact';
import { useContext } from 'preact/hooks';

export interface ProjectFormMetaContextValue {
  isEdit: boolean;
  projectLoading: boolean;
  submitting: Signal<boolean>;
  formError: Signal<string | null>;
  canShowLookup: boolean;
  canShowFields: boolean;
  onSubmit: (e: Event) => void;
  onCancel: () => void;
}

const ProjectFormMetaContext = createContext<ProjectFormMetaContextValue | null>(null);

export const ProjectFormMetaProvider = ProjectFormMetaContext.Provider;

export const useProjectFormMetaContext = (): ProjectFormMetaContextValue => {
  const ctx = useContext(ProjectFormMetaContext);
  if (!ctx) {
    throw new Error('useProjectFormMetaContext must be used inside a ProjectFormMetaProvider');
  }
  return ctx;
};
