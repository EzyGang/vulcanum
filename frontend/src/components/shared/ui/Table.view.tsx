import { clsx } from 'clsx';
import type { ComponentChildren, HTMLAttributes, JSX, MouseEventHandler } from 'preact';

interface TableProps {
  children: ComponentChildren;
  class?: string;
}

const TableRoot = ({ children, class: classProp }: TableProps): JSX.Element => (
  <div class='overflow-x-auto'>
    <table class={clsx('w-full border-collapse', classProp)}>{children}</table>
  </div>
);

interface TableSectionProps {
  children: ComponentChildren;
}

const Head = ({ children }: TableSectionProps): JSX.Element => (
  <thead>
    <tr class='border-b border-border-base'>{children}</tr>
  </thead>
);

interface HeadCellProps {
  children: ComponentChildren;
  class?: string;
}

const HeadCell = ({ children, class: classProp }: HeadCellProps): JSX.Element => (
  <th
    class={clsx('text-text-muted text-xs uppercase tracking-wider text-left px-5 py-3', classProp)}
  >
    {children}
  </th>
);

interface BodyProps {
  children: ComponentChildren;
}

const Body = ({ children }: BodyProps): JSX.Element => <tbody>{children}</tbody>;

interface RowProps {
  children: ComponentChildren;
  class?: string;
  onClick?: MouseEventHandler<HTMLTableRowElement>;
  onKeyDown?: HTMLAttributes<HTMLTableRowElement>['onKeyDown'];
  role?: HTMLAttributes<HTMLTableRowElement>['role'];
  tabIndex?: number;
}

const Row = ({
  children,
  class: classProp,
  onClick,
  onKeyDown,
  role,
  tabIndex
}: RowProps): JSX.Element => (
  <tr
    class={clsx('border-b border-border-base transition-colors hover:bg-bg-hover', classProp)}
    onClick={onClick}
    onKeyDown={onKeyDown}
    role={role}
    tabIndex={tabIndex}
  >
    {children}
  </tr>
);

interface CellProps {
  children: ComponentChildren;
  class?: string;
  colSpan?: number;
  paddingClass?: string;
  onClick?: JSX.MouseEventHandler<HTMLTableCellElement>;
}

const Cell = ({
  children,
  class: classProp,
  colSpan,
  paddingClass = 'px-5 py-3',
  onClick
}: CellProps): JSX.Element => (
  <td class={clsx(paddingClass, classProp)} colSpan={colSpan} onClick={onClick}>
    {children}
  </td>
);

export const Table = Object.assign(TableRoot, { Head, HeadCell, Body, Row, Cell });
