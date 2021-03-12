#[derive(Debug)]
pub enum Program {
    Empty,
    Left(Box<Program>),
    Right(Box<Program>),
    Incr(Box<Program>),
    Decr(Box<Program>),
    Input(Box<Program>),
    Output(Box<Program>),
    Loop(Box<Program>, Box<Program>),
}

impl Program {
    fn step(
        &self,
        idx: &mut usize,
        output: &mut Vec<char>,
        memory: &mut Vec<u8>,
        input: &mut impl Iterator<Item = char>,
    ) -> Option<&Program> {
        use Program::*;
        match self {
            Empty => None,
            Left(next) => {
                *idx -= 1;
                Some(&*next)
            }
            Right(next) => {
                *idx += 1;
                if *idx == memory.len() {
                    memory.push(0u8)
                }
                Some(&*next)
            }
            Incr(next) => {
                memory[*idx] += 1;
                Some(&*next)
            }
            Decr(next) => {
                memory[*idx] -= 1;
                Some(&*next)
            }
            Input(next) => {
                memory[*idx] = input.next().unwrap() as u8;
                Some(&*next)
            }
            Output(next) => {
                output.push(memory[*idx] as char);
                Some(&*next)
            }
            Loop(body, next) => {
                while memory[*idx] != 0 {
                    body.run_context(idx, output, memory, input);
                }
                Some(&*next)
            }
        }
    }

    fn run_context(
        &self,
        idx: &mut usize,
        output: &mut Vec<char>,
        memory: &mut Vec<u8>,
        input: &mut impl Iterator<Item = char>,
    ) {
        let mut step = Some(self);

        while let Some(s) = step {
            step = s.step(idx, output, memory, input);
        }
    }

    pub fn run(self, input: &mut impl Iterator<Item = char>) -> String {
        let mut output = vec![];
        let mut memory = vec![0u8];
        let mut idx = 0;

        self.run_context(&mut idx, &mut output, &mut memory, input);

        output.into_iter().collect()
    }

    pub fn parse(input: &mut impl Iterator<Item = char>) -> Box<Program> {
        use Program::*;
        Box::new(if let Some(char) = input.next() {
            match char {
                '<' => Left(Program::parse(input)),
                '>' => Right(Program::parse(input)),
                '+' => Incr(Program::parse(input)),
                '-' => Decr(Program::parse(input)),
                '.' => Output(Program::parse(input)),
                ',' => Input(Program::parse(input)),
                '[' => {
                    let mut body: Vec<char> = vec![];
                    let mut open_count = 0;
                    while let Some(c) = input.next() {
                        match c {
                            '[' => {
                                open_count += 1;
                            }
                            ']' => {
                                if open_count == 0 {
                                    break;
                                } else {
                                    open_count -= 1;
                                }
                            }
                            _ => (),
                        }
                        body.push(c);
                    }
                    let body = Program::parse(&mut body.drain(..));
                    let rest = Program::parse(input);

                    Loop(body, rest)
                }
                ']' => todo!("Unmatched closing loop"),
                _ => todo!("Unknown input char (ignore this later)"),
            }
        } else {
            Empty
        })
    }
}
