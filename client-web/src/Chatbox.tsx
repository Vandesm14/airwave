import { useAtom } from 'solid-jotai';
import {
  frequencyAtom,
  isRecordingAtom,
  selectedAircraftAtom,
  useTTSAtom,
  useTTSAtomKey,
} from './lib/atoms';
import { createEffect, createSignal } from 'solid-js';
import useGlobalShortcuts, { useStorageAtom } from './lib/hooks';
import { useMessages } from './lib/api';
import { OutgoingCommandReply } from '../bindings/OutgoingCommandReply';

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
  let [useTTS] = useStorageAtom(useTTSAtomKey, useTTSAtom);
  let [showAll, setShowAll] = createSignal(false);
  let [text, setText] = createSignal('');
  let [lastRead, setLastRead] = createSignal(Date.now() / 1000);
  let [voices, setVoices] = createSignal<
    Map<String, { rate: number; pitch: number }>
  >(new Map());
  const messages = useMessages();

  function randBetween(min: number, max: number) {
    return Math.floor(Math.random() * (max - min + 1) + min);
  }

  function speak(message: OutgoingCommandReply) {
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

        if (voices().has(message.id)) {
          utterance.rate = voices().get(message.id)!.rate;
          utterance.pitch = voices().get(message.id)!.pitch;
        } else {
          const voice = {
            rate: (1.0 * randBetween(100, 130)) / 100,
            pitch: (1.3 * randBetween(80, 115)) / 100,
          };
          setVoices((voices) => voices.set(message.id, voice));
          utterance.rate = voice.rate;
          utterance.pitch = voice.pitch;
        }

        utterance.volume = 0.01;
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

  useGlobalShortcuts((e) => {
    if (
      e.key === 't' &&
      chatboxInput instanceof HTMLInputElement &&
      document.activeElement !== chatboxInput
    ) {
      // TODO: this conflicts with pressing "t" in the stripboard.
      chatboxInput.focus();
      e.preventDefault();
    } else if (e.key === 'Escape') {
      chatboxInput.blur();
      e.preventDefault();
    }
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
    const trimmed = text.trim();

    if (trimmed.length === 7 && /\w{3}\d{4}/.test(trimmed)) {
      setSelectedAircraft(trimmed);
    } else {
      sendMessage(trimmed);
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
                onClick={() => setSelectedAircraft(m.id)}
              >
                {m.id}
              </span>
              <span class="text">
                {m.reply
                  .trim()
                  .split('\n')
                  .map((line, i, arr) =>
                    i === arr.length - 1 ? line : [line, <br />]
                  )
                  .flat()}
              </span>
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
