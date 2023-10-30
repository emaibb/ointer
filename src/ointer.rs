pub unsafe trait Ointer<const N: usize> {
    type Pointer;
    const LOW_MASK: usize = { !0usize >> N };
    const HIGH_MASK: usize = { !Self::LOW_MASK };
    const MIN_SIGNED: isize = { isize::MIN >> Self::SHIFT_BITS };
    const MAX_SIGNED: isize = { isize::MAX >> Self::SHIFT_BITS };
    const SHIFT_BITS: usize = {
        if cfg!(target_pointer_width = "128") {
            128 - N
        } else if cfg!(target_pointer_width = "64") {
            64 - N
        } else if cfg!(target_pointer_width = "32") {
            32 - N
        } else if cfg!(target_pointer_width = "16") {
            16 - N
        } else {
            panic!("Unsupported target pointer width")
        }
    };
    /// Get high `N` bits and return `false` if they are all `0`.
    #[inline(always)]
    fn get_bool(&self) -> bool {
        self.get_usize() != 0
    }
    /// Get high `N` bits and cast as `isize`.
    #[inline(always)]
    fn get_isize(&self) -> isize {
        unsafe { *(self as *const Self as *const isize) >> Self::SHIFT_BITS }
    }
    /// Get high `N` bits and cast as `usize`.
    #[inline(always)]
    fn get_usize(&self) -> usize {
        unsafe { *(self as *const Self as *const usize) >> Self::SHIFT_BITS }
    }
    /// Get stored pointer and cast as `usize`.
    #[inline(always)]
    fn get_ptr_as_usize(&self) -> usize {
        unsafe { *(self as *const Self as *const usize) & Self::LOW_MASK }
    }
    /// Set high `N` bits all to `1` if `true`, all to `0` if `false`.
    #[inline(always)]
    fn set_bool(&mut self, b: bool) {
        self.set_isize(if b { -1 } else { 0 })
    }
    /// Set high `N` bits from `isize`.
    #[inline(always)]
    fn set_isize(&mut self, i: isize) {
        if i < Self::MIN_SIGNED || i > Self::MAX_SIGNED {
            panic!("No enough bits to be stolen.")
        }
        let p = self as *mut Self as *mut usize;
        unsafe {
            *p = (*p & Self::LOW_MASK) | ((i as usize) << Self::SHIFT_BITS);
        }
    }
    /// Set high `N` bits from `usize`.
    #[inline(always)]
    fn set_usize(&mut self, u: usize) {
        if (u >> N) != 0 {
            panic!("No enough bits to be stolen.")
        }
        let p = self as *mut Self as *mut usize;
        unsafe {
            *p = (*p & Self::LOW_MASK) | (u << Self::SHIFT_BITS);
        }
    }
    /// Store pointer to low bits.
    #[inline(always)]
    fn set_ptr(&mut self, p: &mut Self::Pointer) {
        let u = unsafe { *(p as *mut Self::Pointer as *mut usize) };
        unsafe {
            *(self as *mut Self as *mut usize) = u;
        }
        self.assert_stealable();
    }
    /// Assert high `N` bits is all `0`.
    #[inline(always)]
    fn assert_stealable(&self) {
        assert_eq!(self.get_bool(), false);
    }
    /// Get high `N` bits and cast as `T`.
    #[inline(always)]
    fn get<T: Copy>(&self) -> T
    where
        Self: OinterGet<T, N>,
    {
        self.get_high_bits()
    }
    /// Set high `N` bits from `T`.
    #[inline(always)]
    fn set_mut<T: Copy>(&mut self, x: T)
    where
        Self: OinterSet<T, N>,
    {
        self.set_high_bits_mut(x);
    }
    /// Map `&Self` as `&T`(from high `N` bits) and `&Self::Pointer`, then map fn `f`.
    #[inline(always)]
    fn map<T: Copy, R, F: FnOnce(T, &Self::Pointer) -> R>(&self, f: F) -> R
    where
        Self: OinterGet<T, N>,
    {
        let x = self.get_high_bits();
        let u = self.get_ptr_as_usize();
        let p = unsafe { &*(&u as *const usize as *const Self::Pointer) };
        f(x, p)
    }
    /// Map `&mut Self` as `&mut T`(from high `N` bits) and `&mut Self::Pointer`, map fn `f`, then store changes back.
    #[inline(always)]
    fn map_mut<T: Copy, R, F: FnOnce(&mut T, &mut Self::Pointer) -> R>(&mut self, f: F) -> R
    where
        Self: OinterGet<T, N> + OinterSet<T, N>,
    {
        let mut x = self.get_high_bits();
        let mut u = self.get_ptr_as_usize();
        let p = unsafe { &mut *(&mut u as *mut usize as *mut Self::Pointer) };
        let ret = f(&mut x, p);
        self.set_ptr(p);
        self.set_high_bits_mut(x);
        ret
    }
}

pub unsafe trait OinterGet<T: Copy, const N: usize>: Ointer<N> {
    /// Get high `N` bits and cast as `T`.
    #[inline(always)]
    fn get_high_bits(&self) -> T {
        use core::mem::size_of;
        let u = self.get_usize();
        let x = if size_of::<T>() == 8 {
            unsafe { *(&(u as u64) as *const u64 as *const T) }
        } else if size_of::<T>() == 4 {
            unsafe { *(&(u as u32) as *const u32 as *const T) }
        } else if size_of::<T>() == 2 {
            unsafe { *(&(u as u16) as *const u16 as *const T) }
        } else if size_of::<T>() == 1 {
            unsafe { *(&(u as u8) as *const u8 as *const T) }
        } else {
            panic!("Unsupported value size")
        };
        x
    }
}

pub unsafe trait OinterSet<T: Copy, const N: usize>: Ointer<N> {
    /// Set high `N` bits from `T`.
    #[inline(always)]
    fn set_high_bits_mut(&mut self, x: T) {
        use core::mem::size_of;
        let u: usize = if size_of::<T>() == 8 {
            unsafe { *(&x as *const T as *const u64) }
                .try_into()
                .unwrap()
        } else if size_of::<T>() == 4 {
            unsafe { *(&x as *const T as *const u32) }
                .try_into()
                .unwrap()
        } else if size_of::<T>() == 2 {
            unsafe { *(&x as *const T as *const u16) }
                .try_into()
                .unwrap()
        } else if size_of::<T>() == 1 {
            unsafe { *(&x as *const T as *const u8) }
                .try_into()
                .unwrap()
        } else {
            panic!("Unsupported value size")
        };
        self.set_usize(u);
    }
}

unsafe impl<const N: usize, T: Copy, Ty: Ointer<N>> OinterGet<T, N> for Ty {}
unsafe impl<const N: usize, T: Copy, Ty: Ointer<N>> OinterSet<T, N> for Ty {}

/// Macro used to define `Weak` like `ointer`s.
#[macro_export]
macro_rules! define_ointer {
    ($ointer:ident, $pointer:ident, $bits:literal) => {
        #[repr(transparent)]
        pub struct $ointer<T>($pointer<T>);

        unsafe impl<T> Ointer<$bits> for $ointer<T> {
            type Pointer = $pointer<T>;
        }

        impl<T> core::convert::From<$pointer<T>> for $ointer<T> {
            fn from(p: $pointer<T>) -> Self {
                let s = Self(p);
                s.assert_stealable();
                s
            }
        }

        impl<T> core::default::Default for $ointer<T>
        where
            Self: Ointer<$bits, Pointer = $pointer<T>>,
            <Self as Ointer<$bits>>::Pointer: core::default::Default,
        {
            fn default() -> Self {
                $pointer::default().into()
            }
        }

        impl<T> core::clone::Clone for $ointer<T>
        where
            Self: Ointer<$bits, Pointer = $pointer<T>>,
            <Self as Ointer<$bits>>::Pointer: Clone,
        {
            fn clone(&self) -> Self {
                self.map(|u: usize, p| {
                    let mut o = Self(p.clone());
                    o.set_usize(u);
                    o
                })
            }
        }

        impl<T> core::fmt::Debug for $ointer<T>
        where
            Self: Ointer<$bits, Pointer = $pointer<T>>,
            <Self as Ointer<$bits>>::Pointer: core::fmt::Debug,
        {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                self.map(|u: usize, p| (u, p).fmt(f))
            }
        }

        impl<T> core::ops::Drop for $ointer<T>
        where
            Self: Ointer<$bits>,
        {
            fn drop(&mut self) {
                self.set_bool(false);
            }
        }

        impl<T> core::hash::Hash for $ointer<T>
        where
            Self: Ointer<$bits, Pointer = $pointer<T>>,
            <Self as Ointer<$bits>>::Pointer: core::hash::Hash,
        {
            fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                self.map(|u: usize, p| (u, p).hash(state))
            }
        }

        impl<T> core::cmp::PartialEq for $ointer<T>
        where
            Self: Ointer<$bits, Pointer = $pointer<T>>,
            <Self as Ointer<$bits>>::Pointer: core::cmp::PartialEq,
        {
            fn eq(&self, rhs: &Self) -> bool {
                self.map(|u: usize, p| rhs.map(|c, q| (u, p).eq(&(c, q))))
            }
        }

        impl<T> core::cmp::PartialOrd for $ointer<T>
        where
            Self: Ointer<$bits, Pointer = $pointer<T>>,
            <Self as Ointer<$bits>>::Pointer: core::cmp::PartialOrd,
        {
            fn partial_cmp(&self, rhs: &Self) -> Option<core::cmp::Ordering> {
                self.map(|u: usize, p| rhs.map(|c, q| (u, p).partial_cmp(&(c, q))))
            }
        }

        impl<T> core::ops::Deref for $ointer<T>
        where
            Self: Ointer<$bits, Pointer = $pointer<T>>,
            <Self as Ointer<$bits>>::Pointer: core::ops::Deref<Target = T>,
        {
            type Target = T;
            fn deref(&self) -> &T {
                self.map(|_: usize, p| unsafe { &*(p.deref() as *const T) })
            }
        }

        impl<T> core::ops::DerefMut for $ointer<T>
        where
            Self: Ointer<$bits, Pointer = $pointer<T>>,
            <Self as Ointer<$bits>>::Pointer: core::ops::DerefMut<Target = T>,
        {
            fn deref_mut(&mut self) -> &mut T {
                self.map_mut(|_: &mut usize, p| unsafe { &mut *(p.deref_mut() as *mut T) })
            }
        }
    };
}

macro_rules! define_ointer_methods {
    ($ointer:ident, $pointer:ident, $bits:literal) => {
        impl<T> $ointer<T>
        where
            Self: Ointer<$bits, Pointer = $pointer<T>>,
        {
            pub fn new(x: T) -> Self {
                $pointer::new(x).into()
            }
            pub fn pin(x: T) -> core::pin::Pin<Self> {
                unsafe { core::pin::Pin::new_unchecked(Self::new(x)) }
            }
        }
    };
}

pub(crate) use define_ointer_methods;

/// Macro used to define `Box`/`Rc`/`Arc` like `ointer`s.
/// This crate defines `BBox`(called byte stolen `Box`) that wraps `Box` and and steal high 8-bits(1-byte), by using
/// `define_ointer_strong!(BBox, Box, 8);`
/// And define `OBox`(called orientable `Box`) by using
/// `define_ointer_strong!(OBox, Box, 1);`
///
/// Tests over`OBox`
/// ```
/// use ointer::{OBox, Ointer};
/// use std::pin::Pin;
/// let mut o = OBox::new(1);
/// assert_eq!(*o, 1);
/// assert_eq!(o.get::<bool>(), false);
/// assert_eq!(*o, 1);
/// *o = i32::default();
/// assert_eq!(*o, i32::default());
/// o.set_bool(true);
/// let b = o.get_bool();
/// assert_eq!(b, true);
/// o.set_mut(false);
/// assert_eq!(o, Pin::into_inner(OBox::pin(Default::default())));
/// ```
#[macro_export]
macro_rules! define_ointer_strong {
    ($ointer:ident, $pointer:ident, $bits:literal) => {
        define_ointer!($ointer, $pointer, $bits);
        define_ointer_methods!($ointer, $pointer, $bits);
    };
}

/// Macro used to define `Rc/Weak` or `Arc/Weak` like shared `ointer`s.
/// This crate defines `BRc/BWeak`(called byte stolen `Rc/Weak`) that wraps `Rc/Weak` and and steal high 8-bits(1-byte), by using
/// `define_shared_ointer!(BRc, Rc, BWeak, Weak, 8);`
/// And define `ORc/OWeak`(called orientable `Rc/Weak`) by using
/// `define_shared_ointer!(ORc, Rc, OWeak, Weak, 1);`
///
/// Tests over `BArc`
/// ```
/// use ointer::{sync::BArc, Ointer};
/// use core::mem::size_of;
/// let mut o = BArc::new(1);
/// assert_eq!(*o, 1);
/// assert_eq!(o.get::<bool>(), false);
///
/// // Define a small enum for testing.
/// #[derive(Clone, Copy, PartialEq, Debug)]
/// enum MySmallEnum {
///     _A,
///     B,
///     _C,
/// }
/// assert_eq!(size_of::<MySmallEnum>(), 1);
///
/// o.set_mut(MySmallEnum::B);
/// assert_eq!(*o, 1);
/// assert_eq!(o.get::<MySmallEnum>(), MySmallEnum::B);
///
/// // Modify the bool and pointer inside the ointer.
/// o.map_mut(|b: &mut bool, p| {
///     *b = !*b;
///     *p = Default::default();
/// });
/// assert_eq!(*o.downgrade().upgrade().unwrap(), Default::default());
/// ```
#[macro_export]
macro_rules! define_shared_ointer {
    ($ointer_strong:ident, $pointer_strong:ident, $ointer_weak:ident, $pointer_weak:ident, $bits:literal) => {
        define_ointer_strong!($ointer_strong, $pointer_strong, $bits);
        define_ointer!($ointer_weak, $pointer_weak, $bits);
        impl<T> $ointer_strong<T> {
            pub fn downgrade(&self) -> $ointer_weak<T> {
                self.map(|u: usize, p| {
                    let mut o: $ointer_weak<T> = $pointer_strong::downgrade(p).into();
                    o.set_usize(u);
                    o
                })
            }
            pub fn strong_count(&self) -> usize {
                self.map(|_: usize, p| $pointer_strong::strong_count(p))
            }
            pub fn weak_count(&self) -> usize {
                self.map(|_: usize, p| $pointer_strong::weak_count(p))
            }
        }
        impl<T> $ointer_weak<T> {
            pub fn upgrade(&self) -> Option<$ointer_strong<T>> {
                self.map(|u: usize, w| {
                    let p = w.upgrade();
                    p.map(|p| {
                        let mut o: $ointer_strong<T> = p.into();
                        o.set_usize(u);
                        o
                    })
                })
            }
        }
    };
}

/// Macro used to define custom enum `ointer`s with the same size of `usize`.
///
/// Example testing usage:
/// ```
/// use ointer::{define_enum_ointers, Ointer};
/// use std::sync::Arc;
/// use core::mem::size_of;
/// let mut a = Arc::new(13);
/// // Define custom enum ointers using MyEnumOinters.
/// define_enum_ointers!(
///     MyEnumOinters {
///         Box<f64> = 1,
///         Arc<i32> = 2,
///         i32 = 5
///     },
///     8
/// );
/// let mut e = MyEnumOinters::new(2, a.clone());
/// assert_eq!(Arc::strong_count(&a), 2);
/// // Perform operations on the enum ointer.
/// assert_eq!(
///     e.map_mut(
///         |_| panic!(),
///         |p| {
///             let i = **p;
///             *p = Arc::new(15);
///             assert_eq!(Arc::strong_count(&a), 1);
///             a = p.clone();
///             i
///         },
///         |_| panic!()
///     ),
///     13
/// );
/// assert_eq!(e.map(|_| panic!(), |p1| **p1, |_| panic!()), 15);
/// assert_eq!(Arc::strong_count(&a), 2);
/// // Set the enum ointer to a new value (Box<f64>).
/// e.set_mut(1, Box::new(2.0));
/// assert_eq!(Arc::strong_count(&a), 1);
/// assert_eq!(e.map(|p| **p, |_| panic!(), |_| panic!()), 2.0);
/// assert_eq!(size_of::<MyEnumOinters>(), size_of::<usize>());
/// ```
#[macro_export]
macro_rules! define_enum_ointers {
    (
        $name:ident {
            $($pointer:ty = $unsigned:literal),*
        },
        $bits:literal
    ) => {
        paste::paste!{
            #[repr(transparent)]
            struct [<$name Inner>](usize);

            unsafe impl Ointer<$bits> for [<$name Inner>] {
                type Pointer = usize;
            }

            #[repr(transparent)]
            pub struct $name([<$name Inner>]);

            impl $name {
                #[inline(always)]
                pub fn new<P: 'static>(u: usize, p: P) -> Self {
                    use core::any::TypeId;
                    use core::mem::size_of;
                    match u {
                        $($unsigned => {
                            if TypeId::of::<P>() != TypeId::of::<$pointer>() {
                                panic!("Unmatched pointer type")
                            }
                            if size_of::<P>() > size_of::<usize>() {
                                panic!("Size overflow")
                            }
                            let mut inner = [<$name Inner>](unsafe {
                                *(&p as *const P as *const usize)
                            });
                            inner.set_usize(u);
                            core::mem::forget(p);
                            $name(inner)
                        }),
                        *,
                        _ => panic!("Unmatched unsigned num")
                    }
                }
                #[inline(always)]
                pub fn set_mut<P: 'static>(&mut self, u: usize, p: P) {
                    *self = Self::new(u, p);
                }
                #[inline(always)]
                pub fn map<
                    R,
                    $([<F $unsigned>]: FnOnce(&$pointer) -> R),
                    *
                >(
                    &self,
                    $([<f $unsigned>]: [<F $unsigned>]),
                    *
                ) -> R {
                    let u = self.0.get_usize();
                    match u {
                        $($unsigned => {
                            let mut u = self.0.get_ptr_as_usize();
                            let p = unsafe {
                                &mut *(&mut u as *mut usize as *mut Self)
                            };
                            p.0.set_usize(0);
                            [<f $unsigned>](unsafe {
                                &*(p as *const Self as *const $pointer)
                            })
                        }),
                        *,
                        _ => panic!("Unmatched unsigned num")
                    }
                }
                #[inline(always)]
                pub fn map_mut<
                    R,
                    $([<F $unsigned>]: FnOnce(&mut $pointer) -> R),
                    *
                >(
                    &mut self,
                    $([<f $unsigned>]: [<F $unsigned>]),
                    *
                ) -> R {
                    let u = self.0.get_usize();
                    match u {
                        $($unsigned => {
                            self.0.set_usize(0);
                            let r = [<f $unsigned>](unsafe {
                                &mut *(self as *mut Self as *mut $pointer)
                            });
                            self.0.set_usize(u);
                            r
                        }),
                        *,
                        _ => panic!("Unmatched unsigned num")
                    }
                }
            }

            impl core::clone::Clone for $name
                where
                    $(
                        $pointer: core::clone::Clone
                    ), *
            {
                fn clone(&self) -> Self {
                    self.map($(|p| Self::new($unsigned, p.clone())), *)
                }
            }

            impl core::ops::Drop for $name {
                fn drop(&mut self) {
                    self.map_mut(
                        $(|p| {
                            $unsigned;
                            unsafe {
                                core::mem::ManuallyDrop::drop(
                                    &mut *(
                                        p as *mut $pointer
                                        as *mut core::mem::ManuallyDrop<$pointer>
                                    )
                                );
                            }
                        }), *
                    );
                }
            }
        }
    };
}

pub use define_enum_ointers;
pub use define_ointer;
pub use define_ointer_strong;
pub use define_shared_ointer;
