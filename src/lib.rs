//! `Ointer` use the first bit of pointer data to store an extra boolean value, with `Box/Rc(Weak)/Arc(Weak)` wrapped.

pub mod boxed;
pub mod ointer;
pub mod rc;
pub mod sync;

pub use {boxed::*, ointer::*};

/// test
#[cfg(test)]
mod tests {
    use super::{rc::ORc, sync::*, *};
    use std::{mem::size_of, pin::Pin, sync::Arc};

    #[test]
    fn test() {
        {
            let mut o = OBox::new(1);
            assert_eq!(*o, 1);
            assert_eq!(o.o(), false);
            o.flip();
            assert_eq!(*o, 1);
            assert_eq!(o.o(), true);
            *o = i32::default();
            assert_eq!(*o, i32::default());
            assert_eq!(o.o(), true);
            o.flip();
            assert_eq!(o, Pin::into_inner(OBox::pin(Default::default())));
        }
        {
            let mut o = OBrc::new(1);
            assert_eq!(*o, 1);
            assert_eq!(o.o(), false);
            o.flip();
            assert_eq!(*o, 1);
            assert_eq!(o.o(), true);
            o.apply_mut(|b, p| {
                *b = !*b;
                *p = Default::default();
            });
            assert_eq!(*o.downgrade().upgrade().unwrap(), Default::default());
        }
        assert_eq!(size_of::<Arc<i32>>(), size_of::<Option<ORc<i32>>>());
    }
}
