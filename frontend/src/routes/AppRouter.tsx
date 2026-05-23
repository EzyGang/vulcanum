import { Route, Switch } from 'wouter-preact';
import { ProtectedRoute } from '../components/auth/ProtectedRoute';
import { Dashboard } from '../pages/Dashboard';
import { Login } from '../pages/Login';
import { Workers } from '../pages/Workers';

export const AppRouter = () => (
  <Switch>
    <Route path='/login' component={Login} />
    <Route path='/workers'>
      <ProtectedRoute>
        <Workers />
      </ProtectedRoute>
    </Route>
    <Route path='/'>
      <ProtectedRoute>
        <Dashboard />
      </ProtectedRoute>
    </Route>
  </Switch>
);
