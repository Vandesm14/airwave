import { useAtom } from 'solid-jotai';
import { frequencyAtom, isRecordingAtom, messagesAtom } from './lib/atoms';
import { createEffect, createSignal } from 'solid-js';

export default function Chatbox() {
  let chatbox;
  let [messages, setMessages] = useAtom(messagesAtom);
  let [isRecording] = useAtom(isRecordingAtom);
  let [frequency] = useAtom(frequencyAtom);
  let [showAll, setShowAll] = createSignal(false);

  createEffect(() => {
    if (chatbox instanceof HTMLDivElement) {
      // TODO: this doesn't scroll down whenever we show all or swap frequency
      let _ = [frequency(), showAll()];
      // Subscribe to frequency and showAll signals

      // Subscribe to messages signal
      if (messages().length > 0) {
        chatbox.scrollTo(0, chatbox.scrollHeight);
      }
    }
  });

  function toggleAll() {
    setShowAll((b) => !b);
  }

  function clearAll() {
    setMessages([]);
  }

  return (
    <div id="chatbox" ref={chatbox} classList={{ live: isRecording() }}>
      <div class="controls">
        <input
          type="button"
          value="Clear All"
          onclick={clearAll}
          class="danger"
        />
        <input
          type="button"
          value={showAll() ? 'Show Yours' : 'Show All'}
          onclick={toggleAll}
        />
      </div>
      <div class="messages">
        {messages()
          .filter((m) => showAll() || m.frequency === frequency())
          .map((m) => (
            <div class="message">
              <span class="frequency">{m.frequency}</span>
              <span class="callsign">{m.id}</span>
              <span class="text">{m.reply}</span>
            </div>
          ))}
      </div>
    </div>
  );
}
