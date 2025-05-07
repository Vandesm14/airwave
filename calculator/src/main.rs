use std::cmp::Ordering;

use clap::{Parser, Subcommand};
use inquire::{CustomType, Select, Text};

/// ATC Utility CLI
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
  #[command(subcommand)]
  command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
  RunwayHeading,
}

fn main() {
  let tools = vec![
    "Runway Heading Expander",
    "Wind Component Calculator",
    "Top of Descent Calculator",
  ];
  while let Some(tool) = select_tool(&tools) {
    run_tool(tool);
  }
}

fn select_tool<'a>(tools: &'a [&'a str]) -> Option<&'a str> {
  Select::new("Select a tool", tools.to_vec()).prompt().ok()
}

fn run_tool(tool: &str) {
  let tool_function = match tool {
    "Runway Heading Expander" => runway_heading_tool,
    "Wind Component Calculator" => wind_component_tool,
    "Top of Descent Calculator" => top_of_descent_tool,
    _ => return,
  };

  loop {
    let result = tool_function();
    println!(
      "\n{}\n\nPress Enter to redo, 'q' to quit, 'm' for menu.",
      result
    );
    match handle_menu_navigation() {
      MenuAction::Menu => return,
      MenuAction::Redo => {}
    }
  }
}

#[derive(Debug, Clone, Copy)]
enum MenuAction {
  Redo,
  Menu,
}
fn handle_menu_navigation() -> MenuAction {
  let input: String = Text::new("Enter your choice (q/m/Enter):")
    .prompt()
    .unwrap_or_default();
  let input = input.trim().to_string();
  if input == "q" {
    std::process::exit(0);
  } else if input == "m" {
    MenuAction::Menu
  } else {
    MenuAction::Redo
  }
}

fn runway_heading_tool() -> String {
  let input = Text::new("Enter the runway heading")
    .prompt()
    .unwrap_or_default();

  match input.parse::<u16>() {
    Ok(heading) => {
      let upwind = heading;
      let downwind = (heading + 180) % 360;
      let left_crosswind = (heading + 270) % 360;
      let right_crosswind = (heading + 90) % 360;

      format!(
        "Expanded Headings:\nUpwind: {}°\nDownwind: {}°\nLeft Crosswind: {}°\nRight Crosswind: {}°",
        upwind, downwind, left_crosswind, right_crosswind
      )
    }
    Err(_) => "Invalid input. Please enter a valid number.".to_string(),
  }
}

fn normalize_deg(deg: f32) -> f32 {
  (deg + 360.0) % 360.0
}

fn wind_component_tool() -> String {
  let runway_heading: f32 = CustomType::new("Enter runway heading:")
    .prompt()
    .unwrap_or(0.0);
  let wind_heading: f32 = CustomType::new("Enter wind heading:")
    .prompt()
    .unwrap_or(0.0);
  let wind_speed: f32 = CustomType::new("Enter wind speed (knots):")
    .prompt()
    .unwrap_or(0.0);

  let angle = normalize_deg(wind_heading - runway_heading).to_radians();
  let headwind = -wind_speed * angle.cos();
  let crosswind = -wind_speed * angle.sin();

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

  format!(
    "{} Component: {:.2} knots\n{} Component: {:.2} knots\nCrosswind Percent: {:.2}%",
    headwind_label,
    headwind.abs(),
    crosswind_label,
    crosswind.abs(),
    (crosswind.abs() / wind_speed) * 100.0
  )
}

fn top_of_descent_tool() -> String {
  let current_altitude: f32 = CustomType::new("Enter current altitude (feet):")
    .prompt()
    .unwrap_or(0.0);
  let target_altitude: f32 = CustomType::new("Enter target altitude (feet):")
    .prompt()
    .unwrap_or(0.0);
  let ground_speed: f32 = CustomType::new("Enter ground speed (knots):")
    .prompt()
    .unwrap_or(0.0);
  let vertical_speed: f32 =
    CustomType::new("Enter vertical speed (feet per min):")
      .prompt()
      .unwrap_or(0.0);

  // Standard 3-degree descent path (approximately 300 feet per NM)
  let altitude_to_descend = current_altitude - target_altitude;

  if altitude_to_descend <= 0.0 {
    return "Error: Current altitude must be higher than target altitude."
      .to_string();
  }

  // Calculate time to descent in minutes
  let time_minutes = altitude_to_descend / vertical_speed;

  // Using standard 3° descent
  let distance_nm = (ground_speed / 60.0) * time_minutes;

  format!(
    "Top of Descent Calculation:\n\nAltitude to descend: {:.0} feet\nDistance needed: {:.1} NM\nEstimated time: {:.1} minutes",
    altitude_to_descend, distance_nm, time_minutes
  )
}
