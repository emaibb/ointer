use std::sync::{Arc, Weak};
use std::rc::{Rc, Weak as Wk};

pub trait Ointer {
    type Pointer;
    fn o(&self) -> bool;
    fn flip(&mut self);
    fn pointer(&self) -> &Self::Pointer;
    fn pointer_mut(&mut self) -> &mut Self::Pointer;
}

macro_rules! define_ointer {
    ($ointer:ident, $pointer:ident) => {

        pub struct $ointer<T>($pointer<T>);

        impl<T> Ointer for $ointer<T> {
            type Pointer = $pointer<T>;

            fn o(&self) -> bool {
                unsafe {
                    *(self as *const $ointer<T> as *const isize) < 0
                }
            }
            fn flip(&mut self) {
                let p = self as *mut $ointer<T> as *mut isize;
                unsafe {
                    *p = -*p;
                }
            }
            fn pointer(&self) -> &Self::Pointer {
                let o = self;
                if o.o() {
                    let p = o as *const $ointer<T> as isize as *mut isize;
                    unsafe {
                        *p = -*p;
                    }
                }
                &o.0
            }
            fn pointer_mut(&mut self) -> &mut Self::Pointer {
                let o = self;
                if o.o() {
                    let p = o as *mut $ointer<T> as *mut isize;
                    unsafe {
                        *p = -*p;
                    }
                }
                &mut o.0
            }
        }

        impl<T> Drop for $ointer<T> where Self: Ointer {
            fn drop(&mut self) {
                if self.o() {
                    self.flip();
                }
            }
        }
    }
}

macro_rules! define_ointer_new {
    ($ointer:ident, $pointer:ident) => {
        impl<T> $ointer<T> {
            pub fn new(value: T) -> Self {
                Self($pointer::new(value))
            }
        }
    };
}

macro_rules! define_ointer_clone {
    ($ointer:ident, $pointer:ident) => {
        impl<T> Clone for $ointer<T> where Self: Ointer<Pointer = $pointer<T>> {
            fn clone(&self) -> Self {
                let mut o = Self(self.pointer().clone());
                if self.o() {
                    o.flip();
                }
                o
            }
        }

        impl<T> $ointer<T> where Self: Ointer + Clone {
            pub fn clone_and_flip(&self) -> Self {
                let mut o = self.clone();
                o.flip();
                o
            }
        }
    };
}

macro_rules! define_shared_ointer {
    ($ointer_strong:ident, $pointer_strong:ident, $ointer_weak:ident, $pointer_weak:ident) => {
        define_ointer!($ointer_strong, $pointer_strong);
        define_ointer!($ointer_weak, $pointer_weak);
        define_ointer_new!($ointer_strong, $pointer_strong);
        define_ointer_clone!($ointer_strong, $pointer_strong);
        define_ointer_clone!($ointer_weak, $pointer_weak);

        impl<T> $ointer_strong<T> {
            pub fn downgrade(&self) -> $ointer_weak<T> {
                let p = $pointer_strong::downgrade(&self.pointer());
                let mut o = $ointer_weak(p);
                if self.o() {
                    o.flip();
                }
                o
            }
        }
        
        impl<T> $ointer_weak<T> {
            pub fn upgrade(&self) -> Option<$ointer_strong<T>> {
                let p = self.pointer().upgrade();
                p.map(|p| {
                    let mut o = $ointer_strong(p);
                    if self.o() {
                        o.flip();
                    }
                    o
                })
            }
        }        
    };
}

define_ointer!(Ox, Box);
define_ointer_new!(Ox, Box);
define_shared_ointer!(Oc, Rc, Ok, Wk);
define_shared_ointer!(Orc, Arc, Oak, Weak);

/// test
#[cfg(test)]
mod tests {
    use std::{ops::Deref, mem::size_of};

    use super::*;

    #[test]
    fn test() {
        {
            let mut p = Ox::new(1);
            assert_eq!(p.o(), false);
            p.flip();
            assert_eq!(p.o(), true);
            assert_eq!(*p.pointer().deref(), 1);
        }
        {
            let mut p = Orc::new(1);
            assert_eq!(p.o(), false);
            p.flip();
            assert_eq!(p.o(), true);
            assert_eq!(*p.pointer().deref(), 1);
            *p.pointer_mut() = Arc::new(2);
            assert_eq!(*p.downgrade().upgrade().unwrap().pointer().deref(), 2);
        }
        assert_eq!(8, size_of::<Option<Oc<i32>>>());
    }
}