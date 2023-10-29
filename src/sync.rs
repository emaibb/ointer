use crate::ointer::*;
use std::sync::{Arc, Weak};

define_shared_ointer!(OArc, Arc, OWeak, Weak, 1);
define_shared_ointer!(BArc, Arc, BWeak, Weak, 8);
