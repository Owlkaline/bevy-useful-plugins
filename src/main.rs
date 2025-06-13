use std::{io::Read, net::TcpListener, process::Command, time::Duration};

use ::twitcheventsub::prelude::TwitchEvent;
use bevy::{
  color::palettes::{
    css::{BLACK, BLUE, WHITE},
    tailwind::{BLUE_400, RED_400, YELLOW_400},
  },
  core_pipeline::{bloom::Bloom, core_2d::graph::Node2d, tonemapping::Tonemapping},
  image::{ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
  input::keyboard::KeyboardInput,
  log::LogPlugin,
  math::{Affine2, VectorSpace},
  prelude::*,
  render::{
    render_resource::{AsBindGroup, ShaderRef},
    view::RenderLayers,
  },
  sprite::{AlphaMode2d, Material2d, Material2dPlugin},
};
use bevy_tunnel::{ConnectTunnel, TunnelEvent};
use clock::{AddTime, Clock, MakeClock};
use draggable_interface::DraggableInterface;
use enable_disable_button::MakeToggleButton;
use particles::{fireworks::CreateFireworks, PARTICLE_LAYER, UI_LAYER};
use twitcheventsub::ManageTwitch;
//use twitcheventsub::ManageTwitch;

mod clock;
mod compression;
mod draggable_interface;
mod enable_disable_button;
mod expire;
mod particles;
mod twitcheventsub;

#[derive(Component)]
pub struct ChatBox;

#[derive(Component)]
pub struct InteractiveButtonsUi;

#[derive(Component)]
struct ProgressBar;

const SHADER_ASSET_PATH: &str = "animate_shader.wgsl";
const VORTEX_SHADER_ASSET_PATH: &str = "vortex.wgsl";
const ADHD_SHADER_ASSET_PATH: &str = "adhd.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct ADHDMaterial {}

impl Material2d for ADHDMaterial {
  fn fragment_shader() -> ShaderRef {
    ADHD_SHADER_ASSET_PATH.into()
  }

  fn alpha_mode(&self) -> bevy::sprite::AlphaMode2d {
    AlphaMode2d::Blend
  }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct VortexMaterial {}

impl Material2d for VortexMaterial {
  fn fragment_shader() -> ShaderRef {
    VORTEX_SHADER_ASSET_PATH.into()
  }

  fn alpha_mode(&self) -> bevy::sprite::AlphaMode2d {
    AlphaMode2d::Blend
  }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {
  #[texture(0)]
  #[sampler(1)]
  colour_texture: Handle<Image>,
  #[uniform(2)]
  percentage: f32,
}

impl Material2d for CustomMaterial {
  fn fragment_shader() -> ShaderRef {
    SHADER_ASSET_PATH.into()
  }

  fn alpha_mode(&self) -> bevy::sprite::AlphaMode2d {
    AlphaMode2d::Blend
  }
}

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
      Material2dPlugin::<CustomMaterial>::default(),
      Material2dPlugin::<VortexMaterial>::default(),
      Material2dPlugin::<ADHDMaterial>::default(),
      draggable_interface::plugin,
      clock::plugin,
      particles::plugin,
      twitcheventsub::plugin,
      bevy_tunnel::plugin,
    ))
    .add_event::<TwitchEvent>()
    .add_systems(Startup, setup)
    .add_systems(
      Update,
      (
        input,
        handle_twitch,
        handle_kofi,
        handle_ad_break,
        spawn_fireworks,
      ),
    );

  app.run();
}

fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
  mut custom_materials: ResMut<Assets<CustomMaterial>>,
  mut vortex_materials: ResMut<Assets<VortexMaterial>>,
  mut adhd_material: ResMut<Assets<ADHDMaterial>>,
  assets: Res<AssetServer>,
) {
  let mut chat_box_width = 300.0;
  let mut chat_box_height = 500.0;
  commands.spawn((
    Node {
      display: Display::Flex,
      flex_direction: FlexDirection::Column,
      align_self: AlignSelf::Center,
      justify_self: JustifySelf::Center,
      max_width: Val::Px(chat_box_width),
      min_width: Val::Px(chat_box_width),
      max_height: Val::Px(chat_box_height),
      min_height: Val::Px(chat_box_height),
      overflow: Overflow::clip(),
      ..default()
    },
    Pickable::IGNORE,
    Visibility::Hidden,
    //Transform::from_translation(Vec3::new(50.0, 50.0, 0.0)),
    //Mesh2d(meshes.add(Rectangle::new(chat_box_width, chat_box_height))),
    //MeshMaterial2d(materials.add(ColorMaterial::from_color(WHITE))),
    BackgroundColor(Color::WHITE),
    //DraggableInterface::new().with_scale_factor(1.0 / chat_box_width),
    children![
      (Node::default(), Text::new("Chat")),
      (
        Node {
          display: Display::Flex,
          flex_direction: FlexDirection::Column,
          align_items: AlignItems::Center,
          justify_content: JustifyContent::FlexEnd,
          ..default()
        },
        BackgroundColor::from(BLACK),
        Pickable::IGNORE,
        ChatBox,
        children![
          (
            Node {
              width: Val::Percent(100.0),
              ..default()
            },
            children![
              (
                Node {
                  display: Display::Flex,
                  flex_direction: FlexDirection::Column,
                  align_items: AlignItems::Center,
                  justify_content: JustifyContent::Center,
                  // padding: UiRect::all(Val::Px(8.0)),
                  margin: UiRect::top(Val::Px(10.0)),
                  width: Val::Percent(80.0),
                  height: Val::Px(100.0),
                  ..default()
                },
                BackgroundColor::from(BLUE),
                children![(
                  Node::default(),
                  Text::new("Chat Message 1"),
                  Pickable::IGNORE
                )]
              ),
              (
                Node {
                  position_type: PositionType::Absolute,
                  top: Val::Px(0.0),
                  left: Val::Px(0.0),
                  ..default()
                },
                Text::new("Username"),
                Pickable::IGNORE
              )
            ]
          ),
          (
            Node { ..default() },
            children![
              (
                Node {
                  display: Display::Flex,
                  flex_direction: FlexDirection::Column,
                  align_items: AlignItems::Center,
                  justify_content: JustifyContent::Center,
                  // padding: UiRect::all(Val::Px(8.0)),
                  margin: UiRect::top(Val::Px(10.0)),
                  height: Val::Px(100.0),
                  ..default()
                },
                BackgroundColor::from(BLUE),
                children![(
                  Node::default(),
                  Text::new("Chat Message 2"),
                  Pickable::IGNORE
                )]
              ),
              (
                Node {
                  position_type: PositionType::Absolute,
                  top: Val::Px(0.0),
                  left: Val::Px(0.0),
                  ..default()
                },
                Text::new("Username 2"),
                Pickable::IGNORE
              )
            ]
          )
        ]
      )
    ],
  ));

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

  let mut progress_bar_texture = assets.load("./progress_bar_inside.png");
  commands
    .spawn((
      Mesh2d(meshes.add(Rectangle::new(50.0, 50.0))),
      MeshMaterial2d(materials.add(ColorMaterial::from(assets.load("./progress_bar_empty.png")))),
      Transform::from_translation(Vec3::new(50.0, 50.0, 0.0)),
      DraggableInterface::new().with_scale_factor(1.0 / 50.0),
      Visibility::Hidden,
    ))
    .with_child((
      Mesh2d(meshes.add(Rectangle::new(50.0, 50.0))),
      MeshMaterial2d(custom_materials.add(CustomMaterial {
        colour_texture: progress_bar_texture,
        percentage: 0.5,
      })),
      Transform::from_translation(Vec3::Z * 1.0),
      ProgressBar,
    ));

  commands.spawn((
    Mesh2d(meshes.add(Circle::new(50.0))), //, 6))),
    MeshMaterial2d(vortex_materials.add(VortexMaterial {})),
    //MeshMaterial2d(materials.add(ColorMaterial::from_color(BLUE_400))),
    Transform::from_translation(Vec3::splat(-50.0)),
    DraggableInterface::new().with_scale_factor(1.0 / 50.0),
    Visibility::Hidden,
  ));

  commands.spawn((
    Mesh2d(meshes.add(Circle::new(50.0))),
    MeshMaterial2d(adhd_material.add(ADHDMaterial {})),
    //MeshMaterial2d(materials.add(ColorMaterial::from_color(BLUE_400))),
    Transform::from_translation(Vec3::splat(80.0)),
    DraggableInterface::new().with_scale_factor(1.0 / 50.0),
    Visibility::Hidden,
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
      Pickable::IGNORE,
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
          //  MakeToggleButton::new("Kofi", ConnectTunnel, ConnectTunnel),
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
          //  MakeToggleButton::new("Kofi", ConnectTunnel, ConnectTunnel),
        ))
        .observe(send_event_on_click(ConnectTunnel))
        .with_child(Text::new("Connect Kofi"));

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

fn handle_ad_break(mut twitch_events: EventReader<TwitchEvent>, mut commands: Commands) {
  for event in twitch_events.read() {
    match event {
      TwitchEvent::AdBreakBegin(ad_break) => {
        commands.trigger(AddTime::new(ad_break.duration_seconds as f32));
      }
      _ => {}
    }
  }
}

fn spawn_fireworks(mut twitch_events: EventReader<TwitchEvent>, mut commands: Commands) {
  for event in twitch_events.read() {
    match event {
      TwitchEvent::PointsCustomRewardRedeem(redeem) => {
        println!("Redeem redeemed: {}", redeem.reward.title);
      }
      TwitchEvent::Follow(_) => {
        commands.trigger(CreateFireworks::new(5.0));
      }
      TwitchEvent::NewSubscription(_) |
      TwitchEvent::Resubscription(_) |
      TwitchEvent::GiftSubscription(_) => {
        commands.trigger(CreateFireworks::new(30.0));
      }
      _ => {}
    }
  }
}

fn handle_kofi(
  mut tunnel_events: EventReader<TunnelEvent>,
  mut twitch_events: EventWriter<TwitchEvent>,
  mut commands: Commands,
) {
  for tunnel_event in tunnel_events.read() {
    match tunnel_event {
      TunnelEvent::KofiPing => {
        println!("kofi ping");
      }
      TunnelEvent::TwitchPing => {
        println!("twitch ping");
      }
      TunnelEvent::Kofi(kofi_donation) => {
        println!(
          "{} donated €{}!\n",
          kofi_donation.from_name, kofi_donation.amount
        );
        commands.trigger(CreateFireworks::new(60.0));
      }
      TunnelEvent::Twitch(twitch_event) => {
        twitch_events.write(twitch_event.to_owned());
      }
    }
  }
}

fn handle_twitch(
  mut twitch_events: EventReader<TwitchEvent>,
  // mut twitch_manager: EventWriter<ManageTwitch>,
  chat_box: Query<Entity, With<ChatBox>>,
  mut commands: Commands,
) {
  for event in twitch_events.read() {
    println!("Got twitch event");
    match event {
      TwitchEvent::ChatMessage(msg) => {
        //println!("Message:  {:?}", msg.message.text);
        //for chat_box in chat_box {
        //  println!("  Adding child");
        //  commands.entity(chat_box).with_child((
        //    Node::default(),
        //    Text::new(&format!("{}: {}", msg.chatter.name, msg.message.text)),
        //    Pickable::IGNORE,
        //  ));
        //}
      }
      TwitchEvent::Ready => {
        //twitch_manager.send(ManageTwitch::SendChatMsg(
        //  "OwlBot reporting for duty!".to_string(),
        //));
        //commands.trigger(CreateFireworks::new(15.0));
      }
      TwitchEvent::Finished => {
        println!("Reiceved twitch finished event.");
      }
      _ => {}
    }
  }
}

fn input(
  buttons: Res<ButtonInput<KeyCode>>,
  mut interactivity_layer: Query<&mut Visibility, With<InteractiveButtonsUi>>,
  mut progress_bar: Query<&MeshMaterial2d<CustomMaterial>, With<ProgressBar>>,
  mut custom_materials: ResMut<Assets<CustomMaterial>>,
  mut commands: Commands,
) {
  for mesh in &progress_bar {
    if let Some(material) = custom_materials.get_mut(mesh.id()) {
      let mut height = material.percentage;
      if buttons.pressed(KeyCode::KeyU) {
        height += 0.01;
      }
      if buttons.pressed(KeyCode::KeyJ) {
        height -= 0.01;
      }
      height = height.min(1.0).max(0.0);
      material.percentage = height;
    }
  }

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
) -> impl Fn(Trigger<Pointer<Pressed>>, EventWriter<E>) {
  move |_trigger, mut event_writer| {
    event_writer.send(event.clone());
  }
}

fn trigger_event_on_click<E: Event + Clone>(
  event: E,
) -> impl Fn(Trigger<Pointer<Pressed>>, Commands) {
  move |_trigger, mut commands| {
    commands.trigger(event.clone());
  }
}
