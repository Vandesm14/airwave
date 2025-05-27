import { frequencyAtom } from './lib/atoms';
import Chatbox from './Chatbox';
import { createMemo, Show } from 'solid-js';
import Canvas from './Canvas';
import StripBoard from './StripBoard';
import FreqSelector from './FreqSelector';
import { useStorageAtom } from './lib/hooks';
import { baseAPIPath, getMessages, usePing } from './lib/api';
import { useQueryClient } from '@tanstack/solid-query';
import Flights from './Airport';
import GameButtons from './GameButtons';

export default function App() {
  const [frequency] = useStorageAtom(frequencyAtom);

  const query = usePing();
  const client = useQueryClient();

  async function sendTextMessage(text: string) {
    await fetch(`${baseAPIPath}/api/comms/text?frequency=${frequency()}`, {
      body: text,
      method: 'POST',
    });
    await client.invalidateQueries({
      queryKey: [getMessages],
      type: 'all',
      exact: true,
    });
    await client.refetchQueries({
      queryKey: [getMessages],
      type: 'all',
      exact: true,
    });
  }

  const isConnected = createMemo(
    () => query.data !== undefined && query.data.connected
  );

  const changeServerURL = (e: Event) => {
    const url = window.location;
    const input = e.target as HTMLInputElement;
    let value = input.value;

    // Add http:// if not provided
    if (!/^https?:\/\//i.test(value)) {
      value = 'http://' + value;
    }

    const search = new URLSearchParams(window.location.search);
    search.set('api', value);
    const newURL = `${url.origin}${url.pathname}?${search.toString()}`;
    window.location.href = newURL;
  };

  // Check if we are in demo mode.
  const isDemoMode = createMemo(() => {
    const root = document.getElementById('root');
    return root?.classList.contains('demo');
  });

  return (
    <>
      <Show when={isConnected()}>
        <Show when={!isDemoMode()}>
          <div class="container left">
            <Flights />
            <div class="spacer"></div>
            <div class="row" id="bottom-left-panel">
              <Chatbox sendMessage={sendTextMessage}></Chatbox>
              <GameButtons />
            </div>
          </div>
        </Show>
        <div id="radar">
          <Canvas></Canvas>
        </div>
        <Show when={!isDemoMode()}>
          <div id="right-panel">
            <StripBoard></StripBoard>
            <FreqSelector></FreqSelector>
          </div>
        </Show>
      </Show>
      <Show when={!isConnected()}>
        <div class="connection-message">
          <h1>Connecting to server {baseAPIPath}</h1>
          <h2>
            Retrying: <input value={baseAPIPath} onchange={changeServerURL} />
          </h2>
        </div>
      </Show>
    </>
  );
}
