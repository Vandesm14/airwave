# Commands

Commands are mirrored from common [ATC phraseology](https://wiki.flightgear.org/ATC_phraseology). For example, to instruct an aircraft to descend to 4,000 feet, you might say: "SkyWest twelve thirty four, descend and maintain four thousand feet."

Because commands are sent through an LLM (`GPT-4o-mini`), the syntax for the commands is very loose. As long as you make your intentions clear, the LLM _should_ understand and generate the approprate response.

**Available Commands:**

All commands should be prefixed with an aircraft's callsign, such as `SkyWest Twelve Thirty Four,` or `SkyWest One Two Three Four.

## Air

### Turn

**Normal Syntax:** `turn [direction] heading <heading>.`  
**Shorthand Syntax:** `TODO`

Changes the direction of the aircraft.

### Speed

**Normal Syntax:** `maintain <knots> knots`  
**Shorthand Syntax:** `TODO`

Changes the speed of the aircraft.

### Altitude

**Normal Syntax:**

- `<climb|descend> [and maintain|to] <feet> feet`
- `<climb|descend> and maintain flight level <flight level>`

**Shorthand Syntax:** `TODO`

Changes the altitude of the aircraft.

**Examples:**

- `climb and maintain flight level one four zero`
- `descend and maintain four thousand feet`

## Frequency

### Contact Named Controller

**Normal Syntax:** `contact <controller> [on] [frequency]`  
**Shorthand Syntax:** `TODO`

Changes the frequency the aircraft is tuned to, using the frequency configured for that controller in the server config.

**Examples:**

- `contact departure`

### Contact Frequency

**Normal Syntax:**

- `contact <controller> on <frequency>`
- `tune to <frequency>`

**Shorthand Syntax:** `TODO`

Changes the frequency the aircraft is tuned to, forcing a specific frequency.

**Examples:**

- `tune to 123.4`
- `contact departure on 123.4`

## Approach and Landing

### Cleared to Land

**Normal Syntax:**

- `cleared to land, runway <runway>`
- `cleared ILS approach runway <runway>`

**Shorthand Syntax:** `TODO`

Clears the aircraft for ILS approach **and** landing.

**Examples:**

- `cleared to land, runway zero one`
- `cleared to land, runway one four left`

### Go Around

**Normal Syntax:** `go around`  
**Shorthand Syntax:** `TODO`

Instructs the aircraft to abort their landing. Any further commands should follow after `go around`.

**Examples:**

- `go around`
- `go around and turn right three six zero`
- `go around, turn right heading zero four zero, climb and maintain four thousand feet`

## Taxi

### Taxi to Runway

**Normal Syntax:** `taxi to [and hold short of] runway <runway> via <taxiway> then <taxiway> then...`  
**Shorthand Syntax:** `TODO`

Instructs the aircraft to taxi to a runway via a list of taxiways.

**Examples:**

- `taxi to and hold short of runway one two left via alpha then bravo then charlie`
- `taxi to runway one two left via alpha`

### Taxi to Gate

**Normal Syntax:** `taxi to gate <gate> via <taxiway> then <taxiway> then...`  
**Shorthand Syntax:** `TODO`

Instructs the aircraft to taxi to a gate via a list of taxiways.

**Examples:**

- `taxi to gate alpha one via alpha then bravo`

### Hold

**Normal Syntax:** `hold position`  
**Shorthand Syntax:** `TODO`

Instructs the aircraft to hold position.

### Continue

**Normal Syntax:** `continue taxi`  
**Shorthand Syntax:** `TODO`

Instructs the aircraft to continue taxiing.

## Takeoff

Both takeoff and line-up clearances can be given while an aircraft is holding at or taxiing to a runway.

### Cleared for Takeoff

**Normal Syntax:** `cleared for takeoff, runway <runway>`  
**Shorthand Syntax:** `TODO`

Clears the aircraft for takeoff.

### Line Up and Wait

**Normal Syntax:** `line up and wait, runway <runway>`  
**Shorthand Syntax:** `TODO`

Tells the aircraft to taxi onto the runway, line up (to its heading), and wait (until instructed for takeoff).

## Departures

### Resume Own Navigation

**Normal Syntax:** `resume own navigation`  
**Shorthand Syntax:** `TODO`

Clears the aircraft (departure) to climb to their filed altitude and follow their departure waypoints.

**Note:** This will happen automatically as an aircraft departs to save on workload, but can be overridden by changing its speed, heading, or altitude, and resumed by issuing this command.

[^1]: Airwave combines the clearence procedures for approaches and landings such that they are interchangable. Once an aircraft is cleared for approach, it does not need to be cleared to land. Thus, the phraseology can be used where "cleared to land runway 22L" and "cleared ILS approach runway 22L" will mean the same thing.
