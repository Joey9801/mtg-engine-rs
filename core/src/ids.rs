#[derive(Clone, Debug)]
pub struct IdGenerator<T> {
    counter: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> IdGenerator<T> {
    pub fn new() -> Self {
        Self {
            counter: 0,
            _phantom: std::marker::PhantomData::<T>,
        }
    }
}

macro_rules! make_id_type {
    ($name:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(usize);

        impl IdGenerator<$name> {
            pub fn next_id(&mut self) -> $name {
                let ret = $name(self.counter);
                self.counter += 1;
                ret
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self)
            }
        }
    };
}

make_id_type!(PlayerId);
make_id_type!(ObjectId);
make_id_type!(ObserverId);
make_id_type!(ZoneId);
