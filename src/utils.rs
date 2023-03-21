/// Newtype pattern wrapper. You can use `W<T>` to implement foreign traits on a foreign type.
pub struct W<T>(pub T);

impl<T> From<T> for W<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}
