local center = { 0, 0 }

local UP = 15
local DOWN = 195
local LEFT = 285
local RIGHT = 105

local runway19R = runway({
  id = "19R",
  start = { 0, 0 },
  heading = DOWN,
  length = 7650
})
runway19R.start = vec2_from(center):move(DOWN, runway19R.length * -0.5):into()

local runway19L = runway({
  id = "19L",
  start = { 1800, 4000 },
  heading = DOWN,
  length = 8650
})

local runway28R = runway({
  id = "28R",
  start = { 0, 0 },
  heading = LEFT,
  length = 11900
})
runway28R.start = vec2_from(center):move(LEFT, runway28R.length * -0.5):move(DOWN, -1500):into()

local runway28L = runway({
  id = "28L",
  start = { 0, 0 },
  heading = LEFT,
  length = 11400
})
runway28L.start = vec2_from(runway28R.start):move(DOWN, 750):into()

local taxiway_l = taxiway({
  id = "L",
  a = vec2_from(runway19L.start):move(RIGHT, 500):into(),
  b = vec2_from(runway19L.start):move(RIGHT, 500):move(DOWN, runway19L.length):into(),
})

local taxiway_l1 = taxiway({
  id = "L1",
  a = runway19L.start,
  b = taxiway_l.a,
})

local taxiway_e = taxiway({
  id = "E",
  a = vec2_from(runway19R.start):move(LEFT, 500):into(),
  b = vec2_from(runway19R.start):move(LEFT, 500):move(DOWN, runway19R.length):into()
})

local taxiway_e1 = taxiway({
  id = "E1",
  a = taxiway_e.a,
  b = vec2_from(runway19R.start):move(RIGHT, 1200):into(),
})

local taxiway_m = taxiway({
  id = "M",
  a = vec2_from(taxiway_e1.a):move(DOWN, runway19R.length):into(),
  b = vec2_from(taxiway_e1.b):move(DOWN, runway19R.length):into(),
})

local taxiway_c = taxiway({
  id = "C",
  a = vec2_from(runway28R.start):move(LEFT, runway28R.length):move(UP, 500):into(),
  b = vec2_from(runway28R.start):move(UP, 500):into(),
})

local taxiway_c1 = taxiway({
  id = "C1",
  a = taxiway_c.b,
  b = vec2_from(runway28L.start):move(DOWN, 500):into(),
})

local taxiway_c3 = taxiway({
  id = "C3",
  a = taxiway_c.a,
  b = vec2_from(runway28R.start):move(LEFT, runway28R.length):into(),
})

local taxiway_r = taxiway({
  id = "R",
  a = vec2_from(taxiway_c3.a):move(RIGHT, 750):into(),
  b = { 0, 0 },
})
taxiway_r.b = vec2_from(taxiway_r.a):move(DOWN, 1750):into()

local taxiway_d = taxiway({
  id = "D",
  a = vec2_from(taxiway_r.a):move(RIGHT, 3000):into(),
  b = vec2_from(taxiway_r.b):move(RIGHT, 3000):move(DOWN, 500):into(),
})

local taxiway_k = taxiway({
  id = "K",
  a = vec2_from(taxiway_d.a):move(LEFT, 1000):into(),
  b = vec2_from(taxiway_d.b):move(LEFT, 1000):into(),
})

local taxiway_b = taxiway({
  id = "B",
  a = taxiway_r.b,
  b = taxiway_c1.b,
})

local taxiway_a = taxiway({
  id = "A",
  a = vec2_from(taxiway_k.b):move(LEFT, 1000):into(),
  b = vec2_from(taxiway_d.b):move(RIGHT, 1000):into(),
})

local terminal_a = terminal({
  id = "A",
  a = vec2_from(taxiway_k.b):move(LEFT, 100):into(),
  b = vec2_from(taxiway_d.b):move(RIGHT, 100):into(),
  c = { 0, 0 },
  d = { 0, 0 },
  gates = {},
  apron = { { 0, 0 }, { 0, 0 } }
})
terminal_a.c = vec2_from(terminal_a.b):move(DOWN, 2200):into()
terminal_a.d = vec2_from(terminal_a.a):move(DOWN, 2200):into()
terminal_a.apron = {
  vec2_from(terminal_a.a):midpoint(vec2_from(terminal_a.b)):into(),
  vec2_from(terminal_a.c):midpoint(vec2_from(terminal_a.d)):into()
}

local count = 6
for i = 1, count do
  local pos = vec2_from(terminal_a.a):move(DOWN, 200):lerp(vec2_from(terminal_a.d), i / count):move(RIGHT, 100):move(UP,
    100):into()
  table.insert(terminal_a.gates, gate({
    id = "A" .. i,
    pos = pos,
    heading = LEFT,
    available = true,
  }))
end
for i = 1, count do
  local pos = vec2_from(terminal_a.b):move(DOWN, 200):lerp(vec2_from(terminal_a.c), i / count):move(LEFT, 100):move(UP,
    100):into()
  table.insert(terminal_a.gates, gate({
    id = "A" .. (count + i),
    pos = pos,
    heading = RIGHT,
    available = true,
  }))
end

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
  taxiways = {
    taxiway_a,
    taxiway_b,
    taxiway_c,
    taxiway_c1,
    taxiway_c3,
    taxiway_d,
    taxiway_e,
    taxiway_e1,
    taxiway_k,
    taxiway_l,
    taxiway_l1,
    taxiway_m,
    taxiway_r,
  },
  terminals = {
    terminal_a
  },
})

return airport
