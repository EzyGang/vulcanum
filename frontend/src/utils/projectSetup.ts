export interface ProjectSetupState {
  hasTaskTrackerProvider: boolean;
  hasModelProvider: boolean;
}

export const getProjectSetupMissingMessages = (state: ProjectSetupState): string[] => {
  const messages: string[] = [];

  if (!state.hasTaskTrackerProvider) {
    messages.push('Create at least one task tracker provider.');
  }

  if (!state.hasModelProvider) {
    messages.push('Connect at least one model provider.');
  }

  return messages;
};

export const isProjectSetupComplete = (state: ProjectSetupState): boolean =>
  getProjectSetupMissingMessages(state).length === 0;

export const getProjectSetupHelpText = (messages: string[]): string =>
  messages.length > 0 ? messages.join(' ') : '';
