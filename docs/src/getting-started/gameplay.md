## Handling Approaches

By default, both approaches and departures will be enabled. Once an arriving aircraft enters your airspace, it will be categorized as an approach. If the stripboard is set to accept approaches, you will see a strip come up as the aircraft enters your airspace.

You will also see the aircraft call out to approach in the chatbox (audible message if TTS is enabled), reporting its position.

### Initial Vectors

Once an aircraft enters your airspace, you should set them on initial vectors to the active runway (of your choosing). Also, descending them to four or five thousand feet will save you time later, so it is best to initiate their descent very early on.

You can group instructions into one transmission that looks something like this:
`SkyWest Twelve Thirty Four, descend and maintain four thousand feet, and turn left heading three zero zero.`

This way, you can ensure that they are at the correct altitude and are pointed towards the general direction for an approach.

### Sequencing

Sequencing is essential when managing a busy airspace. It is important to keep aircraft spaced far enough apart that you can coordinate takeoffs and landings with enough time between each. This also allows incoming aircraft to merge info the sequence if they fall within it, instead of having to tack them on at the end of the line.

The two main ways of ensuring correct spacing is **speed** and **heading**. The speed of an aircraft in your airspace is limited from 250 knots to 180 knots (both inclusive). You can use any speed within that range to aid in keeping your aircraft spaced evenly. You can also utilize heading for spacing, such as having an aircraft make a slightly wider turn, giving it more distance to travel while maintaining the same speed.

### Landing

Every runway is equipped with localizer and glideslope beacons (ILS). The max landing altitude for a runway is 4,000 ft. It is also recommended to slow down incoming aircraft to 200 knots once they are within the localizer so that they can descend down safely to the runway.

**Go-Around**

If sequenced improperly, an aircraft might be too fast or too high for an approach. In this case, it will initiate a go-around on its own (`speed: 250`, `altitude: 4000`). It will also broadcast this message to you, stating that it had to perform a go-around (you will see this in the chatbox, and hear it if TTS is enabled).

In this case, it is your job to vector the aircraft for another approach as you would normally, ensuring that it doesn't come too close to any departing aircraft (always keep an eye out!).

## Handling Ground and Taxiing

### Post-landing

As soon as an aircraft lands on a runway, you need to taxi them off of runway, clearing it for further approaches. In this case, you instruct them to taxi to their gate via a defined path in the airport. For example, you can tell an aircraft to taxi to the open gate `A1` via taxiways `A` and `B` like so: `<callsign>, taxi to gate alpha one via alpha then bravo.`

_Note: The "then" is not required. It is a pattern that I personally use as it's quicker than pausing between instructions._

If no gates are open, you can still get the aircraft off of the runway by instructing them to taxi to and optionally hold short of another taxiway: `<callsign> taxi to and hold short of alpha.`

### Handling Departures

When a parked aircraft is ready for departure, it will show up on the stripboard (as long as the "parked" and "ground" options are checked) and its timer will be greater than zero. A departure with a timer less than zero indicates that it is currently waiting (boarding, refueling, etc) and is not ready for taxi.

Once a departure is ready, you will need to taxi them to a runway.
