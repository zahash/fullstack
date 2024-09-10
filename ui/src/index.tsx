/* @refresh reload */
import { render } from 'solid-js/web';
import { Route, HashRouter } from '@solidjs/router';
import { MetaProvider } from '@solidjs/meta';

import './index.css';
import Home from './pages/Home';

const root = document.getElementById('root');

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    'Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?',
  );
}

render(() => (
  <MetaProvider>
    <HashRouter>
      <Route path="/" component={Home} />
    </HashRouter>
  </MetaProvider>
), root!);
