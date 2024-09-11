**The Radar**

You can view aircraft within your airspace with the radar. You can click and drag to move, and use scroll wheel to zoom in and out.

**Objects**

In the center of your screen, you should see your airport. The large circle around your airport is the airspace that you can control. From this view, you can control inbound and outbound aircraft.

The blue and grey lines stemming from the runways indicates the localizer beacon of the ILS and its range. The blue circles on the localizer line show the recommended approach glideslope from 4,000 feet the furthest away from the runway, then 3,000 feet at the next circle, then 2,000 feet at the last.

There may also be named waypoints within your airspace, which are useful for routing aircraft to locations and through routes.

**Ground View**

If you zoom into your airport, after a certain point, the radar will switch from air to ground mode. In ground mode, you can view the runways, taxiways, terminals, and gates of your airport. All of the objects in this view are labeled. The gates of your airport are represented as a red dot with their respective label next to them.

**Controlling a Departure**

**Frequency and Airspace Selection**

First of all, check that you have the correct airspace selected in the dropdown underneath the stripboard. The name of your airspace will be above the airspace circle in the air view of the radar. This ensures that the frequency selector is using the frequencies for your airspace. Next, select the `ground` frequency so you can command aircraft on the ground.

**The Stripboard**

The stripboard will be the core of your workflow. It separates aircraft based on their state, and sorts them by the aircraft that have been active the longest (lower is older). The stripboard will move aircraft to a section automatically when their state changes. The strips and headers on your stripboard is also controlled by the airspace selector, only showing you the aircraft that you have control over.

If an aircraft is on a different frequency from you, its strip will be grayed out, indicating that you aren't in contact with it. You can resolve this by switching to the frequency that the aircraft is tuned to either manually or through the frequency dropdown.

**Identification on Ground**

In the ground section of the stripboard, you should see an aircraft on your frequency. If you click that aircraft, it will be colored yellow to indicate your selection. In the ground view, you should see the blip of the aircraft holding at the gate, which is also colored yellow for easy identification. Once you have identified them on the radar, you can see that the aircraft's blip includes their callsign and their current speed (will be zero, they are parked).

**Identifying and Preparing Departure**

Next, identify the airport that the aircraft is departing to. The top airport of a strip is where it is departing from, and the bottom is where the aircraft is departing to. You should see your airport at the top of the aircraft's strip, and another at the bottom.

When aircraft depart, it is best for them to depart from a runway that best matches the direction of their departure. So, zoom out, back into the airspace view, and look for the airport that they are departing to. Now, keep a note of that direction and zoom back into ground view. Find the runway on your airport that best matches the direction to their arrival airport.

**Taxiing**

Now that you have a runway you would like to take them off from, find the taxiway that is closest to the beginning of the runway. This ensures that the aircraft will not only taxi to the runway, but taxi to the end so they can take off from the beginning of it.

After you have identified the runway and the taxiway off of the end of it, instruct the aircraft to taxi.

**Issuing Commands**

You can use the input box at the bottom left to issue commands via text, or, if you have your microphone enabled, use the `Talk` button at the bottom right or the `Insert` key on your keyboard as a push-to-talk key. To discard a voice command while you are recording, press the `Delete` key on your keyboard before you then release the `Insert` key.

When addressing an aircraft, you need to prefix your message with their callsign. You can either use the raw callsign letters, such as `SKW1234` or their (more realistic) full callsign `Skywest 1234`. This ensures that your command is received by the correct aircraft.

**Taxi Commands**

So, to issue a taxi command, say `<callsign>, taxi to and hold short of runway <runway> via <taxiway>`. For example, you could speak or type `Skywest one two three four, taxi to and hold short of runway two zero via alpha four`.

After you have issued your command, the aircraft will reply back to you. It will either read the instructions back as you told them, or it will prompt you to re-command them by saying `Say again, <their callsign>` .

**Chatbox and Speech to Text**

If you are using the speech-to-text feature, the chatbox allows you to see the raw transcribed output, allowing you to adjust your pronunciation such that the aircraft correctly understands what you are saying.

**Validation**

Once the aircraft replies back to you, it is important to not only ensure that they have understood the command and read it back correctly, but that they have executed it correctly. In the case of taxiing, you should see the aircraft begin to leave their gate and follow the path you have given them.

**Path**

When an aircraft on the ground is selected, their taxi path will be highlighted in yellow, allowing you to verify the waypoints. Red waypoints mean that the aircraft will stop before them, waiting for you to issue a `continue taxi` command.

As the aircraft is taxiing, you can see their current position, such as `TXWY A4`,  in the third info block of the stripboard.

**Taxi Modification**

While an aircraft is taxiing, you can instruct them to stop at any time by issuing a `hold position` command. To resume their taxi, issue a `continue taxi` command. You can also give them new taxi instructions, which will override their path and begin a new taxi.

**Holding Short**

Once your aircraft is holding short of the runway, you will see their strip move to the **Takeoff** section of the stripbaord. Now that the aircraft is holding short, verify that there are no aircraft close to landing on the runway and that no other aircraft will cross the runway during takeoff.

**Takeoff**

To start, command the aircraft to `continue taxi`, allowing it to taxi onto the runway and line up. Next, once you have verified that the runways is clear and the aircraft is ready for takeoff, you can clear them to take off using this command: `cleared for takeoff, runway <runway>`. The aircraft will readback your takeoff clearance and begin its takeoff.

**Departure**

Now, zoom out on the radar to view your airspace. After the aircraft has taken off of the runway and is above 1,000 feet, it will appear on the radar. After the aircraft is about a third or halfway from your airport to the edge of your airspace circle, or your aircraft reaches 3,000 feet, you can command them to `resume as filed`. This instructs the aircraft to climb to its intended altitude, speed up to its intended cruising speed, and fly direct to the airport (heading).