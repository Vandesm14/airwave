import { useAtom } from 'solid-jotai';
import { WhisperSTT } from './whisper/WhisperSTT';
import {
  frequencyAtom,
  isRecordingAtom,
  messagesAtom,
  pointsAtom,
  selectedAircraftAtom,
  useTTSAtom,
  worldAtom,
} from './lib/atoms';
import { Aircraft, RadioMessage, ServerEvent } from './lib/types';
import Chatbox from './Chatbox';
import {
  createEffect,
  createMemo,
  createSignal,
  onMount,
  Show,
} from 'solid-js';
import Canvas from './Canvas';
import StripBoard from './StripBoard';
import FreqSelector from './FreqSelector';
import { useStorageAtom } from './lib/hooks';
import { formatTime } from './lib/lib';
import { createQuery } from '@tanstack/solid-query';

export default function App() {
  const whisper = new WhisperSTT();

  let [isRecording, setIsRecording] = useAtom(isRecordingAtom);
  let [, setWorld] = useAtom(worldAtom);
  let [messages, setMessages] = useAtom(messagesAtom);
  let [frequency] = useStorageAtom(frequencyAtom);
  let [useTTS, setUseTTS] = useStorageAtom(useTTSAtom);
  let [points, setPoints] = useAtom(pointsAtom);
  let [_, setSelectedAircraft] = useAtom(selectedAircraftAtom);
  const query = createQuery<boolean>(() => ({
    queryKey: ['/api/ping'],
    queryFn: async () => {
      try {
        const result = await fetch('http://localhost:9001/api/ping');
        if (!result.ok) return false;
        return (await result.text()) === 'pong';
      } catch {
        return false;
      }
    },
    staleTime: 2000,
    refetchInterval: 2000,
    throwOnError: true, // Throw an error if the query fails
  }));

  async function getMedia(constraints: MediaStreamConstraints) {
    await navigator.mediaDevices.getUserMedia(constraints);
  }

  function startRecording() {
    whisper.startRecording();
    setIsRecording(true);
  }

  async function stopRecording() {
    setIsRecording(false);
    whisper.stopRecording((blob) => {
      blob.arrayBuffer().then((value) => {
        const data = [...new Uint8Array(value)];
        fetch(
          `http://localhost:9001/api/comms/voice?frequency=${frequency()}`,
          {
            body: value,
            method: 'POST',
          }
        );
      });
    });
  }

  function discardRecording() {
    whisper.abortRecording();
    setIsRecording(false);
  }

  createEffect(() => {
    localStorage.setItem('messages', JSON.stringify(messages()));
  });

  async function sendTextMessage(text: string) {
    await fetch(
      `http://localhost:9001/api/comms/text?frequency=${frequency()}`,
      {
        body: text,
        method: 'POST',
      }
    );
  }

  onMount(async () => {
    await getMedia({ audio: true, video: false });

    document.addEventListener('keydown', (e) => {
      if (e.key === 'Insert' && !isRecording()) {
        startRecording();
      } else if (e.key === 'Pause') {
        sendPause();
      }
    });

    document.addEventListener('keyup', (e) => {
      if (e.key === 'Insert' && isRecording()) {
        stopRecording();
      } else if (e.key === 'Delete' && isRecording()) {
        discardRecording();
      }
    });
  });

  function sendPause() {
    // if (socket !== null) {
    //   socket.send(JSON.stringify({ type: 'ui', value: { type: 'pause' } }));
    // }
  }

  function toggleTTS() {
    setUseTTS((useTTS) => !useTTS);
  }

  createEffect(() => {
    console.log('connected?', query.data);
  });

  return (
    <>
      <Show when={query.data}>
        <div class="bottom-left">
          <div class="points">
            <p>
              <b>Landings:</b> {points().landings} (rate: once every{' '}
              {formatTime(points().landing_rate.rate.secs * 1000)} mins)
            </p>
            <p>
              <b>Takeoffs:</b> {points().takeoffs} (rate: once every{' '}
              {formatTime(points().takeoff_rate.rate.secs * 1000)} mins)
            </p>
          </div>
          <Chatbox sendMessage={sendTextMessage}></Chatbox>
        </div>
        <div id="radar">
          <Canvas></Canvas>
          <div class="top-right">
            <StripBoard></StripBoard>
            <FreqSelector></FreqSelector>
          </div>
          <div class="bottom-right-buttons">
            <button
              classList={{ 'tts-toggle': true, enabled: useTTS() }}
              onClick={toggleTTS}
            >
              {useTTS() ? 'Disable TTS' : 'Enable TTS'}
            </button>
            <button
              class={`talk-button ${isRecording() ? 'recording' : ''}`}
              onMouseDown={startRecording}
              onMouseUp={stopRecording}
            >
              {isRecording() ? 'Recording...' : 'Talk'}
            </button>
            <button class="discard-button" onClick={discardRecording}>
              Discard
            </button>
            <button class="pause-button" onClick={sendPause}>
              Pause
            </button>
          </div>
        </div>
      </Show>
      <Show when={!query.data}>
        <div class="connection-message">
          <h1>Connecting...</h1>
        </div>
      </Show>
    </>
  );
}
