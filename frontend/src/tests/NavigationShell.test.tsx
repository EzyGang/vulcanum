import { fireEvent, render } from '@testing-library/preact';
import type { ComponentChildren } from 'preact';
import { describe, expect, it, vi } from 'vitest';

vi.mock('../components/shared/ui/Select.view', () => ({
  Select: ({
    items,
    value,
    onValueChange,
    placeholder,
    disabled
  }: {
    items: { value: string; label: string }[];
    value: string;
    onValueChange: (value: string) => void;
    placeholder?: string;
    disabled?: boolean;
  }) => (
    <select
      value={value}
      disabled={disabled}
      aria-label={placeholder}
      onInput={(event) => onValueChange((event.target as HTMLSelectElement).value)}
    >
      <option value=''>{placeholder}</option>
      {items.map((item) => (
        <option key={item.value} value={item.value}>
          {item.label}
        </option>
      ))}
    </select>
  )
}));

vi.mock('../components/app-shell/containers/ThemeToggle.container', () => ({
  ThemeToggleContainer: () => <button type='button'>Theme</button>
}));

vi.mock('../components/shared/ui/HamburgerIcon.view', () => ({
  HamburgerIcon: () => <span>Menu</span>
}));

import { NavigationShellView } from '../components/app-shell/ui/NavigationShell.view';

const makeProps = () => ({
  data: {
    navLinks: [
      { href: '/', label: 'Board' },
      { href: '/settings', label: 'Settings' }
    ],
    isActive: (href: string) => href === '/',
    mobileMenuOpen: false,
    selectedTeamId: 'team-1',
    teamOptions: [{ value: 'team-1', label: 'Core team' }],
    selectedProjectKey: 'provider-1/project-1',
    boardOptions: [
      { value: 'provider-1/project-1', label: 'Connected board' },
      { value: 'add:provider-1/project-2', label: 'Add Provider board · Core · Kaneo' }
    ],
    activatingProject: false
  },
  actions: {
    onLogout: vi.fn(),
    onNavigate: vi.fn(),
    onSelectTeam: vi.fn(),
    onSelectBoardOption: vi.fn(),
    onToggleMobileMenu: vi.fn()
  },
  children: (<main>Content</main>) as ComponentChildren
});

describe('NavigationShellView', () => {
  it('selects configured boards and activates available provider projects', () => {
    const props = makeProps();
    const { getByLabelText } = render(<NavigationShellView {...props} />);

    fireEvent.input(getByLabelText('Select or add board'), {
      target: { value: 'provider-1/project-1' }
    });
    fireEvent.input(getByLabelText('Select or add board'), {
      target: { value: 'add:provider-1/project-2' }
    });

    expect(props.actions.onSelectBoardOption).toHaveBeenCalledWith('provider-1/project-1');
    expect(props.actions.onSelectBoardOption).toHaveBeenCalledWith('add:provider-1/project-2');
  });
});
