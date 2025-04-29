import './Flights.scss';
import { createSignal, Show } from 'solid-js';

export default function Flights() {
  const [show, setShow] = createSignal(false);
  // const [selectedAircraft] = useAtom(selectedAircraftAtom);

  // const aircrafts = useAircraft();
  // const world = useWorld();
  // const acceptFlight = useAcceptFlight();
  // const rejectFlight = useRejectFlight();

  // const arrivals = createMemo(() =>
  //   sortByDistance(
  //     aircrafts.data.filter(
  //       (a) =>
  //         !a.accepted &&
  //         a.flight_plan.arriving === HARD_CODED_AIRPORT &&
  //         a.segment !== 'parked'
  //     ),
  //     world.data
  //   )
  // );
  // const departures = createMemo(() =>
  //   aircrafts.data.filter(
  //     (a) => !a.accepted && a.flight_plan.departing === HARD_CODED_AIRPORT
  //   )
  // );

  // function acceptSelected() {
  //   acceptFlight.mutate(selectedAircraft());
  // }

  // function rejectSelected() {
  //   rejectFlight.mutate(selectedAircraft());
  // }

  // function acceptArrivals() {
  //   arrivals().forEach((a) => acceptFlight.mutate(a.id));
  // }

  // function acceptDepartures() {
  //   departures().forEach((a) => acceptFlight.mutate(a.id));
  // }

  return (
    <div class="container border">
      <div class="flights">
        <Show when={!show()}>
          <button onClick={() => setShow(true)}>Flights</button>
        </Show>
        <Show when={show()}>
          <button onClick={() => setShow(false)}>Close</button>
          {/* <hr />
          <button onClick={acceptSelected}>Accept Selected</button>
          <button onClick={rejectSelected}>Reject Selected</button>
          <h2>Arrivals ({arrivals().length})</h2>
          <button onClick={acceptArrivals}>Accept Arrivals</button>
          <div class="list">
            {arrivals().map((f) => (
              <FlightItem flight={f} />
            ))}
          </div>
          <hr />
          <h2>Departures ({departures().length})</h2>
          <button onClick={acceptDepartures}>Accept Departures</button>
          <div class="list">
            {departures().map((f) => (
              <FlightItem flight={f} />
            ))}
          </div> */}
        </Show>
      </div>
    </div>
  );
}
