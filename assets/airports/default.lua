local TAXIWAY_DISTANCE = 400

-- Runways
local runway13 = runway({
  id      = "13",
  start   = { -1500, 0 },
  heading = 135.0,
  length  = 7000.0,
})

local runway22 = runway({
  id      = "22",
  start   = { 1500, 0 },
  heading = 225.0,
  length  = 7000.0,
})

-- Taxiways A–D around runway 22
local taxiwayA = taxiway({
  id = "A",
  a  = vec2(runway22.start):move(runway22.heading + 90, TAXIWAY_DISTANCE):into(),
  b  = vec2(runway22.start):move(runway22.heading, runway22.length):move(runway22.heading + 90, TAXIWAY_DISTANCE):into(),
})

local taxiwayB = taxiway({
  id = "B",
  a  = vec2(runway22.start):move(runway22.heading - 90, TAXIWAY_DISTANCE):into(),
  b  = vec2(runway22.start):move(runway22.heading, runway22.length):move(runway22.heading - 90, TAXIWAY_DISTANCE):into(),
})

-- Taxiways C–D around runway 13
local taxiwayC = taxiway({
  id = "C",
  a  = vec2(runway13.start):move(runway13.heading + 90, TAXIWAY_DISTANCE):into(),
  b  = vec2(runway13.start):move(runway13.heading, runway13.length):move(runway13.heading + 90, TAXIWAY_DISTANCE):into(),
})

local taxiwayD = taxiway({
  id = "D",
  a  = vec2(runway13.start):move(runway13.heading - 90, TAXIWAY_DISTANCE):into(),
  b  = vec2(runway13.start):move(runway13.heading, runway13.length):move(runway13.heading - 90, TAXIWAY_DISTANCE):into(),
})

-- E1–E4 as lerps between A & B
local taxiwayE1 = taxiway({
  id = "E1",
  a  = vec2(taxiwayA.b):lerp(vec2(taxiwayA.a), 1.0):into(),
  b  = vec2(taxiwayB.b):lerp(vec2(taxiwayB.a), 1.0):into(),
})

local taxiwayE2 = taxiway({
  id = "E2",
  a  = vec2(taxiwayA.b):lerp(vec2(taxiwayA.a), 0.5):into(),
  b  = vec2(taxiwayB.b):lerp(vec2(taxiwayB.a), 0.5):into(),
})

local taxiwayE3 = taxiway({
  id = "E3",
  a  = vec2(taxiwayA.b):lerp(vec2(taxiwayA.a), 0.25):into(),
  b  = vec2(taxiwayB.b):lerp(vec2(taxiwayB.a), 0.25):into(),
})

local taxiwayE4 = taxiway({
  id = "E4",
  a  = vec2(taxiwayA.b):lerp(vec2(taxiwayA.a), 0.0):into(),
  b  = vec2(taxiwayB.b):lerp(vec2(taxiwayB.a), 0.0):into(),
})

-- F1–F4 as lerps between C & D
local taxiwayF1 = taxiway({
  id = "F1",
  a  = vec2(taxiwayC.b):lerp(vec2(taxiwayC.a), 1.0):into(),
  b  = vec2(taxiwayD.b):lerp(vec2(taxiwayD.a), 1.0):into(),
})

local taxiwayF2 = taxiway({
  id = "F2",
  a  = vec2(taxiwayC.b):lerp(vec2(taxiwayC.a), 0.5):into(),
  b  = vec2(taxiwayD.b):lerp(vec2(taxiwayD.a), 0.5):into(),
})

local taxiwayF3 = taxiway({
  id = "F3",
  a  = vec2(taxiwayC.b):lerp(vec2(taxiwayC.a), 0.25):into(),
  b  = vec2(taxiwayD.b):lerp(vec2(taxiwayD.a), 0.25):into(),
})

local taxiwayF4 = taxiway({
  id = "F4",
  a  = vec2(taxiwayC.b):lerp(vec2(taxiwayC.a), 0.0):into(),
  b  = vec2(taxiwayD.b):lerp(vec2(taxiwayD.a), 0.0):into(),
})

local TERMINAL_SIZE = 2730

-- Terminal A
local terminalA = terminal({
  id    = "A",
  a     = vec2(taxiwayE4.b):move(runway13.heading, -1):into(),
  b     = vec2(taxiwayE3.b):move(runway13.heading, -1):into(),
  c     = vec2(taxiwayE3.b):move(runway13.heading, TERMINAL_SIZE):into(),
  d     = vec2(taxiwayE4.b):move(runway13.heading, TERMINAL_SIZE):into(),
  gates = {},
  apron = {
    vec2(taxiwayE4.b):lerp(vec2(taxiwayE3.b), 0.5):into(),
    vec2(taxiwayF3.b):lerp(vec2(taxiwayF4.b), 0.5):into(),
  },
})
terminalA.apron = {
  vec2(terminalA.a):midpoint(vec2(terminalA.b)):into(),
  vec2(terminalA.c):midpoint(vec2(terminalA.d)):into()
};

do
  local total_gates = 6
  for i = 1, total_gates do
    table.insert(terminalA.gates, gate({
      id        = "A" .. i,
      pos       = vec2(terminalA.apron[1])
          :lerp(vec2(terminalA.apron[2]), i / (total_gates + 1))
          :move(inverse_degrees(runway22.heading), vec2(terminalA.a):distance(vec2(terminalA.b)) * 0.35)
          :into(),
      heading   = inverse_degrees(runway22.heading),
      available = false,
    }))
  end
  for i = 1, total_gates do
    table.insert(terminalA.gates, gate({
      id        = "A" .. (i + total_gates),
      pos       = vec2(terminalA.apron[1])
          :lerp(vec2(terminalA.apron[2]), i / (total_gates + 1))
          :move(runway22.heading, vec2(terminalA.a):distance(vec2(terminalA.b)) * 0.35)
          :into(),
      heading   = runway22.heading,
      available = false,
    }))
  end
end

-- Terminal B
local terminalB = terminal({
  id    = "B",
  a     = vec2(taxiwayF4.a):move(runway22.heading, -1):into(),
  b     = vec2(taxiwayF3.a):move(runway22.heading, -1):into(),
  c     = vec2(taxiwayF3.a):move(runway22.heading, TERMINAL_SIZE):into(),
  d     = vec2(taxiwayF4.a):move(runway22.heading, TERMINAL_SIZE):into(),
  gates = {},
  apron = { { 0, 0 }, { 0, 0 }, },
})
terminalB.apron = {
  vec2(terminalB.a):midpoint(vec2(terminalB.b)):into(),
  vec2(terminalB.c):midpoint(vec2(terminalB.d)):into()
};

do
  local total_gates = 6
  for i = 1, total_gates do
    table.insert(terminalB.gates, gate({
      id        = "B" .. i,
      pos       = vec2(terminalB.apron[1])
          :lerp(vec2(terminalB.apron[2]), i / (total_gates + 1))
          :move(inverse_degrees(runway13.heading), vec2(terminalB.a):distance(vec2(terminalB.b)) * 0.35)
          :into(),
      heading   = inverse_degrees(runway13.heading),
      available = false,
    }))
  end
  for i = 1, total_gates do
    table.insert(terminalB.gates, gate({
      id        = "B" .. (i + total_gates),
      pos       = vec2(terminalB.apron[1])
          :lerp(vec2(terminalB.apron[2]), i / (total_gates + 1))
          :move(runway13.heading, vec2(terminalB.a):distance(vec2(terminalB.b)) * 0.35)
          :into(),
      heading   = runway13.heading,
      available = false,
    }))
  end
end

-- Assemble airport
local airport = airport({
  id          = "KDEF",
  frequencies = {
    approach = 118.6,
    departure = 118.6,
    tower = 118.6,
    ground = 118.6,
    center = 118.7,
  },
  center      = vec2(taxiwayE3.a):midpoint(vec2(taxiwayF3.b)):into(),
  runways     = { runway13, runway22 },
  taxiways    = { taxiwayA, taxiwayB, taxiwayC, taxiwayD,
    taxiwayE1, taxiwayE2, taxiwayE3, taxiwayE4,
    taxiwayF1, taxiwayF2, taxiwayF3, taxiwayF4 },
  terminals   = { terminalA, terminalB },
})

return airport
