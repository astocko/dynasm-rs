extern crate dynasmrt;

use std::ops::Deref;

use dynasmrt::{DynasmApi, DynasmLabelApi};

use encoding::ndisasm;

include!(concat!(env!("OUT_DIR"), "/tests.rs"));
