import { createSignal } from 'solid-js';
import './App.scss';

function App() {
  const [count, setCount] = createSignal(0);

  return (
    <>
      <canvas></canvas>
    </>
  );
}

export default App;
