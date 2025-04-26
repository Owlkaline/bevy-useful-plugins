use std::time::Duration;

use bevy::prelude::*;

#[derive(Component)]
pub struct Expire(pub Timer);

#[derive(Component)]
pub struct Expired;

impl Expire {
  pub fn new(seconds: f32) -> Expire {
    let timer = Timer::new(Duration::from_secs_f32(seconds), TimerMode::Once);
    Expire(timer)
  }

  pub fn add_time(&mut self, seconds: f32) {
    let mut time_left = self.0.duration().as_secs_f32() - self.0.elapsed_secs();
    time_left += seconds;
    self.0.set_duration(Duration::from_secs_f32(time_left));
    self.0.reset();
  }
}

pub(super) fn plugin(app: &mut App) {
  app.add_systems(FixedUpdate, expire_entities);
}

fn expire_entities(
  mut entities: Query<(Entity, &mut Expire)>,
  mut commands: Commands,
  time: Res<Time>,
) {
  for (entity, mut timer) in &mut entities {
    timer.0.tick(time.delta());
    if timer.0.finished() {
      commands.entity(entity).remove::<Expire>().insert(Expired);
    }
  }
}
