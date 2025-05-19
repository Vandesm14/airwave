import { useAtom } from 'solid-jotai';
import { WhisperSTT } from './whisper/WhisperSTT';
import { frequencyAtom, isRecordingAtom, useTTSAtom } from './lib/atoms';
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
import { baseAPIPath, getMessages, usePing } from './lib/api';
import { useQueryClient } from '@tanstack/solid-query';
import Flights from './Flights';

export default function App() {
  const whisper = new WhisperSTT();

  const [isRecording, setIsRecording] = useAtom(isRecordingAtom);
  const [frequency] = useStorageAtom(frequencyAtom);
  const [useTTS, setUseTTS] = useStorageAtom(useTTSAtom);
  const [downButtons, setDownButtons] = createSignal<number>(0);

  const query = usePing();
  const client = useQueryClient();

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

    function gameLoop() {
      const gamepads = navigator.getGamepads();
      if (gamepads) {
        const gp = gamepads[0];
        if (gp) {
          let newDownButtons = 0;
          for (const i in gp.buttons) {
            const button = gp.buttons[i];
            if (button?.pressed) {
              newDownButtons++;
            }
          }

          if (newDownButtons !== downButtons()) {
            // If one button is pressed, use as PTT.
            if (newDownButtons === 1 && downButtons() === 0) {
              if (!isRecording()) {
                startRecording();
              }

              // If two or more buttons are pressed, stop recording.
            } else if (newDownButtons === 0 && downButtons() === 1) {
              if (isRecording()) {
                stopRecording();
              }
            } else if (newDownButtons > 1) {
              if (isRecording()) {
                discardRecording();
              }
            }

            setDownButtons(newDownButtons);
          }
        }
      }

      requestAnimationFrame(gameLoop);
    }

    gameLoop();
  });

  createEffect(() => {});

  async function sendPause() {
    await fetch(`${baseAPIPath}/api/pause`, {
      method: 'POST',
    });
  }

  function toggleTTS() {
    setUseTTS((useTTS) => !useTTS);
  }

  const isConnected = createMemo(
    () => query.data !== undefined && query.data.connected
  );

  return (
    <>
      <Show when={isConnected()}>
        <div class="container left">
          <Flights />
          <div class="spacer"></div>
          <div class="row" id="bottom-left-panel">
            <Chatbox sendMessage={sendTextMessage}></Chatbox>
            <div id="game-buttons">
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
        </div>
        <div id="radar">
          <Canvas></Canvas>
        </div>
        <div id="right-panel">
          <StripBoard></StripBoard>
          <FreqSelector></FreqSelector>
        </div>
      </Show>
      <Show when={!isConnected()}>
        <div class="connection-message">
          <h1>Connecting to server...</h1>
          <h2>Retrying: {baseAPIPath}</h2>
        </div>
      </Show>
    </>
  );
}
