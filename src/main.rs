extern crate rootmos_bot;

use rootmos_bot::free_runner::*;
use rootmos_bot::irc::*;

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
    rootmos_bot::irc::run("irc-config.json", echo_bot);
}
