import { Tabs as BaseTabs } from '@base-ui/react/tabs';
import { clsx } from 'clsx';
import type { ComponentChildren } from 'preact';

interface TabItem {
  value: string;
  label: string;
}

interface TabsRootProps {
  tabs: TabItem[];
  value: string;
  onValueChange: (value: string) => void;
  children: ComponentChildren;
  class?: string;
}

const TabsRoot = ({ tabs, value, onValueChange, children, class: classProp }: TabsRootProps) => (
  <BaseTabs.Root
    value={value}
    onValueChange={onValueChange}
    className={clsx('flex flex-col gap-6', classProp)}
  >
    <BaseTabs.List className='flex gap-1 border-b border-border-base'>
      {tabs.map((tab) => (
        <BaseTabs.Tab
          key={tab.value}
          value={tab.value}
          className='px-4 py-2 text-sm uppercase tracking-wider transition-colors border-b-2 border-transparent cursor-pointer bg-transparent text-text-muted hover:text-text-secondary data-[active]:text-text-primary data-[active]:border-accent'
        >
          {tab.label}
        </BaseTabs.Tab>
      ))}
    </BaseTabs.List>
    {children}
  </BaseTabs.Root>
);

interface TabsPanelProps {
  value: string;
  children: ComponentChildren;
  class?: string;
}

TabsRoot.Panel = ({ value, children, class: classProp }: TabsPanelProps) => (
  <BaseTabs.Panel value={value} className={clsx('flex flex-col animate-fade-in', classProp)}>
    {children}
  </BaseTabs.Panel>
);

export const Tabs = TabsRoot;
