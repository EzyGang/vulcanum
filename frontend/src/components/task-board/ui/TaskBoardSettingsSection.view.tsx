import type { ComponentChildren, JSX } from 'preact';

interface TaskBoardSettingsSectionProps {
  title: string;
  description: string;
  children: ComponentChildren;
}

export const TaskBoardSettingsSection = ({
  title,
  description,
  children
}: TaskBoardSettingsSectionProps): JSX.Element => (
  <details open class='group border border-border-base bg-bg-panel p-4'>
    <summary class='flex cursor-pointer list-none items-start justify-between gap-4 text-xs font-medium uppercase tracking-wider text-text-muted outline-none transition-colors hover:text-text-primary focus-visible:ring-2 focus-visible:ring-border-focus [&::-webkit-details-marker]:hidden'>
      <span>{title}</span>
      <span class='text-sm leading-none transition-transform group-open:rotate-90'>›</span>
    </summary>
    <div class='mt-3 flex flex-col gap-4'>
      <p class='text-xs normal-case tracking-normal text-text-muted'>{description}</p>
      {children}
    </div>
  </details>
);
