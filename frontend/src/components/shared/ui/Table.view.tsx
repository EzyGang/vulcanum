import { clsx } from 'clsx';
import type { ComponentChildren, JSX } from 'preact';

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
}

const Row = ({ children, class: classProp }: RowProps): JSX.Element => (
  <tr class={clsx('border-b border-border-base', classProp)}>{children}</tr>
);

interface CellProps {
  children: ComponentChildren;
  class?: string;
}

const Cell = ({ children, class: classProp }: CellProps): JSX.Element => (
  <td class={clsx('px-5 py-3', classProp)}>{children}</td>
);

export const Table = Object.assign(TableRoot, { Head, HeadCell, Body, Row, Cell });
