extern crate noser;
extern crate noserc;

pub mod net {
    use noser::{List, Literal};
    use noserc::{Build, SizableDynamic};

    #[derive(Build, SizableDynamic)]
    pub enum Direction {
        North,
        East,
        West,
        South,
    }

    #[derive(Build, SizableDynamic)]
    pub enum Action {
        Move(Direction),
        Shoot(Direction),
        Jump,
    }

    #[derive(Build, SizableDynamic)]
    pub struct Proto<'a> {
        frame: Literal<'a, u8>,
        actions: List<'a, Action>,
    }
}

pub mod game {
    use std::time::{Duration, Instant};

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

        fn take_action(&mut self, action: Action) {
            match action {
                Action::Move(mov) => match mov {
                    Direction::South => self.pos.0 = self.pos.0.wrapping_sub(1),
                    Direction::North => self.pos.0 = self.pos.0.wrapping_add(1),
                    Direction::West => self.pos.1 = self.pos.1.wrapping_sub(1),
                    Direction::East => self.pos.1 = self.pos.1.wrapping_add(1),
                },
                Action::Shoot(_shoot) => println!("shoot"),
                Action::Jump(_jump) => println!("jump"),
            }
        }
    }

    impl GameState {
        pub fn advance<A: ReaderSegments>(
            &mut self,
            player: &mut Player,
            controls: Proto,
        ) {
            debug_assert!(self.frame == controls.frame.read());
            // for i in 0..controls.actions.capacity() {
            //     match a.which() {
            //         Ok(action) => player.take_action(action),
            //         Err(_) => unreachable!(),
            //     }
            // }

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
