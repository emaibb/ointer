//! This module defines `ointer`s that wraps `Arc/Weak`, named `BArc/BWeak`(called byte stolen `Arc/Weak`) and `OArc/OWeak`(called orientable `Arc/Weak`, with 1 bit stolen).

use crate::ointer::*;
use std::sync::{Arc, Weak};

define_shared_ointer!(OArc, Arc, OWeak, Weak, 1);
define_shared_ointer!(BArc, Arc, BWeak, Weak, 8);
