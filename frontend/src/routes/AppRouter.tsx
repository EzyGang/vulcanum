import { Route, Switch, useParams } from 'wouter-preact';
import { ProtectedRoute } from '../components/auth/ProtectedRoute';
import { Dashboard } from '../pages/Dashboard';
import { Login } from '../pages/Login';
import { Projects } from '../pages/Projects';
import { ProjectsFormPage } from '../pages/ProjectsForm';
import { Workers } from '../pages/Workers';

const ProjectsEditRoute = () => {
  const params = useParams();
  return <ProjectsFormPage projectId={params.id} />;
};

export const AppRouter = () => (
  <Switch>
    <Route path='/login' component={Login} />
    <Route path='/workers'>
      <ProtectedRoute>
        <Workers />
      </ProtectedRoute>
    </Route>
    <Route path='/projects/new'>
      <ProtectedRoute>
        <ProjectsFormPage />
      </ProtectedRoute>
    </Route>
    <Route path='/projects/:id/edit'>
      <ProtectedRoute>
        <ProjectsEditRoute />
      </ProtectedRoute>
    </Route>
    <Route path='/projects'>
      <ProtectedRoute>
        <Projects />
      </ProtectedRoute>
    </Route>
    <Route path='/'>
      <ProtectedRoute>
        <Dashboard />
      </ProtectedRoute>
    </Route>
  </Switch>
);
