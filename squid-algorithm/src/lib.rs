//! # squid-algorithm
//!
//! crazy algorithms to quickly rank the most frequently used words in a sentence!
//! Supported algorithms:
//! - HashMap;

#![forbid(unsafe_code)]
#![deny(dead_code, unused_imports, unused_mut, missing_docs)]

/// The most accurate algorithm for ranking.
pub mod hashtable;
