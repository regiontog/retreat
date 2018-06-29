extern crate capnp;

pub mod actions_capnp {
    include!(concat!(env!("OUT_DIR"), "/actions_capnp.rs"));
}

pub mod builder {
    use actions_capnp::Direction;

    pub enum PlaceholderAction {
        Move(Direction),
        Shoot(Direction),
        Jump,
    }

    pub mod action {
        use actions_capnp::client_actions;
        use builder::PlaceholderAction;
        use capnp::message::{Builder, HeapAllocator, TypedReader};

        pub fn actions(
            frame: u8,
            actions: &Vec<PlaceholderAction>,
        ) -> TypedReader<Builder<HeapAllocator>, client_actions::Owned> {
            let mut message = ::capnp::message::Builder::new_default();
            {
                let mut root = message.init_root::<client_actions::Builder>();
                root.set_frame(frame);

                let mut list = root.reborrow().init_actions(actions.len() as u32); //FIXME: Hacky?

                for (i, action) in actions.iter().enumerate() {
                    match action {
                        PlaceholderAction::Move(dir) => {
                            list.reborrow().get(i as u32).set_move(*dir)
                        }
                        PlaceholderAction::Shoot(dir) => {
                            list.reborrow().get(i as u32).set_shoot(*dir)
                        }
                        PlaceholderAction::Jump => list.reborrow().get(i as u32).set_jump(()),
                    }
                }
            }

            TypedReader::from(message)
        }
    }
}

pub mod game {
    use std::time::{Duration, Instant};

    use actions_capnp::{action, client_actions, Direction};
    use capnp::message::{ReaderSegments, TypedReader};

    pub struct Player {
        id: u64,
        pos: (u8, u8),
    }

    pub struct GameState {
        pub frame: u8,
        pub world: [[u8; 3]; 3],
    }

    impl Player {
        pub fn new(id: u64) -> Player {
            Player {
                id: id,
                pos: (0, 0),
            }
        }

        fn take_action(&mut self, action: action::Which) {
            match action {
                action::Which::Move(mov) => match mov {
                    Ok(mov) => match mov {
                        Direction::Forward => self.pos.0 = self.pos.0.wrapping_sub(1),
                        Direction::Backward => self.pos.0 = self.pos.0.wrapping_add(1),
                        Direction::Left => self.pos.1 = self.pos.1.wrapping_sub(1),
                        Direction::Right => self.pos.1 = self.pos.1.wrapping_add(1),
                    },
                    Err(_) => unreachable!(),
                },
                action::Which::Shoot(_shoot) => println!("shoot"),
                action::Which::Jump(_jump) => println!("jump"),
            }
        }
    }

    impl GameState {
        pub fn advance<A: ReaderSegments>(
            &mut self,
            player: &mut Player,
            controls: TypedReader<A, client_actions::Owned>,
        ) {
            let unwrapped = controls.get().unwrap();

            debug_assert!(self.frame == unwrapped.get_frame());
            for a in unwrapped.get_actions().unwrap() {
                match a.which() {
                    Ok(action) => player.take_action(action),
                    Err(_) => unreachable!(),
                }
            }

            self.world = [[0, 0, 0], [0, 0, 0], [0, 0, 0]];
            self.world[(player.pos.0 % 3) as usize][(player.pos.1 % 3) as usize] = player.id as u8;
        }

        pub fn new() -> GameState {
            GameState {
                frame: 0,
                world: [[0, 0, 0], [0, 0, 0], [0, 0, 0]],
            }
        }
    }

    pub fn game_loop<F>(tick_rate: u64, mut ring: F)
    where
        F: FnMut(Instant, u64, Duration),
    {
        let tick_length: Duration = Duration::from_nanos(1_000_000_000 / tick_rate);

        let mut ticks = 0;
        let mut next_tick = tick_length;

        let beginning = Instant::now();

        loop {
            if beginning.elapsed() >= next_tick {
                let mut secs = beginning.elapsed().as_secs() as f64;
                secs += beginning.elapsed().subsec_nanos() as f64 / 1_000_000_000 as f64;
                println!("Tick: {}", ticks);
                println!("Second: {}", secs);
                println!("Ticks per second: {}", ticks as f64 / secs);

                ticks += 1;
                next_tick += tick_length;
            } else {
                ring(beginning, ticks, next_tick);
            }
        }
    }
}
