import type { JSX } from 'preact';
import { SettingsContainer } from '../components/settings/containers/Settings.container';
import { PageLayout } from '../components/shared/ui/PageLayout.view';

export const Settings = (): JSX.Element => (
  <PageLayout maxWidth='settings'>
    <SettingsContainer />
  </PageLayout>
);
