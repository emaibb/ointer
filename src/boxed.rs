use std::boxed::Box;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Error, Formatter, Pointer};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;

use crate::ointer::*;

define_ointer!(OBox, Box);
define_ointer_deref!(OBox);
define_ointer_deref_mut!(OBox);
define_ointer_methods_and_traits!(OBox, Box);
