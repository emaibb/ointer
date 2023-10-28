use std::cmp::Ordering;
use std::fmt::{Debug, Display, Error, Formatter, Pointer};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::pin::Pin;

use crate::ointer::*;
use std::rc::{Rc, Weak};

define_shared_ointer!(ORc, Rc, OWeak, Weak);
