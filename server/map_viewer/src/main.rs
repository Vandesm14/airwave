// use nannou::{
//   color::*,
//   prelude::{App, Frame, Update},
// };

// fn main() {
//   nannou::app(model).update(update).simple_window(view).run();
// }

// #[derive(Debug, Clone, PartialEq, Default)]
// pub struct Model {
//   entities: Vec<ParsedEntity>,
// }

// fn model(_app: &App) -> Model {
//   Model { entities: vec![] }
// }
// fn update(_app: &App, _model: &mut Model, _update: Update) {}

// fn view(app: &App, model: &Model, frame: Frame) {
//   frame.clear(BLACK);
// }

use map_viewer::ParsedEntity;

fn main() {
  let entities: Vec<ParsedEntity> =
    ron::de::from_bytes(include_bytes!("../../airport.ron")).unwrap();

  println!("{:?}", entities);
}
