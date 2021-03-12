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

macro_rules! make_cons {
    () => {
        Nil
    };
    ($e:ty $(, $rest:ty)*) => {
        Cons<$e, make_cons!($($rest),*)>
    };
}

macro_rules! run_bf {
    (($($arg:ty),*) {$($ins:tt)*}) => {
        <<MakeMachine<make_cons!($($arg),*)> as RunMachine<compile_bf!($($ins)*)>>::Next as GetOutput>::Output
    };
}

fn main() {
    {
        use comptime::*;
        type HelloWorld = run_bf!(()
        {
            ++++++++[>++++[>++>+++>+++>+< < < < -]>+>+> - > >+[<] < -] >
            > .>---.+++++++. .+++.> >.< -.<.+++.------.--------.> >+.>++.
        });

        let hello_world: Vec<u8> = HelloWorld::make();

        println!(
            "Ascii output from HelloWorld: {:?}",
            std::str::from_utf8(&hello_world).unwrap()
        );

        type Cat = run_bf!(
            (One, Three, Five, <Five as Add<Two>>::Sum, Two)
            { ,[.>,] });
        let cat: Vec<u8> = Cat::make();

        println!("u8 values from cat: {:?}", &cat);
    }

    {
        let hello_world = runtime::Program::parse(&mut "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.".chars());
        // println!("{}", program.comptime_rep());
        let result = hello_world.run(&mut "".chars());
        println!("Ascii output from HelloWorld (runtime): {:?}", result);

        let echo = runtime::Program::parse(&mut ",[.>,]".chars());
        // println!("{}", program.comptime_rep());
        let result = echo.run(&mut "Hi there!".chars());
        println!("Ascii output from cat (runtime): {:?}", result);
    }
}
