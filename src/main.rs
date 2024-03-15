#![feature(rustc_private)]
extern crate rustc_ast_pretty;
extern crate rustc_driver;
extern crate rustc_error_codes;
extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_infer;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_type_ir;

use proc_macro2::Ident;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use rustc_errors::registry;
use rustc_errors::DIAGNOSTICS;
use rustc_hir::def;
use rustc_hir::def_id::LOCAL_CRATE;
use rustc_middle::mir::BasicBlock;
use rustc_middle::mir::Statement;
use rustc_middle::mir::Terminator;
use rustc_middle::ty::FnSig;
use rustc_middle::ty::{Ty, TyCtxt, TyKind};
use rustc_middle::{mir::Body, ty::AdtDef};
use rustc_session::config::{self, CheckCfg};
use rustc_span::def_id::DefId;
use rustc_span::sym::result;
use std::alloc::System;
use std::collections::HashMap;
use std::env;
use std::fmt::format;
use std::fs;
use std::fs::File;
use std::fs::{write, OpenOptions};
use std::io::Write;
use std::panic;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::vec;
use std::{path, process, str};
use syn::parse_str;
use syn::visit_mut;
use syn::visit_mut::VisitMut;
use syn::Expr;
use syn::Item;
use syn::Local;
use syn::PatType;
use syn::{parse_file, parse_quote, ItemFn, Pat, Stmt, Token, Type};

struct Trans {}

impl Trans {
    pub(crate) fn trans(input_file: PathBuf) {
        println!("now process file: {:?}", input_file.to_str().unwrap());
        let r = panic::catch_unwind(|| {
            let file_name = input_file.clone();
            let config = Trans::get_rustc_config(file_name.clone());
            rustc_interface::run_compiler(config, |compiler| {
                compiler.enter(|queries| {
                    queries.global_ctxt().unwrap().enter(|tcx| {
                        let ret = Trans::parse_mir(tcx);
                        for r in ret {
                            r.my_cout();
                        }
                    });
                });
            });
        });
    }

    fn get_rustc_config(input_path: PathBuf) -> rustc_interface::Config {
        let out = std::process::Command::new("rustc")
            .arg("--print=sysroot")
            .current_dir(".")
            .output()
            .unwrap();
        let sysroot = std::str::from_utf8(&out.stdout).unwrap().trim();
        let config = rustc_interface::Config {
            crate_cfg: Vec::new(),
            crate_check_cfg: Vec::new(),
            hash_untracked_state: None,
            ice_file: None,
            using_internal_features: Arc::new(AtomicBool::new(false)),
            expanded_args: Vec::new(),
            opts: config::Options {
                maybe_sysroot: Some(path::PathBuf::from(sysroot)),
                ..config::Options::default()
            },
            input: config::Input::File(input_path.clone()),
            output_dir: None,
            output_file: None,
            file_loader: None,
            locale_resources: rustc_driver::DEFAULT_LOCALE_RESOURCES,
            lint_caps: rustc_hash::FxHashMap::default(),
            psess_created: None,
            register_lints: None,
            override_queries: None,
            make_codegen_backend: None,
            registry: registry::Registry::new(DIAGNOSTICS),
        };
        config
    }

    pub fn parse_mir<'tcx>(tcx: TyCtxt<'tcx>) -> Vec<FnBlocks> {
        let mut ret: Vec<FnBlocks> = vec![];
        let hir_krate = tcx.hir();
        for id in hir_krate.items() {
            let item = id.owner_id.def_id;
            match tcx.def_kind(item) {
                def::DefKind::Fn => {
                    //函数
                    let fn_name = format!("{:?}", item.to_def_id());
                    let mut fn_blocks: Vec<MyBlock> = vec![];
                    let mut mir = tcx.optimized_mir(item);
                    let mut mir2 = mir.clone();
                    let blocks = &mir.basic_blocks;
                    println!("{:#?}", blocks.reverse_postorder());
                    println!("{:#?}", blocks.predecessors());
                    let block_names = blocks.reverse_postorder();
                    let pre_blocks = blocks.predecessors();
                    let mut bblocks = mir2.basic_blocks.as_mut_preserves_cfg();
                    let mut suc_infos: HashMap<BasicBlock, Vec<BasicBlock>> = HashMap::new();
                    for block_name in block_names {
                        suc_infos.insert(block_name.clone(), vec![]);
                    }
                    for (block, data) in bblocks.iter().enumerate() {
                        println!("{:#?}", block);
                        // println!("{:#?}", blocks.reverse_postorder()[block]);
                        // println!("{:#?}", data);
                        let block_name = block_names[block];
                        // println!("{:#?}", block);
                        // println!("{:#?}", pre_blocks[block_name]);
                        let statements = data.statements.clone();
                        let terminator = data.terminator.clone();
                        let mut block_pre_blocks: Vec<BasicBlock> = vec![];
                        for b in pre_blocks[block_name].clone() {
                            block_pre_blocks.push(b);
                            suc_infos.get_mut(&b).unwrap().push(block_name);
                        }
                        let mut block_suc_blocks: Vec<BasicBlock> = vec![];
                        let mut a_block = MyBlock {
                            block_name: block_name,
                            statements: statements,
                            terminator: terminator,
                            pre_blocks: block_pre_blocks,
                            suc_blocks: block_suc_blocks,
                        };
                        fn_blocks.push(a_block);
                        // println!("{:#?}", a_block.clone());
                    }
                    for block in &mut fn_blocks {
                        block.suc_blocks = suc_infos.get(&block.block_name).unwrap().clone();
                    }
                    // println!("{:#?}", fn_blocks);
                    let a_fn_block = FnBlocks {
                        fn_name: fn_name,
                        blocks: fn_blocks.clone(),
                    };
                    ret.push(a_fn_block);
                }
                _ => {
                    // println!("mir other kind: {:?}", tcx.def_kind(item));
                }
            }
        }
        ret
    }
}

#[derive(Clone, Debug)]
struct MyBlock<'a> {
    block_name: BasicBlock,
    statements: Vec<Statement<'a>>,
    terminator: Option<Terminator<'a>>,
    pre_blocks: Vec<BasicBlock>,
    suc_blocks: Vec<BasicBlock>,
}

#[derive(Clone, Debug)]
struct FnBlocks<'a> {
    fn_name: String,
    blocks: Vec<MyBlock<'a>>,
}

impl FnBlocks<'_> {
    fn my_cout(&self) {
        println!("{:?}", self.fn_name);
        println!();
        for block in &self.blocks {
            println!("{:?}", block.block_name);
            let mut i = 0;
            for statement in &block.statements {
                println!("{}: {:?}", i, statement);
                i = i + 1;
            }
            println!("terminator {:?}", block.terminator);
            println!("preds {:?}", block.pre_blocks);
            println!("succs {:?}", block.suc_blocks);
            println!();
        }
    }
}

fn main() {
    // 获取要分析的Rust文件路径
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <path-to-rust-file>", args[0]);
        return;
    }
    let input_file = PathBuf::from_str(args[1].as_str()).unwrap();
    Trans::trans(input_file);
}
