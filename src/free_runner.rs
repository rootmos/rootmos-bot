extern crate time;

use std::thread;
use self::time::{Tm, now_utc};
use std::thread::JoinHandle;
use std::sync::mpsc::{channel, Sender};

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
        let tx_copy = tx.clone();
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
        (tx_copy, t)
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

#[cfg(test)]
mod test {
    extern crate time;
    use free_runner::*;

    enum TestEvent {
        Foo(u32),
        Bar(String),
    }

    enum TestEffect {
        Increment(u32),
        AppendBar(String),
    }

    #[test]
    fn complete_an_event_into_effect_into_event_loop() {
        let f = |e| match e {
            Event::Event { time: _, event: TestEvent::Foo(7) } => effect(TestEffect::Increment(7)),
            Event::Event { time: _, event: TestEvent::Foo(8) } => return_(13),
            _ => noop(),
        };
        let g = |eff| match eff {
            TestEffect::Increment(i) => event(TestEvent::Foo(i+1)),
            _ => noop(),
        };
        let (tx, t) = run(f, g);
        tx.send(Event::Event { time: time::now_utc(), event: TestEvent::Foo(7) }).unwrap();
        assert_eq!(t.join().unwrap(), 13)
    }
}

