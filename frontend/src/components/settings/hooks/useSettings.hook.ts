import { useSignal } from '@preact/signals';

const TABS = [
  { value: 'providers', label: 'Providers' },
  { value: 'projects', label: 'Projects' },
  { value: 'github', label: 'GitHub App' }
];

export const useSettings = () => {
  const activeTab = useSignal('providers');

  return {
    tabs: TABS,
    activeTab,
    setActiveTab: (value: string) => {
      activeTab.value = value;
    }
  };
};
