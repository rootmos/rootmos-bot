extern crate time;
extern crate schedule_recv;

use std::thread;
use self::time::{Tm, now_utc};
use std::thread::JoinHandle;
use std::sync::mpsc::{channel, Sender, SendError};
use std::time::Duration;

pub enum Event<E> {
    Heartbeat { time: Tm },
    Event { time: Tm, event: E },
}

pub enum Effect<E, T> {
    Return(T),
    Effect(E),
}

pub fn run<Ev: Send + 'static, Eff, T: Send + 'static, F, G>(f: F, g: G) -> (Sender<Event<Ev>>, JoinHandle<T>)
    where F: Fn(Event<Ev>) -> Option<Effect<Eff, T>> + Send + 'static, G: Fn(Eff) -> Option<Ev> + Send + 'static {
        let (tx, rx) = channel();
        let tx_clone = tx.clone();
        let t = thread::spawn(move || {
            loop {
                let ev = rx.recv().unwrap();
                match f(ev) {
                    Some(Effect::Return(t)) => return t,
                    Some(Effect::Effect(eff)) => match g(eff) {
                        Some(new_ev) => tx.send(Event::Event { time: now_utc(), event: new_ev }).unwrap(),
                        None => ()
                    },
                    None => ()
                }
            }
        });

        (tx_clone, t)
    }

pub fn effect<Eff, T>(eff: Eff) -> Option<Effect<Eff, T>> {
    Some(Effect::Effect(eff))
}

pub fn return_<Eff, T>(t: T) -> Option<Effect<Eff, T>> {
    Some(Effect::Return(t))
}

pub fn noop<T>() -> Option<T> {
    None
}

pub fn event<E>(e: E) -> Option<E> {
    Some(e)
}

pub fn send<Ev>(s: Sender<Event<Ev>>, ev: Ev) -> Result<(), SendError<Event<Ev>>> {
    s.send(Event::Event { time: now_utc(), event: ev })
}

pub fn heartbeats<Ev: Send + 'static>(s: Sender<Event<Ev>>, duration: Duration) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let rx = schedule_recv::periodic(duration);
        loop {
            match rx.recv() {
                Ok(_) => match s.send(Event::Heartbeat { time: now_utc() }) {
                    Ok(_) => (),
                    Err(_) => return ()
                },
                Err(e) => panic!("{}", e),
            }
        }
    })
}

#[cfg(test)]
mod test {
    extern crate time;
    use free_runner::*;
    use std::time::Duration;

    enum TestEvent {
        Foo(u32),
        Bar(String),
    }

    enum TestEffect {
        Increment(u32),
        AppendBar(String),
    }

    #[test]
    fn complete_an_event_into_effect_into_event_loop_test() {
        let f = |e| match e {
            Event::Event { time: _, event: TestEvent::Foo(7) } => effect(TestEffect::Increment(7)),
            Event::Event { time: _, event: TestEvent::Foo(8) } => return_(13),
            _ => noop(),
        };
        let g = |eff| match eff {
            TestEffect::Increment(i) => event(TestEvent::Foo(i+1)),
            _ => noop(),
        };
        let (sender, t) = run(f, g);
        send(sender, TestEvent::Foo(7)).unwrap();
        assert_eq!(t.join().unwrap(), 13)
    }

    #[test]
    fn receive_heartbeat_test() {
        let f = |e| match e { Event::Heartbeat { time: _ } => return_(true), _ => noop() };
        let g = |_: ()| noop::<()>();
        let (s, t) = run(f, g);
        let t2 = heartbeats(s, Duration::from_millis(100));
        assert!(t.join().unwrap());
        assert_eq!(t2.join().unwrap(), ())
    }
}

