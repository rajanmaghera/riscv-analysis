use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use riscv_analysis::analysis::{AvailableValuePass, LivenessPass};
use riscv_analysis::cfg::{Cfg, Function, RegisterSet};
use riscv_analysis::gen::{FunctionMarkupPass, NodeDirectionPass};
use riscv_analysis::parser::{
    LabelString, LabelStringToken, Position, ProgramEntryType, RVParserOutput, RVRegister,
    RVStringParser, RVToken, Range, TokenType,
};
use riscv_analysis::passes::{DiagnosticManager, GenerationPass, Manager};
use std::collections::HashSet;
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

fn read_file(filename: &str) -> String {
    std::fs::read_to_string(filename).unwrap()
}

fn parse_items(s: &str) -> RVParserOutput {
    RVStringParser::parse_from_text(s)
}

fn make_cfg(
    o: &RVParserOutput,
    ext: &Option<HashSet<(LabelStringToken, RegisterSet, RegisterSet)>>,
) -> Cfg {
    Manager::gen_empty_cfg(o, ext, &ProgramEntryType::None).unwrap()
}

fn add_direction_info(cfg: &mut Cfg) {
    NodeDirectionPass::run(cfg).unwrap()
}

fn markup_function_info(cfg: &mut Cfg) {
    FunctionMarkupPass::run(cfg).unwrap()
}

fn add_available_value_info(cfg: &mut Cfg) {
    AvailableValuePass::run(cfg).unwrap()
}

fn add_liveness_info(
    cfg: &mut Cfg,
    ext: Option<HashSet<(LabelStringToken, RegisterSet, RegisterSet)>>,
) {
    LivenessPass::inject_return_registers_into_function(
        cfg,
        ext.into_iter().flat_map(std::iter::IntoIterator::into_iter),
    );
    LivenessPass::run(cfg).unwrap();
}

fn lint_items(cfg: &Cfg) -> DiagnosticManager {
    let mut errs = DiagnosticManager::new();
    let mut manager = Manager::new();
    manager.register_and_enable_built_in_passes();
    manager.run_diagnostics(cfg, &mut errs);
    errs
}

fn str_to_reg_set(s: &str) -> RegisterSet {
    s.split(",")
        .filter_map(|x| {
            if x.is_empty() {
                None
            } else {
                Some(RVRegister::from_str(x).unwrap())
            }
        })
        .collect::<RegisterSet>()
}

fn get_external_functions() -> HashSet<(LabelStringToken, RegisterSet, RegisterSet)> {
    let str = "__adddf3:a0,a1,a2,a3:a0,a1;__divdi3:a0,a1,a2,a3:a0,a1;__fixdfdi:a0,a1:a0,a1;__floatdidf:a0,a1:a0,a1;__isoc99_sscanf:a0,a1:a0;__moddi3:a0,a1,a2,a3:a0,a1;__muldf3:a0,a1,a2,a3:a0,a1;calloc:a0,a1:a0;exit:a0:;fclose:a0:a0;fflush:a0:a0;fgets:a0,a1,a2:a0;fopen64:a0,a1:a0;fprintf:a0,a1:a0;free:a0:a0;fwrite:a0,a1,a2,a3:a0;malloc:a0:a0;memset:a0,a1,a2:a0;printf:a0:a0;putchar:a0:a0;puts:a0:a0;realloc:a0,a1:a0;sprintf:a0,a1:a0;strcpy:a0,a1:a0;strtol:a0,a1,a2:a0,a1;main:a0,a1:a0";
    let mut map = HashSet::new();
    let text_uuid = Uuid::new_v4();
    for item in str.split(";") {
        let (name, rest) = item.split_once(':').unwrap();
        let (args_str, rets_str) = rest.split_once(':').unwrap();

        let args = str_to_reg_set(args_str);
        let rets = str_to_reg_set(rets_str);

        map.insert((
            LabelStringToken::new(
                LabelString::new(name.to_owned()),
                RVToken::new(
                    TokenType::Newline,
                    "\n",
                    Range::new(Position::new(0, 0, 0), Position::new(0, 1, 1)),
                    text_uuid,
                ),
            ),
            args,
            rets,
        ));
    }
    map
}
pub fn parser_benchmark(c: &mut Criterion) {
    let text = read_file("/Users/rajan/combined.s");
    let mut group = c.benchmark_group("sized_group_505");
    group
        .significance_level(0.1)
        .sample_size(10)
        .measurement_time(Duration::new(30, 0));
    group.bench_function("parse_from_string", |b| {
        b.iter(|| parse_items(black_box(text.as_str())))
    });
    let parsed = parse_items(text.as_str());

    let ext = Some(get_external_functions());

    group.bench_function("cfg_from_parsed", |b| {
        b.iter(|| make_cfg(black_box(&parsed), black_box(&ext)));
    });

    let mut cfg = make_cfg(&parsed, &ext);

    group.bench_function("add_direction", |b| {
        b.iter_batched(
            || make_cfg(&parsed, &ext),
            |mut x| add_direction_info(black_box(&mut x)),
            BatchSize::SmallInput,
        );
    });

    group.bench_function("markup_function_info", |b| {
        b.iter_batched(
            || {
                let mut cfg = make_cfg(&parsed, &ext);
                add_direction_info(&mut cfg);
                cfg
            },
            |mut x| markup_function_info(black_box(&mut x)),
            BatchSize::SmallInput,
        );
    });

    group.bench_function("add_available_value", |b| {
        b.iter_batched(
            || {
                let mut cfg = make_cfg(&parsed, &ext);
                add_direction_info(&mut cfg);
                markup_function_info(&mut cfg);
                cfg
            },
            |mut x| add_available_value_info(black_box(&mut x)),
            BatchSize::SmallInput,
        );
    });

    group.bench_function("add_liveness_information", |b| {
        b.iter_batched(
            || {
                let mut cfg = make_cfg(&parsed, &ext);
                add_direction_info(&mut cfg);
                markup_function_info(&mut cfg);
                add_available_value_info(&mut cfg);
                (cfg, ext.clone())
            },
            |(mut x, ext)| add_liveness_info(black_box(&mut x), ext),
            BatchSize::SmallInput,
        );
    });

    add_direction_info(&mut cfg);
    markup_function_info(&mut cfg);
    add_available_value_info(&mut cfg);
    add_liveness_info(&mut cfg, ext);

    group.bench_function("lint_from_cfg", |b| {
        b.iter(|| lint_items(black_box(&cfg)));
    });

    group.finish();
}

criterion_group!(benches, parser_benchmark);
criterion_main!(benches);
