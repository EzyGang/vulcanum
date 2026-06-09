import type { JSX } from 'preact';

interface HamburgerIconProps {
  open: boolean;
  class?: string;
}

export const HamburgerIcon = ({ open, class: classProp }: HamburgerIconProps): JSX.Element => (
  <svg
    width='18'
    height='18'
    viewBox='0 0 18 18'
    fill='none'
    stroke='currentColor'
    stroke-width='2'
    stroke-linecap='round'
    class={classProp}
  >
    <title>{open ? 'Close menu' : 'Open menu'}</title>
    {open ? <path d='M4.5 4.5L13.5 13.5M13.5 4.5L4.5 13.5' /> : <path d='M3 5H15M3 9H15M3 13H15' />}
  </svg>
);
