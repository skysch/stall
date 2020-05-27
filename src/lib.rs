////////////////////////////////////////////////////////////////////////////////
// Stall -- a simple local configuration management utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer.
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Stall library modules.
////////////////////////////////////////////////////////////////////////////////
// #![doc(html_root_url = "https://docs.rs/palette/0.2.1")]
#![warn(anonymous_parameters)]
#![warn(bad_style)]
#![warn(bare_trait_objects)]
#![warn(const_err)]
#![warn(dead_code)]
#![warn(elided_lifetimes_in_paths)]
#![warn(improper_ctypes)]
// #![warn(missing_copy_implementations)]
// #![warn(missing_debug_implementations)]
// #![warn(missing_docs)]
// #![warn(missing_doc_code_examples)]
#![warn(no_mangle_generic_items)]
#![warn(non_shorthand_field_patterns)]
#![warn(overflowing_literals)]
#![warn(path_statements)]
#![warn(patterns_in_fns_without_body)]
#![warn(private_in_public)]
#![warn(rust_2018_idioms)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unconditional_recursion)]
#![warn(unreachable_pub)]
// #![warn(unused)]
#![warn(unused_allocation)]
#![warn(unused_comparisons)]
#![warn(unused_parens)]
// #![warn(unused_qualifications)]
// #![warn(unused_results)]
#![warn(variant_size_differences)]
#![warn(while_true)]


mod stall_file;
pub mod utility;

pub use stall_file::*;
