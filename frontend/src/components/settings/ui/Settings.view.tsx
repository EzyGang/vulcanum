import { IconBrain, IconBrandGithub, IconCpu, IconSettings, IconTicket } from '@tabler/icons-react';
import { clsx } from 'clsx';
import type { JSX } from 'preact';

interface SettingsTab {
  value: string;
  label: string;
  group: string;
}

interface SettingsViewProps {
  tabs: SettingsTab[];
  activeTab: string;
  onTabChange: (value: string) => void;
  panels: {
    teamDefaults: JSX.Element;
    modelSelection: JSX.Element;
    providers: JSX.Element;
    modelProviders: JSX.Element;
    github: JSX.Element;
  };
}

const TAB_ICONS: Record<string, typeof IconSettings> = {
  'team-defaults': IconSettings,
  'model-selection': IconBrain,
  providers: IconTicket,
  'model-providers': IconCpu,
  github: IconBrandGithub
};

const panelForTab = (activeTab: string, panels: SettingsViewProps['panels']): JSX.Element => {
  switch (activeTab) {
    case 'model-selection':
      return panels.modelSelection;
    case 'providers':
      return panels.providers;
    case 'model-providers':
      return panels.modelProviders;
    case 'github':
      return panels.github;
    default:
      return panels.teamDefaults;
  }
};

export const SettingsView = ({
  tabs,
  activeTab,
  onTabChange,
  panels
}: SettingsViewProps): JSX.Element => {
  const groups = [...new Set(tabs.map((tab) => tab.group))];

  return (
    <div class='flex min-h-[calc(100vh-9rem)] flex-col gap-6 animate-fade-in'>
      <div class='flex flex-col gap-2'>
        <span class='text-xs font-medium uppercase tracking-wider text-accent'>Control plane</span>
        <h2 class='text-2xl font-semibold text-text-primary'>Settings</h2>
        <p class='max-w-2xl text-sm leading-relaxed text-text-muted'>
          Configure team defaults, runtime model choices, and external integrations from one place.
        </p>
      </div>

      <div class='grid flex-1 gap-6 lg:grid-cols-[18rem_minmax(0,1fr)]'>
        <aside class='flex flex-col gap-5 border border-border-base bg-bg-card p-4'>
          <div class='flex flex-col gap-1'>
            <span class='text-xs font-medium uppercase tracking-wider text-text-muted'>
              Settings index
            </span>
            <span class='text-sm text-text-secondary'>Choose a category to edit.</span>
          </div>

          <nav class='flex flex-col gap-5' aria-label='Settings categories'>
            {groups.map((group) => (
              <div key={group} class='flex flex-col gap-2'>
                <span class='text-xs uppercase tracking-wider text-text-muted'>{group}</span>
                <div class='flex flex-col gap-1'>
                  {tabs
                    .filter((tab) => tab.group === group)
                    .map((tab) => {
                      const Icon = TAB_ICONS[tab.value] ?? IconSettings;
                      const active = tab.value === activeTab;

                      return (
                        <button
                          key={tab.value}
                          id={`settings-tab-${tab.value}`}
                          type='button'
                          aria-current={active ? 'page' : undefined}
                          onClick={() => onTabChange(tab.value)}
                          class={clsx(
                            'flex min-h-10 w-full cursor-pointer items-center gap-3 border px-3 py-2 text-left text-sm transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-border-focus',
                            active
                              ? 'border-border-focus bg-bg-active text-text-primary'
                              : 'border-transparent text-text-muted hover:border-border-base hover:bg-bg-hover hover:text-text-primary'
                          )}
                        >
                          <Icon size={16} stroke={1.75} aria-hidden='true' />
                          <span class='min-w-0 truncate'>{tab.label}</span>
                        </button>
                      );
                    })}
                </div>
              </div>
            ))}
          </nav>
        </aside>

        <main
          aria-labelledby={`settings-tab-${activeTab}`}
          class='min-w-0 border border-border-base bg-bg-page p-5'
        >
          {panelForTab(activeTab, panels)}
        </main>
      </div>
    </div>
  );
};
