/* @refresh reload */
import { render } from 'solid-js/web';

import App from './App';
import { QueryClient, QueryClientProvider } from '@tanstack/solid-query';

const root = document.getElementById('root');
const client = new QueryClient();
if (!(root instanceof HTMLElement)) {
  throw new Error(
    'Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?'
  );
}

render(
  () => (
    <QueryClientProvider client={client}>
      <App />
    </QueryClientProvider>
  ),
  root!
);
