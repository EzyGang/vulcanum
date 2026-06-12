import type { ComponentChildren, JSX } from 'preact';
import { Route, Switch, useParams } from 'wouter-preact';
import { NavigationShellContainer } from '../components/app-shell/containers/NavigationShell.container';
import { Dashboard } from '../pages/Dashboard';
import { InviteAccept } from '../pages/InviteAccept';
import { Login } from '../pages/Login';
import { ProjectsFormPage } from '../pages/ProjectsForm';
import { Runs } from '../pages/Runs';
import { Settings } from '../pages/Settings';
import { Teams } from '../pages/Teams';
import { Workers } from '../pages/Workers';
import { ProtectedRoute } from './ProtectedRoute';

const AuthenticatedLayout = ({ children }: { children: ComponentChildren }): JSX.Element => (
  <ProtectedRoute>
    <NavigationShellContainer>{children}</NavigationShellContainer>
  </ProtectedRoute>
);

const ProjectsEditRoute = () => {
  const params = useParams<{ id?: string }>();
  return <ProjectsFormPage projectId={params.id} />;
};

const InviteAcceptRoute = () => {
  const params = useParams<{ token: string }>();
  return <InviteAccept token={params.token} />;
};

export const AppRouter = () => (
  <Switch>
    <Route path='/login' component={Login} />
    <Route path='/invites/:token' component={InviteAcceptRoute} />
    <Route path='/workers'>
      <AuthenticatedLayout>
        <Workers />
      </AuthenticatedLayout>
    </Route>
    <Route path='/settings'>
      <AuthenticatedLayout>
        <Settings />
      </AuthenticatedLayout>
    </Route>
    <Route path='/teams'>
      <AuthenticatedLayout>
        <Teams />
      </AuthenticatedLayout>
    </Route>
    <Route path='/projects/connect'>
      <AuthenticatedLayout>
        <ProjectsFormPage />
      </AuthenticatedLayout>
    </Route>
    <Route path='/projects/:id/edit'>
      <AuthenticatedLayout>
        <ProjectsEditRoute />
      </AuthenticatedLayout>
    </Route>
    <Route path='/runs'>
      <AuthenticatedLayout>
        <Runs />
      </AuthenticatedLayout>
    </Route>
    <Route path='/'>
      <AuthenticatedLayout>
        <Dashboard />
      </AuthenticatedLayout>
    </Route>
  </Switch>
);
