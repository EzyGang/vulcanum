import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { useLocation } from 'wouter-preact';

const TABS = [
  { value: 'providers', label: 'Task Tracker Providers' },
  { value: 'model-providers', label: 'Model Providers' },
  { value: 'projects', label: 'Projects' },
  { value: 'github', label: 'GitHub App' }
];

const DEFAULT_TAB = 'providers';

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
