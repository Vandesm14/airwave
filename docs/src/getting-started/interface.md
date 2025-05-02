# Interface

Airwave has four major parts of the interface:

1. Radar
2. Frequency Panel
3. Stripboard
4. Chatbox
5. Flight Panel

## Radar

To view aircraft in the air and on the ground, you will utilize the radar. The radar is the paramount for situational awareness, if not already obvious. Most of your time will be spent looking at the radar and issuing commands to aircraft based on their state and position on it.

Aircraft arriving at your airport will be displayed green, aircraft leaving your airport displayed blue, and if you've selected an aircraft, it will be displayed yellow. If an aircraft is not on your frequency, it will be displayed as a simple grey dot, without any further information (unless selected). Finally, if two or more aircraft encounter a TCAS RA (collision avoidance resolution), the involved aircraft will be colored red.

This color pattern of green, blue, yellow, and red carries throughout the other tools in Airwave such as the stripboard and chatbox.

### Controls

You can click and drag to move the view around, and use the scroll wheel to zoom in and out.

When you zoom in far enough into an airport, the radar type will switch from air to ground, allowing you to view taxiways, terminals, and gates. You can also use `PageUp` and `PageDown` to switch between air and ground views respectively.

## Frequency Panel

This panel is second to the radar. Here you will manage the frequencies you occupy. It can hold up to 10 total slots, which you can switch between using the number row of your keyboard. To the right of every frequency box is a dropdown selector to choose a named frequency set up by the server config.

## Stripboard

The stripboard is the core part of your workflow. It allows you to organize aircraft into discreet steps, see aircraft that need attention in the inbox, and reference when giving commands. You can click & drag headers and strips as you please. You can also rename, add, and remove headers to customize the stripboard to your workflow.

The only hard-coded header is the `Inbox` as it is required for providing you with incoming strips that you are given control over.

The stripboard includes an options section (toggleable via the `Opts` button) that tells it which kind of aircraft you are looking to control. For example, if you check the "Approach" box, Airwave will create strips for all aircraft on approach, keeping your stripboard up to date with approach operations.

By default, the stripboard includes all major segments of flight: approach, departure, landing, takeoff, parked, and ground. This is to allow any player to jump in and customize as they'd like while ensuring all aircraft within their control are added to the stripboard. However, these defaults can easily be changed and will be saved to the client storage between sessions.

The `Clr` button will remove any aircraft that is in a state of flight that is not checked by options. This allows players to quickly discard strips that they no longer have control over, or clear the stripboard between game sessions. Headers and aircraft that are still under your control will not be removed and will stay the way you might have them sorted.

## Chatbox

The chatbox displays a log of all of your messages between aircraft and allows you to toggle between your current frequency and all frequencies. The chatbox includes an input box to send text commands instead of using your voice.

ATC messages are shown in red and aircraft messages are shown in green (blue is not used here). You can click a callsign in the log to select the aircraft that sent the message and the callsign will be displayed as yellow (because of selection).