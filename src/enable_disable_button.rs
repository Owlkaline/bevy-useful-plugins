use bevy::prelude::*;

#[derive(Component)]
pub struct MakeToggleButton<E: Event + Clone, D: Event + Clone> {
  name: String,
  enable: E,
  disable: D,
}

impl<E: Event + Clone, D: Event + Clone> MakeToggleButton<E, D> {
  pub fn new<S: Into<String>>(name: S, enable: E, disable: D) -> MakeToggleButton<E, D> {
    MakeToggleButton {
      name: name.into(),
      enable,
      disable,
    }
  }
}

#[derive(Component)]
struct EnableDisableButton;

#[derive(Component, Deref, DerefMut)]
struct Enabled(bool);

pub(super) fn plugin<E: Event + Clone, D: Event + Clone>(app: &mut App) {
  app.add_observer(make_toggle_button::<E, D>);
}

fn make_toggle_button<E: Event + Clone, D: Event + Clone>(
  trigger: Trigger<OnAdd, MakeToggleButton<E, D>>,
  make_toggle_button: Query<&MakeToggleButton<E, D>>,
  mut commands: Commands,
) {
  if let Ok(make_toggle) = make_toggle_button.get(trigger.target()) {
    commands
      .entity(trigger.target())
      .insert(Name::new(make_toggle.name.to_owned()))
      .insert(Enabled(false))
      .remove::<MakeToggleButton<E, D>>()
      .observe(trigger_toggle_event_on_click(
        make_toggle.enable.clone(),
        make_toggle.disable.clone(),
      ))
      .with_child(Text::new(format!("Enable {}", make_toggle.name.to_owned())));
  }
}

fn trigger_toggle_event_on_click<E: Event + Clone, D: Event + Clone>(
  enable_event: E,
  disable_event: D,
) -> impl Fn(
  Trigger<Pointer<Pressed>>,
  Query<(&mut Enabled, &Name, &Children)>,
  Query<&mut Text>,
  EventWriter<E>,
  EventWriter<D>,
  Commands,
) {
  move |trigger,
        mut button,
        mut text,
        mut enable_event_writer,
        mut disable_event_writer,
        mut commands| {
    if let Ok((mut enabled, name, children)) = button.get_mut(trigger.target()) {
      for child in children {
        if let Ok(mut text) = text.get_mut(*child) {
          text.0 = format!("{} {}", if enabled.0 { "Disable" } else { "Enable" }, name);
        }
      }

      if enabled.0 {
        disable_event_writer.send(disable_event.clone());
      } else {
        enable_event_writer.send(enable_event.clone());
      }
      enabled.0 = !enabled.0;
    }
  }
}
