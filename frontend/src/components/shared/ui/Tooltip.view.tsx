import { Tooltip as BaseTooltip } from '@base-ui/react/tooltip';
import { clsx } from 'clsx';
import type { ComponentChildren } from 'preact';

interface TooltipRootProps {
  children: ComponentChildren;
  delay?: number;
}

const TooltipRoot = ({ children, delay = 350 }: TooltipRootProps) => (
  <BaseTooltip.Provider delay={delay}>
    <BaseTooltip.Root>{children}</BaseTooltip.Root>
  </BaseTooltip.Provider>
);

interface TooltipTriggerProps {
  children: ComponentChildren;
  class?: string;
}

TooltipRoot.Trigger = ({ children, class: classProp }: TooltipTriggerProps) => (
  <BaseTooltip.Trigger
    class={clsx(
      'cursor-help focus:outline-none focus-visible:ring-2 focus-visible:ring-border-focus',
      classProp
    )}
  >
    {children}
  </BaseTooltip.Trigger>
);

interface TooltipPopupProps {
  children: ComponentChildren;
  class?: string;
}

TooltipRoot.Popup = ({ children, class: classProp }: TooltipPopupProps) => (
  <BaseTooltip.Portal>
    <BaseTooltip.Positioner sideOffset={8}>
      <BaseTooltip.Popup
        class={clsx(
          'z-50 max-w-80 border border-border-base bg-bg-active px-3 py-2 text-xs text-text-secondary',
          'shadow-none outline-none',
          classProp
        )}
      >
        {children}
      </BaseTooltip.Popup>
    </BaseTooltip.Positioner>
  </BaseTooltip.Portal>
);

export const Tooltip = TooltipRoot;
