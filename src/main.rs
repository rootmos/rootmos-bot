extern crate rootmos_bot;

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

fn handle_chat_effect(server: IrcServer) -> Box<Fn(ChatEffect) -> ()> {
    Box::new(move |eff| {
        match eff {
            ChatEffect::ChannelMsg { channel, msg } =>
                server.send_privmsg(channel.as_str(), msg.as_str()).unwrap(),
            ChatEffect::PrivateMsg { to, msg } =>
                server.send_privmsg(to.as_str(), msg.as_str()).unwrap(),
        }
    })
}

fn main() {
    let server = IrcServer::new("irc-config.json").unwrap();
    server.identify().unwrap();
    let handler = handle_chat_effect(server.clone());
    for maybe_message in server.iter() {
        let nickname = server.current_nickname();
        match maybe_message {
            Ok(Message { prefix: Some(ref who), command: Command::PRIVMSG(ref to, ref msg), .. }) if to == nickname => {
                let nickname = String::from(User::new(who).get_nickname());
                let ev = ChatEvent::PrivateMsg { from: nickname.clone(), msg: msg.clone() };
                println!("{:?}", ev);
                handler(ChatEffect::PrivateMsg { to: nickname.clone(), msg: format!("I agree {}, I also: {}", nickname, msg)})

            },
            Ok(Message { prefix: Some(ref who), command: Command::PRIVMSG(ref to, ref msg), .. }) => {
                let nickname = String::from(User::new(who).get_nickname());
                let ev = ChatEvent::ChannelMsg { channel: to.clone(), from: nickname.clone(), msg: msg.clone() };
                println!("{:?}", ev);
                handler(ChatEffect::ChannelMsg { channel: to.clone(), msg: format!("Did {} just say: {}", nickname, msg)})
            },
            Ok(Message { prefix: Some(ref who), command: Command::JOIN(ref channel, _, _), .. }) => {
                let nickname = String::from(User::new(who).get_nickname());
                let ev = ChatEvent::JoinedChannel { channel: channel.clone(), who: nickname };
                println!("{:?}", ev)
            },
            Ok(Message { prefix: Some(ref who), command: Command::PART(ref channel, ref maybe_comment), .. }) => {
                let nickname = String::from(User::new(who).get_nickname());
                let ev = ChatEvent::PartedChannel { channel: channel.clone(), who: nickname, comment: maybe_comment.clone() };
                println!("{:?}", ev)
            },
            Ok(message) => println!("Unhandled: {}", message),
            Err(err) => println!("{}", err),
        }
    }
}
