#[macro_use]
extern crate lazy_static;

extern crate rootmos_bot;
use rootmos_bot::free_runner::*;
use rootmos_bot::irc::*;
use rootmos_bot::db;
use rootmos_bot::db::KV;

extern crate chrono;
use chrono::*;

extern crate regex;
use regex::Regex;

extern crate sha2;
use sha2::Sha256;
use sha2::Digest;

extern crate serde;
extern crate serde_json;

include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));

use std::path::Path;

fn tag_bot<KV>(event: Event<ChatEvent>, kv: &mut KV) -> Option<Effect<ChatEffect, ()>> where KV: db::KV<String, String> {
    println!("Event: {:?}", event);
    lazy_static! {
        static ref LIST_CMD: Regex = Regex::new(r"^!list\s+(#[a-zA-Z0-9]+)$").unwrap();
        static ref UNTAG_CMD: Regex = Regex::new(r"^!untag\s+(#[a-zA-Z0-9]+)\s+([a-fA-F0-9]+)$").unwrap();
        static ref TAGGED: Regex = Regex::new(r"(\s|^)(#[a-zA-Z0-9]+)(\s|$)").unwrap();
    }

    match event {
        Event::Event { time, event: ChatEvent::ChannelMsg { channel, msg, from } } =>
            if let Some(cap) = LIST_CMD.captures(msg.as_str()) {
                let tag = cap.at(1).unwrap();
                effect(list_cmd(channel, tag.to_owned(), kv))
            } else if let Some(cap) = UNTAG_CMD.captures(msg.as_str()) {
                let tag = cap.at(1).unwrap();
                let hash = cap.at(2).unwrap();
                effect(untag_cmd(channel, tag.to_owned(), hash.to_owned(), kv))
            } else if let Some(cap) = TAGGED.captures(msg.as_str()) {
                let tag = cap.at(2).unwrap();
                effect(tag_line(time, from, channel, tag.to_owned(), msg.clone(), kv))
            } else {
                noop()
            },
        _  => noop(),
    }
}

fn hash(s: &String) -> String {
    let mut hasher = Sha256::new();
    hasher.input_str(s.as_str());
    let mut h = hasher.result_str();
    h.truncate(8);
    h
}

fn mk_key(channel: &String, tag: &String, hash: &String) -> String {
    format!("{}{}", mk_key_prefix(channel, tag), hash)
}

fn mk_key_prefix(channel: &String, tag: &String) -> String {
    format!("{}-{}-", channel, tag)
}


fn list_cmd<KV>(channel: String, tag: String, kv: &KV) -> ChatEffect where KV: db::KV<String, String> {

    let tag_prefix = mk_key_prefix(&channel, &tag);
    let mut tagged_lines = kv.get_prefix(&tag_prefix).iter().map(|p| serde_json::from_str(&p.1).unwrap()).collect::<Vec<TaggedLine>>();
    tagged_lines.sort_by(|a, b| a.time.cmp(&b.time));

    let mut msg = tagged_lines.iter()
        .map(|l| format!("{} (by: {}, at: {})", &l.line, &l.user, &l.time.with_timezone(&Local).to_rfc2822()))
        .collect::<Vec<String>>();
    msg.insert(0, format!("Listing tag {}:", tag));
    ChatEffect::ChannelMsg { channel: channel, msg: msg }
}

fn tag_line<KV>(time: DateTime<UTC>, user: String, channel: String, tag: String, line: String, kv: &mut KV) -> ChatEffect where KV: db::KV<String, String> {
    let line_hash = hash(&line);
    let key = mk_key(&channel, &tag, &line_hash);
    let tagged_line = TaggedLine {
        channel: channel.clone(),
        tag: tag.clone(),
        time: time,
        user: user,
        line: line,
        hash: line_hash};
    let json = serde_json::to_string(&tagged_line).unwrap();
    kv.put(&key, &json).unwrap();
    let response = vec![format!("Line tagged, recall using: !list {}", tag)];
    ChatEffect::ChannelMsg { channel: channel, msg: response }
}

fn untag_cmd<KV>(channel: String, tag: String, hash: String, kv: &mut KV) -> ChatEffect where KV: db::KV<String, String> {
    let key = mk_key(&channel, &tag, &hash);
    match kv.get(&key).unwrap() {
        Some(json) => {
            kv.remove(&key).unwrap();
            let tl: TaggedLine = serde_json::from_str(json.as_str()).unwrap();
            let msg = vec![format!("Removed tag {} from line: {}", tag, tl.line)];
            ChatEffect::ChannelMsg { channel: channel, msg: msg }
        },
        None => {
            let error = format!("Unable to find line tagged with {} and with hash {}", tag, hash);
            ChatEffect::ChannelMsg { channel: channel, msg: vec![error] }
        },
    }
}

#[test]
fn save_line_without_tag_test() {
    let mut kv = db::hashmap_kv::HashMapKV::new();
    let input = Event::Event { time: UTC::now(), event: ChatEvent::ChannelMsg {
        channel: "my_channel".to_owned(),
        from: "user1".to_owned(),
        msg: "test line tag".to_owned() } };

    assert_eq!(tag_bot(input, &mut kv), None)
}

#[test]
fn ignore_line_with_almost_tags_test() {
    let mut kv = db::hashmap_kv::HashMapKV::new();
    let input = Event::Event { time: UTC::now(), event: ChatEvent::ChannelMsg {
        channel: "my_channel".to_owned(),
        from: "user1".to_owned(),
        msg: "not#tag #not(a-tag)".to_owned() } };

    assert_eq!(tag_bot(input, &mut kv), None)
}

#[test]
fn save_line_with_tag_in_the_middle_test() {
    let tag = "#tag".to_owned();
    save_line_with_tag_test(format!("test line {} some more text", tag), tag)
}

#[test]
fn save_line_with_tag_at_the_end_test() {
    let tag = "#tag".to_owned();
    save_line_with_tag_test(format!("another test line {}", tag), tag)
}

#[test]
fn save_line_with_tag_at_the_start_test() {
    let tag = "#tag".to_owned();
    save_line_with_tag_test(format!("{} yet another test line", tag), tag)
}


#[cfg(test)]
fn save_line_with_tag_test(line: String, tag: String) {
    let mut kv = db::hashmap_kv::HashMapKV::new();
    let channel = "my_channel".to_owned();
    let input = Event::Event { time: UTC::now(), event: ChatEvent::ChannelMsg {
        channel: channel.clone(),
        from: "user1".to_owned(),
        msg: line.clone() } };

    match tag_bot(input, &mut kv) {
        Some(Effect::Effect(ChatEffect::ChannelMsg { channel: to_channel, .. })) => {
            assert_eq!(to_channel, channel)
        },
        _ => panic!(),
    }

    let expected_key = format!("{}-{}-{}", channel, tag, hash(&line));
    match kv.get(&expected_key).unwrap() {
        Some(raw_data) => {
            let tagged_line: TaggedLine = serde_json::from_str(&raw_data).unwrap();
            assert_eq!(tagged_line.line, line)
        },
        _ => panic!(),
    }
}

#[test]
fn recall_tag_one_line_test() {
    let mut kv = db::hashmap_kv::HashMapKV::new();
    let channel = "my_channel".to_owned();
    let tag = "#tag".to_owned();
    let line = format!("a test line {}", tag);

    let input_line = Event::Event { time: UTC::now(), event: ChatEvent::ChannelMsg {
        channel: channel.clone(),
        from: "user1".to_owned(),
        msg: line.clone() } };
    match tag_bot(input_line, &mut kv) {
        Some(_) => (),
        _ => panic!(),
    }

    let recall_cmdline = format!("!list {}", tag);
    let recall_event = Event::Event { time: UTC::now(), event: ChatEvent::ChannelMsg {
        channel: channel.clone(),
        from: "user2".to_owned(),
        msg: recall_cmdline } };

    match tag_bot(recall_event, &mut kv) {
        Some(Effect::Effect(ChatEffect::ChannelMsg { channel: to_channel, msg })) => {
            assert_eq!(to_channel, channel);
            assert!(msg[0].contains(tag.as_str()));
            assert!(msg[1].contains(line.as_str()));
        },
        _ => panic!(),
    }
}

#[test]
fn recall_tag_several_lines_test() {
    let mut kv = db::hashmap_kv::HashMapKV::new();
    let channel = "my_channel".to_owned();
    let tag = "#tag".to_owned();

    let line1 = format!("a test line {}", tag);
    let time1 = UTC::now() - Duration::hours(3);
    let user1 = "user1".to_owned();
    run_tag_bot_for_line_in_channel(&time1, &channel, &user1, &line1, &mut kv);

    let line2 = format!("another {} test line", tag);
    let time2 = time1 + Duration::hours(1);
    let user2 = "user2".to_owned();
    run_tag_bot_for_line_in_channel(&time2, &channel, &user2, &line2, &mut kv);

    let line3 = format!("{} yet another line", tag);
    let time3 = time2 + Duration::hours(1);
    let user3 = "user3".to_owned();
    run_tag_bot_for_line_in_channel(&time3, &channel, &user3, &line3, &mut kv);

    assert!(time1 < time2);
    assert!(time2 < time3);

    let recall_cmdline = format!("!list {}", tag);
    let recall_event = Event::Event { time: UTC::now(), event: ChatEvent::ChannelMsg {
        channel: channel.clone(),
        from: "user4".to_owned(),
        msg: recall_cmdline } };

    match tag_bot(recall_event, &mut kv) {
        Some(Effect::Effect(ChatEffect::ChannelMsg { channel: to_channel, msg })) => {
            assert_eq!(to_channel, channel);
            assert_eq!(msg[0], format!("Listing tag {}:", tag));

            assert!(msg[1].contains(line1.as_str()));
            assert!(msg[1].contains(user1.as_str()));
            assert!(msg[1].contains(&time1.with_timezone(&Local).to_rfc2822()));

            assert!(msg[2].contains(line2.as_str()));
            assert!(msg[2].contains(user2.as_str()));
            assert!(msg[2].contains(&time2.with_timezone(&Local).to_rfc2822()));

            assert!(msg[3].contains(line3.as_str()));
            assert!(msg[3].contains(user3.as_str()));
            assert!(msg[3].contains(&time3.with_timezone(&Local).to_rfc2822()));
        },
        _ => panic!(),
    }
}

#[test]
fn untag_nonexisting_line_comlains_test() {
    let mut kv = db::hashmap_kv::HashMapKV::new();
    let channel = "my_channel".to_owned();
    let tag = "#tag".to_owned();

    let nonexisting_hash = "abbababe";
    let untag_cmdline = format!("!untag {} {}", tag, nonexisting_hash);
    let untag_event = Event::Event { time: UTC::now(), event: ChatEvent::ChannelMsg {
        channel: channel.clone(),
        from: "user3".to_owned(),
        msg: untag_cmdline } };

    match tag_bot(untag_event, &mut kv) {
        Some(Effect::Effect(ChatEffect::ChannelMsg { msg, .. })) => {
            let error = format!("Unable to find line tagged with {} and with hash {}", tag, nonexisting_hash);
            assert_eq!(msg[0], error);
        },
        _ => panic!(),
    }
}

#[test]
fn untag_line_test() {
    let mut kv = db::hashmap_kv::HashMapKV::new();
    let channel = "my_channel".to_owned();
    let tag = "#tag".to_owned();

    let line1 = format!("a test line {}", tag);
    let time1 = UTC::now() - Duration::hours(3);
    let user1 = "user1".to_owned();
    let line1_hash = hash(&line1);
    run_tag_bot_for_line_in_channel(&time1, &channel, &user1, &line1, &mut kv);

    let expected_key = mk_key(&channel, &tag, &line1_hash);
    assert!(kv.get(&expected_key).unwrap().is_some());

    let untag_cmdline = format!("!untag {} {}", tag, line1_hash);
    let untag_event = Event::Event { time: UTC::now(), event: ChatEvent::ChannelMsg {
        channel: channel.clone(),
        from: "user3".to_owned(),
        msg: untag_cmdline } };

    match tag_bot(untag_event, &mut kv) {
        Some(Effect::Effect(ChatEffect::ChannelMsg { msg, .. })) => {
            assert_eq!(msg[0], format!("Removed tag {} from line: {}", tag, line1));
        }
        _ => panic!(),
    }

    let expected_key = mk_key(&channel, &tag, &line1_hash);
    assert!(kv.get(&expected_key).unwrap().is_none());
}

#[cfg(test)]
fn run_tag_bot_for_line_in_channel<KV>(time: &DateTime<UTC>, channel: &String, user: &String, line: &String, kv: &mut KV) -> () where KV: db::KV<String, String> {
    let input = Event::Event { time: time.clone(), event: ChatEvent::ChannelMsg {
        channel: channel.clone(),
        from: user.clone(),
        msg: line.clone() } };
    match tag_bot(input, kv) {
        Some(_) => (),
        _ => panic!(),
    }
}


fn main() {
    let kv = db::rocksdb_kv::RocksDBKV::new(Path::new("tag_bot_db"));
    rootmos_bot::irc::run("irc-config.json", tag_bot, kv);
}
