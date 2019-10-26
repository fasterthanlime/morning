mod ast;
mod checker;
mod ir;
mod parser;

use clap::{App, Arg};

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
    let file = parser::parse(source)?;
    println!("AST: {:#?}", file);

    Ok(())
}
