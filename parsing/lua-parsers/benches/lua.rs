use criterion::{criterion_group, criterion_main, Criterion};
use lua_parsers::*;

fn lua54(c: &mut Criterion) {
    c.bench_function("parser_creation", |b| b.iter(|| Lua54Parser::new()));

    let parser = Lua54Parser::new();

    c.bench_function("function_calls_in_function", |b| {
        b.iter(|| parser.parse("function a() print() print() end"))
    });

    c.bench_function("function_as_exp", |b| {
        b.iter(|| parser.parse("local a = a()"))
    });

    c.bench_function("function_as_prefixexp", |b| {
        b.iter(|| parser.parse("a()()"))
    });

    c.bench_function("function_as_prefixexp2", |b| {
        b.iter(|| parser.parse("a:a():a():a()"))
    });

    c.bench_function("for_loop", |b| {
        b.iter(|| parser.parse("for i = 0,10 do print('hi') end"))
    });

    c.bench_function("multiline_string", |b| {
        b.iter(|| parser.parse("[[multiline\nstring]]"))
    });

    c.bench_function("indexing", |b| b.iter(|| parser.parse("a[1] = 3")));

    c.bench_function("liberation_mission", |b| {
        b.iter(|| parser.parse(include_str!("./liberation_mission.lua")))
    });

    c.bench_function("blizzardman", |b| {
        b.iter(|| parser.parse(include_str!("./blizzardman.lua")))
    });
}

criterion_group!(lua, lua54);
criterion_main!(lua);
