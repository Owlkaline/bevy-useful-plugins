use bevy::{
  color::palettes::tailwind::{BLUE_400, YELLOW_400},
  prelude::*,
};
use draggable_interface::DraggableInterface;

mod draggable_interface;

fn main() {
  let mut app = App::new();

  app
    .add_plugins((
      DefaultPlugins,
      MeshPickingPlugin,
      draggable_interface::plugin,
    ))
    .add_systems(Startup, setup);

  app.run();
}

fn setup(
  mut commands: Commands,
  assets: Res<AssetServer>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  commands.spawn(Camera2d::default());
  commands.spawn((
    Mesh2d(meshes.add(Triangle2d::new(
      Vec2::new(0.5, -0.5),
      Vec2::new(0.0, 0.5),
      Vec2::new(-0.5, -0.5),
    ))),
    MeshMaterial2d(materials.add(ColorMaterial::from_color(YELLOW_400))),
    Transform::from_translation(Vec3::splat(50.0)).with_scale(Vec3::splat(50.0)),
    DraggableInterface::new(),
  ));

  commands.spawn((
    Mesh2d(meshes.add(RegularPolygon::new(50.0, 6))),
    MeshMaterial2d(materials.add(ColorMaterial::from_color(BLUE_400))),
    Transform::from_translation(Vec3::splat(-50.0)),
    DraggableInterface::new().with_scale_factor(1.0 / 50.0),
  ));
}
