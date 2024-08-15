/* @refresh reload */
import { render } from 'solid-js/web';

import App from './app';

const root = document.getElementById('root');

if (!(root instanceof HTMLElement)) {
  throw new Error(
    'Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?'
  );
}

// @ts-expect-error: React isn't being use, TS
render(() => <App />, root!);
