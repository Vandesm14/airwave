import { Show } from 'solid-js';
import { useGame } from './lib/api';
import { formatTime } from './lib/lib';

export default function Points() {
  const query = useGame();

  return (
    <Show when={query.data !== undefined}>
      <div class="points">
        <p>
          <b>Landings:</b> {query.data!.points.landings} (rate: once every{' '}
          {formatTime(query.data!.points.landing_rate.rate.secs * 1000)} mins)
        </p>
        <p>
          <b>Takeoffs:</b> {query.data!.points.takeoffs} (rate: once every{' '}
          {formatTime(query.data!.points.takeoff_rate.rate.secs * 1000)} mins)
        </p>
      </div>
    </Show>
  );
}
