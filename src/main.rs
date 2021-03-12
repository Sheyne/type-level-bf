mod comptime;
mod runtime;

fn main() {
    {
        use comptime::*;

        println!("{}", <Five as Mul<Six>>::Prod::u8());

        println!("{:?}", <Cons<Three, Cons<Five, Nil>>>::to_value());
    }

    let program = runtime::Program::parse(&mut "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.".chars());
    let result = program.run(&mut "".chars());
    println!("Result: {}", result);
}
