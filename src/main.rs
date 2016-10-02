extern crate rootmos_bot;
use rootmos_bot::free_runner::*;

extern crate irc;
use irc::client::prelude::*;
use irc::client::data::user::User;

#[derive(Debug)]
enum ChatEvent {
    ChannelMsg { channel: String, from: String, msg: String },
    PrivateMsg { from: String, msg: String },
    JoinedChannel { channel: String, who: String },
    PartedChannel { channel: String, who: String, comment: Option<String> },
}

#[derive(Debug)]
enum ChatEffect {
    ChannelMsg { channel: String, msg: String },
    PrivateMsg { to: String, msg: String },
}


fn echo_bot(event: Event<ChatEvent>) -> Option<Effect<ChatEffect, ()>> {
    println!("Event: {:?}", event);
    match event {
        Event::Event { event: ChatEvent::PrivateMsg { from, msg }, .. } =>
            effect(ChatEffect::PrivateMsg { to: from.clone(), msg: format!("I agree {}, I also: {}", from, msg)}),
        Event::Event { event: ChatEvent::ChannelMsg { channel, from, msg }, .. } =>
            effect(ChatEffect::ChannelMsg { channel: channel, msg: format!("Did {} just say: {}", from, msg)}),
        _  => noop(),
    }
}

fn main() {

    let server = IrcServer::new("irc-config.json").unwrap();
    server.identify().unwrap();


    let server_clone = server.clone();
    let handle_chat_effect = move |eff| {
        match eff {
            ChatEffect::ChannelMsg { channel, msg } => {
                server_clone.send_privmsg(channel.as_str(), msg.as_str()).unwrap();
                noop()
            }
            ChatEffect::PrivateMsg { to, msg } => {
                server_clone.send_privmsg(to.as_str(), msg.as_str()).unwrap();
                noop()
            }
        }
    };

    let runner = Runner::new(echo_bot, handle_chat_effect);

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
            Ok(message) => println!("Unhandled: {}", message),
            Err(err) => println!("{}", err),
        }
    }
}
