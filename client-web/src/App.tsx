import { useAtom } from 'solid-jotai';
import { WhisperSTT } from './whisper/WhisperSTT';
import { frequencyAtom, isRecordingAtom, useTTSAtom } from './lib/atoms';
import Chatbox from './Chatbox';
import { createEffect, onMount, Show } from 'solid-js';
import Canvas from './Canvas';
import StripBoard from './StripBoard';
import FreqSelector from './FreqSelector';
import { useStorageAtom } from './lib/hooks';
import { baseAPIPath, getMessages, usePing } from './lib/api';
import Points from './Points';
import { QueryClient } from '@tanstack/solid-query';
import Flights from './Flights';

export default function App() {
  const whisper = new WhisperSTT();

  let [isRecording, setIsRecording] = useAtom(isRecordingAtom);
  let [frequency] = useStorageAtom(frequencyAtom);
  let [useTTS, setUseTTS] = useStorageAtom(useTTSAtom);
  const query = usePing();
  const client = new QueryClient();

  async function getMedia(constraints: MediaStreamConstraints) {
    await navigator.mediaDevices.getUserMedia(constraints);
  }

  function startRecording() {
    whisper.startRecording();
    setIsRecording(true);
  }

  async function stopRecording() {
    setIsRecording(false);
    whisper.stopRecording(async (blob) => {
      const value = await blob.arrayBuffer();
      await fetch(`${baseAPIPath}/api/comms/voice?frequency=${frequency()}`, {
        body: value,
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
    });
  }

  function discardRecording() {
    whisper.abortRecording();
    setIsRecording(false);
  }

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

  async function sendPause() {
    await fetch(`${baseAPIPath}/api/pause`, {
      method: 'POST',
    });
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
        <div class="top-left">
          <Flights />
        </div>
        <div class="bottom-left">
          <Points />
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
          <h1>Connecting to server...</h1>
          <h2>Retrying: {baseAPIPath}</h2>
        </div>
      </Show>
    </>
  );
}
