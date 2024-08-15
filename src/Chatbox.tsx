import { useAtom } from 'solid-jotai';
import { isRecordingAtom, messagesAtom } from './lib/atoms';
import { createEffect } from 'solid-js';

export default function Chatbox() {
  let chatbox;
  let [messages, _] = useAtom(messagesAtom);
  let [isRecording] = useAtom(isRecordingAtom);

  createEffect(() => {
    if (chatbox instanceof HTMLDivElement) {
      // Subscribe to messages signal
      if (messages().length > 0) {
        chatbox.scrollTo(0, chatbox.scrollHeight);
      }
    }
  });

  return (
    <div id="chatbox" ref={chatbox} classList={{ live: isRecording() }}>
      {messages().map((m) => (
        <div class="message">
          <span class="callsign">{m.id}</span>
          <span class="text">{m.reply}</span>
        </div>
      ))}
    </div>
  );
}
