import { Accordion as BaseAccordion } from '@base-ui/react/accordion';
import { clsx } from 'clsx';
import type { ComponentChildren, JSX } from 'preact';

interface AccordionRootProps {
  children: ComponentChildren;
  class?: string;
  defaultValue?: string[];
  value?: string[];
  onValueChange?: (value: string[]) => void;
  multiple?: boolean;
  keepMounted?: boolean;
}

interface AccordionItemProps {
  value: string;
  children: ComponentChildren;
  class?: string;
  disabled?: boolean;
}

interface AccordionTriggerProps {
  children: ComponentChildren;
  class?: string;
  indicator?: ComponentChildren;
}

interface AccordionPanelProps {
  children: ComponentChildren;
  class?: string;
}

const AccordionRoot = ({
  children,
  class: classProp,
  defaultValue,
  value,
  onValueChange,
  multiple = true,
  keepMounted = true
}: AccordionRootProps): JSX.Element => (
  <BaseAccordion.Root
    defaultValue={defaultValue}
    value={value}
    onValueChange={onValueChange}
    multiple={multiple}
    keepMounted={keepMounted}
    class={clsx('flex flex-col gap-3', classProp)}
  >
    {children}
  </BaseAccordion.Root>
);

AccordionRoot.Item = ({
  value,
  children,
  class: classProp,
  disabled
}: AccordionItemProps): JSX.Element => (
  <BaseAccordion.Item
    value={value}
    disabled={disabled}
    class={clsx('group border border-border-base bg-bg-panel', classProp)}
  >
    {children}
  </BaseAccordion.Item>
);

AccordionRoot.Trigger = ({
  children,
  class: classProp,
  indicator = '›'
}: AccordionTriggerProps): JSX.Element => (
  <BaseAccordion.Header>
    <BaseAccordion.Trigger
      class={clsx(
        'flex w-full cursor-pointer list-none items-start justify-between gap-4 text-left outline-none',
        'transition-colors hover:text-text-primary focus-visible:ring-2 focus-visible:ring-border-focus',
        classProp
      )}
    >
      <span class='min-w-0'>{children}</span>
      <span class='shrink-0 text-sm leading-none transition-transform group-data-open:rotate-90'>
        {indicator}
      </span>
    </BaseAccordion.Trigger>
  </BaseAccordion.Header>
);

AccordionRoot.Panel = ({ children, class: classProp }: AccordionPanelProps): JSX.Element => (
  <BaseAccordion.Panel class={clsx('`data-closed:hidden', classProp)}>
    {children}
  </BaseAccordion.Panel>
);

export const Accordion = AccordionRoot;
