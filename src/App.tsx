import { useAtom } from 'solid-jotai';
import { WhisperSTT } from '../vendor/whisper-speech-to-text/src/';
import {
  airspaceSizeAtom,
  isRecordingAtom,
  messagesAtom,
  runwaysAtom,
} from './lib/atoms';
import { Aircraft, RadioMessage, ServerEvent } from './lib/types';
import Chatbox from './Chatbox';
import { createSignal, onMount } from 'solid-js';
import Canvas from './Canvas';
import StripBoard from './StripBoard';

export default function App() {
  const whisper = new WhisperSTT();

  let [isRecording, setIsRecording] = useAtom(isRecordingAtom);

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
  let [, setMessages] = useAtom(messagesAtom);

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
                value: [...new Uint8Array(value)],
              })
            );

            console.log('sent voice request');
          });
        });
      }
    });
  });

  function speakAsAircraft(message: RadioMessage) {
    setMessages((messages) => [...messages, message]);
    speak(`${message.reply}, ${message.id}`);
  }

  function speakAsATC(message: RadioMessage) {
    setMessages((messages) => [...messages, message]);
  }

  let socket = new WebSocket(`ws://${window.location.hostname}:9001`);
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
        speakAsATC({ id: 'ATC', reply: json.value });
        break;
      case 'reply':
        speakAsAircraft(json.value);
        break;
      case 'size':
        console.log('size', json.value);
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
      <StripBoard aircrafts={aircrafts}></StripBoard>
    </div>
  );
}