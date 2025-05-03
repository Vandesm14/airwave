local center = {0, 0}

local runway19R = runway({
  id = "19R",
  start = {0, 0},
  heading = 195,
  length = 7650
})
runway19R.start = vec2(table.unpack(center)):move(runway19R.heading, runway19R.length * -0.5):into()

local runway19L = runway({
  id = "19L",
  start = {1800, 4000},
  heading = 195,
  length = 8650
})

local runway28R = runway({
  id = "28R",
  start = {0, 0},
  heading = 285,
  length = 11900
})
runway28R.start = vec2(table.unpack(center)):move(runway28R.heading, runway28R.length * -0.5):move(runway19R.heading, -1500):into()

local runway28L = runway({
  id = "28R",
  start = {0, 0},
  heading = 285,
  length = 11400
})
runway28L.start = vec2(table.unpack(runway28R.start)):move(runway19R.heading, 750):into()

local airport = airport({
  id = "KSFO",
  frequencies = {
    approach = 118.6,
    departure = 118.6,
    tower = 118.6,
    ground = 118.6,
    center = 118.7,
  },
  center = center,

  runways = {
    runway19R,
    runway19L,
    runway28R,
    runway28L,
  },
  taxiways = {},
  terminals = {},
})

return airport
