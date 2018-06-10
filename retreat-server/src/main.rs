extern crate mio;

use mio::net::UdpSocket;
use mio::{Events, Poll, PollOpt, Ready, Token};

fn main() {
    const ECHOER: Token = Token(1);

    let server = UdpSocket::bind(&"0.0.0.0:9050".parse().unwrap()).unwrap();
    println!("Server bound to {:?}", server.local_addr());

    let poll = Poll::new().unwrap();
    poll.register(&server, ECHOER, Ready::readable(), PollOpt::edge())
        .unwrap();

    let mut buffer = [0; 9];
    let mut events = Events::with_capacity(128);

    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                ECHOER => {
                    let (num_recv, addr) = server.recv_from(&mut buffer).unwrap();
                    server.send_to(&buffer, &addr).unwrap();
                    println!("echo {:?} -> {:?}", buffer, num_recv);
                    buffer = [0; 9];
                }
                _ => unreachable!(),
            }
        }
    }
}
