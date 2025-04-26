use std::time::Duration;

use ::twitcheventsub::TwitchEvent;
use bevy::{
  color::palettes::tailwind::{BLUE_400, RED_400, YELLOW_400},
  core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
  input::keyboard::KeyboardInput,
  math::VectorSpace,
  prelude::*,
  render::view::RenderLayers,
};
use clock::{AddTime, Clock, MakeClock};
use draggable_interface::DraggableInterface;
use particles::{fireworks::CreateFireworks, PARTICLE_LAYER, UI_LAYER};
use twitcheventsub::ManageTwitch;

mod clock;
mod compression;
mod draggable_interface;
mod expire;
mod particles;
mod twitcheventsub;

#[derive(Component)]
pub struct InteractiveButtonsUi;

fn main() {
  let mut app = App::new();

  app
    .add_plugins((
      DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
          title: "UsefulOverlayThings".into(),
          decorations: true,
          transparent: true,
          resizable: false,
          //window_level: WindowLevel::AlwaysOnBottom,
          ..default()
        }),
        ..default()
      }),
      MeshPickingPlugin,
      draggable_interface::plugin,
      clock::plugin,
      particles::plugin,
      twitcheventsub::plugin,
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, (input, handle_twitch, spawn_fireworks));

  app.run();
}

fn setup(
  mut commands: Commands,
  assets: Res<AssetServer>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  // commands.spawn((
  //   Camera2d::default(),
  //   Camera {
  //     clear_color: Color::NONE.into(),
  //     order: 1,
  //     ..default()
  //   },
  // ));
  commands.spawn((
    Camera2d::default(),
    Camera {
      //hdr: true,
      clear_color: Color::NONE.into(),
      ..default()
    },
    //  RenderLayers::layer(UI_LAYER),
  ));
  commands.spawn((
    Transform::from_translation(Vec3::new(0., 20., 50.)),
    Camera3d::default(),
    Camera {
      hdr: true,
      clear_color: Color::NONE.into(),
      order: 2,
      ..Default::default()
    },
    Tonemapping::None,
    Bloom {
      intensity: 0.8,
      ..default()
    },
    RenderLayers::layer(PARTICLE_LAYER),
  ));

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

  commands.spawn((
    MakeClock(Clock::new(120.0)),
    Transform::from_translation(Vec3::ZERO),
    DraggableInterface::new().with_scale_factor(0.25),
  ));

  commands
    .spawn((
      Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Start,
        justify_content: JustifyContent::Start,
        border: UiRect::all(Val::Percent(2.0)),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        row_gap: Val::Px(20.0),
        ..Default::default()
      },
      PickingBehavior::IGNORE,
      InteractiveButtonsUi,
    ))
    .with_children(|parent| {
      parent
        .spawn((
          Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
          },
          BackgroundColor(RED_400.into()),
        ))
        .observe(send_event_on_click(ManageTwitch::Connect))
        .with_child(Text::new("Connect Twitch"));

      parent
        .spawn((
          Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
          },
          BackgroundColor(RED_400.into()),
        ))
        .observe(send_event_on_click(ManageTwitch::Disconnect(Some(
          "OwlBot shutting down!".to_string(),
        ))))
        .with_child(Text::new("Disconnect Twitch"));

      parent
        .spawn((
          Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
          },
          BackgroundColor(RED_400.into()),
        ))
        .observe(trigger_event_on_click(CreateFireworks::new(15.0)))
        .with_child(Text::new("Fireworks!!!"));
    });
}

fn spawn_fireworks(mut twitch_events: EventReader<TwitchEvent>, mut commands: Commands) {
  for event in twitch_events.read() {
    match event {
      TwitchEvent::PointsCustomRewardRedeem(redeem) => {
        if redeem.reward.title.contains("Fireworks") {
          commands.trigger(CreateFireworks::new(15.0));
        }
      }
      TwitchEvent::NewSubscription(_)
      | TwitchEvent::Resubscription(_)
      | TwitchEvent::GiftSubscription(_) => {
        commands.trigger(CreateFireworks::new(30.0));
      }
      _ => {}
    }
  }
}

fn handle_twitch(
  mut twitch_events: EventReader<TwitchEvent>,
  mut twitch_manager: EventWriter<ManageTwitch>,
  mut commands: Commands,
) {
  for event in twitch_events.read() {
    match event {
      TwitchEvent::Ready => {
        //twitch_manager.send(ManageTwitch::SendChatMsg(
        //  "OwlBot reporting for duty!".to_string(),
        //));
        //commands.trigger(CreateFireworks::new(15.0));
      }
      TwitchEvent::Finished => {}
      _ => {}
    }
  }
}

fn input(
  buttons: Res<ButtonInput<KeyCode>>,
  mut interactivity_layer: Query<&mut Visibility, With<InteractiveButtonsUi>>,
  mut commands: Commands,
) {
  if buttons.just_pressed(KeyCode::KeyP) {
    commands.trigger(AddTime::new(10.0));
  }
  if buttons.just_pressed(KeyCode::Space) {
    for mut visibility in &mut interactivity_layer {
      if visibility.eq(&Visibility::Visible) {
        *visibility = Visibility::Hidden;
      } else {
        *visibility = Visibility::Visible;
      }
    }
  }
}

fn send_event_on_click<E: Event + Clone>(
  event: E,
) -> impl Fn(Trigger<Pointer<Down>>, EventWriter<E>) {
  move |_trigger, mut event_writer| {
    event_writer.send(event.clone());
  }
}

fn trigger_event_on_click<E: Event + Clone>(event: E) -> impl Fn(Trigger<Pointer<Down>>, Commands) {
  move |_trigger, mut commands| {
    commands.trigger(event.clone());
  }
}
