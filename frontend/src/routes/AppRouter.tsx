import type { ComponentChildren, JSX } from 'preact';
import { Route, Switch, useParams } from 'wouter-preact';
import { NavigationShellContainer } from '../components/app-shell/containers/NavigationShell.container';
import { Dashboard } from '../pages/Dashboard';
import { Login } from '../pages/Login';
import { Projects } from '../pages/Projects';
import { ProjectsFormPage } from '../pages/ProjectsForm';
import { Providers } from '../pages/Providers';
import { Runs } from '../pages/Runs';
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

export const AppRouter = () => (
  <Switch>
    <Route path='/login' component={Login} />
    <Route path='/workers'>
      <AuthenticatedLayout>
        <Workers />
      </AuthenticatedLayout>
    </Route>
    <Route path='/providers'>
      <AuthenticatedLayout>
        <Providers />
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
    <Route path='/projects'>
      <AuthenticatedLayout>
        <Projects />
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
