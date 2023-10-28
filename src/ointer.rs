pub unsafe trait Ointer {
    type Pointer;
    /// Get orientation
    #[inline(always)]
    fn o(&self) -> bool {
        unsafe { *(self as *const Self as *const isize) < 0 }
    }
    /// Flip orientation
    #[inline(always)]
    fn flip(&mut self) {
        let p = self as *mut Self as *mut isize;
        unsafe { *p = -*p };
    }
    /// Apply fn
    #[inline(always)]
    fn apply<R, F: FnOnce(bool, &Self::Pointer) -> R>(&self, f: F) -> R {
        let b = self.o();
        let p = unsafe {
            let p = *(self as *const Self as *const isize);
            if b {
                -p
            } else {
                p
            }
        };
        let o = unsafe { &*(&p as *const isize as *const Self::Pointer) };
        f(b, o)
    }
    /// Apply fn mut
    #[inline(always)]
    fn apply_mut<R, F: FnOnce(&mut bool, &mut Self::Pointer) -> R>(&mut self, f: F) -> R {
        let mut b = self.o();
        let mut p = unsafe {
            let p = *(self as *mut Self as *mut isize);
            if b {
                -p
            } else {
                p
            }
        };
        let o = unsafe { &mut *(&mut p as *mut isize as *mut Self::Pointer) };
        let ret = f(&mut b, o);
        if b {
            p = -p;
        }
        unsafe {
            *(self as *mut Self as *mut isize) = p;
        }
        ret
    }
}

macro_rules! define_ointer {
    ($ointer:ident, $pointer:ident) => {
        #[repr(transparent)]
        pub struct $ointer<T>($pointer<T>);

        unsafe impl<T> Ointer for $ointer<T> {
            type Pointer = $pointer<T>;
        }

        impl<T> From<$pointer<T>> for $ointer<T> {
            fn from(p: $pointer<T>) -> Self {
                let o = Self(p);
                assert_eq!(o.o(), false);
                o
            }
        }

        impl<T> Default for $ointer<T>
        where
            Self: Ointer<Pointer = $pointer<T>>,
            <Self as Ointer>::Pointer: Default,
        {
            fn default() -> Self {
                $pointer::default().into()
            }
        }

        impl<T> Clone for $ointer<T>
        where
            Self: Ointer<Pointer = $pointer<T>>,
            <Self as Ointer>::Pointer: Clone,
        {
            fn clone(&self) -> Self {
                self.apply(|b, p| {
                    let mut o = Self(p.clone());
                    if b {
                        o.flip();
                    }
                    o
                })
            }
        }

        impl<T> $ointer<T>
        where
            Self: Ointer + Clone,
        {
            pub fn clone_and_flip(&self) -> Self {
                let mut o = self.clone();
                o.flip();
                o
            }
        }

        impl<T> Debug for $ointer<T>
        where
            Self: Ointer<Pointer = $pointer<T>>,
            <Self as Ointer>::Pointer: Debug,
        {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                self.apply(|b, p| (b, p).fmt(f))
            }
        }

        impl<T> Drop for $ointer<T>
        where
            Self: Ointer,
        {
            fn drop(&mut self) {
                if self.o() {
                    self.flip();
                }
            }
        }
    };
}

macro_rules! define_ointer_methods_and_traits {
    ($ointer:ident, $pointer:ident) => {
        impl<T> $ointer<T>
        where
            Self: Ointer<Pointer = $pointer<T>>,
        {
            pub fn new(x: T) -> Self {
                $pointer::new(x).into()
            }
            pub fn pin(x: T) -> Pin<Self> {
                unsafe { Pin::new_unchecked(Self::new(x)) }
            }
        }

        impl<T> Hash for $ointer<T>
        where
            Self: Ointer<Pointer = $pointer<T>>,
            <Self as Ointer>::Pointer: Hash,
        {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.apply(|b, p| (b, p).hash(state))
            }
        }

        impl<T> PartialEq for $ointer<T>
        where
            Self: Ointer<Pointer = $pointer<T>>,
            <Self as Ointer>::Pointer: PartialEq,
        {
            fn eq(&self, rhs: &Self) -> bool {
                self.apply(|b, p| rhs.apply(|c, q| (b, p).eq(&(c, q))))
            }
        }

        impl<T> PartialOrd for $ointer<T>
        where
            Self: Ointer<Pointer = $pointer<T>>,
            <Self as Ointer>::Pointer: PartialOrd,
        {
            fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
                self.apply(|b, p| rhs.apply(|c, q| (b, p).partial_cmp(&(c, q))))
            }
        }

        impl<T> Display for $ointer<T>
        where
            Self: Ointer<Pointer = $pointer<T>>,
            <Self as Ointer>::Pointer: Debug,
        {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                self.apply(|b, p| (b, p).fmt(f))
            }
        }

        impl<T> Pointer for $ointer<T>
        where
            Self: Ointer<Pointer = $pointer<T>>,
            <Self as Ointer>::Pointer: Debug,
        {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                self.apply(|b, p| (b, p).fmt(f))
            }
        }
    };
}

macro_rules! define_ointer_deref {
    ($ointer:ident) => {
        impl<T> Deref for $ointer<T> {
            type Target = T;
            fn deref(&self) -> &Self::Target {
                self.apply(|_, p| unsafe { &*(p.deref() as *const Self::Target) })
            }
        }
    };
}

macro_rules! define_ointer_deref_mut {
    ($ointer:ident) => {
        impl<T> DerefMut for $ointer<T> {
            fn deref_mut(&mut self) -> &mut T {
                self.apply_mut(|_, p| unsafe { &mut *(p.deref_mut() as *mut T) })
            }
        }
    };
}

macro_rules! define_shared_ointer {
    ($ointer_strong:ident, $pointer_strong:ident, $ointer_weak:ident, $pointer_weak:ident) => {
        define_ointer!($ointer_strong, $pointer_strong);
        define_ointer!($ointer_weak, $pointer_weak);
        define_ointer_deref!($ointer_strong);
        define_ointer_methods_and_traits!($ointer_strong, $pointer_strong);
        impl<T> $ointer_strong<T> {
            pub fn downgrade(&self) -> $ointer_weak<T> {
                self.apply(|b, p| {
                    let mut o: $ointer_weak<T> = $pointer_strong::downgrade(p).into();
                    if b {
                        o.flip();
                    }
                    o
                })
            }
            pub fn strong_count(&self) -> usize {
                self.apply(|_, p| $pointer_strong::strong_count(p))
            }
            pub fn weak_count(&self) -> usize {
                self.apply(|_, p| $pointer_strong::weak_count(p))
            }
        }
        impl<T> $ointer_weak<T> {
            pub fn upgrade(&self) -> Option<$ointer_strong<T>> {
                self.apply(|b, w| {
                    let p = w.upgrade();
                    p.map(|p| {
                        let mut o: $ointer_strong<T> = p.into();
                        if b {
                            o.flip();
                        }
                        o
                    })
                })
            }
        }
    };
}

pub(crate) use define_ointer;
pub(crate) use define_ointer_deref;
pub(crate) use define_ointer_deref_mut;
pub(crate) use define_ointer_methods_and_traits;
pub(crate) use define_shared_ointer;
