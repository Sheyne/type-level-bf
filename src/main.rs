mod comptime;
pub mod peano;
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
    (,$($i:tt)*) => {
        Cons<Read, compile_bf!($($i)*)>
    };
    ([$($i:tt)*]$($j:tt)*) => {
        Cons<Loop<compile_bf!($($i)*)>, compile_bf!($($j)*)>
    };
 }

fn main() {
    {
        use comptime::*;
        {
            type HelloWorld = compile_bf!(++++++++[>++++[>++>+++>+++>+< < < < -]>+>+> - > >+[<] < -] > > .>---.+++++++. .+++.> >.< -.<.+++.------.--------.> >+.>++.);

            type Initial = MakeMachine<Nil>;
            type Output = <<Initial as RunMachine<HelloWorld>>::Next as GetOutput>::Output;

            let out = <Output as Make<Vec<u8>>>::make();

            println!(
                "From type calculation: {:?}",
                std::str::from_utf8(&out).unwrap()
            );
        }
        {
            type Echo = compile_bf!(,[.>,]);
            type Initial =
                MakeMachine<Cons<One, Cons<Three, Cons<Five, Cons<<Five as Add<Two>>::Sum, Nil>>>>>;
            type Output = <<Initial as RunMachine<Echo>>::Next as GetOutput>::Output;

            let out = <Output as Make<Vec<u8>>>::make();

            println!("From type calculation: {:?}", &out);
        }
    }

    {
        let hello_world = runtime::Program::parse(&mut "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.".chars());
        // println!("{}", program.comptime_rep());
        let result = hello_world.run(&mut "".chars());
        println!("From runtime calculation: {:?}", result);

        let echo = runtime::Program::parse(&mut ",[.>,]".chars());
        // println!("{}", program.comptime_rep());
        let result = echo.run(&mut "Hi there!".chars());
        println!("From runtime calculation: {:?}", result);
    }
}
