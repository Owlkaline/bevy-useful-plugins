use bevy::{
  input::mouse::{MouseButtonInput, MouseWheel},
  prelude::*,
};

#[derive(Component)]
pub struct DraggableInterface {
  scale_factor: f32,
}

#[derive(Component)]
struct Selected;

#[derive(Component)]
struct PendingUnselect;

pub(super) fn plugin(app: &mut App) {
  app
    .add_systems(Update, mouse_up)
    .add_systems(Update, zoom)
    .add_observer(draggable_added);
}

fn mouse_up(
  mouse_buttons: Res<ButtonInput<MouseButton>>,
  mut unselected_entites: Query<Entity, With<PendingUnselect>>,
  mut commands: Commands,
) {
  if mouse_buttons.just_released(MouseButton::Left) {
    for entity in &mut unselected_entites {
      commands
        .entity(entity)
        .remove::<Selected>()
        .remove::<PendingUnselect>();
    }
  }
}

fn zoom(
  mut scroll_event: EventReader<MouseWheel>,
  mut transforms: Query<(&mut Transform, &DraggableInterface), With<Selected>>,
) {
  for scroll in scroll_event.read() {
    for (mut transform, draggable) in &mut transforms {
      transform.scale += scroll.y * draggable.scale_factor;
    }
  }
}

fn draggable_added(trigger: Trigger<OnAdd, DraggableInterface>, mut commands: Commands) {
  commands
    .entity(trigger.target())
    .observe(get_dragged)
    .observe(insert_selected_on::<Pointer<DragEnd>>())
    .observe(insert_selected_on::<Pointer<Over>>())
    .observe(remove_selected_on::<Pointer<Out>>());
}

fn get_dragged(trigger: Trigger<Pointer<Drag>>, mut transform: Query<&mut Transform>) {
  if let Ok(mut transform) = transform.get_mut(trigger.target()) {
    transform.translation.x += trigger.delta.x;
    transform.translation.y -= trigger.delta.y;
  }
}

fn insert_selected_on<E>() -> impl Fn(Trigger<E>, Query<Entity, With<Selected>>, Commands) {
  move |trigger, other_entities_selected, mut commands| {
    if other_entities_selected.iter().count() == 0 {
      commands.entity(trigger.target()).insert(Selected);
    }
  }
}

fn remove_selected_on<E>() -> impl Fn(Trigger<E>, Res<ButtonInput<MouseButton>>, Commands) {
  move |trigger, mouse_buttons, mut commands| {
    if mouse_buttons.pressed(MouseButton::Left) {
      commands.entity(trigger.target()).insert(PendingUnselect);
    } else {
      commands.entity(trigger.target()).remove::<Selected>();
    }
  }
}

impl DraggableInterface {
  pub fn new() -> DraggableInterface {
    DraggableInterface::default()
  }

  pub fn with_scale_factor(mut self, factor: f32) -> DraggableInterface {
    self.scale_factor = factor;
    self
  }
}

impl Default for DraggableInterface {
  fn default() -> Self {
    DraggableInterface { scale_factor: 1.0 }
  }
}
