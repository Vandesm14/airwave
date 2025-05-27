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
runway19R.start = vec2({ 0, 0 }):move(DOWN, runway19R.length * -0.5):into()

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
runway28R.start = vec2({ 0, 0 }):move(LEFT, runway28R.length * -0.5):move(DOWN, -1500):into()

local runway28L = runway({
  id = "28L",
  start = { 0, 0 },
  heading = LEFT,
  length = 11400
})
runway28L.start = vec2(runway28R.start):move(DOWN, 750):into()

local taxiway_s = taxiway({
  id = "S",
  a = vec2(runway28L.start):move(LEFT, runway28L.length):into(),
  b = vec2(runway28R.start):move(LEFT, runway28R.length):into(),
})

local taxiway_l = taxiway({
  id = "L",
  a = vec2(runway19L.start):move(RIGHT, 500):into(),
  b = vec2(runway19L.start):move(RIGHT, 500):move(DOWN, runway19L.length):into(),
})

local taxiway_l1 = taxiway({
  id = "L1",
  a = runway19L.start,
  b = taxiway_l.a,
})

local taxiway_l2 = taxiway({
  id = "L1",
  a = vec2(runway19L.start):move(DOWN, runway19L.length):into(),
  b = taxiway_l.b,
})

local taxiway_e = taxiway({
  id = "E",
  a = vec2(runway19R.start):move(LEFT, 500):into(),
  b = vec2(runway19R.start):move(LEFT, 500):move(DOWN, runway19R.length):into()
})

local taxiway_e1 = taxiway({
  id = "E1",
  a = taxiway_e.a,
  b = vec2(runway19R.start):move(RIGHT, 1200):into(),
})

local taxiway_v = taxiway({
  id = "V",
  a = vec2(taxiway_e1.a):move(DOWN, 700):into(),
  b = vec2(taxiway_e1.b):move(DOWN, 700):into(),
})

local taxiway_m = taxiway({
  id = "M",
  a = vec2(taxiway_e1.a):move(DOWN, runway19R.length):into(),
  b = vec2(taxiway_e1.b):move(DOWN, runway19R.length):into(),
})

local taxiway_g = taxiway({
  id = "G",
  a = vec2(taxiway_m.a):move(UP, 2200):into(),
  b = vec2(taxiway_m.b):move(UP, 2200):into(),
})

local taxiway_c = taxiway({
  id = "C",
  a = vec2(runway28R.start):move(LEFT, runway28R.length):move(UP, 500):into(),
  b = vec2(runway28R.start):move(UP, 500):into(),
})

local taxiway_c1 = taxiway({
  id = "C1",
  a = taxiway_c.b,
  b = vec2(runway28L.start):move(DOWN, 500):into(),
})

local taxiway_n = taxiway({
  id = "N",
  a = vec2(taxiway_c1.a):move(LEFT, 2500):into(),
  b = vec2(taxiway_c1.b):move(LEFT, 2500):into(),
})

local taxiway_p = taxiway({
  id = "P",
  a = vec2(taxiway_c1.a):move(LEFT, 3500):into(),
  b = vec2(taxiway_c1.b):move(LEFT, 3500):into(),
})

local taxiway_c3 = taxiway({
  id = "C3",
  a = taxiway_c.a,
  b = vec2(runway28R.start):move(LEFT, runway28R.length):into(),
})

local taxiway_r = taxiway({
  id = "R",
  a = vec2(taxiway_c3.a):move(RIGHT, 1000):into(),
  b = { 0, 0 },
})
taxiway_r.b = vec2(taxiway_r.a):move(DOWN, 1750):into()

local taxiway_d = taxiway({
  id = "D",
  a = vec2(taxiway_r.a):move(RIGHT, 3000):into(),
  b = vec2(taxiway_r.b):move(RIGHT, 3000):move(DOWN, 300):into(),
})

local taxiway_k = taxiway({
  id = "K",
  a = vec2(taxiway_d.a):move(LEFT, 1000):into(),
  b = vec2(taxiway_d.b):move(LEFT, 1000):into(),
})

local taxiway_b = taxiway({
  id = "B",
  a = taxiway_r.b,
  b = taxiway_c1.b,
})

local taxiway_a = taxiway({
  id = "A",
  a = vec2(taxiway_k.b):move(LEFT, 1000):into(),
  b = vec2(taxiway_d.b):move(RIGHT, 1400):into(),
})

local count = 5
local gate_size = 175
local gate_spacing = 30
local terminal_padding = 100 + gate_spacing
local terminal_size = terminal_padding + (gate_size + gate_spacing) * count;

local terminal_a = terminal({
  id = "A",
  a = vec2(taxiway_a.b):move(LEFT, 50):into(),
  b = vec2(taxiway_a.b):move(LEFT, 50 + gate_size * 4):into(),
  c = { 0, 0 },
  d = { 0, 0 },
  gates = {},
  apron = { { 0, 0 }, { 0, 0 } }
})
terminal_a.c = vec2(terminal_a.b):move(DOWN, terminal_size):into()
terminal_a.d = vec2(terminal_a.a):move(DOWN, terminal_size):into()
terminal_a.apron = {
  vec2(terminal_a.a):midpoint(vec2(terminal_a.b)):move(UP, 1):into(),
  vec2(terminal_a.c):midpoint(vec2(terminal_a.d)):into()
}

for i = 1, count do
  table.insert(terminal_a.gates, gate({
    id = "A" .. i,
    pos = vec2(terminal_a.b):move(DOWN, terminal_padding):lerp(vec2(terminal_a.c), i / count):move(RIGHT,
      gate_size * 0.5 + gate_spacing):move(
      UP,
      gate_size * 0.5 + gate_spacing):into(),
    heading = LEFT,
    available = true,
  }))
  table.insert(terminal_a.gates, gate({
    id = "A" .. (count + i),
    pos = vec2(terminal_a.a):move(DOWN, terminal_padding):lerp(vec2(terminal_a.d), i / count):move(LEFT,
      gate_size * 0.5 + gate_spacing):move(
      UP,
      gate_size * 0.5 + gate_spacing):into(),
    heading = RIGHT,
    available = true,
  }))
end

local terminal_b = terminal({
  id = "B",
  a = vec2(terminal_a.b):move(LEFT, gate_size):into(),
  b = vec2(terminal_a.b):move(LEFT, gate_size * 5):into(),
  c = { 0, 0 },
  d = { 0, 0 },
  gates = {},
  apron = { { 0, 0 }, { 0, 0 } }
})
terminal_b.c = vec2(terminal_b.b):move(DOWN, terminal_size):into()
terminal_b.d = vec2(terminal_b.a):move(DOWN, terminal_size):into()
terminal_b.apron = {
  vec2(terminal_b.a):midpoint(vec2(terminal_b.b)):move(UP, 1):into(),
  vec2(terminal_b.c):midpoint(vec2(terminal_b.d)):into()
}

for i = 1, count do
  table.insert(terminal_b.gates, gate({
    id = "B" .. i,
    pos = vec2(terminal_b.b):move(DOWN, terminal_padding):lerp(vec2(terminal_b.c), i / count):move(RIGHT,
      gate_size * 0.5 + gate_spacing):move(
      UP,
      gate_size * 0.5 + gate_spacing):into(),
    heading = LEFT,
    available = true,
  }))
  table.insert(terminal_b.gates, gate({
    id = "B" .. (count + i),
    pos = vec2(terminal_b.a):move(DOWN, terminal_padding):lerp(vec2(terminal_b.d), i / count):move(LEFT,
      gate_size * 0.5 + gate_spacing):move(
      UP,
      gate_size * 0.5 + gate_spacing):into(),
    heading = RIGHT,
    available = true,
  }))
end

local terminal_c = terminal({
  id = "C",
  a = vec2(terminal_a.d):move(RIGHT, 100):move(DOWN, gate_size):into(),
  b = vec2(terminal_a.d):move(RIGHT, 100):move(DOWN, gate_size * 5):into(),
  c = { 0, 0 },
  d = { 0, 0 },
  gates = {},
  apron = { { 0, 0 }, { 0, 0 } }
})
terminal_c.c = vec2(terminal_c.b):move(LEFT, terminal_size):into()
terminal_c.d = vec2(terminal_c.a):move(LEFT, terminal_size):into()
terminal_c.apron = {
  vec2(terminal_c.a):midpoint(vec2(terminal_c.b)):move(RIGHT, 1):into(),
  vec2(terminal_c.c):midpoint(vec2(terminal_c.d)):into()
}

for i = 1, count do
  table.insert(terminal_c.gates, gate({
    id = "C" .. i,
    pos = vec2(terminal_c.a):move(LEFT, terminal_padding):lerp(vec2(terminal_c.d), i / count):move(DOWN,
      gate_size * 0.5 + gate_spacing):move(
      RIGHT,
      gate_size * 0.5 + gate_spacing):into(),
    heading = UP,
    available = true,
  }))
  table.insert(terminal_c.gates, gate({
    id = "C" .. (count + i),
    pos = vec2(terminal_c.b):move(LEFT, terminal_padding):lerp(vec2(terminal_c.c), i / count):move(UP,
      gate_size * 0.5 + gate_spacing):move(
      RIGHT,
      gate_size * 0.5 + gate_spacing):into(),
    heading = DOWN,
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
  center = { 600, -100 },

  runways = {
    runway19L,
    runway19R,
    runway28L,
    runway28R,
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
    taxiway_g,
    taxiway_k,
    taxiway_l,
    taxiway_l1,
    taxiway_l2,
    taxiway_m,
    taxiway_n,
    taxiway_p,
    taxiway_r,
    taxiway_s,
    taxiway_v,
  },
  terminals = {
    terminal_a,
    terminal_b,
    terminal_c,
  },
})

return airport
