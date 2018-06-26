extern crate mio;
extern crate rlib;

use mio::net::UdpSocket;
use mio::{Events, Poll, PollOpt, Ready, Token};

use rlib::game::game_loop;

const TICKS_PER_SECOND: u64 = 128;

fn main() {
    const ACTIONS: Token = Token(1);

    let server = UdpSocket::bind(&"0.0.0.0:9050".parse().unwrap()).unwrap();
    println!("Server bound to {:?}", server.local_addr());

    let poll = Poll::new().unwrap();
    poll.register(&server, ACTIONS, Ready::readable(), PollOpt::edge())
        .unwrap();

    let mut buffer = [0; 9];
    let mut events = Events::with_capacity(128);

    game_loop(TICKS_PER_SECOND, |start, tick, next_tick| {
        println!("ring {}", tick);
        let wait = next_tick.checked_sub(start.elapsed());

        if wait.is_some() {
            poll.poll(&mut events, wait).unwrap();
            for event in events.iter() {
                match event.token() {
                    ACTIONS => {
                        let (num_recv, addr) = server.recv_from(&mut buffer).unwrap();
                        server.send_to(&buffer, &addr).unwrap();
                        println!("echo {:?} -> {:?}", buffer, num_recv);
                        buffer = [0; 9];
                    }
                    _ => unreachable!(),
                }
            }
        }
    });
}
