//! This module defines `ointer`s that wraps `Box`, named `BBox`(called byte stolen `Box`) and `OBox`(called orientable `Box`, with 1 bit stolen).

use crate::ointer::*;

define_ointer_strong!(OBox, Box, 1);
define_ointer_strong!(BBox, Box, 8);
