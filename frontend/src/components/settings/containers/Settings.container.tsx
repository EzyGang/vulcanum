import type { JSX } from 'preact';
import { selectedTeamId } from '../../../stores/auth.store';
import { GitHubAppCardContainer } from '../../github/containers/GitHubAppCard.container';
import { ModelProvidersContainer } from '../../model-providers/containers/ModelProviders.container';
import { ProvidersContainer } from '../../providers/containers/Providers.container';
import { TeamDefaultsContainer } from '../../team-defaults/containers/TeamDefaults.container';
import { useSettings } from '../hooks/useSettings.hook';
import { SettingsView } from '../ui/Settings.view';

export const SettingsContainer = (): JSX.Element => {
  const { tabs, activeTab, setActiveTab } = useSettings();

  return (
    <SettingsView
      tabs={tabs}
      activeTab={activeTab.value}
      onTabChange={setActiveTab}
      panels={{
        teamDefaults: <TeamDefaultsContainer teamId={selectedTeamId.value} section='defaults' />,
        modelSelection: <TeamDefaultsContainer teamId={selectedTeamId.value} section='models' />,
        providers: <ProvidersContainer />,
        modelProviders: <ModelProvidersContainer />,
        github: <GitHubAppCardContainer />
      }}
    />
  );
};
