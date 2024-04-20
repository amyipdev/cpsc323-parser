#[derive(Debug, Eq, PartialEq)]
enum Token {
    Id,
    Add,
    Mul,
    Open,
    Close,
    End,
}

impl Token {
    pub const fn tostr(&self) -> &'static str {
        match self {
            Token::Id => "id",
            Token::Add => "+",
            Token::Mul => "*",
            Token::Open => "(",
            Token::Close => ")",
            Token::End => "$",
        }
    }
}

fn format_tokvec(inp: &std::collections::VecDeque<Token>) -> String {
    let mut builder: String = String::new();
    for t in inp {
        builder.push_str(t.tostr());
    }
    builder
}

fn format_states(inp: &Vec<u8>) -> String {
    let mut builder: String = String::new();
    for t in 0..inp.len() {
        builder.push('s');
        builder.push_str(&inp[t].to_string());
    }
    builder
}

macro_rules! osh {
    ($outstack:ident,$tokens:ident) => {
        $outstack.push($tokens.pop_front().unwrap().tostr().to_string())
    };
}

macro_rules! er1 {
    ($outstack:ident,$states:ident,$tokens:ident,$st:literal) => {{
        $outstack.pop().expect("outstack is empty");
        $states.pop().expect("empty state stack");
        $outstack.push($st.to_string());
        let gt = goto($states[$states.len() - 1], $st);
        $states.push(gt);
        gt
    }};
}

macro_rules! er3 {
    ($outstack:ident,$states:ident,$tokens:ident,$st:literal) => {{
        $outstack.pop().expect("outstack is empty");
        $outstack.pop().expect("outstack is empty");
        $outstack.pop().expect("outstack is empty");
        $states.pop().expect("empty state stack");
        $states.pop().expect("empty state stack");
        $states.pop().expect("empty state stack");
        $outstack.push($st.to_string());
        let gt = goto($states[$states.len() - 1], $st);
        $states.push(gt);
        gt
    }};
}

macro_rules! s45 {
    ($tokens:ident,$states:ident,$outstack:ident,$step:ident,$css:ident,$cop:ident,$cip:ident,$output:ident,$term:ident) => {
        match $tokens[0] {
            Token::Id => {
                $states.push(5);
                osh!($outstack, $tokens);
                $output.push(ParserStep::new($step, $css, $cop, $cip, "s5".to_string()));
            }
            Token::Open => {
                $states.push(4);
                osh!($outstack, $tokens);
                $output.push(ParserStep::new($step, $css, $cop, $cip, "s4".to_string()));
            }
            _ => {
                $term = 2;
                $output.push(ParserStep::new(
                    $step,
                    $css,
                    $cop,
                    $cip,
                    "error".to_string(),
                ));
            }
        }
    };
}

#[derive(tabled::Tabled)]
struct ParserStep {
    step: usize,
    states: String,
    output: String,
    input: String,
    action: String,
}

impl ParserStep {
    const fn new(
        step: usize,
        states: String,
        output: String,
        input: String,
        action: String,
    ) -> Self {
        Self {
            step,
            states,
            output,
            input,
            action,
        }
    }
}

fn main() {
    // mini lexer
    let arg: String = std::env::args().nth(1).expect("No parse string passed");
    let mut tokens: std::collections::VecDeque<Token> = std::collections::VecDeque::new();
    let mut cs: u8 = 0;
    for c in arg.chars() {
        if cs == 1 {
            if c != 'd' {
                eprintln!("parse error: invalid token (i followed by non-d)");
                std::process::exit(2);
            }
            tokens.push_back(Token::Id);
            cs = 0;
            continue;
        }
        match c {
            'i' => cs = 1,
            '+' => tokens.push_back(Token::Add),
            '*' => tokens.push_back(Token::Mul),
            '(' => tokens.push_back(Token::Open),
            ')' => tokens.push_back(Token::Close),
            '$' => tokens.push_back(Token::End),
            _ => panic!("Illegal character passed in input"),
        }
    }
    if tokens[tokens.len() - 1] != Token::End {
        eprintln!("parse error: end token is not End");
        std::process::exit(3);
    }
    for tok in 0..tokens.len() - 1 {
        if tokens[tok] == Token::End {
            eprintln!("parse error: premature End");
        }
    }

    // Actual parser time
    let mut states: Vec<u8> = vec![0];
    let mut outstack: Vec<String> = vec![];
    let mut step: usize = 1;
    let mut output: Vec<ParserStep> = vec![];
    // 0 = not finished, 1 = accepted, 2 = rejected
    let mut term: u8 = 0;
    loop {
        if term != 0 {
            break;
        }
        if tokens.len() == 0 {
            eprintln!("parse error: ran out of tokens");
            std::process::exit(4);
        }
        if states.len() == 0 {
            eprintln!("parse error: ran out of states");
            std::process::exit(5);
        }
        let css = format_states(&states);
        let cip = format_tokvec(&tokens);
        let cop = outstack.join(" ");
        match states[states.len() - 1] {
            0 => s45!(tokens, states, outstack, step, css, cop, cip, output, term),
            1 => match tokens[0] {
                Token::Add => {
                    states.push(6);
                    osh!(outstack, tokens);
                    output.push(ParserStep::new(step, css, cop, cip, "s6".to_string()));
                }
                Token::End => {
                    term = 1;
                    output.push(ParserStep::new(step, css, cop, cip, "accept".to_string()));
                }
                _ => {
                    term = 2;
                    output.push(ParserStep::new(step, css, cop, cip, "error".to_string()));
                }
            },
            2 => match tokens[0] {
                Token::Add | Token::End | Token::Close => output.push(ParserStep::new(
                    step,
                    css,
                    cop,
                    cip,
                    format!("r2g{}", er1!(outstack, states, tokens, 'E')),
                )),
                Token::Mul => {
                    states.push(7);
                    osh!(outstack, tokens);
                    output.push(ParserStep::new(step, css, cop, cip, "s7".to_string()));
                }
                _ => {
                    term = 2;
                    output.push(ParserStep::new(step, css, cop, cip, "error".to_string()));
                }
            },
            3 => match tokens[0] {
                Token::Add | Token::End | Token::Close | Token::Mul => {
                    output.push(ParserStep::new(
                        step,
                        css,
                        cop,
                        cip,
                        format!("r4g{}", er1!(outstack, states, tokens, 'T')),
                    ))
                }
                _ => {
                    term = 2;
                    output.push(ParserStep::new(step, css, cop, cip, "error".to_string()));
                }
            },
            4 => s45!(tokens, states, outstack, step, css, cop, cip, output, term),
            5 => match tokens[0] {
                Token::Add | Token::End | Token::Close | Token::Mul => {
                    output.push(ParserStep::new(
                        step,
                        css,
                        cop,
                        cip,
                        format!("r6g{}", er1!(outstack, states, tokens, 'F')),
                    ))
                }
                _ => {
                    term = 2;
                    output.push(ParserStep::new(step, css, cop, cip, "error".to_string()));
                }
            },
            6 => s45!(tokens, states, outstack, step, css, cop, cip, output, term),
            7 => s45!(tokens, states, outstack, step, css, cop, cip, output, term),
            8 => match tokens[0] {
                Token::Add => {
                    states.push(6);
                    osh!(outstack, tokens);
                    output.push(ParserStep::new(step, css, cop, cip, "s6".to_string()));
                }
                Token::Close => {
                    states.push(11);
                    osh!(outstack, tokens);
                    output.push(ParserStep::new(step, css, cop, cip, "s11".to_string()));
                }
                _ => {
                    term = 2;
                    output.push(ParserStep::new(step, css, cop, cip, "error".to_string()));
                }
            },
            9 => match tokens[0] {
                Token::Add | Token::Close | Token::End => output.push(ParserStep::new(
                    step,
                    css,
                    cop,
                    cip,
                    format!("r1g{}", er3!(outstack, states, tokens, 'E')),
                )),
                Token::Mul => {
                    states.push(7);
                    osh!(outstack, tokens);
                    output.push(ParserStep::new(step, css, cop, cip, "s7".to_string()));
                }
                _ => {
                    term = 2;
                    output.push(ParserStep::new(step, css, cop, cip, "error".to_string()));
                }
            },
            10 => match tokens[0] {
                Token::Add | Token::End | Token::Close | Token::Mul => {
                    output.push(ParserStep::new(
                        step,
                        css,
                        cop,
                        cip,
                        format!("r3g{}", er3!(outstack, states, tokens, 'T')),
                    ))
                }
                _ => {
                    term = 2;
                    output.push(ParserStep::new(step, css, cop, cip, "error".to_string()));
                }
            },
            11 => match tokens[0] {
                Token::Add | Token::End | Token::Close | Token::Mul => {
                    output.push(ParserStep::new(
                        step,
                        css,
                        cop,
                        cip,
                        format!("r5g{}", er3!(outstack, states, tokens, 'F')),
                    ))
                }
                _ => {
                    term = 2;
                    output.push(ParserStep::new(step, css, cop, cip, "error".to_string()));
                }
            },
            _ => panic!("impossible state reached!"),
        }
        step += 1;
    }
    println!("{}", tabled::Table::new(output));
    if term == 1 {
        println!("String is accepted");
    } else {
        println!("String is not accepted")
    }
}

fn goto(state: u8, sym: char) -> u8 {
    match state {
        0 => match sym {
            'E' => 1,
            'T' => 2,
            'F' => 3,
            _ => panic!("impossible"),
        },
        4 => match sym {
            'E' => 8,
            'T' => 2,
            'F' => 3,
            _ => panic!("impossible"),
        },
        6 => match sym {
            'T' => 9,
            'F' => 3,
            _ => panic!("goto error"),
        },
        7 => match sym {
            'F' => 10,
            _ => panic!("goto error"),
        },
        _ => panic!("goto error"),
    }
}
