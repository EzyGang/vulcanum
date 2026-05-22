import { Route, Switch } from 'wouter-preact';

import { ProtectedRoute } from '../components/auth/ProtectedRoute';
import { Dashboard } from '../pages/Dashboard';
import { Login } from '../pages/Login';

export const AppRouter = () => (
  <Switch>
    <Route path='/login' component={Login} />
    <Route path='/'>
      <ProtectedRoute>
        <Dashboard />
      </ProtectedRoute>
    </Route>
  </Switch>
);
