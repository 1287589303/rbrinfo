#![feature(rustc_private)]
#![feature(custom_mir)]

extern crate rustc_ast;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;
#[macro_use]
extern crate log;

// Modules for static analyses
pub mod analysis {
    // Definitions of callbacks for rustc
    pub mod callback;
    pub mod option;
}

// Useful utilities
pub mod utils;
