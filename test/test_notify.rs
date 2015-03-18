use {sleep_ms};
use mio::*;
use mio::tcp::*;
use super::localhost;
use std::thread;

struct TestHandler {
    sender: EventLoopSender<String>,
    notify: usize
}

impl TestHandler {
    fn new(sender: EventLoopSender<String>) -> TestHandler {
        TestHandler {
            sender: sender,
            notify: 0
        }
    }
}

impl Handler for TestHandler {
    type Timeout = usize;
    type Message = String;

    fn notify(&mut self, event_loop: &mut EventLoop<TestHandler>, msg: String) {
        match self.notify {
            0 => {
                assert!(msg.as_slice() == "First", "actual={}", msg);
                self.sender.send("Second".to_string()).unwrap();
            }
            1 => {
                assert!(msg.as_slice() == "Second", "actual={}", msg);
                event_loop.shutdown();
            }
            v => panic!("unexpected value for notify; val={}", v)
        }

        self.notify += 1;
    }
}

#[test]
pub fn test_notify() {
    debug!("Starting TEST_NOTIFY");
    let mut event_loop = EventLoop::new().unwrap();

    let addr = localhost();

    // Setup a server socket so that the event loop blocks
    let srv = tcp::v4().unwrap();
    srv.set_reuseaddr(true).unwrap();
    srv.bind(&addr).unwrap();

    let srv = srv.listen(256).unwrap();

    event_loop.register_opt(&srv, Token(0), Interest::all(), PollOpt::edge()).unwrap();

    let sender = event_loop.channel();

    thread::spawn(move || {
        sleep_ms(1_000);
        sender.send("First".to_string()).unwrap();
    });

    let sender = event_loop.channel();
    let mut handler = TestHandler::new(sender);

    // Start the event loop
    event_loop.run(&mut handler).unwrap();

    assert!(handler.notify == 2, "actual={}", handler.notify);
}

#[test]
pub fn test_notify_capacity() {
    use std::default::Default;
    use std::sync::mpsc::*;
    use std::thread;

    struct Capacity(Receiver<i32>);

    impl Handler for Capacity {
        fn notify(&mut self, event_loop: EventLoop<Capacity>, msg: i32) {
            if msg == 1 {
                self.0.recv().unwrap();
            } else {
                event_loop.shutdown();
            }
        }
    }

    let config = EventLoopConfig {
        notify_capacity: 1,
        .. EventLoopConfig::default()
    };

    let (tx, rx) = channel::<i32>();
    let mut event_loop = EventLoop::configured(config).unwrap();
    let notify = event_loop.channel();

    //let guard = thread::scoped(move || {
        let mut handler = Capacity(rx);
        event_loop.run(&mut handler);
    //});

    // assert!(notify.send(1).is_ok());
    // assert!(notify.send(2).is_err());
}
