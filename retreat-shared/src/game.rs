mod game {
    #[derive(Serialize, Deserialize)]
    enum Action {
        MOVE(x: u8, y: u8)
    }

    #[derive(Serialize, Deserialize)]
    struct Actions {
        frame: u8,
        actions: Vec<Action>
    }

    struct Player {
        id: u8
        pos: (x: u8, y: u8)
    }

    struct GameState {
        frame: u8,
        world: [u8, 3*3],
    }

    impl GameState {
        fn advance(&self, player: Player, actions: Actions) {

        }
    }
}
