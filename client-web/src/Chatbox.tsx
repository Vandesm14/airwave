import { useAtom } from 'solid-jotai';
import {
  frequencyAtom,
  isRecordingAtom,
  selectedAircraftAtom,
  useTTSAtom,
} from './lib/atoms';
import { createEffect, createSignal, onMount } from 'solid-js';
import { createQuery } from '@tanstack/solid-query';
import { RadioMessage } from './lib/types';
import { useStorageAtom } from './lib/hooks';

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
  let [useTTS, setUseTTS] = useStorageAtom(useTTSAtom);
  let [showAll, setShowAll] = createSignal(false);
  let [text, setText] = createSignal('');
  let [lastRead, setLastRead] = createSignal(Date.now() / 1000);
  const messages = createQuery<Array<RadioMessage>>(() => ({
    queryKey: ['/api/messages'],
    queryFn: async () => {
      const result = await fetch('http://localhost:9001/api/messages');
      if (!result.ok) return [];
      return result.json();
    },
    initialData: [],
    staleTime: 500,
    refetchInterval: 500,
    refetchOnMount: 'always',
    throwOnError: true, // Throw an error if the query fails
  }));

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
        speak(message);
        setLastRead(message.created.secs);
      }
    }
  });

  onMount(() => {
    document.addEventListener('keydown', (e) => {
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
    });
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
