//! This crate provides a set of traits and macros to enable the creation of custom pointers in Rust, allowing for the storage of extra information in the high bits of a pointer. This extra information can be of various types, and the crate provides utilities for working with these custom pointers efficiently. The crate also offers convenient macros for defining custom `ointer`s and `enum ointers` and managing them.

pub mod boxed;
pub mod ointer;
pub mod rc;
pub mod sync;

pub use {boxed::*, ointer::*};

/// Test the Ointer Library
#[cfg(test)]
mod tests {
    // Import the necessary modules and types.
    use super::{rc::*, sync::*, *};
    use std::{mem::size_of, pin::Pin, rc::Rc, sync::*};

    // Define a test function.
    #[test]
    fn test() {
        {
            // Test custom ointers (OBox).
            let mut o = OBox::new(1);
            assert_eq!(*o, 1);
            assert_eq!(o.get::<bool>(), false);
            assert_eq!(*o, 1);
            *o = i32::default();
            assert_eq!(*o, i32::default());
            o.set_bool(true);
            let b = o.get_bool();
            assert_eq!(b, true);
            o.set_mut(false);
            assert_eq!(o, Pin::into_inner(OBox::pin(Default::default())));
        }
        {
            // Test custom strong ointers (BArc).
            let mut o = BArc::new(1);
            assert_eq!(*o, 1);
            assert_eq!(o.get::<bool>(), false);

            // Define a small enum for testing.
            #[derive(Clone, Copy, PartialEq, Debug)]
            enum MySmallEnum {
                _A,
                B,
                _C,
            }
            assert_eq!(size_of::<MySmallEnum>(), 1);

            o.set_mut(MySmallEnum::B);
            assert_eq!(*o, 1);
            assert_eq!(o.get::<MySmallEnum>(), MySmallEnum::B);

            // Modify the bool and pointer inside the ointer.
            o.map_mut(|b: &mut bool, p| {
                *b = !*b;
                *p = Default::default();
            });
            assert_eq!(*o.downgrade().upgrade().unwrap(), Default::default());
        }
        {
            let mut a = Arc::new(13);

            // Define custom enum ointers using MyEnumOinters.
            define_enum_ointers!(
                MyEnumOinters {
                    Box<f64> = 1,
                    Arc<i32> = 2,
                    i32 = 5
                },
                8
            );

            let mut e = MyEnumOinters::new(2, a.clone());
            assert_eq!(Arc::strong_count(&a), 2);

            // Perform operations on the enum ointer.
            assert_eq!(
                e.map_mut(
                    |_| panic!(),
                    |p| {
                        let i = **p;
                        *p = Arc::new(15);
                        assert_eq!(Arc::strong_count(&a), 1);
                        a = p.clone();
                        i
                    },
                    |_| panic!()
                ),
                13
            );
            assert_eq!(e.map(|_| panic!(), |p1| **p1, |_| panic!()), 15);
            assert_eq!(Arc::strong_count(&a), 2);

            // Set the enum ointer to a new value (Box<f64>).
            e.set_mut(1, Box::new(2.0));
            assert_eq!(Arc::strong_count(&a), 1);
            assert_eq!(e.map(|p| **p, |_| panic!(), |_| panic!()), 2.0);
            assert_eq!(size_of::<MyEnumOinters>(), size_of::<usize>());
        }

        // Test size comparison of Rc<i32> and Option<BRc<i32>>.
        assert_eq!(size_of::<Rc<i32>>(), size_of::<Option<BRc<i32>>>());
    }
}
