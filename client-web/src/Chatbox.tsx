import { useAtom } from 'solid-jotai';
import {
  frequencyAtom,
  isRecordingAtom,
  messagesAtom,
  selectedAircraftAtom,
} from './lib/atoms';
import { createEffect, createSignal } from 'solid-js';

export default function Chatbox({
  sendMessage,
}: {
  sendMessage: (text: string) => void;
}) {
  let chatbox;
  let chatboxInput;
  let [messages, setMessages] = useAtom(messagesAtom);
  let [isRecording] = useAtom(isRecordingAtom);
  let [frequency] = useAtom(frequencyAtom);
  let [selectedAircraft] = useAtom(selectedAircraftAtom);
  let [showAll, setShowAll] = createSignal(false);
  let [text, setText] = createSignal('');

  function resetText() {
    if (selectedAircraft()) {
      setText(`${selectedAircraft()} `);
    } else {
      setText('');
    }
  }

  createEffect(() => {
    if (
      selectedAircraft() &&
      chatboxInput instanceof HTMLInputElement &&
      document.activeElement !== chatboxInput
    ) {
      resetText();
    }
  });

  createEffect(() => {
    if (chatbox instanceof HTMLDivElement) {
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
    if (confirm('Clear all messages?')) {
      setMessages([]);
    }
  }

  function handleSendMessage(text) {
    sendMessage(text);
    resetText();
  }

  return (
    <div id="chatbox" classList={{ live: isRecording() }}>
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
      <div class="messages" ref={chatbox}>
        {messages()
          .filter((m) => showAll() || m.frequency === frequency())
          .map((m) => (
            <div
              classList={{
                message: true,
                selected: m.id === selectedAircraft(),
              }}
            >
              {showAll() ? <span class="frequency">{m.frequency}</span> : null}
              <span
                classList={{
                  callsign: true,
                  atc: m.id === 'ATC',
                }}
              >
                {m.id}
              </span>
              <span class="text">{m.reply}</span>
            </div>
          ))}
      </div>
      <div class="input">
        <input
          type="text"
          value={text()}
          oninput={(e) => setText(e.currentTarget.value)}
          onkeydown={(e) => e.key === 'Enter' && handleSendMessage(text())}
          ref={chatboxInput}
          placeholder="Type a message..."
        />
        <input
          type="button"
          value="Send"
          onclick={() => handleSendMessage(text())}
        />
      </div>
    </div>
  );
}
