use core::{cmp::Ordering, fmt, ops::ControlFlow};
use std::io::Write;

use inquire::{CustomType, Select};

static TOOLS: &'static [Tool] = &[
  Tool::Quit,
  Tool::RunwayHeading,
  Tool::WindComponent,
  Tool::TopOfDescent,
];

fn main() {
  loop {
    let Ok(tool) = Select::new("Select a tool to use", TOOLS.to_vec())
      .with_help_message("Choose an option or 'Quit' to exit")
      .prompt()
    else {
      break;
    };

    while tool.run() != ControlFlow::Break(()) {}

    print!("Press Enter to continue...");
    let _ = std::io::stdout().flush();
    if std::io::stdin().lines().next().is_none() {
      break;
    }
  }
}

#[derive(Clone, Copy)]
enum Tool {
  Quit,
  RunwayHeading,
  WindComponent,
  TopOfDescent,
}

impl Tool {
  fn run(&self) -> ControlFlow<()> {
    match self {
      Self::Quit => std::process::exit(0),
      Self::RunwayHeading => runway_heading_tool(),
      Self::WindComponent => wind_component_tool(),
      Self::TopOfDescent => top_of_descent_tool(),
    }
  }
}

impl fmt::Display for Tool {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Quit => f.write_str("Quit"),
      Self::RunwayHeading => f.write_str("Runway Heading"),
      Self::WindComponent => f.write_str("Wind Component"),
      Self::TopOfDescent => f.write_str("Top of Descent"),
    }
  }
}

fn runway_heading_tool() -> ControlFlow<()> {
  let Ok(runway_heading) = CustomType::<f64>::new("Enter the runway heading:")
    .with_default(0.0)
    .with_error_message("Please type a valid number")
    .with_help_message("Type the heading in degrees")
    .prompt_skippable()
  else {
    return ControlFlow::Continue(());
  };

  let Some(runway_heading) = runway_heading else {
    return ControlFlow::Break(());
  };

  let upwind = runway_heading;
  let downwind = (runway_heading + 180.0) % 360.0;
  let left_crosswind = (runway_heading + 270.0) % 360.0;
  let right_crosswind = (runway_heading + 90.0) % 360.0;

  println!("\tUpwind:          {upwind}");
  println!("\tDownwind:        {downwind}");
  println!("\tLeft Crosswind:  {left_crosswind}");
  println!("\tRight Crosswind: {right_crosswind}");

  ControlFlow::Break(())
}

fn wind_component_tool() -> ControlFlow<()> {
  let Ok(runway_heading) = CustomType::<f64>::new("Enter the runway heading:")
    .with_default(0.0)
    .with_error_message("Please type a valid number")
    .with_help_message("Type the heading in degrees")
    .prompt_skippable()
  else {
    return ControlFlow::Continue(());
  };

  let Some(runway_heading) = runway_heading else {
    return ControlFlow::Break(());
  };

  let Ok(wind_heading) = CustomType::<f64>::new("Enter the wind heading:")
    .with_default(0.0)
    .with_error_message("Please type a valid number")
    .with_help_message("Type the heading in degrees")
    .prompt_skippable()
  else {
    return ControlFlow::Continue(());
  };

  let Some(wind_heading) = wind_heading else {
    return ControlFlow::Break(());
  };

  let Ok(wind_speed) = CustomType::<f64>::new("Enter the wind speed:")
    .with_default(0.0)
    .with_error_message("Please type a valid number")
    .with_help_message("Type the speed in knots")
    .prompt_skippable()
  else {
    return ControlFlow::Continue(());
  };

  let Some(wind_speed) = wind_speed else {
    return ControlFlow::Break(());
  };

  let angle = normalize_deg(wind_heading - runway_heading).to_radians();
  let headwind = (-wind_speed * angle.cos()).abs();
  let crosswind = (-wind_speed * angle.sin()).abs();
  let crosswind_percent = (crosswind.abs() / wind_speed) * 100.0;

  let headwind_label = if headwind >= 0.0 {
    "Headwind"
  } else {
    "Tailwind"
  };
  let crosswind_label = match crosswind.partial_cmp(&0.0).unwrap() {
    Ordering::Less => "Left Crosswind",
    Ordering::Equal => "Crosswind",
    Ordering::Greater => "Right Crosswind",
  };

  println!("\t{headwind_label} Component:  {headwind:.2}");
  println!("\t{crosswind_label} Component: {crosswind:.2}");
  println!("\tCrosswind Percent:           {crosswind_percent:.2}%");

  ControlFlow::Break(())
}

fn top_of_descent_tool() -> ControlFlow<()> {
  let Ok(current_altitude) =
    CustomType::<f64>::new("Enter the current altitude:")
      .with_default(0.0)
      .with_error_message("Please type a valid number")
      .with_help_message("Type the speed in feet")
      .prompt_skippable()
  else {
    return ControlFlow::Continue(());
  };

  let Some(current_altitude) = current_altitude else {
    return ControlFlow::Break(());
  };

  let Ok(target_altitude) =
    CustomType::<f64>::new("Enter the target altitude:")
      .with_default(0.0)
      .with_error_message("Please type a valid number")
      .with_help_message("Type the speed in feet")
      .prompt_skippable()
  else {
    return ControlFlow::Continue(());
  };

  let Some(target_altitude) = target_altitude else {
    return ControlFlow::Break(());
  };

  let Ok(ground_speed) = CustomType::<f64>::new("Enter the ground speed:")
    .with_default(0.0)
    .with_error_message("Please type a valid number")
    .with_help_message("Type the speed in knots")
    .prompt_skippable()
  else {
    return ControlFlow::Continue(());
  };

  let Some(ground_speed) = ground_speed else {
    return ControlFlow::Break(());
  };

  let Ok(vertical_speed) = CustomType::<f64>::new("Enter the vertical speed:")
    .with_default(0.0)
    .with_error_message("Please type a valid number")
    .with_help_message("Type the speed in feet per minute")
    .prompt_skippable()
  else {
    return ControlFlow::Continue(());
  };

  let Some(vertical_speed) = vertical_speed else {
    return ControlFlow::Break(());
  };

  // Standard 3-degree descent path (approximately 300 feet per NM)
  let altitude_to_descend = current_altitude - target_altitude;

  if altitude_to_descend <= 0.0 {
    eprintln!("ERROR: Current altitude must be higher than target altitude");
    return ControlFlow::Continue(());
  }

  // Calculate time to descent in minutes
  let time_minutes = altitude_to_descend / vertical_speed;

  // Using standard 3° descent
  let distance_nm = (ground_speed / 60.0) * time_minutes;

  println!("\tAltitude to descend: {altitude_to_descend:.0} feet");
  println!("\tDistance needed:     {distance_nm:.1} NM");
  println!("\tEstimated time:      {time_minutes:.1} minutes");

  ControlFlow::Break(())
}

fn normalize_deg(deg: f64) -> f64 {
  (deg + 360.0) % 360.0
}
