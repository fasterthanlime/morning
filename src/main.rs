pub mod ast;
pub mod ir;
pub mod middle;
pub mod parser;

use clap::{App, Arg};
use ir::*;

fn main() -> Result<(), parser::Error> {
    let matches = App::new("Morning Language")
        .version("0.1.0")
        .author("@fasterthanlime's twitch chat")
        .about("?")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();
    println!("Compiling: {}", input);

    let source = parser::Source::from_path(input)?;
    let unit = parser::parse(source)?;
    println!("AST: {:#?}", unit);

    {
        let mut buf: Vec<u8> = Vec::new();
        middle::transform(&mut buf, &unit)?;

        let asm_path = "./samples/hello.asm";
        std::fs::write(asm_path, buf)?;

        let status = std::process::Command::new("bat")
            .arg(asm_path)
            .status()
            .expect("bat should run");
        if !status.success() {
            panic!("bat failed with {:?}", status)
        }

        let status = std::process::Command::new("nasm")
            .arg("-f")
            .arg("win64")
            .arg("-o")
            .arg("./samples/hello.obj")
            .arg("./samples/hello.asm")
            .status()
            .expect("nasm should run");
        if !status.success() {
            panic!("nasm failed with {:?}", status)
        }

        let link_path = r#"D:\Programs\Microsoft Visual Studio\2019\Community\VC\Tools\MSVC\14.23.28105\bin\Hostx64\x64\link.exe"#;
        let status = std::process::Command::new(link_path)
            .arg("/SUBSYSTEM:CONSOLE")
            .arg("/ENTRY:_start")
            .arg(r#"samples\hello.obj"#)
            .arg(r#"/OUT:samples\hello.exe"#)
            .status()
            .expect("link should run");
        if !status.success() {
            panic!("link failed with {:?}", status)
        }

        println!("Compiled to samples/hello.exe");
    }

    Ok(())
}

#[allow(dead_code)]
fn manual_ir() -> ir::Func {
    let mut main = Func::new("_start");
    main.public = true;

    {
        let entry = main.entry;
        let f = &mut main;

        // let x = 1
        let x = f.push_local("x", Type::I64);
        entry.push_op(f, Op::mov(x, 1));

        // let y = 0
        let y = f.push_local("y", Type::I64);
        entry.push_op(f, Op::mov(y, 0));

        // loop
        let loopstart = entry.new_label(f);
        let loopend = entry.new_label(f);

        entry.push_op(f, loopstart);

        // y += x
        entry.push_op(f, Op::add(y, x));

        // x += 1
        entry.push_op(f, Op::add(x, 1));

        // if x > 10
        entry.push_op(f, Op::cmp(x, 10));

        // break
        entry.push_op(f, Op::jg(loopend));
        // continue (implicit)
        entry.push_op(f, Op::jmp(loopstart));

        entry.push_op(f, loopend);
        entry.push_op(f, Op::ret_some(y));
    }

    main
}
