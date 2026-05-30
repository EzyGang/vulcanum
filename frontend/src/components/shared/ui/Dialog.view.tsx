import { Dialog as BaseDialog } from '@base-ui/react/dialog';
import { clsx } from 'clsx';
import type { ComponentChildren } from 'preact';

interface DialogRootProps {
  children: ComponentChildren;
  defaultOpen?: boolean;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}

const DialogRoot = ({ children, ...props }: DialogRootProps) => (
  <BaseDialog.Root {...props}>{children}</BaseDialog.Root>
);

interface DialogTriggerProps {
  children: ComponentChildren;
  class?: string;
}

DialogRoot.Trigger = ({ children, class: classProp }: DialogTriggerProps) => (
  <BaseDialog.Trigger
    class={clsx(
      'cursor-pointer focus:outline-none focus-visible:ring-2 focus-visible:ring-border-focus',
      classProp
    )}
  >
    {children}
  </BaseDialog.Trigger>
);

interface DialogPortalProps {
  children: ComponentChildren;
}

DialogRoot.Portal = ({ children }: DialogPortalProps) => (
  <BaseDialog.Portal>{children}</BaseDialog.Portal>
);

interface DialogBackdropProps {
  class?: string;
}

DialogRoot.Backdrop = ({ class: classProp }: DialogBackdropProps) => (
  <BaseDialog.Backdrop
    class={clsx('fixed inset-0 bg-bg-page/80 backdrop-blur-sm transition-opacity', classProp)}
  />
);

interface DialogPopupProps {
  children: ComponentChildren;
  class?: string;
}

DialogRoot.Popup = ({ children, class: classProp }: DialogPopupProps) => (
  <BaseDialog.Popup
    class={clsx(
      'fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2',
      'bg-bg-card border border-border-base p-5 min-w-[320px]',
      'focus:outline-none',
      classProp
    )}
  >
    {children}
  </BaseDialog.Popup>
);

interface DialogTitleProps {
  children: ComponentChildren;
  class?: string;
}

DialogRoot.Title = ({ children, class: classProp }: DialogTitleProps) => (
  <BaseDialog.Title class={clsx('text-text-primary text-lg font-medium', classProp)}>
    {children}
  </BaseDialog.Title>
);

interface DialogDescriptionProps {
  children: ComponentChildren;
  class?: string;
}

DialogRoot.Description = ({ children, class: classProp }: DialogDescriptionProps) => (
  <BaseDialog.Description class={clsx('text-text-secondary text-sm', classProp)}>
    {children}
  </BaseDialog.Description>
);

interface DialogCloseProps {
  children: ComponentChildren;
  class?: string;
}

DialogRoot.Close = ({ children, class: classProp }: DialogCloseProps) => (
  <BaseDialog.Close
    class={clsx(
      'cursor-pointer focus:outline-none focus-visible:ring-2 focus-visible:ring-border-focus',
      classProp
    )}
  >
    {children}
  </BaseDialog.Close>
);

export const Dialog = DialogRoot;
