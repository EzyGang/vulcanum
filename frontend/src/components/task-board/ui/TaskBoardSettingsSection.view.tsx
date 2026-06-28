import type { ComponentChildren, JSX } from 'preact';
import { Accordion } from '../../shared/ui/Accordion.view';

interface TaskBoardSettingsSectionProps {
  title: string;
  description: string;
  children: ComponentChildren;
  hasOverrides?: boolean;
}

export const TaskBoardSettingsSection = ({
  title,
  description,
  children,
  hasOverrides = false
}: TaskBoardSettingsSectionProps): JSX.Element => (
  <Accordion class='gap-0'>
    <Accordion.Item value={title}>
      <Accordion.Trigger class='p-4 text-xs font-medium uppercase tracking-wider text-text-muted'>
        <span class='flex min-w-0 items-center gap-2'>
          <span>{title}</span>
          {hasOverrides && (
            <span class='border border-accent/60 px-1.5 py-0.5 text-[9px] text-accent'>Set</span>
          )}
        </span>
      </Accordion.Trigger>
      <Accordion.Panel class='flex flex-col gap-4 px-4 pt-3 pb-4'>
        <p class='text-xs normal-case tracking-normal text-text-muted'>{description}</p>
        {children}
      </Accordion.Panel>
    </Accordion.Item>
  </Accordion>
);
