local airport = airport({
  id = "KSFO",
  frequencies = {
    approach = 118.6,
    departure = 118.6,
    tower = 118.6,
    ground = 118.6,
    center = 118.7,
  },
  center = {0.0, 0.0},

  runways = {
    runway({
      id = "13",
      pos = {0.0, 0.0},
      heading = 135.0,
      length = 7000.0,
    })
  },
  taxiways = {
    taxiway({
      id = "A",
      a = {0.0, 0.0},
      b = {0.0, 0.0},
    }),
    taxiway({
      id = "B",
      a = {0.0, 0.0},
      b = {0.0, 0.0},
    })
  },
  terminals = {},
})

return airport
