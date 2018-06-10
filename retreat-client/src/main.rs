extern crate mio;

use mio::net::UdpSocket;
use mio::{Events, Poll, PollOpt, Ready, Token};

use std::time::{Duration, Instant};

fn main() {
    let server = "127.0.0.1:9050".parse().unwrap();

    let poll = Poll::new().unwrap();
    let clients: Vec<UdpSocket> = (0..1000)
        .into_iter()
        .map(|i| {
            let client = UdpSocket::bind(&"0.0.0.0:0".parse().unwrap()).unwrap();
            println!(
                "Got client address {:?} connected to server {:?}",
                client.local_addr(),
                server
            );
            client.connect(server).unwrap();

            poll.register(
                &client,
                Token(i),
                Ready::readable() | Ready::writable(),
                PollOpt::edge(),
            ).unwrap();

            return client;
        })
        .collect();

    let mut buffer = [0; 9];
    let mut events = Events::with_capacity(128);

    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                Token(i) => {
                    if event.readiness().is_writable() {
                        let msg_to_send = [i as u8; 9];
                        let bytes_sent = clients[i].send(&msg_to_send).unwrap();
                        assert_eq!(bytes_sent, 9);
                        println!("sent {:?} -> {:?} bytes", msg_to_send, bytes_sent);
                    }
                    if event.readiness().is_readable() {
                        let bytes_recv = clients[i].recv(&mut buffer).unwrap();
                        assert_eq!(bytes_recv, 9);
                        println!("recv {:?} -> {:?} bytes", buffer, bytes_recv);
                        buffer = [0; 9];
                    }
                }
            }
        }
    }
}
