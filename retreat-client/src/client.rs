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
use std::marker::PhantomData;

use capnp::message::{Builder, HeapAllocator, TypedReader};
use capnp::serialize_packed;

use mio::net::UdpSocket;
use mio::{Events, Poll, PollOpt, Ready, Token};

use winit::{Event, WindowEvent::*};

use rlib::actions_capnp::{client_actions, Direction};
use rlib::builder;
use rlib::game::{GameState, Player};

mod proto {
    use std::collections::HashMap;

    pub trait DefaultLit {
        fn provide_default_type() -> Self;
    }

    pub trait ProtoSize {
        fn size(&self) -> usize;
    }

    pub trait ProtoType<T>: ProtoSize {
        fn typed_read(&self, slice: &[u8]) -> T {
            assert!(slice.len() >= self.size());
            self.typed_read_nocheck(slice)
        }

        fn typed_write(&self, slice: &mut [u8], val: T) {
            assert!(slice.len() >= self.size());
            self.typed_write_nocheck(slice, val)
        }

        fn typed_read_nocheck(&self, &[u8]) -> T;
        fn typed_write_nocheck(&self, &mut [u8], T);
    }

    pub trait Literal<T> {
        fn read(slice: &[u8]) -> T {
            assert!(slice.len() >= Self::size());
            Self::read_nocheck(slice)
        }

        fn write(slice: &mut [u8], val: T) {
            assert!(slice.len() >= Self::size());
            Self::write_nocheck(slice, val)
        }

        fn read_nocheck(this: &[u8]) -> T;
        fn write_nocheck(&mut [u8], T);
        fn size() -> usize;
    }

    impl<T> Literal<T> for T
    where
        T: DefaultLit + ProtoType<T>,
    {
        fn read_nocheck(this: &[u8]) -> T {
            T::provide_default_type().typed_read(this)
        }

        fn write_nocheck(this: &mut [u8], value: T) {
            T::provide_default_type().typed_write(this, value)
        }

        fn size() -> usize {
            T::provide_default_type().size()
        }
    }

    impl DefaultLit for u8 {
        fn provide_default_type() -> u8 {
            0
        }
    }

    impl ProtoType<u8> for u8 {
        fn typed_read_nocheck(&self, slice: &[u8]) -> Self {
            Self::from(slice[0])
        }

        fn typed_write_nocheck(&self, slice: &mut [u8], value: Self) {
            slice[0] = Self::into(value);
        }
    }

    impl ProtoSize for u8 {
        fn size(&self) -> usize {
            1
        }
    }

    pub struct ListType<T: ProtoType<T>> {
        t_type: T,
        length: usize,
        elem_size: usize,
    }

    impl<T: ProtoType<T>> ListType<T> {
        pub fn typed_with_length(t_type: T, len: usize) -> ListType<T> {
            ListType {
                length: len,
                elem_size: t_type.size(),
                t_type: t_type,
            }
        }
    }

    impl<T: DefaultLit + ProtoType<T>> ListType<T> {
        pub fn with_length(len: usize) -> ListType<T> {
            Self::typed_with_length(T::provide_default_type(), len)
        }
    }

    impl<'a, T: ProtoType<T>> ProtoType<Vec<T>> for ListType<T> {
        fn typed_read_nocheck(&self, slice: &[u8]) -> Vec<T> {
            let mut result = Vec::with_capacity(self.length);

            for i in 0..self.length {
                let ptr = &slice[i * self.elem_size..];
                result.push(self.t_type.typed_read_nocheck(ptr));
            }

            result
        }

        fn typed_write_nocheck(&self, slice: &mut [u8], value: Vec<T>) {
            let mut i = 0;

            for x in value {
                let mut ptr = &mut slice[i * self.elem_size..];
                self.t_type.typed_write_nocheck(&mut ptr, x);
                i += 1;
            }
        }
    }

    impl<T: ProtoType<T>> ProtoSize for ListType<T> {
        fn size(&self) -> usize {
            self.length * self.elem_size
        }
    }

    macro_rules! protostruct {
        ($field:ident, $field_type:ty) => {
            pub fn $field<'a>(&'a mut self,
                            $field: $field_type) -> &'a mut Self {
                self.$field = $field;
                self
            }
        };
    }

    pub struct StructType<'a> {
        field_test1_type: &'a ProtoType<u8>,
        field_test2_type: &'a ListType<u8>,
        field_sizes: &'a [usize],
    }

    impl<'a> StructType<'a> {
        pub fn with_fields<T>(test1: &'a T, test2: &'a ListType<u8>) -> StructType<'a>
        where
            T: ProtoType<u8>,
        {
            let types: Vec<&ProtoSize> = vec![test1, test2];
            let mut ptr = 0;

            StructType {
                field_sizes: types
                    .iter()
                    .map(|proto| {
                        ptr += proto.size();
                        ptr
                    })
                    .collect::<Vec<usize>>()
                    .as_slice(),
                field_test1_type: test1,
                field_test2_type: test2,
            }
        }

        pub fn test1_mut(self, arena: &mut [u8]) -> &mut [u8] {}
    }

    trait L {
        type Item;
        fn write(&mut [u8], Self::Item);
        fn read(&mut [u8]) -> Self::Item;
    }

    trait Owned<K> {
        fn get<T>(K) -> T;
        fn set<T>(K, T);
    }

    struct OwnedL<T> {
        p: T,
    }

    impl<T> Owned<()> for OwnedL<T> {
        fn get<R>(void: ()) -> R {
            3
        }
        fn set<R>(void: (), value: R) {}
    }

    struct OwnedS {}

    impl OwnedS {
        pub fn with_fields<K, T: L>(fields: HashMap<K, T>) -> OwnedS {
            OwnedS {}
        }

        pub fn get<K, R>(key: K) -> R {}
    }
}

#[cfg(test)]
mod test {
    use proto::*;

    #[test]
    fn test_struct() {
        let mut arena = [0, 0, 0];

        let test1 = u8::provide_default_type();
        let test2 = ListType::with_length(5);

        let s_type = StructType::with_fields(&test1, &test2);

        u8::write(s_type.test1_mut(&mut arena), 8);
        let val = u8::read(&arena);
        assert_eq!(val, 8);
    }

    #[test]
    fn test_u8() {
        let mut arena = [0, 0, 0];
        u8::write(&mut arena, 8);
        let val = u8::read(&arena);
        assert_eq!(val, 8);
    }

    #[test]
    #[should_panic]
    fn test_u8_panics() {
        let mut arena = [];
        u8::write(&mut arena, 8);
    }

    #[test]
    fn test_list() {
        let mut arena = [0, 0, 0];
        let list = ListType::with_length(3);

        list.typed_write(&mut arena, vec![1, 2, 3]);
        let val = list.typed_read(&arena);
        assert_eq!(val, [1, 2, 3]);
    }

    #[test]
    #[should_panic]
    fn test_list_panics() {
        let mut arena = [0, 0];
        let list = ListType::with_length(3);

        list.typed_write(&mut arena, vec![1, 2, 3]);
    }
}

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
                        serialize_packed::write_message(&mut buffer, &message);

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
