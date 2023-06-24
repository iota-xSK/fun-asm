use logos::{Lexer, Logos};
use std::{collections::HashMap, env};

use std::fs::File;
use std::io::{Read, Write};

fn main() {
    // Retrieve the command-line arguments
    let args: Vec<String> = env::args().collect();

    // Ensure that an argument (file path) is provided
    if args.len() != 3 {
        println!("Usage: asm <in_file> <out_file>");
        return;
    }

    let in_file = &args[1];
    let out_file = &args[2];

    // Open the file
    let mut file = match File::open(in_file) {
        Ok(file) => file,
        Err(err) => {
            println!("Error opening file: {}", err);
            return;
        }
    };

    // Read the file contents into a string
    let mut file_content = String::new();
    if let Err(err) = file.read_to_string(&mut file_content) {
        println!("Error reading file: {}", err);
        return;
    }
    if let Ok(mut file_handle) = File::create(out_file) {
        if let Some(assembled) = assemble(&file_content) {
            if let Err(err) = file_handle.write_all(&assembled) {
                println!("{err}")
            }
        } else {
            println!("couldn't parse the code")
        }
    } else {
        println!("couldn't create file")
    }
}

#[derive(Logos, Debug, Clone)]
#[logos(skip r"[ \t]+|(;.*)")] // Ignore this regex pattern between tokens
enum Token {
    #[token("lit")]
    Lit,
    #[token("jmp")]
    Jmp,
    #[token("cjmp")]
    Cjmp,
    #[token("tac")]
    Tac,
    #[token("tre")]
    Tre,
    #[token("r")]
    R,
    #[token("w")]
    W,
    #[token("eq")]
    Eq,
    #[token("cmp")]
    Cmp,
    #[token("add")]
    Add,
    #[token("sub")]
    Sub,
    #[token("lsf")]
    Lsf,
    #[token("rsf")]
    Rsf,
    #[token("or")]
    Or,
    #[token("and")]
    And,
    #[token("not")]
    Not,
    #[token("call")]
    Call,
    #[token("ret")]
    Ret,
    #[token("halt")]
    Halt,
    #[regex("r[0-9a-f]", |lex| u8::from_str_radix(lex.slice().trim_start_matches("r"), 16).ok())]
    Reg(u8),
    #[regex("[0-9a-f]{2}", |lex| u8::from_str_radix(lex.slice().trim_start_matches("r"), 16).ok())]
    Num(u8),
    #[regex(r"[a-zA-Z_][a-zA-Z_0-9]*:", |lex| lex.slice().trim_end_matches(":").to_string())]
    LabelDef(String),
    #[regex(r"[hl]{1}_[a-zA-Z_][a-zA-Z_0-9]*", parse_label)]
    Label(Label),
    #[regex(r"\|[0-9a-f]{4}", |lex| u16::from_str_radix(lex.slice().trim_start_matches("|"), 16).ok())]
    MemPos(u16),
    #[token("\n")]
    Newline,
}

fn parse_label(lex: &mut Lexer<Token>) -> Label {
    match lex.slice().chars().nth(0) {
        Some('h') => Label {
            name: lex.slice().trim_start_matches("h_").to_owned(),
            nibble: Nibble::High,
        },
        Some('l') => Label {
            name: lex.slice().trim_start_matches("l_").to_owned(),
            nibble: Nibble::Low,
        },
        _ => panic!(),
    }
}

#[derive(Clone, Debug)]
struct Label {
    name: String,
    nibble: Nibble,
}

#[derive(Clone, Debug)]
enum Nibble {
    High,
    Low,
}

fn assemble(program: &str) -> Option<[u8; 65536]> {
    let mut rom = [0; 65536];
    let tokens: Result<Vec<Token>, _> = Token::lexer(program).collect();

    let tokens = tokens.ok()?;
    let mut labels = HashMap::new();

    let mut pp = 0;

    let mut line = Vec::new();
    use Token::*;

    // label pass
    for token in &tokens {
        match token {
            Token::Newline => {
                if line.len() == 0 {
                    continue;
                }
                match &line[0] {
                    LabelDef(name) => {
                        labels.insert(name.clone(), pp);
                    }
                    MemPos(position) => pp = *position,
                    _ => pp += 1,
                }
                line.clear();
            }
            _ => line.push(token.clone()),
        }
    }
    pp = 0;

    println!("{labels:?}");

    line.clear();
    for token in &tokens {
        match &token {
            Token::Newline => {
                match (line.get(0), line.get(1)) {
                    (None, None) => (),
                    (None, Some(_)) => unreachable!(),
                    (Some(LabelDef(_)), _) => (),
                    (Some(Label(label)), _) => rom[pp as usize] = get_label(&labels, label)?,
                    (Some(Jmp), None) => {
                        rom[pp as usize] = 0x10;
                    }
                    (Some(Call), None) => {
                        rom[pp as usize] = 0x11;
                    }
                    (Some(Ret), None) => {
                        rom[pp as usize] = 0x12;
                    }
                    (Some(Halt), None) => {
                        rom[pp as usize] = 0x13;
                    }
                    (Some(MemPos(_)), _) => (),
                    (Some(Num(num)), _) => rom[pp as usize] = *num,
                    (Some(Lit), Some(Reg(r))) => rom[pp as usize] = r | 0x00,
                    (Some(Lit), Some(Label(label))) => {
                        rom[pp as usize] = 0x00 | get_label(&labels, &label)?;
                    }
                    // (Some(Jmp), Some(Reg(r))) => rom[pp as usize] = r | 0x10,
                    (Some(Jmp), Some(Label(label))) => {
                        rom[pp as usize] = 0x10 | get_label(&labels, &label)?;
                    }
                    (Some(Cjmp), Some(Reg(r))) => rom[pp as usize] = r | 0x20,
                    (Some(Cjmp), Some(Label(label))) => {
                        rom[pp as usize] = 0x20 | get_label(&labels, &label)?;
                    }
                    (Some(Tac), Some(Reg(r))) => rom[pp as usize] = r | 0x30,
                    (Some(Tac), Some(Label(label))) => {
                        rom[pp as usize] = 0x30 | get_label(&labels, &label)?;
                    }
                    (Some(Tre), Some(Reg(r))) => rom[pp as usize] = r | 0x40,
                    (Some(Tre), Some(Label(label))) => {
                        rom[pp as usize] = 0x40 | get_label(&labels, &label)?;
                    }
                    (Some(R), Some(Reg(r))) => rom[pp as usize] = r | 0x50,
                    (Some(R), Some(Label(label))) => {
                        rom[pp as usize] = 0x50 | get_label(&labels, &label)?;
                    }
                    (Some(W), Some(Reg(r))) => rom[pp as usize] = r | 0x60,
                    (Some(W), Some(Label(label))) => {
                        rom[pp as usize] = 0x60 | get_label(&labels, &label)?;
                    }
                    (Some(Eq), Some(Reg(r))) => rom[pp as usize] = r | 0x70,
                    (Some(Eq), Some(Label(label))) => {
                        rom[pp as usize] = 0x70 | get_label(&labels, &label)?;
                    }
                    (Some(Cmp), Some(Reg(r))) => rom[pp as usize] = r | 0x80,
                    (Some(Cmp), Some(Label(label))) => {
                        rom[pp as usize] = 0x80 | get_label(&labels, &label)?;
                    }
                    (Some(Add), Some(Reg(r))) => rom[pp as usize] = r | 0x90,
                    (Some(Add), Some(Label(label))) => {
                        rom[pp as usize] = 0x90 | get_label(&labels, &label)?;
                    }
                    (Some(Sub), Some(Reg(r))) => rom[pp as usize] = r | 0xa0,
                    (Some(Sub), Some(Label(label))) => {
                        rom[pp as usize] = 0xa0 | get_label(&labels, &label)?;
                    }
                    (Some(Lsf), Some(Reg(r))) => rom[pp as usize] = r | 0xb0,
                    (Some(Lsf), Some(Label(label))) => {
                        rom[pp as usize] = 0xb0 | get_label(&labels, &label)?;
                    }
                    (Some(Rsf), Some(Reg(r))) => rom[pp as usize] = r | 0xc0,
                    (Some(Rsf), Some(Label(label))) => {
                        rom[pp as usize] = 0xc0 | get_label(&labels, &label)?;
                    }
                    (Some(Or), Some(Reg(r))) => rom[pp as usize] = r | 0xd0,
                    (Some(Or), Some(Label(label))) => {
                        rom[pp as usize] = 0xd0 | get_label(&labels, &label)?;
                    }
                    (Some(And), Some(Reg(r))) => rom[pp as usize] = r | 0xe0,
                    (Some(And), Some(Label(label))) => {
                        rom[pp as usize] = 0xe0 | get_label(&labels, &label)?;
                    }
                    (Some(Not), Some(Reg(r))) => rom[pp as usize] = r | 0xf0,
                    (Some(Not), Some(Label(label))) => {
                        rom[pp as usize] = 0xf0 | get_label(&labels, &label)?;
                    }
                    _ => {
                        println!("none matched");
                        println!("{:?}", line);
                        return None;
                    }
                };
                match line.get(0) {
                    Some(MemPos(pos)) => pp = *pos,
                    Some(LabelDef(_)) => (),
                    None => (),
                    _ => pp += 1,
                }
                line.clear();
            }
            _ => line.push(token.clone()),
        }
    }

    Some(rom)
}

fn get_label(labels: &HashMap<String, u16>, label: &Label) -> Option<u8> {
    println!("getting label");
    use Nibble::*;
    if let Some(pp) = labels.get(&label.name) {
        match label.nibble {
            High => Some((pp >> 8) as u8),
            Low => Some((pp & 0b00001111) as u8),
        }
    } else {
        None
    }
}
