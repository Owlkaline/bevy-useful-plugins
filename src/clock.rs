use std::time::Duration;

use bevy::prelude::*;

const SECONDS_IN_HOUR: u32 = 3600;
const SECONDS_IN_MINUTE: u32 = 60;

#[derive(Event)]
pub struct AddTime {
  seconds: f32,
}

#[derive(Component)]
pub struct MakeClock(pub Clock);

#[derive(Component)]
pub struct Clock {
  duration: Timer,
}

pub(super) fn plugin(app: &mut App) {
  app
    .add_systems(FixedUpdate, update_clocks)
    .add_systems(Update, update_font_size)
    .add_observer(make_clock)
    .add_observer(add_time)
    .add_observer(add_clock);
}

fn add_time(trigger: Trigger<AddTime>, mut clocks: Query<&mut Clock>) {
  for mut clock in &mut clocks {
    let new_seconds = clock.duration.remaining_secs() + trigger.seconds;
    clock
      .duration
      .set_duration(Duration::from_secs_f32(new_seconds));
    clock.duration.reset();
  }
}

fn update_font_size(
  mut clocks: Query<(Entity, &Parent, &mut TextFont), With<Clock>>,
  mut transforms: Query<&mut Transform>,
  mut commands: Commands,
) {
  for (entity, parent, mut text_font) in &mut clocks {
    if let Ok(mut transform) = transforms.get_mut(parent.get()) {
      let new_font_size = text_font.font_size * transform.scale.x;
      transform.scale = Vec3::splat(1.0);

      commands
        .entity(entity)
        .insert(TextFont::from_font_size(new_font_size));
    }
  }
}

fn update_clocks(mut clocks: Query<(&mut Clock, &mut Text2d)>, time: Res<Time>) {
  for (mut clock, mut text) in &mut clocks {
    clock.duration.tick(time.delta());

    let mut seconds_left = clock.duration.remaining_secs();
    let hr = (seconds_left.floor() as u32) / SECONDS_IN_HOUR;
    seconds_left -= (hr * SECONDS_IN_HOUR) as f32;
    let mm = (seconds_left.floor() as u32) / SECONDS_IN_MINUTE;
    seconds_left -= (mm * SECONDS_IN_MINUTE) as f32;
    let secs = seconds_left as u32;

    text.0 = format!(
      "{}{}:{}{}:{}{}",
      if hr >= 10 { "" } else { "0" },
      hr,
      if mm >= 10 { "" } else { "0" },
      mm,
      if secs >= 10 { "" } else { "0" },
      secs
    );
  }
}

fn make_clock(
  trigger: Trigger<OnAdd, MakeClock>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut commands: Commands,
) {
  commands
    .entity(trigger.entity())
    .insert((Mesh2d(meshes.add(Rectangle::new(80.0, 20.0))),))
    .with_child(Clock::new(120.0))
    .remove::<MakeClock>();
}

fn add_clock(trigger: Trigger<OnAdd, Clock>, mut commands: Commands) {
  commands
    .entity(trigger.entity())
    .insert(Text2d::new("00:00:00"));
}

impl Clock {
  pub fn new(seconds: f32) -> Clock {
    Clock {
      duration: Timer::from_seconds(seconds, TimerMode::Once),
    }
  }
}

impl AddTime {
  pub fn new(seconds: f32) -> AddTime {
    AddTime { seconds }
  }
}
