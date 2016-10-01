extern crate time;
extern crate schedule_recv;

use std::thread;
use self::time::{Tm, now_utc};
use std::thread::JoinHandle;
use std::sync::mpsc::{channel, Sender, SendError};
use std::time::Duration;
use std::any::Any;

pub enum Event<E> {
    Heartbeat { time: Tm },
    Event { time: Tm, event: E },
}

pub enum Effect<E, T> {
    Return(T),
    Effect(E),
}

pub struct Runner<Ev, T> {
    sender: Sender<Event<Ev>>,
    join_handle: JoinHandle<T>,
    heartbeats_join_handle: Option<JoinHandle<()>>,
}

impl <Ev, T> Runner<Ev, T> {
    pub fn new<F, G, Eff>(f: F, g: G) -> Runner<Ev, T>
        where F: Fn(Event<Ev>) -> Option<Effect<Eff, T>> + Send + 'static, G: Fn(Eff) -> Option<Ev> + Send + 'static, T: Send + 'static, Ev: Send + 'static {

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

        Runner { sender: tx_clone, join_handle: t, heartbeats_join_handle: None }
    }

    pub fn join(self) -> Result<T, Box<Any + Send + 'static>> {
        match self.heartbeats_join_handle {
            Some(t) => assert_eq!(t.join().unwrap(), ()),
            None => ()
        };
        self.join_handle.join()
    }

    pub fn send(&self, ev: Ev) -> Result<(), SendError<Event<Ev>>> {
        self.sender.send(Event::Event { time: now_utc(), event: ev })
    }

    pub fn heartbeats(&mut self, duration: Duration) -> () where Ev: Send + 'static {
        assert!(self.heartbeats_join_handle.is_none());

        let cloned_sender = self.sender.clone();
        self.heartbeats_join_handle = Some(thread::spawn(move || {
            let rx = schedule_recv::periodic(duration);
            loop {
                match rx.recv() {
                    Ok(_) => match cloned_sender.send(Event::Heartbeat { time: now_utc() }) {
                        Ok(_) => (),
                        Err(_) => return ()
                    },
                    Err(e) => panic!("{}", e),
                }
            }
        }))
    }
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
    use free_runner::*;
    use std::time::Duration;

    enum TestEvent {
        Foo(u32),
    }

    enum TestEffect {
        Increment(u32),
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
        };
        let runner = Runner::new(f, g);
        runner.send(TestEvent::Foo(7)).unwrap();
        assert_eq!(runner.join().unwrap(), 13)
    }

    #[test]
    fn receive_heartbeat_test() {
        let f = |e| match e { Event::Heartbeat { time: _ } => return_(true), _ => noop() };
        let g = |_: ()| noop::<()>();
        let mut runner = Runner::new(f, g);
        runner.heartbeats(Duration::from_millis(100));
        assert!(runner.join().unwrap());
    }
}

