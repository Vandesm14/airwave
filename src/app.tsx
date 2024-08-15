import { onMount } from 'solid-js';
import { init } from '.';

export default function App() {
  let canvas;

  onMount(init);

  return (
    <div class="radar">
      <div id="chatbox"></div>
      <canvas id="canvas" ref={canvas}></canvas>
    </div>
  );
}

{
  /* <template id="message-template">
<div class="message">
  <span class="callsign">{{callsign}}</span>
  <span class="text">{{text}}</span>
</div>
</template> */
}
