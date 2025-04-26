use bevy::{prelude::*, render::view::RenderLayers};
use bevy_hanabi::prelude::*;

use crate::draggable_interface::DraggableInterface;

pub const PARTICLE_LAYER: usize = 2;
pub const UI_LAYER: usize = 1;

pub mod fireworks;

#[derive(Resource)]
pub struct Particles {
  effects: Handle<EffectAsset>,
}

pub(super) fn plugin(app: &mut App) {
  app
    .add_plugins((HanabiPlugin, fireworks::plugin))
    .add_systems(Startup, (create_click_effect))
    .add_systems(Update, mouse_click);
}

fn mouse_click(
  mouse_event: Res<ButtonInput<MouseButton>>,
  window: Single<&Window>,
  mut particle_effect: Query<(&mut Transform, &mut EffectSpawner, &Name), With<ParticleEffect>>,
  mut commands: Commands,
) {
  let position = window.cursor_position();
  let resolution = window.resolution.size();

  if mouse_event.just_pressed(MouseButton::Left) {
    for (mut transform, mut spawner_settings, name) in &mut particle_effect {
      if name.contains("ClickEffect") {
        if let Some(position) = position {
          let pos = position - resolution * 0.5;
          transform.translation.x = pos.x;
          transform.translation.y = resolution.y * 0.5 - position.y;
        }
        spawner_settings.reset();
      }
    }
  }
}

fn setup(mut effects: ResMut<Assets<EffectAsset>>, mut commands: Commands) {
  let max_particles = 32768;
  let mut module = Module::default();

  let mut gradient = Gradient::new();
  gradient.add_key(0.0, Vec4::new(1., 0., 0., 1.));
  gradient.add_key(1.0, Vec4::splat(0.));

  let init_pos = SetPositionSphereModifier {
    center: module.lit(Vec3::ZERO),
    radius: module.lit(2.),
    dimension: ShapeDimension::Surface,
  };

  let init_vel = SetVelocitySphereModifier {
    center: module.lit(Vec3::ZERO),
    speed: module.lit(60.),
  };

  let accel = module.lit(Vec3::new(0., -30., 0.));
  let update_accel = AccelModifier::new(accel);

  let lifetime = module.lit(10.); // literal value "10.0"
  let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
  let particle_size = SetAttributeModifier::new(Attribute::SIZE, module.lit(50.0));

  // Every frame, add a gravity-like acceleration downward
  let accel = module.lit(Vec3::new(0., -30., 0.));
  let update_accel = AccelModifier::new(accel);

  let effect = EffectAsset::new(
    // Maximum number of particles alive at a time
    max_particles,
    // Spawn at a rate of 5 particles per second
    SpawnerSettings::rate(5.0.into()),
    // Move the expression module into the asset
    module,
  )
  .with_name("ClickEffect")
  .init(particle_size)
  .init(init_pos)
  .init(init_vel)
  .init(init_lifetime)
  .update(update_accel)
  .render(ColorOverLifetimeModifier {
    gradient,
    blend: ColorBlendMode::Modulate,
    mask: ColorBlendMask::all(),
  });

  let effect_handle = effects.add(effect);
  commands.spawn((
    ParticleEffect::new(effect_handle),
    Transform::from_translation(Vec3::Y),
    DraggableInterface::new(),
  ));
}

fn create_click_effect(mut effects: ResMut<Assets<EffectAsset>>, mut commands: Commands) {
  let max_particles = 32768;
  let mut module = Module::default();

  let mut gradient = Gradient::new();
  gradient.add_key(0.0, Vec4::new(0., 0., 1., 1.));
  gradient.add_key(1.0, Vec4::splat(0.));

  let init_vel = SetVelocityCircleModifier {
    center: module.lit(Vec3::ZERO),
    axis: module.lit(Vec3::Z),
    speed: module.lit(300.),
  };

  //let init_pos = SetPositionSphereModifier {
  //  center: module.lit(Vec3::ZERO),
  //  radius: module.lit(2.0),
  //  dimension: ShapeDimension::Surface,
  //};

  let init_pos = SetPositionSphereModifier {
    center: module.lit(Vec3::ZERO),
    //axis: module.lit(Vec3::Z),
    radius: module.lit(1.0),
    dimension: ShapeDimension::Surface,
  };

  //let init_vel = SetVelocitySphereModifier {
  //  center: module.lit(Vec3::ZERO),
  //  speed: module.lit(600.),
  //};

  let lifetime = module.lit(0.5); // literal value "10.0"
  let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
  let particle_size = SetAttributeModifier::new(Attribute::SIZE, module.lit(10.0));

  // Every frame, add a gravity-like acceleration downward
  //let accel = module.lit(Vec3::new(0., -30., 0.));
  //let update_accel = AccelModifier::new(accel);

  let effect = EffectAsset::new(
    // Maximum number of particles alive at a time
    max_particles,
    // Spawn at a rate of 5 particles per second
    SpawnerSettings::once(32.0.into()),
    // Move the expression module into the asset
    module,
  )
  .with_name("MyEffect")
  .init(particle_size)
  .init(init_pos)
  .init(init_vel)
  .init(init_lifetime)
  //.update(update_accel)
  .render(ColorOverLifetimeModifier {
    gradient,
    blend: ColorBlendMode::Modulate,
    mask: ColorBlendMask::all(),
  });

  let effect_handle = effects.add(effect);
  commands.spawn((
    Name::new("ClickEffect"),
    ParticleEffect::new(effect_handle.clone()),
    Transform::from_translation(Vec3::Y),
    //RenderLayers::layer(UI_LAYER),
  ));

  commands.insert_resource(Particles {
    effects: effect_handle,
  });
}
