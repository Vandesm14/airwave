use engine::structs::Airport;

pub fn setup(_airport: &mut Airport) {
  todo!()
}

// #[allow(dead_code)]
// fn cross_roads_airport(airport: &mut Airport, airspace_size: f32) {
//   let runway_01 = Runway {
//     id: "01".into(),
//     pos: Vec2::new(airspace_size * 0.5, airspace_size * 0.5)
//       + Vec2::new(750.0, 750.0),
//     heading: 10.0,
//     length: 7000.0,
//   };

//   let runway_14 = Runway {
//     id: "14".into(),
//     pos: Vec2::new(airspace_size * 0.5, airspace_size * 0.5),
//     heading: 140.0,
//     length: 9000.0,
//   };

//   let taxiway_b = Taxiway {
//     id: "B".into(),
//     a: move_point(
//       runway_14.start(),
//       add_degrees(runway_14.heading, 90.0),
//       -500.0,
//     ),
//     b: move_point(
//       runway_14.end(),
//       add_degrees(runway_14.heading, 90.0),
//       -500.0,
//     ),
//     kind: TaxiwayKind::Normal,
//   };

//   let taxiway_c = Taxiway {
//     id: "C".into(),
//     a: runway_01.end(),
//     b: move_point(runway_01.end(), 180.0, 3600.0),
//     kind: TaxiwayKind::Normal,
//   };

//   let taxiway_hs14 = Taxiway {
//     id: "HS14".into(),
//     a: runway_14.start(),
//     b: taxiway_b.a,
//     kind: TaxiwayKind::HoldShort("14".into()),
//   };

//   let taxiway_a1 = Taxiway {
//     id: "A1".into(),
//     a: move_point(runway_14.start(), runway_14.heading - 90.0, 3250.0),
//     b: move_point(taxiway_b.a, runway_14.heading - 90.0, 3250.0),
//     kind: TaxiwayKind::Normal,
//   };

//   let taxiway_a2 = Taxiway {
//     id: "A2".into(),
//     a: move_point(runway_14.end(), runway_14.heading + 90.0, 2750.0),
//     b: move_point(taxiway_b.b, runway_14.heading + 90.0, 2750.0),
//     kind: TaxiwayKind::Normal,
//   };

//   let taxiway_a3 = Taxiway {
//     id: "A3".into(),
//     a: runway_14.end(),
//     b: taxiway_b.b,
//     kind: TaxiwayKind::Normal,
//   };

//   let taxiway_hs01 = Taxiway {
//     id: "HS01".into(),
//     a: runway_01.start(),
//     b: runway_14.end(),
//     kind: TaxiwayKind::HoldShort("01".into()),
//   };

//   let mut terminal_a = Terminal {
//     id: 'A',
//     a: move_point(taxiway_b.b, runway_14.heading + 90.0, 2750.0),
//     b: taxiway_b.b,
//     c: move_point(taxiway_b.b, runway_14.heading + 180.0, 1000.0),
//     d: move_point(
//       move_point(taxiway_b.b, runway_14.heading + 180.0, 1000.0),
//       runway_14.heading + 90.0,
//       2750.0,
//     ),
//     gates: Vec::new(),
//     apron: Line::default(),
//   };
//   terminal_a.apron = Line::new(terminal_a.a, terminal_a.b);

//   let gate_count = 8;

//   for i in 1..=gate_count {
//     terminal_a.gates.push(Gate {
//       id: format!("A{i}"),
//       heading: 0.0,
//       pos: move_point(
//         move_point(taxiway_b.b, runway_14.heading + 180.0, 1000.0),
//         runway_14.heading + 90.0,
//         2400.0 / gate_count as f32 * i as f32,
//       ),
//     });
//   }

//   airport.add_taxiway(taxiway_a1);
//   airport.add_taxiway(taxiway_a2);
//   airport.add_taxiway(taxiway_a3);

//   airport.add_taxiway(taxiway_b);
//   airport.add_taxiway(taxiway_c);

//   airport.add_taxiway(taxiway_hs14);
//   airport.add_taxiway(taxiway_hs01);

//   airport.add_runway(runway_01);
//   airport.add_runway(runway_14);

//   airport.terminals.push(terminal_a);
// }
