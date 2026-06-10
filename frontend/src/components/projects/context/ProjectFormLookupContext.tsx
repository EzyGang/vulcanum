import type { Signal } from '@preact/signals';
import { createContext } from 'preact';
import { useContext } from 'preact/hooks';

interface SelectOption {
  value: string;
  label: string;
}

export interface ProjectFormLookupContextValue {
  externalProjectId: Signal<string>;
  lookupProjectName: Signal<string>;
  lookupError: Signal<string | null>;
  lookedUp: Signal<boolean>;
  workspaceOptions: Signal<SelectOption[]>;
  workspaceId: Signal<string>;
  projectOptions: Signal<SelectOption[]>;
  workspacesLoading: Signal<boolean>;
  projectsLoading: Signal<boolean>;
  workspaceSelectDisabled: Signal<boolean>;
  projectSelectDisabled: Signal<boolean>;
  onLookup: () => void;
  onProjectIdChange: (value: string) => void;
  onWorkspaceChange: (value: string) => void;
  onProjectSelectById: (id: string) => void;
  fetchProjects: (workspaceId: string) => void;
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
