#![crate_name = "mithril"]
#![crate_type = "lib"]
#![feature(repr_simd)]
#![feature(integer_atomics)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate thiserror;

#[cfg(test)]
#[macro_use]
extern crate test_case;

pub mod bandit_tools;
pub mod byte_string;
pub mod config;
pub mod metric;
pub mod randomx;
pub mod stratum;
pub mod timer;
pub mod worker;
