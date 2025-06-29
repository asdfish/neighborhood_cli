use {std::mem::ManuallyDrop, zeroizing::Zeroizing};

pub trait ZeroizingExt<T> {
    fn read<F>(&self)
    where
        F: Fn(ManuallyDrop<T>);
}
