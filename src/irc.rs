extern crate irc;
use self::irc::client::prelude::*;
use self::irc::client::data::user::User;

use free_runner::*;

#[derive(Debug, PartialEq)]
pub enum ChatEvent {
    ChannelMsg { channel: String, from: String, msg: String },
    PrivateMsg { from: String, msg: String },
    JoinedChannel { channel: String, who: String },
    PartedChannel { channel: String, who: String, comment: Option<String> },
}

#[derive(Debug, PartialEq)]
pub enum ChatEffect {
    ChannelMsg { channel: String, msg: Vec<String> },
    PrivateMsg { to: String, msg: Vec<String> },
}

pub fn run<F, S: Send + 'static>(config: &str, f: F, s: S) where F: Fn(Event<ChatEvent>, &mut S) -> Option<Effect<ChatEffect, ()>> + Send + 'static  {
    let server = IrcServer::new(config).unwrap();
    server.identify().unwrap();

    let server_clone = server.clone();
    let handle_chat_effect = move |eff| {
        match eff {
            ChatEffect::ChannelMsg { channel, msg } => {
                for line in msg.iter() {
                    server_clone.send_privmsg(channel.as_str(), line.as_str()).unwrap();
                }
                noop()
            }
            ChatEffect::PrivateMsg { to, msg } => {
                for line in msg.iter() {
                    server_clone.send_privmsg(to.as_str(), line.as_str()).unwrap();
                }
                noop()
            }
        }
    };

    let runner = Runner::new(f, handle_chat_effect, s);

    for maybe_message in server.iter() {
        let nickname = server.current_nickname();
        match maybe_message {
            Ok(Message { prefix: Some(ref who), command: Command::PRIVMSG(ref to, ref msg), .. }) if to == nickname => {
                let nickname = String::from(User::new(who).get_nickname());
                runner.send(ChatEvent::PrivateMsg { from: nickname.clone(), msg: msg.clone() }).unwrap()
            },
            Ok(Message { prefix: Some(ref who), command: Command::PRIVMSG(ref to, ref msg), .. }) => {
                let nickname = String::from(User::new(who).get_nickname());
                runner.send(ChatEvent::ChannelMsg { channel: to.clone(), from: nickname.clone(), msg: msg.clone() }).unwrap()
            },
            Ok(Message { prefix: Some(ref who), command: Command::JOIN(ref channel, _, _), .. }) => {
                let nickname = String::from(User::new(who).get_nickname());
                runner.send(ChatEvent::JoinedChannel { channel: channel.clone(), who: nickname }).unwrap()
            },
            Ok(Message { prefix: Some(ref who), command: Command::PART(ref channel, ref maybe_comment), .. }) => {
                let nickname = String::from(User::new(who).get_nickname());
                runner.send(ChatEvent::PartedChannel { channel: channel.clone(), who: nickname, comment: maybe_comment.clone() }).unwrap()
            },
            Ok(message) => print!("Unhandled: {}", message),
            Err(err) => println!("{}", err),
        }
    }
}
