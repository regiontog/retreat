/*
 * TODO:
 * - gfxrs
 * - threading & parallelism
 * - lyon
 */
#![feature(const_fn)]
extern crate capnp;
extern crate mio;
extern crate winit;

extern crate rlib;

use std::io::ErrorKind;

use capnp::message::{Builder, HeapAllocator, TypedReader};
use capnp::serialize_packed;

use mio::net::UdpSocket;
use mio::{Events, Poll, PollOpt, Ready, Token};

use winit::{Event, WindowEvent::*};

use rlib::actions_capnp::{client_actions, Direction};
use rlib::builder;
use rlib::game::{GameState, Player};

const fn keys_length() -> usize {
    // This ain't pretty!
    (winit::VirtualKeyCode::Cut as usize) + 1
}

fn render(game: &GameState) {
    print!("{}[2J", 27 as char); // Clear terminal
    println!(
        "{}{}{}",
        game.world[0][0], game.world[0][1], game.world[0][2]
    );
    println!(
        "{}{}{}",
        game.world[1][0], game.world[1][1], game.world[1][2]
    );
    println!(
        "{}{}{}",
        game.world[2][0], game.world[2][1], game.world[2][2]
    );
}

enum ClientLoopControl {
    NewAction(builder::PlaceholderAction),
    Exit,
}

struct ClientState {
    exit: bool,
    actions: Vec<builder::PlaceholderAction>,
    keys: [winit::ElementState; keys_length()],
}

impl ClientState {
    fn new() -> ClientState {
        ClientState {
            exit: false,
            actions: vec![],
            keys: [winit::ElementState::Released; keys_length()],
        }
    }

    fn push_action(&mut self, ctrl: ClientLoopControl) {
        match ctrl {
            ClientLoopControl::Exit => self.exit = true,
            ClientLoopControl::NewAction(action) => self.actions.push(action),
        }
    }

    fn get_actions(&self, frame: u8) -> TypedReader<Builder<HeapAllocator>, client_actions::Owned> {
        builder::action::actions(frame, &self.actions)
    }

    fn truncate_actions(&mut self) {
        self.actions.truncate(0);
    }

    fn key_up(&self, key: winit::VirtualKeyCode) -> Option<ClientLoopControl> {
        None
    }

    fn key_down(&self, key: winit::VirtualKeyCode) -> Option<ClientLoopControl> {
        match key {
            winit::VirtualKeyCode::F => Some(ClientLoopControl::Exit),
            winit::VirtualKeyCode::W => Some(ClientLoopControl::NewAction(
                builder::PlaceholderAction::Move(Direction::Forward),
            )),
            winit::VirtualKeyCode::S => Some(ClientLoopControl::NewAction(
                builder::PlaceholderAction::Move(Direction::Backward),
            )),
            winit::VirtualKeyCode::A => Some(ClientLoopControl::NewAction(
                builder::PlaceholderAction::Move(Direction::Left),
            )),
            winit::VirtualKeyCode::D => Some(ClientLoopControl::NewAction(
                builder::PlaceholderAction::Move(Direction::Right),
            )),
            _ => None,
        }
    }

    fn handle_input(&mut self, input: winit::KeyboardInput) -> Option<ClientLoopControl> {
        input.virtual_keycode.and_then(|vkc| {
            let current_key_state = self.keys[vkc as usize];

            if current_key_state == input.state {
                None
            } else {
                self.keys[vkc as usize] = input.state;

                match input.state {
                    winit::ElementState::Pressed => self.key_down(vkc),
                    winit::ElementState::Released => self.key_up(vkc),
                }
            }
        })
    }
}

fn main() {
    const CLIENT_TOKEN: Token = Token(0);

    let mut events_loop = winit::EventsLoop::new();
    let poll = Poll::new().unwrap();

    let _window = winit::Window::new(&events_loop).unwrap();

    let server = "127.0.0.1:9050".parse().unwrap();
    let client = UdpSocket::bind(&"0.0.0.0:0".parse().unwrap()).unwrap();
    client.connect(server).unwrap();

    println!(
        "Got client address {:?} connected to server {:?}",
        client.local_addr(),
        server
    );

    poll.register(
        &client,
        CLIENT_TOKEN,
        Ready::readable() | Ready::writable(),
        PollOpt::edge(),
    ).unwrap();

    let mut buffer = Vec::with_capacity(128);
    let mut events = Events::with_capacity(128);
    let mut game_state = GameState::new();
    let mut client_state = ClientState::new();

    let mut message = Builder::new_default();

    let mut player = Player::new(1);

    while !client_state.exit {
        // Advance game state
        let actions = client_state.get_actions(game_state.frame);
        game_state.advance(&mut player, actions);
        client_state.truncate_actions();
        // TODO: remove ack'ed actions

        // Render game state
        render(&game_state);

        // Process input
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    KeyboardInput { input, .. } => match client_state.handle_input(input) {
                        Some(action) => client_state.push_action(action),
                        _ => (),
                    },
                    CloseRequested => client_state.push_action(ClientLoopControl::Exit),
                    _ => (),
                },
                _ => (),
            };
        });

        // Synchronize client <-> server
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                CLIENT_TOKEN => {
                    if event.readiness().is_writable() {
                        // message.set_root(actions);
                        // serialize_packed::write_message(&mut buffer, &message);

                        match client.send(&buffer) {
                            Err(e) => match e.kind() {
                                ErrorKind::ConnectionRefused => {
                                    client_state.push_action(ClientLoopControl::Exit)
                                }
                                _ => panic!(e),
                            },
                            Ok(bytes_sent) => {
                                // assert_eq!(bytes_sent, 9);
                                // println!("sent {:?} -> {:?} bytes", msg_to_send, bytes_sent);
                            }
                        }
                    }
                    if event.readiness().is_readable() {
                        let bytes_recv = client.recv(&mut buffer).unwrap();
                        match client.recv(&mut buffer) {
                            Ok(bytes_recv) => {
                                // println!("recv {:?} -> {:?} bytes", buffer, bytes_recv);
                                buffer.truncate(0);
                            }
                            Err(e) => match e.kind() {
                                ErrorKind::ConnectionReset => {
                                    println!("Connection reset!");
                                    client_state.push_action(ClientLoopControl::Exit);
                                }
                                ErrorKind::WouldBlock => (),
                                _ => {
                                    println!("{:?}", e);
                                    unreachable!();
                                }
                            },
                        }
                        // assert_eq!(bytes_recv, 9);
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
