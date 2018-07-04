use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy)]
struct Token(usize);

trait SizeChangeListener {
    fn size_changed(&self, u32, Token);
}

trait ListenerManager<'a> {
    fn register(&mut self, &'a SizeChangeListener, Token);
    fn size_changed(&self, u32);
}

struct VecListenerManager<'a> {
    listeners: Vec<(&'a SizeChangeListener, Token)>,
}

impl<'a> VecListenerManager<'a> {
    fn new() -> Self {
        VecListenerManager {
            listeners: Vec::new(),
        }
    }
}

impl<'a> ListenerManager<'a> for VecListenerManager<'a> {
    fn register(&mut self, listener: &'a SizeChangeListener, id: Token) {
        self.listeners.push((listener, id));
    }

    fn size_changed(&self, size: u32) {
        for (listener, id) in &self.listeners {
            listener.size_changed(size, *id);
        }
    }
}

struct NoopListenerManager;

impl<'a> ListenerManager<'a> for NoopListenerManager {
    fn register(&mut self, listener: &'a SizeChangeListener, id: Token) {}
    fn size_changed(&self, size: u32) {}
}

const NOOP_CALLBACK_MANAGER: NoopListenerManager = NoopListenerManager {};

trait NoserDynSized<'a> {
    fn listener_manager(&self) -> &'a ListenerManager;
    fn initial_size() -> u32;
}

trait NoserStaticallySized {
    fn size() -> u32;
}

impl<T: NoserStaticallySized> NoserDynSized<'static> for T {
    fn initial_size() -> u32 {
        Self::size()
    }

    fn listener_manager(&self) -> &'static ListenerManager {
        // Noop because statically sized types never change size, duh.
        &NOOP_CALLBACK_MANAGER
    }
}

trait NoserType<'a>: NoserDynSized<'a> {
    type Input;
    type Output;

    fn read(&[u8]) -> Self::Output;
    fn write(&mut [u8], Self::Input);
}

struct Owned<'a, N> {
    phantom: PhantomData<N>,
    arena: &'a mut [u8],
}

impl<'a, N: NoserType<'a>> Owned<'a, N> {
    fn new(arena: &'a mut [u8]) -> Owned<'a, N> {
        Owned {
            arena: arena,
            phantom: PhantomData,
        }
    }

    fn get(&self) -> N::Output {
        N::read(self.arena)
    }

    fn set(&mut self, value: N::Input) {
        N::write(&mut self.arena, value)
    }
}

struct NoserList<'a, T> {
    items: Vec<Owned<'a, T>>,
    lm: VecListenerManager<'a>,
}

impl<'a, T: NoserType<'a>> Index<u32> for NoserList<'a, T> {
    type Output = Owned<'a, T>;

    fn index<'b>(&'b self, index: u32) -> &'b Owned<'a, T> {
        &self.items[index as usize]
    }
}

impl<'a, T: NoserType<'a>> IndexMut<u32> for NoserList<'a, T> {
    fn index_mut<'b>(&'b mut self, index: u32) -> &'b mut Owned<'a, T> {
        &mut self.items[index as usize]
    }
}

impl<'a, T: NoserType<'a>> NoserList<'a, T> {
    fn with_capacity(arena: &'a mut [u8], num_elems: u32) -> NoserList<'a, T> {
        let mut items = Vec::with_capacity(num_elems as usize);

        let elem_size = T::initial_size() as usize;

        for i in 0..num_elems as usize {
            items.push(Owned::new(&mut arena[elem_size * i..elem_size * (i + 1)]));
        }

        NoserList {
            items: items,
            lm: VecListenerManager::new(),
        }
    }
}

impl<'a, T: NoserType<'a>> NoserDynSized<'a> for NoserList<'a, T> {
    fn listener_manager(&self) -> &'a ListenerManager {
        &self.lm
    }

    fn initial_size() -> u32 {
        0
    }
}

impl<'a, T: NoserType<'a>> NoserType<'a> for NoserList<'a, T> {
    type Input = Vec<T>;
    type Output = Vec<T>;

    fn read(arena: &[u8]) -> Vec<T> {
        // let result = Vec::new();

        // let num_elems = u32::read(arena);
        // let bytes = T::size() * num_elems;

        // for i in 0..bytes {}
        unimplemented!()
    }

    fn write(arena: &mut [u8], input: Vec<T>) {}
}

impl NoserType<'static> for u8 {
    type Input = u8;
    type Output = u8;

    fn read(arena: &[u8]) -> u8 {
        arena[0]
    }

    fn write(arena: &mut [u8], input: u8) {
        arena[0] = input
    }
}

impl NoserStaticallySized for u8 {
    fn size() -> u32 {
        1
    }
}

impl NoserType<'static> for u64 {
    type Input = u64;
    type Output = u64;

    fn read(arena: &[u8]) -> u64 {
        let mut result = 0;
        result |= arena[0] as u64;
        result |= (arena[1] as u64) << 8;
        result |= (arena[2] as u64) << 16;
        result |= (arena[3] as u64) << 24;
        result |= (arena[4] as u64) << 32;
        result |= (arena[5] as u64) << 40;
        result |= (arena[6] as u64) << 48;
        result |= (arena[7] as u64) << 56;
        result
    }

    fn write(arena: &mut [u8], input: u64) {
        arena[0] = input as u8;
        arena[1] = (input << 8) as u8;
        arena[2] = (input << 16) as u8;
        arena[3] = (input << 24) as u8;
        arena[4] = (input << 32) as u8;
        arena[5] = (input << 40) as u8;
        arena[6] = (input << 48) as u8;
        arena[7] = (input << 56) as u8;
    }
}

impl NoserStaticallySized for u64 {
    fn size() -> u32 {
        8
    }
}
