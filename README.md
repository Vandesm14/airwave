# Airwave

Airwave is an ATC (Air Traffic Control) simulator based on real-world units and commands. It features:

1. Landing, taxiing, and departing operations
2. Frequency switching
3. Multiplayer
4. Tower/approach and ground radar
5. Voice control powered by OpenAI's Whisper model
6. Natural language parsing powered by GPT-4o-mini (also by OpenAI)
7. Multiple runways and airports (coming soon)

## Getting Started

### Installation

#### Prerequisites

- **Backend:** [Rust and Cargo](https://www.rust-lang.org/tools/install)
- **Frontend:** [NodeJS](https://nodejs.org/en/learn/getting-started/how-to-install-nodejs) and [pnpm](https://pnpm.io/installation) (you can install both with [volta](https://volta.sh)!)

**Step 1:**

Clone the repository

```bash
git clone https://github.com/vandesm14/airwave
```

**Step 2:**

Build the client (frontend)

```bash
pnpm i
pnpm client-web:build
cargo build --release --bin serve
```

**Step 3:**

Build the server (backend)

```bash
cargo build --release --bin server
```

### Running (Self-Hosted)

**Step 0:**

In the base directory of the project (where the README is), create a `.env` file with the following contents:

```env
OPENAI_API_KEY=<your-openai-api-key>
```

This is a requirement for the server to be able to generate reply text for incoming commands and to perform speech recognition (for voice commands).

**Step 1:**

Launch the server

```bash
cargo run --release --bin server
```

**Step 2:**

Launch the client

```bash
cargo run --release --bin serve client-web/dist
```

**Step 3:**

Open your browser of choice and go to http://localhost:8080 to connect to the client

### Running (Connecting to Another Server)

**Step 1:**

Launch the client

```bash
cargo run --release --bin serve client-web/dist
```

**Step 2:**

Open your browser of choice and go to http://localhost:8080?ws=ip, where `ip` is the IP or hostname of the server you want to connect to.

Example: http://localhost:8080?ws=127.0.0.1 or http://localhost:8080?ws=my.server.com

<!-- TODO: Remove this when we add SSL -->

_Note: Currently, Airwave does not contain a self-signed certificate so either both the server and client need to mutually use HTTP or HTTPS. Mixing HTTP and HTTPS will prevent the websockets from connecting._

## Gameplay Guide

### Handling Approaches

Aircraft will spawn just outside of the airspace at a random direction from the airport. As an approach controller, you are responsible for setting up the aircraft on approach to land.

#### Initial Vectors

Once an aircraft enters your airspace, you should set them on initial vectors to the closest runway. Also, descending them to four or five thousand feet will save you time later, so it is best to initiate their descent very early on.

You can group instructions into one transmission that looks something like this:
`SkyWest one two three four, descend and maintain four thousand feet, and turn left heading three zero zero.`

#### Sequencing

Sequencing is essential when managing a busy airspace. It is important to keep aircraft spaced far enough apart that you can coordinate takeoffs and landings with enough time between each. This also allows incoming aircraft to merge info the sequence if they fall within it, instead of having to tack them on at the end of the line.

The two main ways of ensuring correct spacing is **speed** and **heading**. The speed of an aircraft in your airspace is limited from 250 knots to 180 knots (both inclusive). You can use any speed within that range to aid in keeping your aircraft spaced evenly.

You can also utilize heading for spacing, such as having an aircraft make a slightly wider turn, giving it more distance to travel while maintaining the same speed.

#### Landing

Every runway is equipped with localizer and glideslope beacons (ILS). The max landing altitude for a runway is 4,000 ft. It is also recommended to slow down incoming aircraft to 200 knots once they are within the localizer so that they can descend down safely to the runway.

**Go-Around**

Sometimes an aircraft might be too fast or too high for an approach. In this case, it will initiate a go-around on its own (`speed: 250`, `altitude: 3000`). It will also broadcast this message to you, stating that it had to perform a go-around.

In this case, it is your job to fly the aircraft safely away from the airport and back into your landing sequence.

### Handling Ground and Taxiing

#### Post-landing

As soon as an aircraft lands on a runway, you need to taxi them off of runway, clearing it for further approaches. In this case, you can instruct them to taxi to their gate via a defined path in the airport. For example, you can tell an aircraft to taxi to the open gate `A1` via taxiways `A` and `B` like so: `<callsign>, taxi to gate alpha one via taxiways alpha then bravo.`

_Note: The "then" is to allow speech-to-text to understand the spacing of the taxiway codes, like an explicit comma, but this is not necessary when typing out commands via text._

**Important:** Ground control is currently being reimplemented, so no docs will be written for this section until development is complete.

<!-- TODO: make this section better -->

### Handling Departures

**Important:** Ground control is currently being reimplemented, so no docs will be written for this section until development is complete.

<!-- TODO: write this out -->

## Gameplay Reference

### Issuing Commands

You can use your own voice to command the aircraft in your airspace.

#### Frequency Behavior

First of all, ensure that the aircraft you would like to reach is on your frequency. If an aircraft is not on your frequency, its strip in the stripboard will be darkened. In this case, you will need to change your frequency in order to make contact with the aircraft (see: [changing frequency](#changing-frequency)).

#### Command Format

Commands are mirrored from aviation standards. For example, to instruct an aircraft to descend to 4,000 feet, you might say: "SkyWest one two three four, descend and maintain four thousand feet."

Because commands are sent through an LLM (`GPT-4o-mini`), the syntax for the commands is very loose. As long as you make your intentions clear, the LLM should understand and generate the approprate response.

**Available Commands:**

All commands should be prefixed with an aircraft's callsign, such as `SkyWest one two three four,`.

- **Heading:** `turn [direction] heading <heading>.`
  - **Description:** Changes the direction of the aircraft.
- **Speed:** `<increase|decrees> speed to <knots> knots`
  - **Description:** Changes the speed of the aircraft.
- **Altitude:** `<climb|descend> and maintain <feet> feet` or `<climb|descend> and maintain flight level <flight level>`
  - **Examples:**
    - `climb and maintain flight level one four zero`
    - `descend and maintain four thousand feet`
- **Frequency:** `tune to [kind] <frequency>`
  - **Description:** Changes the frequency that the aircraft is tuned to.
  - **Examples:**
    - `tune to tower on one one eight point five`
    - `tune to one one eight decimal six`
- **Land:** `cleared to land, runway <runway>`
  - **Description:** Clears the aircraft for ILS approach **and** landing _(this will be split into approach and landing later on)_.
  - **Examples:**
    - `cleared to land, runway zero one`
    - `cleared to land, runway one four left`
- **Go-Around:** `go around`
  - **Description:** Instructs the aircraft to abort their landing. Keep in mind when adding further commands to a go around, that the `go around` should _always_ come first such that the aircraft aborts, _then_ executes the other commands.
  - **Examples:**
    - `go around`
    - `go around and turn right three six zero`
    - `go around, turn right heading zero four zero, climb and maintain four thousand feet`
- **Takeoff:** `cleared for takeoff, runway <runway>`
  - **Description:** Clears the aircraft for takeoff, which it will then proceed to take off.
- **Resume Navigation:** `resume own navigation`
  - **Description:** For departures, clears the aircraft to climb to FL130 (13,000 feet), fly its own heading, and cruise at its desired speed.
- **Taxi:**
  - **To Runway:** `taxi to [and hold short of] runway <runway> via <taxiway> then <taxiway> then...`
    - **Description:** Instructs the aircraft to taxi to a runway via a list of taxiways.
    - **Examples:**
      - `taxi to and hold short of runway one two left via alpha then bravo then charlie`
      - `taxi to runway one two left via alpha`
  - **To Gate:** `taxi to gate <gate> via <taxiway> then <taxiway> then...`
    - **Description:** Instructs the aircraft to taxi to a gate via a list of taxiways.
    - **Examples:**
      - `taxi to gate alpha one via alpha then bravo`
  - **Hold:** `hold position`
    - **Description:** Instructs the aircraft to hold their position.
  - **Continue:** `continue taxi`
    - **Description:** Instructs the aircraft to continue taxi.

#### Text vs Voice Commands

**Text:**

Type in your command in the text box at the bottom left of the page. Then, press `enter` or click the "Send" button to submit your command.

**Voice:**

Voice commands require that you provide the webpage access to your microphone, which it prompts for when it first loads up.

To begin a command, press the `Insert` key on your keyboard and begin speaking your command. This acts like a push-to-talk switch. Once you are finished speaking your command, depress the `Insert` key. While you are transmitting, the log box will be outlined in red.

To discard your command, while holding down the `Insert` key, press and release the `Delete` key on your keyboard, then release the `Insert` key. The log box's outline will go back to white, signaling that you discarded the transmission. Discarded commands are not sent to the speech recognition service (OpenAI Whisper) and are never transmitted to the server or to OpenAI.

### Changing Frequency

<!-- TODO: do it -->
