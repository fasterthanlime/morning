pub mod ast;
pub mod checker;
pub mod ir;
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

    let _input = matches.value_of("INPUT").unwrap();
    // println!("Compiling: {}", input);

    // let source = parser::Source::from_path(input)?;
    // let file = parser::parse(source)?;
    // println!("AST: {:#?}", file);

    {
        let mut main = Func::new();

        {
            let entry = main.entry.borrow_mut(&mut main);
            let x = entry.add_local("x", Type::I64);
            entry.add_op(Op::Mov(Mov {
                dst: Location::Local(x),
                src: Location::Imm64(1),
            }));
            let y = entry.add_local("y", Type::I64);
            entry.add_op(Op::Mov(Mov {
                dst: Location::Local(y),
                src: Location::Imm64(0),
            }));

            entry.add_op(Op::Mov(Mov {
                dst: Location::Register(Register::RAX),
                src: Location::Local(x),
            }));

            entry.add_op(Op::Cmp(Cmp {
                lhs: Location::Register(Register::RAX),
                rhs: Location::Local(y),
            }));
            entry.add_op(Op::Jg(Jg { dst: entry.start }))
        }

        let mut buf: Vec<u8> = Vec::new();
        ir::emit::emit_main(&mut buf, &main)?;
        // println!("{}", std::str::from_utf8(&buf).unwrap());

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
