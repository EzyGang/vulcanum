import { useSignal } from '@preact/signals';
import { useEffect } from 'preact/hooks';
import { useLocation } from 'wouter-preact';

const TABS = [
  { value: 'providers', label: 'Providers' },
  { value: 'projects', label: 'Projects' },
  { value: 'teams', label: 'Teams' },
  { value: 'github', label: 'GitHub App' }
];

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
  return new URLSearchParams(query).get('tab') ?? 'providers';
};
