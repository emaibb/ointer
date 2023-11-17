//! This module defines `ointer`s that wraps `Rc/Weak`, named `BRc/BWeak`(called byte stolen `Rc/Weak`) and `ORc/OWeak`(called orientable `Rc/Weak`, with 1 bit stolen).

use crate::ointer::*;
use std::rc::{Rc, Weak};

define_shared_ointer!(ORc, Rc, OWeak, Weak, 1);
define_shared_ointer!(BRc, Rc, BWeak, Weak, 8);
