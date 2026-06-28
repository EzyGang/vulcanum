import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { useLocation } from 'wouter-preact';

const TABS = [
  { value: 'team-defaults', label: 'Agent defaults', group: 'Team' },
  { value: 'model-selection', label: 'Model selection', group: 'Team' },
  { value: 'providers', label: 'Task trackers', group: 'Integrations' },
  { value: 'model-providers', label: 'Model providers', group: 'Integrations' },
  { value: 'github', label: 'GitHub app', group: 'Integrations' }
];

const DEFAULT_TAB = 'team-defaults';

export const useSettings = () => {
  const [location] = useLocation();
  const activeTab = useSignal(getTabFromLocation(location));

  useEffect(() => {
    activeTab.value = getTabFromLocation(location);
  }, [location]);

  return {
    tabs: TABS,
    activeTab,
    setActiveTab: (value: string) => {
      activeTab.value = value;
    }
  };
};

const getTabFromLocation = (location: string): string => {
  const [, query = ''] = location.split('?');
  const tab = new URLSearchParams(query).get('tab');

  if (tab && TABS.some((item) => item.value === tab)) {
    return tab;
  }

  return DEFAULT_TAB;
};
