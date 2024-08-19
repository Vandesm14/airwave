import { Accessor, createEffect, createSignal, For, onMount } from 'solid-js';
import { Aircraft } from './lib/types';
import { useAtom } from 'solid-jotai';
import { frequencyAtom } from './lib/atoms';

type StripType =
  | {
      type: 'header';
      value: string;
    }
  | { type: 'strip'; value: Aircraft };

const Separator = () => <div class="separator"></div>;

type StripProps = {
  strip: StripType;
  onmousedown: () => void;
  onmousemove: () => void;
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

function Strip({ strip, onmousedown, onmousemove }: StripProps) {
  let target: string = '&nbsp;'.repeat(4);
  let frequency: number | null = null;
  let sinceCreated: string | null = null;
  let isOverTime = false;
  let [ourFrequency] = useAtom(frequencyAtom);

  if (strip.type === 'strip') {
    if (strip.value.state.type === 'landing') {
      target = `RW${strip.value.state.value.id}`;
      // TODO: update stripboard on intention/state
      // } else if (strip.value.intention.type === 'willdepart') {
      //   target = `RW${strip.value.state.value.runway.id}`;
      // } else if (strip.value.intention.type === 'departing') {
      //   target = `${strip.value.state.value.toString().padStart(3, '0')}&nbsp;`;
    } else if (strip.value.state.type === 'taxiing') {
      switch (strip.value.state.value.current.wp.type) {
        case 'gate':
          target = `G&nbsp;${strip.value.state.value.current.wp.value[1].id}`;
          break;
        case 'runway':
          target = `RW${strip.value.state.value.current.wp.value.id}`;
          break;
        case 'taxiway':
          target = `&nbsp;&nbsp;${strip.value.state.value.current.wp.value.id}`;
          break;
      }
    }

    frequency = strip.value.frequency;
    sinceCreated = formatTime(Date.now() - strip.value.created);
    isOverTime = !(
      sinceCreated.startsWith('0') || sinceCreated.startsWith('-')
    );
  }

  if (strip.type === 'strip') {
    let intention = strip.value.intention.type === 'depart' ? 'D' : 'A';
    return (
      <div
        classList={{
          strip: true,
          theirs: frequency !== ourFrequency(),
          overtime: isOverTime,
        }}
        onmousedown={onmousedown}
        onmousemove={onmousemove}
      >
        <span class="callsign">{strip.value.callsign}</span>
        <span class="intention">{intention}</span>
        <span class="target" innerHTML={target}></span>
        <span class="frequency">{frequency}</span>
        <span class="timer">{sinceCreated}</span>
      </div>
    );
  } else if (strip.type === 'header') {
    return (
      <div
        classList={{ header: true }}
        onmousedown={onmousedown}
        onmousemove={onmousemove}
      >
        {strip.value}
      </div>
    );
  }
}

function getStripsLocalStorage() {
  let item = localStorage.getItem('strips');
  if (typeof item === 'string') {
    return JSON.parse(item);
  } else {
    return [
      { type: 'header', value: 'Approach' },
      { type: 'header', value: 'Landing RW20' },
      { type: 'header', value: 'Landing RW27' },
      { type: 'header', value: 'Ground' },
      { type: 'header', value: 'Departure' },
    ];
  }
}

export default function StripBoard({
  aircrafts,
}: {
  aircrafts: Accessor<Array<Aircraft>>;
}) {
  let [dragged, setDragged] = createSignal<string | null>(null);
  let [separator, setSeparator] = createSignal<number | null>(null);
  let [strips, setStrips] = createSignal<Array<StripType>>(
    getStripsLocalStorage(),
    {
      equals: false,
    }
  );

  createEffect(() => {
    localStorage.setItem('strips', JSON.stringify(strips()));
  });

  createEffect(() => {
    // This is to prevent initial loading state from removing saved strips.
    //
    // When we first load, aircrafts() will be blank, since they havent been
    // loaded from the server yet. So, when we run the purge function to clean
    // up nonexistent callsigns from the strips, all are cleaned up.
    if (aircrafts().length > 0) {
      let allsigns = new Set(aircrafts().map((s) => s.callsign));
      setStrips((state) => {
        return state.filter((s) => {
          if (s.type === 'header') {
            return true;
          } else if (s.type === 'strip') {
            return allsigns.has(s.value.callsign);
          }
        });
      });

      for (let aircraft of aircrafts()) {
        setStrips((state) => {
          let index = state.findIndex(
            (s) => s.type === 'strip' && s.value.callsign === aircraft.callsign
          );

          if (index === -1) {
            if (aircraft.intention.type !== 'depart') {
              return [
                state[0],
                { type: 'strip', value: aircraft },
                ...state.slice(1),
              ];
            } else if (aircraft.intention.type === 'depart') {
              let takeoffIndex = state.findIndex(
                (s) => s.type === 'header' && s.value === 'Ground'
              );
              if (takeoffIndex !== -1) {
                return [
                  ...state.slice(0, takeoffIndex + 1),
                  { type: 'strip', value: aircraft },
                  ...state.slice(takeoffIndex + 1),
                ];
              } else {
                return [{ type: 'strip', value: aircraft }, ...state];
              }
            }
          } else {
            return state.map((e, i) =>
              i === index ? { type: 'strip', value: aircraft } : e
            );
          }
        });
      }
    }
  });

  function handleMouseDown(callsign: string) {
    setDragged(callsign);
  }

  function handleMouseUp() {
    setStrips((strips) => {
      let callsign = dragged();
      let fromIndex = strips.findIndex(
        (s) => s.type === 'strip' && s.value.callsign === callsign
      );
      let toIndex = separator();

      if (fromIndex !== -1) {
        let newStrips = [];
        for (let i = 0; i < strips.length; i++) {
          if (i !== fromIndex) {
            newStrips.push(strips[i]);
          }

          if (i === toIndex) {
            newStrips.push(strips[fromIndex]);
          }
        }

        return newStrips;
      } else {
        return strips;
      }
    });

    resetDrag();
  }

  function handleMouseMove(index: number) {
    if (dragged()) {
      setSeparator(index);
    }
  }

  function resetDrag() {
    setDragged(null);
    setSeparator(null);
  }

  return (
    <div
      id="stripboard"
      onmouseleave={() => resetDrag()}
      onmouseup={() => handleMouseUp()}
    >
      {strips().map((s, i) => (
        <>
          <Strip
            strip={s}
            onmousedown={() => {
              s.type === 'strip' ? handleMouseDown(s.value.callsign) : {};
            }}
            onmousemove={() => {
              handleMouseMove(i);
            }}
          ></Strip>
          {i === separator() ? <Separator /> : null}
        </>
      ))}
    </div>
  );
}
