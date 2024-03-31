pub trait RefTo<T> {
    fn ref_to(&self) -> T;
}