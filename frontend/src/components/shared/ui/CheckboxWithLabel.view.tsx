import type { JSX } from 'preact';
import { Checkbox } from './Checkbox.view';

interface CheckboxWithLabelProps {
  id: string;
  checked: boolean;
  disabled?: boolean;
  children: string;
  onCheckedChange: (checked: boolean) => void;
}

export const CheckboxWithLabel = ({
  id,
  checked,
  disabled,
  children,
  onCheckedChange
}: CheckboxWithLabelProps): JSX.Element => (
  <label for={id} class='flex items-center gap-2 cursor-pointer'>
    <Checkbox id={id} checked={checked} disabled={disabled} onCheckedChange={onCheckedChange} />
    <span class='text-sm text-text-primary'>{children}</span>
  </label>
);
