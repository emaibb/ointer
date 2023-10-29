use crate::ointer::*;
use std::rc::{Rc, Weak};

define_shared_ointer!(ORc, Rc, OWeak, Weak, 1);
define_shared_ointer!(BRc, Rc, BWeak, Weak, 8);
