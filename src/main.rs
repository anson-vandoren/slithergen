mod args;

fn main() {
    let args: args::Args = argh::from_env();
    let config = args.normalize();
    println!("{config:#?}");
}
