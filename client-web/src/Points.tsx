import { Show } from 'solid-js';
import { usePoints } from './lib/api';
import { formatTime } from './lib/lib';

export default function Points() {
  const query = usePoints();

  return (
    <Show when={query.data !== undefined}>
      <div class="points">
        <p>
          <b>Landings:</b> {query.data!.landings} (rate: once every{' '}
          {formatTime(query.data!.landing_rate.rate.secs * 1000)} mins)
        </p>
        <p>
          <b>Takeoffs:</b> {query.data!.takeoffs} (rate: once every{' '}
          {formatTime(query.data!.takeoff_rate.rate.secs * 1000)} mins)
        </p>
      </div>
    </Show>
  );
}
