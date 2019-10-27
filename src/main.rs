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
                src: Location::Immediate(1),
            }));
            let y = entry.add_local("y", Type::I64);
            entry.add_op(Op::Mov(Mov {
                dst: Location::Local(y),
                src: Location::Immediate(0),
            }));
        }

        // println!("{:#?}", main);

        let mut buf: Vec<u8> = Vec::new();
        ir::emit::emit(&mut buf, &main)?;
        println!("{}", std::str::from_utf8(&buf).unwrap());
    }

    Ok(())
}
