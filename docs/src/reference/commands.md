# Commands

Commands are mirrored from common [ATC phraseology](https://wiki.flightgear.org/ATC_phraseology). For example, to instruct an aircraft to descend to 4,000 feet, you might say: "SkyWest twelve thirty four, descend and maintain four thousand feet."

Because commands are sent through an LLM (`GPT-4o-mini`), the syntax for the commands is very loose. As long as you make your intentions clear, the LLM _should_ understand and generate the approprate response.

**Available Commands:**

All commands should be prefixed with an aircraft's callsign, such as `SkyWest Twelve Thirty Four,` or `SkyWest One Two Three Four.

## Air

### Altitude

**Normal Syntax:**

- `<climb|descend> [and maintain|to] <feet> feet`
- `<climb|descend> and maintain flight level <flight level>`

**Shorthand Syntax:** `a`, `alt`, `altitude`: `descend and maintain 4000 feet` = `a 040`

Changes the altitude of the aircraft.

**Examples:**

- `climb and maintain flight level one four zero`
- `descend and maintain four thousand feet`

### Turn

**Normal Syntax:** `turn [direction] heading <heading>.`

**Shorthand Syntax:** `t`, `turn`, `h`, `heading`: `turn right heading 360` = `t 360`

Changes the direction of the aircraft.

### Speed

**Normal Syntax:** `maintain <knots> knots`

**Shorthand Syntax:** `s`, `spd` `speed`: `maintain 250 knots` = `s 250`

Changes the speed of the aircraft.

### Direct

**Normal Syntax:** `direct <waypoint>`

**Shorthand Syntax:** `d`, `dt`, `direct`: `direct waypoint` = `d waypoint`

Changes the aircraft's flight plan to fly directly to a waypoint (that is already in the flight plan).

## Frequency

### Contact Named Controller

**Normal Syntax:** `contact <controller> [on] [frequency]`

**Shorthand Syntax:** `f`, `freq`, `frequency`, `tune`, `contact`: `contact departure` = `f departure`

Changes the frequency the aircraft is tuned to, using the frequency configured for that controller in the server config.

**Examples:**

- `contact departure`

### Contact Frequency

**Normal Syntax:**

- `contact <controller> on <frequency>`
- `tune to <frequency>`

**Shorthand Syntax:** (same as above): `contact departure on 123.4` = `f 123.4`

Changes the frequency the aircraft is tuned to, forcing a specific frequency.

**Examples:**

- `tune to 123.4`
- `contact departure on 123.4`

## Approach and Landing

### Cleared to Land

**Normal Syntax:**

- `cleared to land, runway <runway>`
- `cleared ILS approach runway <runway>`

**Shorthand Syntax:** `l`, `cl`, `land`: `cleared to land runway 22L` = `l 22L`

Clears the aircraft for ILS approach **and** landing[^1].

**Examples:**

- `cleared to land, runway zero one`
- `cleared to land, runway one four left`

### Go Around

**Normal Syntax:** `go around`

**Shorthand Syntax:** `g`, `ga`, `go`: `go around` = `g`

Instructs the aircraft to abort their landing. Any further commands should follow after `go around`.

**Examples:**

- `go around`
- `go around and turn right three six zero`
- `go around, turn right heading zero four zero, climb and maintain four thousand feet`

## Taxi

### Taxi

**Normal Syntax:** `taxi to [and hold short of] <runway|gate|taxiway> [via] <taxiway> [then] <taxiway> [then]...`

**Shorthand Syntax:** `tx`, `taxi`: `taxi to and hold short of runway one two left via alpha then bravo` = `tx short 12L via a b`

Instructs the aircraft to taxi to a runway via a list of taxiways.

**Examples:**

- `taxi to and hold short of runway one two left via alpha then bravo then charlie`
- `taxi to runway one two left via alpha`
- `taxi to gate A1 via alpha bravo charlie`
- `taxi to and hold short of alpha`

**Shorthand Examples:**

- `tx short 12L via a b c`
- `tx short 12L via a`
- `tx gate A1 via a b c`
- `tx short a`

### Hold

**Normal Syntax:** `hold position`

**Shorthand Syntax:** `th`, `hold`, `stop`: `hold position` = `th`

Instructs the aircraft to hold position.

### Continue

**Normal Syntax:** `continue taxi`

**Shorthand Syntax:** `c`, `tc`, `continue`: `continue taxi` = `c`

Instructs the aircraft to continue taxiing.

## Takeoff

Both takeoff and line-up clearances can be given while an aircraft is holding at or taxiing to a runway.

### Cleared for Takeoff

**Normal Syntax:** `cleared for takeoff, runway <runway>`

**Shorthand Syntax:** `ct`, `to`, `takeoff`: `cleared for takeoff runway 22L` = `ct 22L`

Clears the aircraft for takeoff.

### Line Up and Wait

**Normal Syntax:** `line up and wait, runway <runway>`

**Shorthand Syntax:** `lu`, `line`, `wait`: `line up and wait runway 22L` = `lu 22L`

Tells the aircraft to taxi onto the runway, line up (to its heading), and wait (until instructed for takeoff).

## Departures

### Resume As Filed

**Normal Syntax:** `resume as filed`

**Shorthand Syntax:** `r`, `raf`, `resume`, `own`: `resume as filed` = `r`

Clears the aircraft (departure) to climb to their filed altitude and follow their departure waypoints.

**Note:** This will happen automatically as an aircraft departs to save on workload, but can be overridden by changing its speed, heading, or altitude, and resumed by issuing this command.

## Miscellaneous

### Ident

**Normal Syntax:** `ident` or `identify`

**Shorthand Syntax:** `i`, `id`, `ident`: `ident` = `i`

Selects the aircraft on the client.

[^1]: Airwave combines the clearence procedures for approaches and landings such that they are interchangable. Once an aircraft is cleared for approach, it does not need to be cleared to land. Thus, the phraseology can be used where "cleared to land runway 22L" and "cleared ILS approach runway 22L" will mean the same thing.
