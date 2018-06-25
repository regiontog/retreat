/*
 * TODO:
 * - gfxrs
 * - threading & parallelism
 * - lyon
 */
extern crate capnp;
extern crate mio;
extern crate winit;

extern crate rlib;

use std::io::ErrorKind;

use capnp::message::{Builder, HeapAllocator, TypedReader};
use mio::net::UdpSocket;
use mio::{Events, Poll, PollOpt, Ready, Token};

use winit::{Event, WindowEvent::*};

use rlib::actions_capnp::{client_actions, Direction};
use rlib::builder;
use rlib::game::{GameState, Player};

fn handle_input(input: winit::KeyboardInput) -> Option<ClientLoopControl> {
    input.virtual_keycode.and_then(|vkc| match vkc {
        winit::VirtualKeyCode::F => Some(ClientLoopControl::Exit),
        winit::VirtualKeyCode::W => Some(ClientLoopControl::NewAction(
            builder::PlaceholderAction::Move(Direction::Forward),
        )),
        _ => None,
    })
}

fn render(game: &GameState) {
    print!("{}[2J", 27 as char); // Clear terminal
    println!("{}{}{}", game.world[0], game.world[1], game.world[2]);
    println!("{}{}{}", game.world[3], game.world[4], game.world[5]);
    println!("{}{}{}", game.world[6], game.world[7], game.world[8]);
}

enum ClientLoopControl {
    NewAction(builder::PlaceholderAction),
    Exit,
}

struct ClientLoop {
    exit: bool,
    actions: Vec<builder::PlaceholderAction>,
}

impl ClientLoop {
    fn new() -> ClientLoop {
        ClientLoop {
            exit: false,
            actions: vec![],
        }
    }

    fn push(&mut self, ctrl: ClientLoopControl) {
        match ctrl {
            ClientLoopControl::Exit => self.exit = true,
            ClientLoopControl::NewAction(action) => self.actions.push(action),
        }
    }

    fn actions(&self, frame: u8) -> TypedReader<Builder<HeapAllocator>, client_actions::Owned> {
        builder::action::actions(frame, &self.actions)
    }
}

fn main() {
    let mut events_loop = winit::EventsLoop::new();
    let _window = winit::Window::new(&events_loop).unwrap();

    let server = "127.0.0.1:9050".parse().unwrap();

    let poll = Poll::new().unwrap();
    let clients: Vec<UdpSocket> = (0..1)
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
    let mut game = GameState::new();
    let mut cloop = ClientLoop::new();

    let player = Player::new(1);

    while !cloop.exit {
        // Advance game state
        let actions = cloop.actions(game.frame);
        game.advance(&player, actions);
        // TODO: remove ack'ed actions

        // Render game state
        render(&game);

        // Process input
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    KeyboardInput { input, .. } => match handle_input(input) {
                        Some(action) => cloop.push(action),
                        _ => (),
                    },
                    CloseRequested => cloop.push(ClientLoopControl::Exit),
                    _ => (),
                },
                _ => (),
            };
        });

        // Synchronize client <-> server
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                Token(i) => {
                    if event.readiness().is_writable() {
                        let msg_to_send = [i as u8; 9];
                        match clients[i].send(&msg_to_send) {
                            Err(e) => match e.kind() {
                                ErrorKind::ConnectionRefused => cloop.push(ClientLoopControl::Exit),
                                _ => panic!(e),
                            },
                            Ok(bytes_sent) => {
                                assert_eq!(bytes_sent, 9);
                                println!("sent {:?} -> {:?} bytes", msg_to_send, bytes_sent);
                            }
                        }
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
