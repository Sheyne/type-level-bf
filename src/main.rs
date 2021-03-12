mod comptime;
mod runtime;

macro_rules! compile_bf {
    () => {
        Nil
    };
    (.$($i:tt)*) => {
        Cons<Write, compile_bf!($($i)*)>
    };
    (+$($i:tt)*) => {
        Cons<Incr, compile_bf!($($i)*)>
    };
    (-$($i:tt)*) => {
        Cons<Decr, compile_bf!($($i)*)>
    };
    (<$($i:tt)*) => {
        Cons<Left, compile_bf!($($i)*)>
    };
    (>$($i:tt)*) => {
        Cons<Right, compile_bf!($($i)*)>
    };
    ([$($i:tt)*]$($j:tt)*) => {
        Cons<Loop<compile_bf!($($i)*)>, compile_bf!($($j)*)>
    };
 }

fn main() {
    {
        use comptime::*;

        println!("{}", <Five as Mul<Six>>::Prod::u8());

        println!("{:?}", <Cons<Three, Cons<Five, Nil>>>::to_value());

        type HelloWorld = compile_bf!(++++++++[>++++[>++>+++>+++>+< < < < -]>+>+> - > >+[<] < -] > > .>---.+++++++. .+++.> >.< -.<.+++.------.--------.> >+.>++.);

        type Initial = MakeMachine<Nil>;
        type Output = <<Initial as RunMachine<HelloWorld>>::Next as GetOutput>::Output;

        let out = <Output as AsString>::as_string();

        println!("From type calculation: {:?}", out);
    }

    let program = runtime::Program::parse(&mut "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.".chars());
    // println!("{}", program.comptime_rep());
    let result = program.run(&mut "".chars());
    println!("From runtime calculation: {:?}", result);
}
