import type { JSX } from 'preact';
import { Tabs } from '../../shared/ui/Tabs.view';

interface SettingsViewProps {
  tabs: { value: string; label: string }[];
  activeTab: string;
  onTabChange: (value: string) => void;
  panels: {
    teamDefaults: JSX.Element;
    providers: JSX.Element;
    modelProviders: JSX.Element;
    github: JSX.Element;
  };
}

export const SettingsView = ({
  tabs,
  activeTab,
  onTabChange,
  panels
}: SettingsViewProps): JSX.Element => (
  <div class='flex flex-col gap-8 animate-fade-in'>
    <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>Settings</h2>

    <Tabs tabs={tabs} value={activeTab} onValueChange={onTabChange}>
      <Tabs.Panel value='team-defaults'>{panels.teamDefaults}</Tabs.Panel>
      <Tabs.Panel value='providers'>{panels.providers}</Tabs.Panel>
      <Tabs.Panel value='model-providers'>{panels.modelProviders}</Tabs.Panel>
      <Tabs.Panel value='github'>{panels.github}</Tabs.Panel>
    </Tabs>
  </div>
);
