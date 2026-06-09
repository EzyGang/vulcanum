import type { JSX } from 'preact';
import { useSettings } from '../hooks/useSettings.hook';
import { SettingsView } from '../ui/Settings.view';

export const SettingsContainer = (): JSX.Element => {
  const { tabs, activeTab, setActiveTab } = useSettings();

  return <SettingsView tabs={tabs} activeTab={activeTab.value} onTabChange={setActiveTab} />;
};
