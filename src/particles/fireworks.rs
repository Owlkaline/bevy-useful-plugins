use bevy::{color::palettes::css::FIRE_BRICK, prelude::*, render::view::RenderLayers};
use bevy_hanabi::prelude::*;

use crate::expire::{self, Expire, Expired};

use super::PARTICLE_LAYER;

const FIREWORKS_SCALE: f32 = 8.0;
const FIREWORKS_SIZE: f32 = 3.0;

#[derive(Event, Clone)]
pub struct CreateFireworks(pub f32);

#[derive(Component)]
struct Fireworks;

impl CreateFireworks {
  pub fn new(duration: f32) -> CreateFireworks {
    CreateFireworks(duration)
  }
}

pub(super) fn plugin(app: &mut App) {
  app
    .add_observer(create_effect)
    .add_observer(fireworks_expired)
    .add_plugins(expire::plugin);
}

fn fireworks_expired(
  trigger: Trigger<OnAdd, Expired>,
  mut particle_effect: Query<
    (&mut Transform, &mut EffectSpawner),
    (With<ParticleEffect>, With<Fireworks>),
  >,
  mut commands: Commands,
) {
  if let Ok((mut transform, mut effect_spawner)) = particle_effect.get_mut(trigger.entity()) {
    effect_spawner.active = false;
  }
  commands.entity(trigger.entity()).remove::<Expired>();
}

fn create_effect(
  trigger: Trigger<CreateFireworks>,
  mut fireworks: Query<(Entity, &mut EffectSpawner, Option<&mut Expire>), With<Fireworks>>,
  mut commands: Commands,
  mut effects: ResMut<Assets<EffectAsset>>,
) {
  for (entity, mut firework_effect, expire) in &mut fireworks {
    firework_effect.active = true;
    if let Some(mut expire) = expire {
      expire.add_time(trigger.0);
    } else {
      commands.entity(entity).insert(Expire::new(trigger.0));
    }
    return;
  }

  // Rocket
  let rocket_effect = effects.add(create_rocket_effect());
  let rocket_entity = commands
    .spawn((
      Name::new("rocket"),
      Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
      ParticleEffect::new(rocket_effect),
      RenderLayers::layer(PARTICLE_LAYER),
      Expire::new(trigger.0),
      Fireworks,
    ))
    .id();

  // Sparkle trail
  let sparkle_trail_effect = effects.add(create_sparkle_trail_effect());
  commands.spawn((
    Name::new("sparkle_trail"),
    ParticleEffect::new(sparkle_trail_effect),
    // Set the rocket effect as parent. This gives access to the rocket effect's particles,
    // which in turns allows inheriting their position (and other attributes if needed).
    EffectParent::new(rocket_entity),
    RenderLayers::layer(PARTICLE_LAYER),
    Expire::new(trigger.0),
  ));

  // Trails
  let trails_effect = effects.add(create_trails_effect());
  commands.spawn((
    Name::new("trails"),
    ParticleEffect::new(trails_effect),
    // Set the rocket effect as parent. This gives access to the rocket effect's particles,
    // which in turns allows inheriting their position (and other attributes if needed).
    EffectParent::new(rocket_entity),
    RenderLayers::layer(PARTICLE_LAYER),
    Expire::new(trigger.0),
  ));
}

fn create_rocket_effect() -> EffectAsset {
  let writer = ExprWriter::new();

  // Always start from the same launch point
  let init_pos = SetPositionCircleModifier {
    center: writer.lit(Vec3::ZERO).expr(),
    axis: writer.lit(Vec3::Y).expr(),
    radius: writer.lit(30.0).expr(),
    dimension: ShapeDimension::Volume,
  };

  // Give a bit of variation by randomizing the initial speed and direction
  let zero = writer.lit(0.);
  let y = writer
    .lit(14.)
    .uniform(writer.lit(16.))
    .mul(writer.lit(FIREWORKS_SCALE));
  let v = zero.clone().vec3(y, zero);
  let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, v.expr());

  let age = writer.lit(0.).expr();
  let init_age = SetAttributeModifier::new(Attribute::AGE, age);

  // Store a random color per particle, which will be inherited by the spark ones
  // on explosion. We don't store it in Attribute::COLOR otherwise it's going to
  // affect the color of the rocket particle itself.
  let rgb = writer.rand(VectorType::VEC3F) * writer.lit(0.9) + writer.lit(0.1);
  let color = rgb.vec4_xyz_w(writer.lit(1.)).pack4x8unorm();
  let init_trails_color = SetAttributeModifier::new(Attribute::U32_0, color.expr());

  // Give a bit of variation by randomizing the lifetime per particle
  let lifetime = writer.lit(0.8).uniform(writer.lit(1.2)).expr();
  let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

  // Add constant downward acceleration to simulate gravity
  let accel = writer.lit(Vec3::Y * -16.).expr();
  let update_accel = AccelModifier::new(accel);

  // Add drag to make particles slow down as they ascend
  let drag = writer.lit(4.).expr();
  let update_drag = LinearDragModifier::new(drag);

  // As the rocket particle rises in the air, it leaves behind a trail of
  // sparkles. To achieve this, the particle emits spawn events for its child
  // effect.
  let update_spawn_trail = EmitSpawnEventModifier {
    condition: EventEmitCondition::Always,
    count: 5u32,
    // We use channel #0 for those sparkle trail events; see EffectParent
    child_index: 0,
  };

  // When the rocket particle dies, it "explodes" and spawns the actual firework
  // particles. To achieve this, when a rocket particle dies, it emits spawn
  // events for its child(ren) effects.
  let update_spawn_on_die = EmitSpawnEventModifier {
    condition: EventEmitCondition::OnDie,
    count: 1000u32,
    // We use channel #1 for the explosion itself; see EffectParent
    child_index: 1,
  };

  let spawner = SpawnerSettings::rate((1., 3.).into());

  EffectAsset::new(32, spawner, writer.finish())
    .with_name("rocket")
    .init(init_pos)
    .init(init_vel)
    .init(init_age)
    .init(init_lifetime)
    .init(init_trails_color)
    .update(update_drag)
    .update(update_accel)
    .update(update_spawn_trail)
    .update(update_spawn_on_die)
    .render(ColorOverLifetimeModifier {
      gradient: Gradient::constant(Vec4::ONE),
      blend: ColorBlendMode::Overwrite,
      mask: ColorBlendMask::RGBA,
    })
    .render(SizeOverLifetimeModifier {
      gradient: Gradient::constant(Vec3::ONE * 0.1 * FIREWORKS_SIZE),
      screen_space_size: true,
    })
}

/// Create the effect for the sparkle trail coming out of the rocket as it
/// raises in the air.
fn create_sparkle_trail_effect() -> EffectAsset {
  let writer = ExprWriter::new();

  // Inherit the start position from the parent effect (the rocket particle)
  let init_pos = InheritAttributeModifier::new(Attribute::POSITION);

  // The velocity is random in any direction
  let vel = writer.rand(VectorType::VEC3F).normalized();
  let speed = writer.lit(1.).uniform(writer.lit(4.));
  let vel = (vel * speed).expr();
  let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, vel);

  let age = writer.lit(0.).expr();
  let init_age = SetAttributeModifier::new(Attribute::AGE, age);

  // Give a bit of variation by randomizing the lifetime per particle
  let lifetime = writer.lit(0.2).expr(); //.uniform(writer.lit(0.4)).expr();
  let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

  // Add constant downward acceleration to simulate gravity
  let accel = writer.lit(Vec3::Y * -16. * FIREWORKS_SCALE).expr();
  let update_accel = AccelModifier::new(accel);

  // Add drag to make particles slow down as they ascend
  let drag = writer.lit(4. * FIREWORKS_SCALE).expr();
  let update_drag = LinearDragModifier::new(drag);

  // The (CPU) spawner is unused
  let spawner = SpawnerSettings::default();

  let mut color_gradient = Gradient::new();
  color_gradient.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
  color_gradient.add_key(0.8, Vec4::new(4.0, 4.0, 4.0, 1.0));
  color_gradient.add_key(1.0, Vec4::new(4.0, 4.0, 4.0, 0.0));

  EffectAsset::new(1000, spawner, writer.finish())
    .with_name("sparkle_trail")
    .init(init_pos)
    .init(init_vel)
    .init(init_age)
    .init(init_lifetime)
    .update(update_drag)
    .update(update_accel)
    .render(ColorOverLifetimeModifier {
      gradient: color_gradient,
      blend: ColorBlendMode::Modulate,
      mask: ColorBlendMask::RGBA,
    })
    .render(SizeOverLifetimeModifier {
      gradient: Gradient::constant(Vec3::ONE * 0.02 * FIREWORKS_SIZE),
      screen_space_size: true,
    })
}

/// Create the effect for the trails coming out of the rocket explosion. They
/// spawn in burst each time a rocket particle dies (= "explodes").
fn create_trails_effect() -> EffectAsset {
  let writer = ExprWriter::new();

  // Inherit the start position from the parent effect (the rocket particle)
  let init_pos = InheritAttributeModifier::new(Attribute::POSITION);

  // Pull the color from the parent's Attribute::U32_0.
  let init_color = SetAttributeModifier::new(
    Attribute::COLOR,
    writer.parent_attr(Attribute::U32_0).expr(),
  );

  // The velocity is random in any direction
  let center = writer.attr(Attribute::POSITION);
  let speed = writer
    .lit(4.)
    .uniform(writer.lit(6.))
    .mul(writer.lit(FIREWORKS_SCALE));
  let dir = writer
    .rand(VectorType::VEC3F)
    .mul(writer.lit(2.0))
    .sub(writer.lit(1.0))
    .normalized();
  let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, (center + dir * speed).expr());

  let age = writer.lit(0.).expr();
  let init_age = SetAttributeModifier::new(Attribute::AGE, age);

  // Give a bit of variation by randomizing the lifetime per particle
  let lifetime = writer.lit(0.8).uniform(writer.lit(1.2)).expr();
  let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

  // Add constant downward acceleration to simulate gravity
  let accel = writer.lit(Vec3::Y * -16.).expr();
  let update_accel = AccelModifier::new(accel);

  // Add drag to make particles slow down as they ascend
  let drag = writer.lit(4.).expr();
  let update_drag = LinearDragModifier::new(drag);

  // Orient particle toward its velocity to create a cheap 1-particle trail
  let orient = OrientModifier::new(OrientMode::AlongVelocity);

  // The (CPU) spawner is unused
  let spawner = SpawnerSettings::default();

  let mut color_gradient = Gradient::new();
  color_gradient.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
  color_gradient.add_key(0.6, Vec4::new(4.0, 4.0, 4.0, 1.0));
  color_gradient.add_key(1.0, Vec4::new(4.0, 4.0, 4.0, 0.0));

  EffectAsset::new(10000, spawner, writer.finish())
    .with_name("trail")
    .init(init_pos)
    .init(init_vel)
    .init(init_age)
    .init(init_lifetime)
    .init(init_color)
    .update(update_drag)
    .update(update_accel)
    .render(ColorOverLifetimeModifier {
      gradient: color_gradient,
      blend: ColorBlendMode::Modulate,
      mask: ColorBlendMask::RGBA,
    })
    .render(SizeOverLifetimeModifier {
      gradient: Gradient::constant(Vec3::new(0.2, 0.05, 0.05) * FIREWORKS_SIZE),
      screen_space_size: true,
    })
    .render(orient)
}
