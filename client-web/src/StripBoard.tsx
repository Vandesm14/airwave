import { Accessor, createEffect, createSignal, For, onMount } from 'solid-js';
import { Aircraft } from './lib/types';
import { useAtom } from 'solid-jotai';
import { frequencyAtom, selectedAircraftAtom } from './lib/atoms';

type Strips = Record<string, Array<Aircraft>>;

const Separator = () => <div class="separator"></div>;

type StripProps = {
  strip: Aircraft;
};

function formatTime(duration: number): string {
  const isNegative = duration < 0;
  let absDuration = Math.abs(duration);
  let durationSeconds = Math.floor(absDuration / 1000);
  let seconds = (durationSeconds % 60).toString().padStart(2, '0');
  let minutes = Math.floor(durationSeconds / 60)
    .toString()
    .padStart(2, '0');
  let timeString = `${minutes}:${seconds}`;
  if (isNegative) {
    timeString = `-${timeString}`;
  }

  return timeString;
}

function Strip({ strip }: StripProps) {
  let target: string = '&nbsp;'.repeat(4);
  let frequency: number | null = null;
  let sinceCreated: string | null = null;
  let isOverTime = false;
  let [ourFrequency] = useAtom(frequencyAtom);
  let [selectedAircraft, setSelectedAircraft] = useAtom(selectedAircraftAtom);

  function handleMouseDown() {
    setSelectedAircraft(strip.callsign);
  }

  return (
    <div
      classList={{
        strip: true,
        theirs: frequency !== ourFrequency(),
        overtime: isOverTime,
        selected: selectedAircraft() === strip.callsign,
      }}
      onmousedown={handleMouseDown}
    >
      <span class="callsign">{strip.callsign}</span>
      <span class="target" innerHTML={target}></span>
      <span class="frequency">{frequency}</span>
      <span class="timer">{sinceCreated}</span>
    </div>
  );
}

// function getStripsLocalStorage() {
//   let item = localStorage.getItem('strips');
//   if (typeof item === 'string') {
//     return JSON.parse(item);
//   } else {
//     return [
//       { type: 'header', value: 'Approach' },
//       { type: 'header', value: 'Landing RW20' },
//       { type: 'header', value: 'Landing RW27' },
//       { type: 'header', value: 'Ground' },
//       { type: 'header', value: 'Departure' },
//     ];
//   }
// }

export default function StripBoard({
  aircrafts,
}: {
  aircrafts: Accessor<Array<Aircraft>>;
}) {
  let [strips, setStrips] = createSignal<Strips>(
    {},
    {
      equals: false,
    }
  );

  // createEffect(() => {
  //   localStorage.setItem('strips', JSON.stringify(strips()));
  // });

  createEffect(() => {
    // This is to prevent initial loading state from removing saved strips.
    //
    // When we first load, aircrafts() will be blank, since they havent been
    // loaded from the server yet. So, when we run the purge function to clean
    // up nonexistent callsigns from the strips, all are cleaned up.
    if (aircrafts().length > 0) {
      // TODO: implement stripboard logic
      let strips: Strips = {
        Approach: [],
        Tower: [],
        Ground: [],
        Departure: [],
      };

      for (let aircraft of aircrafts()) {
        strips['Approach'].push(aircraft);
      }

      setStrips(strips);
    }
  });

  return (
    <div id="stripboard">
      {Object.entries(strips()).map(([key, list]) => (
        <>
          <div class="header">{key}</div>
          {list.map((strip) => (
            <Strip strip={strip}></Strip>
          ))}
        </>
      ))}
    </div>
  );
}
