import { useAtom } from 'solid-jotai';
import { WhisperSTT } from '../vendor/whisper-speech-to-text/src/index';
import {
  frequencyAtom,
  isRecordingAtom,
  messagesAtom,
  selectedAircraftAtom,
  worldAtom,
} from './lib/atoms';
import { Aircraft, RadioMessage, ServerEvent } from './lib/types';
import Chatbox from './Chatbox';
import { createEffect, createSignal, onMount } from 'solid-js';
import Canvas from './Canvas';
import StripBoard from './StripBoard';
import FreqSelector from './FreqSelector';

export default function App() {
  const whisper = new WhisperSTT();

  let [isRecording, setIsRecording] = useAtom(isRecordingAtom);

  let [aircrafts, setAircrafts] = createSignal<Array<Aircraft>>([], {
    equals: false,
  });
  let [, setWorld] = useAtom(worldAtom);
  let [messages, setMessages] = useAtom(messagesAtom);
  let [frequency] = useAtom(frequencyAtom);
  let [_, setSelectedAircraft] = useAtom(selectedAircraftAtom);

  async function getMedia(constraints) {
    await navigator.mediaDevices.getUserMedia(constraints);
  }

  function callsignString(id: string): string {
    const airlines: Record<string, string> = {
      AAL: 'American Airlines',
      SKW: 'Sky West',
      JBL: 'Jet Blue',
    };

    return `${airlines[id.slice(0, 3)]} ${id.slice(3, 7)}`;
  }

  function speak(message: RadioMessage) {
    if ('speechSynthesis' in window && frequency() === message.frequency) {
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

  function startRecording() {
    whisper.startRecording();
    setIsRecording(true);
  }

  function stopRecording() {
    setIsRecording(false);
    whisper.stopRecording((blob) => {
      blob.arrayBuffer().then((value) => {
        console.log('send voice request');
        socket.send(
          JSON.stringify({
            type: 'voice',
            value: {
              data: [...new Uint8Array(value)],
              frequency: frequency(),
            },
          })
        );
        console.log('sent voice request');
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

  function sendTextMessage(text: string) {
    socket.send(
      JSON.stringify({ type: 'text', value: { text, frequency: frequency() } })
    );
  }

  onMount(async () => {
    await getMedia({ audio: true, video: false });

    document.addEventListener('keydown', (e) => {
      if (e.key === 'Insert' && !isRecording()) {
        startRecording();
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

  function speakAsAircraft(message: RadioMessage) {
    message.reply = `${message.reply}, ${callsignString(message.id)}`;
    setMessages((messages) => [...messages, message]);
    speak(message);
  }

  function speakAsATC(message: RadioMessage) {
    setMessages((messages) => [...messages, message]);
  }

  const search = new URLSearchParams(window.location.search);
  const hostname = search.get('ws') ?? window.location.hostname;
  const port = search.get('port') ?? '9001';

  let socket = new WebSocket(`ws://${hostname}:${port}`);
  socket.onopen = function (_) {
    console.log('[open] Connection established');
    console.log('Sending to server');

    socket.send(JSON.stringify({ type: 'connect' }));
  };

  socket.onmessage = function (event) {
    // console.log(`[message] Data received from server: ${event.data}`);

    let json: ServerEvent = JSON.parse(event.data);
    switch (json.type) {
      case 'aircraft':
        setAircrafts(json.value);
        break;
      case 'world':
        setWorld(json.value);
        break;
      case 'atcreply':
        speakAsATC(json.value);
        break;
      case 'reply':
        if (json.value.frequency == frequency()) {
          setSelectedAircraft(json.value.id);
        }
        if (json.value.reply != '') speakAsAircraft(json.value);
        break;
    }
  };

  socket.onclose = function (event: { wasClean: any; code: any; reason: any }) {
    if (event.wasClean) {
      console.log(
        `[close] Connection closed cleanly, code=${event.code} reason=${event.reason}`
      );
    } else {
      // e.g. server process killed or network down
      // event.code is usually 1006 in this case
      console.log('[close] Connection died');
    }
  };

  socket.onerror = function () {
    console.log(`[error]`);
  };

  return (
    <div id="radar">
      <Chatbox sendMessage={sendTextMessage}></Chatbox>
      <Canvas aircrafts={aircrafts}></Canvas>
      <div class="top-right">
        <StripBoard aircrafts={aircrafts}></StripBoard>
        <FreqSelector></FreqSelector>
      </div>
      <div class="bottom-right-buttons">
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
      </div>
    </div>
  );
}
