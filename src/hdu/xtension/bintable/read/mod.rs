use std::{mem::size_of, ptr::copy_nonoverlapping};

use log::error;

use crate::error::Error;

pub mod bytes;
pub mod deser;
pub mod visitor;
