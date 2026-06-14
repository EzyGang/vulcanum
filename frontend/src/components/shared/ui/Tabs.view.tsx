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
    class={clsx('flex min-w-0 flex-col gap-6', classProp)}
  >
    <BaseTabs.List class='grid w-full grid-cols-2 gap-1 border-b border-border-base sm:flex sm:flex-nowrap'>
      {tabs.map((tab) => (
        <BaseTabs.Tab
          key={tab.value}
          value={tab.value}
          class='flex min-w-0 w-full items-center justify-center border-b-2 border-transparent bg-transparent px-2 py-3 text-center text-xs uppercase leading-tight tracking-wider transition-colors cursor-pointer text-text-muted hover:text-text-secondary data-[active]:border-accent data-[active]:text-text-primary sm:w-auto sm:px-4 sm:py-2 sm:text-sm sm:whitespace-nowrap'
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
  <BaseTabs.Panel value={value} class={clsx('flex flex-col animate-fade-in', classProp)}>
    {children}
  </BaseTabs.Panel>
);

export const Tabs = TabsRoot;
