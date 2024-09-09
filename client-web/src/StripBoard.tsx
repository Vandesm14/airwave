import {
  Accessor,
  createEffect,
  createMemo,
  createSignal,
  For,
  onMount,
} from 'solid-js';
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
  let [ourFrequency] = useAtom(frequencyAtom);
  let [selectedAircraft, setSelectedAircraft] = useAtom(selectedAircraftAtom);

  let sinceCreated = formatTime(Date.now() - strip.created);
  let overtime = !(
    sinceCreated.startsWith('0') || sinceCreated.startsWith('-')
  );

  function handleMouseDown() {
    setSelectedAircraft(strip.callsign);
  }

  return (
    <tr
      classList={{
        strip: true,
        theirs: strip.frequency !== ourFrequency(),
        overtime,
        selected: selectedAircraft() === strip.callsign,
      }}
      onmousedown={handleMouseDown}
    >
      <td class="callsign">{strip.callsign}</td>
      <td class="vertical">
        <span>{strip.flight_plan[0]}</span>
        <span>{strip.flight_plan[1]}</span>
      </td>
      <td class="vertical">
        <span>FOO</span>
        <span>BAR</span>
      </td>
      <td class="vertical">
        <span class="frequency">{strip.frequency}</span>
        <span class="timer">{sinceCreated}</span>
      </td>
    </tr>
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

  let stripEntries = createMemo(() => Object.entries(strips()));

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
    <table id="stripboard">
      {stripEntries().map(([key, list]) => (
        <>
          <tr class="header">{key}</tr>
          {list.map((strip) => (
            <Strip strip={strip}></Strip>
          ))}
        </>
      ))}
    </table>
  );
}
