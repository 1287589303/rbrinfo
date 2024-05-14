use crate::analysis::option::AnalysisOption;
use core::mem::size_of;
use log::info;
use rustc_driver::Compilation;
use rustc_hir::def;
use rustc_interface::interface;
use rustc_interface::Queries;
use rustc_middle::mir::BasicBlock;
use rustc_middle::mir::Statement;
use rustc_middle::mir::Terminator;
use rustc_middle::ty::TyCtxt;
use std::collections::HashMap;

pub struct MirCheckerCallbacks {
    pub analysis_options: AnalysisOption,
    pub source_name: String,
}

impl MirCheckerCallbacks {
    pub fn new(options: AnalysisOption) -> Self {
        Self {
            analysis_options: options,
            source_name: String::new(),
        }
    }
}

impl rustc_driver::Callbacks for MirCheckerCallbacks {
    /// Called before creating the compiler instance
    fn config(&mut self, config: &mut interface::Config) {
        self.source_name = format!("{:?}", config.input.source_name());
        config.crate_cfg.push("mir_checker".to_string());
        info!("Source file: {}", self.source_name);
    }

    /// Called after analysis. Return value instructs the compiler whether to
    /// continue the compilation afterwards (defaults to `Compilation::Continue`)
    fn after_analysis<'compiler, 'tcx>(
        &mut self,
        compiler: &'compiler interface::Compiler,
        queries: &'tcx Queries<'tcx>,
    ) -> Compilation {
        queries
            .global_ctxt()
            .unwrap()
            .enter(|tcx| self.run_analysis(compiler, tcx));
        Compilation::Continue
    }
}

#[derive(Clone, Debug)]
struct MyBlock<'a> {
    block_name: BasicBlock,
    statements: Vec<Statement<'a>>,
    terminator: Terminator<'a>,
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

impl MirCheckerCallbacks {
    fn run_analysis<'tcx, 'compiler>(
        &mut self,
        compiler: &'compiler interface::Compiler,
        tcx: TyCtxt<'tcx>,
    ) {
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
                    // println!("{:#?}", blocks.reverse_postorder());
                    // println!("{:#?}", blocks.predecessors());
                    let pre_blocks = blocks.predecessors();
                    let mut bblocks = mir2.basic_blocks.as_mut_preserves_cfg();
                    let mut suc_infos: HashMap<BasicBlock, Vec<BasicBlock>> = HashMap::new();
                    for i in 0..=blocks.len() - 1 {
                        suc_infos.insert(BasicBlock::from_usize(i), vec![]);
                    }
                    for (block, data) in bblocks.iter().enumerate() {
                        // println!("{:#?}", block);
                        // println!("{:#?}", blocks.reverse_postorder()[block]);
                        // println!("{:#?}", data);
                        let block_name = BasicBlock::from_usize(block);
                        println!("{:#?}", blocks[block_name]);
                        // println!("{:#?}", block);
                        // println!("{:#?}", pre_blocks[block_name]);
                        let statements = data.statements.clone();
                        let terminator = data.terminator.clone().unwrap();
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
        for block in ret {
            block.my_cout();
        }
    }
}
