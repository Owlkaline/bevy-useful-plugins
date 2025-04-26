use std::time::Duration;

use bevy::prelude::*;

fn wiseman_score(compression_ratio: f32, duration: Duration) -> f32 {
  0.0
}

fn compression_algorithm(data: String) -> String {
  data
}

pub(super) fn plugin(app: &mut App) {
  app.add_systems(Startup, setup);
}

fn setup(commands: Commands) {}
