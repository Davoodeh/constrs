#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), no_main, no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod ip;
pub mod num;
pub mod percent_encoding;
pub mod url;
pub mod utf8;
