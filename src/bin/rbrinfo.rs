#![feature(rustc_private)]
#![feature(box_patterns)]

extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;

use log::info;

use rbrinfo::analysis::option;
use rbrinfo::{analysis, utils};
use rustc_errors::emitter::HumanReadableErrorType;
use rustc_errors::ColorConfig;
use rustc_session::config::ErrorOutputType;
use rustc_session::EarlyDiagCtxt;
use std::env;
use std::process;

/// Exit status code used for successful compilation and help output.
pub const EXIT_SUCCESS: i32 = 0;

/// Exit status code used for compilation failures and invalid flags.
pub const EXIT_FAILURE: i32 = 1;

fn main() {
    let result = rustc_driver::catch_fatal_errors(move || {
        let mut rustc_args = env::args_os()
            .enumerate()
            .map(|(i, arg)| arg.into_string().unwrap())
            .collect::<Vec<_>>();

        if let Some(sysroot) = utils::compile_time_sysroot() {
            let sysroot_flag = "--sysroot";
            if !rustc_args.iter().any(|e| e == sysroot_flag) {
                // We need to overwrite the default that librustc would compute.
                rustc_args.push(sysroot_flag.to_owned());
                rustc_args.push(sysroot);
            }
        }

        // If this environment variable is set, we behave just like the real rustc
        if env::var_os("RBRINFO_BE_RUSTC").is_some() {
            let early_diag_ctxt: EarlyDiagCtxt = EarlyDiagCtxt::new(
                ErrorOutputType::HumanReadable(HumanReadableErrorType::Default(ColorConfig::Auto)),
            );
            rustc_driver::init_rustc_env_logger(&early_diag_ctxt);
            // We cannot use `rustc_driver::main` as we need to adjust the CLI arguments.
            let mut callbacks = rustc_driver::TimePassesCallbacks::default();
            let run_compiler = rustc_driver::RunCompiler::new(&rustc_args, &mut callbacks);
            run_compiler.run()
        } else {
            let always_encode_mir = "-Zalways_encode_mir";
            if !rustc_args.iter().any(|e| e == always_encode_mir) {
                // Get MIR code for all code related to the crate (including the dependencies and standard library)
                rustc_args.push(always_encode_mir.to_owned());
            }

            // Add this to support analyzing no_std libraries
            // rustc_args.push("-Clink-arg=-nostartfiles".to_owned());

            // Disable unwind to simplify the CFG
            rustc_args.push("-Cpanic=abort".to_owned());

            let analysis_options = option::AnalysisOption::from_args(&mut rustc_args);
            info!("Analysis Option: {:?}", analysis_options);

            let mut callbacks = analysis::callback::MirCheckerCallbacks::new(analysis_options);

            let run_compiler = rustc_driver::RunCompiler::new(&rustc_args, &mut callbacks);
            run_compiler.run()
        }
    });

    let exit_code = match result {
        Ok(_) => EXIT_SUCCESS,
        Err(_) => EXIT_FAILURE,
    };

    process::exit(exit_code);
}
