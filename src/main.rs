use clap::{Arg, Command};
use mdbook_custom_chaptering::handle_preprocessing;

fn make_app() -> Command {
    Command::new("mdbook-custom-chaptering")
        .about("An mdbook preprocessor for custom chapter numbering")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    let matches = make_app().get_matches();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        let renderer = sub_args
            .get_one::<String>("renderer")
            .expect("Required argument");
        let supported = renderer == "html"
            || renderer == "pdf"
            || renderer == "latex"
            || renderer == "epub";
        
        if supported {
            std::process::exit(0);
        } else {
            std::process::exit(1);
        }
    } else if let Err(e) = handle_preprocessing() {
        eprintln!("{e:?}");
        std::process::exit(1);
    }
}