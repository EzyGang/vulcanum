import type { JSX } from 'preact';
import { GitHubAppCardContainer } from '../../github/containers/GitHubAppCard.container';
import { ModelProvidersContainer } from '../../model-providers/containers/ModelProviders.container';
import { ProjectsContainer } from '../../projects/containers/Projects.container';
import { ProvidersContainer } from '../../providers/containers/Providers.container';
import { Tabs } from '../../shared/ui/Tabs.view';

interface SettingsViewProps {
  tabs: { value: string; label: string }[];
  activeTab: string;
  onTabChange: (value: string) => void;
}

export const SettingsView = ({ tabs, activeTab, onTabChange }: SettingsViewProps): JSX.Element => (
  <div class='flex flex-col gap-8 animate-fade-in'>
    <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>Settings</h2>

    <Tabs tabs={tabs} value={activeTab} onValueChange={onTabChange}>
      <Tabs.Panel value='providers'>
        <ProvidersContainer />
      </Tabs.Panel>
      <Tabs.Panel value='model-providers'>
        <ModelProvidersContainer />
      </Tabs.Panel>
      <Tabs.Panel value='projects'>
        <ProjectsContainer />
      </Tabs.Panel>
      <Tabs.Panel value='github'>
        <GitHubAppCardContainer />
      </Tabs.Panel>
    </Tabs>
  </div>
);
