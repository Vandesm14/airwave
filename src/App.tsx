import { useAtom } from 'solid-jotai';
import { WhisperSTT } from '../vendor/whisper-speech-to-text/src/';
import {
  airspaceSizeAtom,
  frequencyAtom,
  isRecordingAtom,
  messagesAtom,
  runwaysAtom,
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

  function callsignString(id: string): string {
    const airlines: Record<string, string> = {
      AAL: 'American Airlines',
      SKW: 'Sky West',
      JBL: 'Jet Blue',
    };

    return `${airlines[id.slice(0, 3)]} ${id.slice(3, 7)}`;
  }

  function speak(text: string) {
    if ('speechSynthesis' in window) {
      if (window.speechSynthesis.speaking || isRecording()) {
        setTimeout(() => speak(text), 500);
      } else {
        const utterance = new SpeechSynthesisUtterance(
          text.replace(/[0-9]/g, '$& ')
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

  let [, setAirspaceSize] = useAtom(airspaceSizeAtom);
  let [aircrafts, setAircrafts] = createSignal<Array<Aircraft>>([], {
    equals: false,
  });
  let [, setRunways] = useAtom(runwaysAtom);
  let [messages, setMessages] = useAtom(messagesAtom);
  let [frequency] = useAtom(frequencyAtom);

  createEffect(() => {
    localStorage.setItem('messages', JSON.stringify(messages()));
  });

  onMount(() => {
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Insert' && !isRecording()) {
        whisper.startRecording();
        setIsRecording(true);
      } else if (e.key === 'Delete' && isRecording()) {
        whisper.abortRecording();
        setIsRecording(false);
      }
    });

    document.addEventListener('keyup', (e) => {
      if (e.key === 'Insert' && isRecording()) {
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
    });
  });

  function speakAsAircraft(message: RadioMessage) {
    message.reply = `${message.reply}, ${callsignString(message.id)}`;
    setMessages((messages) => [...messages, message]);
    speak(message.reply);
  }

  function speakAsATC(message: RadioMessage) {
    setMessages((messages) => [...messages, message]);
  }

  const search = new URLSearchParams(window.location.search);
  const hostname = search.get('ws') ?? window.location.hostname;

  let socket = new WebSocket(`ws://${hostname}:9001`);
  socket.onopen = function (_) {
    console.log('[open] Connection established');
    console.log('Sending to server');

    socket.send(JSON.stringify({ type: 'connect' }));
  };

  function posToXY<T extends { pos: [number, number]; x: number; y: number }>(
    obj: T
  ): T {
    obj.x = obj.pos[0];
    obj.y = obj.pos[1];

    return obj;
  }

  socket.onmessage = function (event) {
    // console.log(`[message] Data received from server: ${event.data}`);

    let json: ServerEvent = JSON.parse(event.data);
    switch (json.type) {
      case 'aircraft':
        setAircrafts(json.value.map(posToXY));
        break;
      case 'runways':
        setRunways(json.value.map(posToXY));
        break;
      case 'atcreply':
        speakAsATC(json.value);
        break;
      case 'reply':
        speakAsAircraft(json.value);
        break;
      case 'size':
        setAirspaceSize(json.value);
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
      <Chatbox></Chatbox>
      <Canvas aircrafts={aircrafts}></Canvas>
      <div class="top-right">
        <StripBoard aircrafts={aircrafts}></StripBoard>
        <FreqSelector></FreqSelector>
      </div>
    </div>
  );
}
