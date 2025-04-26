use std::{
  fs::exists,
  sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex,
  },
  thread,
  time::Duration,
};

use bevy::prelude::*;

use twitcheventsub::*;

#[derive(Event, Clone)]
pub enum ManageTwitch {
  Connect,
  Disconnect(Option<String>),
  SendChatMsg(String),
}

#[derive(Resource)]
pub struct TwitchResource {
  new_events: Mutex<Receiver<TwitchEvent>>,
  sender: Mutex<Sender<ManageTwitch>>,
}

pub(super) fn plugin(app: &mut App) {
  app
    .add_event::<ManageTwitch>()
    .add_event::<TwitchEvent>()
    .add_systems(
      Update,
      (
        send_twitch_events.run_if(resource_exists::<TwitchResource>),
        manage_twitch_connection,
      ),
    );
}

fn manage_twitch_connection(
  mut manage_twitch_events: EventReader<ManageTwitch>,
  mut twitch_resource: Option<Res<TwitchResource>>,
  mut commands: Commands,
) {
  for manage_twitch in manage_twitch_events.read() {
    match manage_twitch {
      ManageTwitch::Connect => {
        let (sender, receiver) = channel::<TwitchEvent>();
        let (sender2, receiver2) = channel::<ManageTwitch>();

        commands.insert_resource(TwitchResource {
          new_events: Mutex::new(receiver),
          sender: Mutex::new(sender2),
        });

        twitch_thread(sender, receiver2);
      }
      ManageTwitch::Disconnect(msg) => {
        if let Some(twitch) = &mut twitch_resource {
          let _ = twitch.sender.lock().and_then(|t| {
            let _ = t.send(ManageTwitch::Disconnect(msg.clone()));

            Ok(())
          });
        }
        commands.remove_resource::<TwitchResource>();
      }
      manage_twitch => {
        if let Some(twitch) = &mut twitch_resource {
          let _ = twitch.sender.lock().and_then(|t| {
            let _ = t.send(manage_twitch.clone());

            Ok(())
          });
        }
      }
    }
  }
}

fn send_twitch_events(twitch: Res<TwitchResource>, mut twitch_events: EventWriter<TwitchEvent>) {
  if let Ok(twitch) = twitch.new_events.try_lock() {
    if let Ok(new_event) = twitch.recv_timeout(Duration::ZERO) {
      twitch_events.send(new_event);
    }
  }
}

fn twitch_thread(sender: Sender<TwitchEvent>, new_commands: Receiver<ManageTwitch>) {
  thread::spawn(move || {
    if let Ok(keys) = TwitchKeys::from_secrets_env(vec![".secrets.env".to_string()]) {
      if let Ok(mut twitch) = TwitchEventSubApi::builder(keys)
        .enable_irc("owlkalinevt", "owlkalinevt")
        .auto_save_load_created_tokens(".user_token", ".refresh_token")
        .generate_access_token_on_expire(true)
        .set_redirect_url("http://localhost:3000")
        .generate_new_token_if_none(true)
        .add_subscriptions([
          Subscription::AdBreakBegin,
          Subscription::ChannelPointsCustomRewardRedeem,
          Subscription::ChannelFollow,
          Subscription::ChannelNewSubscription,
          Subscription::ChannelResubscription,
          Subscription::ChannelGiftSubscription,
          Subscription::ChannelCheer,
          Subscription::ChannelRaid,
          Subscription::ChatMessage,
          Subscription::PermissionIRCRead,
          Subscription::PermissionIRCWrite,
          Subscription::PermissionReadModerator,
        ])
        .build()
      {
        'thread: loop {
          match twitch.receive_single_message(Duration::from_millis(1)) {
            Some(response) => match response {
              ResponseType::Event(event) => {
                if let Err(err) = sender.send(event) {
                  dbg!(err);
                  break 'thread;
                }
              }
              ResponseType::Ready => {
                let _ = sender.send(TwitchEvent::Ready);
              }
              _ => {}
            },
            None => {}
          }

          for new_command in new_commands.try_iter() {
            match new_command {
              ManageTwitch::Disconnect(exit_msg) => {
                if let Some(msg) = exit_msg {
                  let _ = twitch.send_chat_message(msg);
                }
                // Do exit twitch things
                let _ = sender.send(TwitchEvent::Finished);

                // stop this thread
                break 'thread;
              }
              ManageTwitch::SendChatMsg(msg) => {
                let _ = twitch.send_chat_message(msg);
              }
              _ => {}
            }
          }
        }
      }
    };
  });
}
