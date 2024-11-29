import { useAtom } from 'solid-jotai';
import {
  frequencyAtom,
  isRecordingAtom,
  selectedAircraftAtom,
  useTTSAtom,
} from './lib/atoms';
import { createEffect, createSignal, onMount, onCleanup } from 'solid-js';
import { RadioMessage } from './lib/types';
import { useStorageAtom } from './lib/hooks';
import { useMessages } from './lib/api';

export default function Chatbox({
  sendMessage,
}: {
  sendMessage: (text: string) => void;
}) {
  let chatbox!: HTMLDivElement;
  let chatboxInput!: HTMLInputElement;
  let [isRecording] = useAtom(isRecordingAtom);
  let [frequency] = useAtom(frequencyAtom);
  let [selectedAircraft, setSelectedAircraft] = useAtom(selectedAircraftAtom);
  let [useTTS] = useStorageAtom(useTTSAtom);
  let [showAll, setShowAll] = createSignal(false);
  let [text, setText] = createSignal('');
  let [lastRead, setLastRead] = createSignal(Date.now() / 1000);
  const messages = useMessages();

  function speak(message: RadioMessage) {
    if (
      useTTS() &&
      'speechSynthesis' in window &&
      frequency() === message.frequency
    ) {
      if (window.speechSynthesis.speaking || isRecording()) {
        setTimeout(() => speak(message), 500);
      } else {
        const utterance = new SpeechSynthesisUtterance(
          message.reply.replace(/[0-9]/g, '$& ')
        );
        utterance.volume = 0.01;
        utterance.rate = 1.0;
        utterance.pitch = 1.3;
        window.speechSynthesis.speak(utterance);
      }
    } else {
      console.log("Sorry, your browser doesn't support text to speech!");
    }
  }

  createEffect(() => {
    for (const message of messages.data) {
      if (message.created.secs > lastRead() && message.id !== 'ATC') {
        if (message.reply !== '') {
          speak(message);
        } else {
          setSelectedAircraft(message.id);
        }
        setLastRead(message.created.secs);
      }
    }
  });

  function onKeydown(e: KeyboardEvent) {
    if (
      e.key === 't' &&
      chatboxInput instanceof HTMLInputElement &&
      document.activeElement !== chatboxInput
    ) {
      chatboxInput.focus();
      e.preventDefault();
    } else if (e.key === 'Escape') {
      chatboxInput.blur();
      e.preventDefault();
    }
  }

  onMount(() => {
    document.addEventListener('keydown', onKeydown);
  });

  onCleanup(() => {
    document.removeEventListener('keydown', onKeydown);
  });

  function resetText() {
    if (selectedAircraft()) {
      setText(`${selectedAircraft()} `);
    } else {
      setText('');
    }
  }

  createEffect(() => {
    if (selectedAircraft()) {
      resetText();
    }
  });

  createEffect(() => {
    if (chatbox instanceof HTMLDivElement) {
      // Subscribe to frequency and showAll signals
      frequency();
      showAll();

      // Subscribe to messages signal
      if (messages.data.length > 0) {
        chatbox.scrollTo(0, chatbox.scrollHeight);
      }
    }
  });

  function toggleAll() {
    setShowAll((b) => !b);
  }

  function handleSendMessage(text: string) {
    if (text.length === 7 && /\w{3}\d{4}/.test(text)) {
      setSelectedAircraft(text);
    } else {
      sendMessage(text);
    }

    resetText();
  }

  return (
    <div id="chatbox" classList={{ live: isRecording() }}>
      <div class="controls">
        <input
          type="button"
          value={showAll() ? 'Show Yours' : 'Show All'}
          onclick={toggleAll}
        />
      </div>
      <div class="messages" ref={chatbox}>
        {messages.data
          .filter(
            (m) => (showAll() || m.frequency === frequency()) && m.reply !== ''
          )
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
