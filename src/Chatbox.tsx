import { useAtom } from 'solid-jotai';
import { frequencyAtom, isRecordingAtom, messagesAtom } from './lib/atoms';
import { createEffect, createSignal } from 'solid-js';

export default function Chatbox() {
  let chatbox;
  let [messages, _] = useAtom(messagesAtom);
  let [isRecording] = useAtom(isRecordingAtom);
  let [frequency] = useAtom(frequencyAtom);
  let [showAll, setShowAll] = createSignal(false);

  createEffect(() => {
    if (chatbox instanceof HTMLDivElement) {
      // Subscribe to messages signal
      if (messages().length > 0) {
        chatbox.scrollTo(0, chatbox.scrollHeight);
      }
    }
  });

  function toggleAll() {
    setShowAll((b) => !b);
  }

  return (
    <div id="chatbox" ref={chatbox} classList={{ live: isRecording() }}>
      <input
        type="button"
        value={showAll() ? 'Show Yours' : 'Show All'}
        onclick={toggleAll}
      />
      {messages()
        .filter((m) => showAll() || m.frequency === frequency())
        .map((m) => (
          <div class="message">
            <span class="callsign">{m.id}</span>
            <span class="text">{m.reply}</span>
          </div>
        ))}
    </div>
  );
}
