use std::env;

fn main(){
    // Test to see that we can build
    let line_args = env::args().skip(1);
    
    for argument in line_args {
        println!("{}", argument);
    }
}