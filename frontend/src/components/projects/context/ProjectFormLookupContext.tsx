import type { Signal } from '@preact/signals';
import { createContext } from 'preact';
import { useContext } from 'preact/hooks';

export interface ProjectFormLookupContextValue {
  externalProjectId: Signal<string>;
  lookupProjectName: Signal<string>;
  lookupError: Signal<string | null>;
  onLookup: () => void;
  onProjectIdChange: (value: string) => void;
}

const ProjectFormLookupContext = createContext<ProjectFormLookupContextValue | null>(null);

export const ProjectFormLookupProvider = ProjectFormLookupContext.Provider;

export const useProjectFormLookupContext = (): ProjectFormLookupContextValue => {
  const ctx = useContext(ProjectFormLookupContext);
  if (!ctx) {
    throw new Error('useProjectFormLookupContext must be used inside a ProjectFormLookupProvider');
  }
  return ctx;
};
