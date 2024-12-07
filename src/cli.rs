use argh::FromArgs;

#[derive(FromArgs)]
/// Reach new heights.
struct Hello {
    /// turn on verbose output
    #[argh(switch, short = 'v')]
    verbose: bool,

    /// year to hello to
    #[argh(option)]
    year: i32,
}

pub fn main() {
    let hello: Hello = argh::from_env();
    if hello.verbose {
        println!("Hello, {}!", hello.year);
    } else {
        println!("{}!", hello.year);
    }
}