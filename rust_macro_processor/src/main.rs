// do not remove this line since you are not allowed to use unsafe code
#![deny(unsafe_code)]

// use statements
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::panic;

// a similar 'die' macro with the C version
macro_rules! die {
    ($($arg:tt)*) => {
        eprintln!("Rust Macro Processor: {}", format_args!($($arg)*));
        panic!();
    };
}

/* ################################# Define Structs ################################# */

struct MacroArgs {
    arg_1: String,
    arg_2: String,
    arg_3: String,
}

/* ################################# State Machines ################################# */

enum CommentState {
    PlainText,
    Escape,
    StartComment,
    EndComment,
}

enum ParseState {
    Text,
    Backslash,
    Macro,
    Process,
}

enum MacroState {
    Def,
    UnDef,
    If,
    IfDef,
    ExpandAfter,
    Include,
    Custom,
}

enum ArgState {
    Arg1,
    Arg2,
    Arg3,
}

/* ################################# Strip Comments Function ################################# */

fn strip_comments(input: &mut String, input_file: Option<&str>) {
    let mut file: Box<dyn BufRead> = match input_file {
        Some(filename) => {
            let file = File::open(filename);
            if file.is_err() {
                die!("Unable to open file!");
            }
            else {
                Box::new(io::BufReader::new(file.unwrap()))
            }
        }
        None => Box::new(io::BufReader::new(io::stdin())),
    };

    let mut state = CommentState::PlainText;
    let mut buffer = String::new();
    let mut str = String::new();

    while let Ok(bytes) = file.read_to_string(&mut buffer) {
        if bytes == 0 {
            break;
        }

        for c in buffer.chars() {
            match state {
                CommentState::PlainText => {
                    if c == '\\' {
                        state = CommentState::Escape;
                        str.push('\\');
                    } else if c == '%' {
                        state = CommentState::StartComment;
                    } else {
                        str.push(c);
                    }
                }
                CommentState::Escape => {
                    if c == '%' {
                        str.pop();
                    }
                    str.push(c);
                    state = CommentState::PlainText;
                }
                CommentState::StartComment => {
                    if c == '\n' {
                        state = CommentState::EndComment;
                    }
                }
                CommentState::EndComment => {
                    if c == '%' {
                        state = CommentState::StartComment;
                    }
                    else if c == '\\' {
                        state = CommentState::Escape;
                        str.push('\\');
                    }
                    else if c != '\t' && c != ' ' {
                        state = CommentState::PlainText;
                        str.push(c);
                    }
                }
            }
        }

        str = str.chars().rev().collect::<String>();
        input.push_str(&str);


        buffer.clear();
        str.clear();
    }
}

/* ################################# State Machine Function ################################# */

fn state_machine(input: &mut String, output: &mut String, macro_hash: &mut HashMap<String, String>) {
    let mut bracket_count: usize = 0;

    let mut parse_state: ParseState = ParseState::Text;
    let mut macro_state: MacroState = MacroState::Def;
    let mut arg_state: ArgState = ArgState::Arg1;

    let mut macro_name: String = String::new();

    let mut macro_args: MacroArgs = MacroArgs {
        arg_1: String::new(),
        arg_2: String::new(),
        arg_3: String::new(),
    };

    while input.len() > 0 {
        let mut c: char = input.pop().unwrap();

        match parse_state {
            ParseState::Text => {
                if c == '\\' {
                    parse_state = ParseState::Backslash;
                } else {
                    output.push(c);
                    parse_state = ParseState::Text;
                }
            }

            ParseState::Backslash => {
                if c == '\\' || c == '#' || c == '%' || c == '{' || c == '}' {
                    output.push(c);
                    parse_state = ParseState::Text;
                } else if c.is_alphanumeric() {
                    macro_name.push(c);
                    parse_state = ParseState::Macro;
                } else {
                    output.push('\\');
                    output.push(c);
                    parse_state = ParseState::Text;
                }
            }

            ParseState::Macro => {
                if c.is_alphanumeric() {
                    macro_name.push(c);
                } else if c == '{' {
                    bracket_count += 1;

                    if macro_name == "def" {
                        macro_state = MacroState::Def;
                    } else if macro_name == "undef" {
                        macro_state = MacroState::UnDef;
                    } else if macro_name == "if" {
                        macro_state = MacroState::If;
                    } else if macro_name == "ifdef" {
                        macro_state = MacroState::IfDef;
                    } else if macro_name == "expandafter" {
                        macro_state = MacroState::ExpandAfter;
                    } else if macro_name == "include" {
                        macro_state = MacroState::Include;
                    } else {
                        macro_state = MacroState::Custom;
                        if !macro_hash.contains_key(&macro_name) {
                            die!("Cannot find undefined macro");
                        }
                    }

                    parse_state = ParseState::Process;

                } else {
                    die!("Invalid macro name");
                }
            }

            ParseState::Process => {
                match macro_state {
                    MacroState::Def => {
                        match arg_state {
                            ArgState::Arg1 => {
                                if c.is_alphanumeric() {
                                    macro_args.arg_1.push(c);
                                } else if c == '}' {
                                    bracket_count -= 1;
                                    
                                    if let Some(last_char) = input.chars().last() {
                                        if last_char == '{' {
                                            input.pop().unwrap();
                                            arg_state = ArgState::Arg2;
                                            bracket_count += 1;
                                        } else {
                                            die!("Invalid argument provided");
                                        }
                                    }
                                }
                                else {
                                    die!("Invalid argument provided");
                                }
                            }

                            ArgState::Arg2 => 'def_arg2: {
                                if c == '\\' {
                                    macro_args.arg_2.push(c);
                                    c = input.pop().unwrap();
                                } else if c == '{' {
                                    bracket_count += 1;
                                } else if c == '}' {
                                    bracket_count -= 1;

                                    if bracket_count == 0 {
                                        if macro_args.arg_1.is_empty() {
                                            die!("Empty macro name argument");
                                        } else if macro_hash.contains_key(&macro_args.arg_1) {
                                            die!("Macro already defined");
                                        } else {
                                            macro_hash.insert(macro_args.arg_1.clone(), macro_args.arg_2.clone());

                                            macro_name.clear();
                                            macro_args.arg_1.clear();
                                            macro_args.arg_2.clear();
                                            macro_args.arg_3.clear();

                                            arg_state = ArgState::Arg1;
                                            parse_state = ParseState::Text;

                                            break 'def_arg2;
                                        }
                                    }
                                }
                                macro_args.arg_2.push(c);
                            }

                            ArgState::Arg3 => {}
                        }
                    }

                    MacroState::UnDef => {
                        match arg_state {
                            ArgState::Arg1 => {
                                if c.is_alphanumeric() {
                                    macro_args.arg_1.push(c);
                                } else if c == '}' {
                                    bracket_count -= 1;

                                    if macro_args.arg_1.is_empty() {
                                        die!("Empty macro name argument");
                                    } else if !macro_hash.contains_key(&macro_args.arg_1) {
                                        die!("Cannot delete undefined macro");
                                    }

                                    macro_hash.remove(&macro_args.arg_1);

                                    macro_name.clear();
                                    macro_args.arg_1.clear();
                                    macro_args.arg_2.clear();
                                    macro_args.arg_3.clear();

                                    arg_state = ArgState::Arg1;
                                    parse_state = ParseState::Text;

                                } else {
                                    die!("Invalid argument provided");
                                }
                            }

                            ArgState::Arg2 => {}

                            ArgState::Arg3 => {}
                        }
                    }

                    MacroState::If => {
                        match arg_state {
                            ArgState::Arg1 => 'if_arg1: {
                                if c == '\\' {
                                    macro_args.arg_1.push(c);
                                    c = input.pop().unwrap();
                                } else if c == '{' {
                                    bracket_count += 1;
                                } else if c == '}' {
                                    bracket_count -= 1;

                                    if bracket_count == 0 {
                                        if let Some(last_char) = input.chars().last() {
                                            if last_char == '{' {
                                                input.pop().unwrap();
                                                arg_state = ArgState::Arg2;
                                                bracket_count += 1;
                                                break 'if_arg1;
                                            } else {
                                                die!("Invalid argument provided");
                                            }
                                        }
                                    }
                                }
                                macro_args.arg_1.push(c);
                            }

                            ArgState::Arg2 => 'if_arg2: {
                                if c == '\\' {
                                    macro_args.arg_2.push(c);
                                    c = input.pop().unwrap();
                                } else if c == '{' {
                                    bracket_count += 1;
                                } else if c == '}' {
                                    bracket_count -= 1;

                                    if bracket_count == 0 {
                                        if let Some(last_char) = input.chars().last() {
                                            if last_char == '{' {
                                                input.pop().unwrap();
                                                arg_state = ArgState::Arg3;
                                                bracket_count += 1;
                                                break 'if_arg2;
                                            } else {
                                                die!("Invalid argument provided");
                                            }
                                        }
                                    }
                                }
                                macro_args.arg_2.push(c);
                            }

                            ArgState::Arg3 => 'if_arg3: {
                                if c == '\\' {
                                    macro_args.arg_3.push(c);
                                    c = input.pop().unwrap();
                                } else if c == '{' {
                                    bracket_count += 1;
                                } else if c == '}' {
                                    bracket_count -= 1;
                                    if bracket_count == 0 {
                                        if !macro_args.arg_1.is_empty() {
                                            macro_args.arg_2 = macro_args.arg_2.chars().rev().collect::<String>();
                                            input.push_str(&macro_args.arg_2);
                                        } else {
                                            macro_args.arg_3 = macro_args.arg_3.chars().rev().collect::<String>();
                                            input.push_str(&macro_args.arg_3);
                                        }

                                        macro_name.clear();
                                        macro_args.arg_1.clear();
                                        macro_args.arg_2.clear();
                                        macro_args.arg_3.clear();

                                        arg_state = ArgState::Arg1;
                                        parse_state = ParseState::Text;

                                        break 'if_arg3;
                                    }
                                }
                                macro_args.arg_3.push(c);
                            }
                        }
                    }

                    MacroState::IfDef => {
                        match arg_state {
                            ArgState::Arg1 => {
                                if c.is_alphanumeric() {
                                    macro_args.arg_1.push(c);
                                } else if c == '}' {
                                    bracket_count -= 1;

                                    if let Some(last_char) = input.chars().last() {
                                        if last_char == '{' {
                                            input.pop().unwrap();
                                            arg_state = ArgState::Arg2;
                                            bracket_count += 1;
                                        } else {
                                            die!("Invalid argument provided");
                                        }
                                    }

                                } else {
                                    die!("Invalid argument provided");
                                }
                            }

                            ArgState::Arg2 => 'ifdef_arg2: {
                                if c == '\\' {
                                    macro_args.arg_2.push(c);
                                    c = input.pop().unwrap();
                                } else if c == '{' {
                                    bracket_count += 1;
                                } else if c == '}' {
                                    bracket_count -= 1;

                                    if bracket_count == 0 {
                                        if let Some(last_char) = input.chars().last() {
                                            if last_char == '{' {
                                                input.pop().unwrap();
                                                arg_state = ArgState::Arg3;
                                                bracket_count += 1;
                                                break 'ifdef_arg2;

                                            } else {
                                                die!("Invalid argument provided");
                                            }
                                        }
                                    }
                                }
                                macro_args.arg_2.push(c);
                            }

                            ArgState::Arg3 => 'ifdef_arg3: {
                                if c == '\\' {
                                    macro_args.arg_3.push(c);
                                    c = input.pop().unwrap();
                                } else if c == '{' {
                                    bracket_count += 1;
                                } else if c == '}' {
                                    bracket_count -= 1;

                                    if bracket_count == 0 {
                                        if macro_hash.contains_key(&macro_args.arg_1) {
                                            macro_args.arg_2 = macro_args.arg_2.chars().rev().collect::<String>();
                                            input.push_str(&macro_args.arg_2);
                                        } else {
                                            macro_args.arg_3 = macro_args.arg_3.chars().rev().collect::<String>();
                                            input.push_str(&macro_args.arg_3);
                                        }

                                        macro_name.clear();
                                        macro_args.arg_1.clear();
                                        macro_args.arg_2.clear();
                                        macro_args.arg_3.clear();

                                        arg_state = ArgState::Arg1;
                                        parse_state = ParseState::Text;

                                        break 'ifdef_arg3;
                                    }
                                }
                                macro_args.arg_3.push(c);
                            }
                        }
                    }

                    MacroState::ExpandAfter => {
                        match arg_state {
                            ArgState::Arg1 => 'expandafter_arg1: {
                                if c == '\\' {
                                    macro_args.arg_1.push(c);
                                    c = input.pop().unwrap();
                                } else if c == '{' {
                                    bracket_count += 1;
                                } else if c == '}' {
                                    bracket_count -= 1;

                                    if bracket_count == 0 {
                                        if let Some(last_char) = input.chars().last() {
                                            if last_char == '{' {
                                                input.pop().unwrap();
                                                arg_state = ArgState::Arg2;
                                                bracket_count += 1;
                                                break 'expandafter_arg1;
                                            } else {
                                                die!("Invalid argument provided");
                                            }
                                        }
                                    }
                                }
                                macro_args.arg_1.push(c);
                            }

                            ArgState::Arg2 => 'expandafter_arg2: {
                                if c == '\\' {
                                    macro_args.arg_2.push(c);
                                    c = input.pop().unwrap();
                                } else if c == '{' {
                                    bracket_count += 1;
                                } else if c == '}' {
                                    bracket_count -= 1;

                                    if bracket_count == 0 {
                                        let mut temp_1: String = String::new();
                                        let mut temp_2: String = String::new();

                                        macro_args.arg_2 = macro_args.arg_2.chars().rev().collect::<String>();
                                        temp_1.push_str(&macro_args.arg_2);

                                        state_machine(&mut temp_1, &mut temp_2, macro_hash);

                                        temp_2 = temp_2.chars().rev().collect::<String>();
                                        input.push_str(&temp_2);

                                        macro_args.arg_1 = macro_args.arg_1.chars().rev().collect::<String>();
                                        input.push_str(&macro_args.arg_1);

                                        macro_name.clear();
                                        macro_args.arg_1.clear();
                                        macro_args.arg_2.clear();
                                        macro_args.arg_3.clear();

                                        arg_state = ArgState::Arg1;
                                        parse_state = ParseState::Text;

                                        break 'expandafter_arg2;
                                    }
                                }
                                macro_args.arg_2.push(c);
                            }

                            ArgState::Arg3 => {}
                        }
                    }

                    MacroState::Include => 'include_arg1: {
                        match arg_state {
                            ArgState::Arg1 => {
                                if c == '\\' {
                                    macro_args.arg_1.push(c);
                                    c = input.pop().unwrap();
                                } else if c == '{' {
                                    bracket_count += 1;
                                } else if c == '}' {
                                    bracket_count -= 1;

                                    if bracket_count == 0 {
                                        strip_comments(input, Some(&macro_args.arg_1));

                                        macro_name.clear();
                                        macro_args.arg_1.clear();
                                        macro_args.arg_2.clear();
                                        macro_args.arg_3.clear();

                                        arg_state = ArgState::Arg1;
                                        parse_state = ParseState::Text;

                                        break 'include_arg1;
                                    }
                                }
                                macro_args.arg_1.push(c);
                            }

                            ArgState::Arg2 => {}

                            ArgState::Arg3 => {}
                        }
                    }

                    MacroState::Custom => {
                        match arg_state {
                            ArgState::Arg1 => 'custom_arg1: {
                                if c == '\\' {
                                    macro_args.arg_1.push(c);
                                    c = input.pop().unwrap();
                                } else if c == '{' {
                                    bracket_count += 1;
                                } else if c == '}' {
                                    bracket_count -= 1;

                                    if bracket_count == 0 {
                                        let value: &String = macro_hash.get(&macro_name).unwrap();
                                        let mut process: String = String::new();
                                        let mut escape_flag: bool = false;

                                        for i in value.chars() {
                                            if escape_flag == true {
                                                process.push(i);
                                                escape_flag = false;
                                            } else if i == '\\' {
                                                process.push(i);
                                                escape_flag = true;
                                            } else if i == '#' {
                                                for j in macro_args.arg_1.chars() {
                                                    process.push(j)
                                                }
                                            } else {
                                                process.push(i);
                                            }
                                        }

                                        process = process.chars().rev().collect::<String>();
                                        input.push_str(&process);

                                        process.clear();
                                        macro_name.clear();
                                        macro_args.arg_1.clear();
                                        macro_args.arg_2.clear();
                                        macro_args.arg_3.clear();

                                        arg_state = ArgState::Arg1;
                                        parse_state = ParseState::Text;

                                        break 'custom_arg1;
                                    }
                                }
                                macro_args.arg_1.push(c);
                            }

                            ArgState::Arg2 => {}

                            ArgState::Arg3 => {}
                        }
                    }
                }
            }
        }
    }

    if !matches!(parse_state, ParseState::Text) {
        if matches!(parse_state, ParseState::Backslash) {
            output.push('\\');
        }
        else {
            die!("Exited on bad parse state");
        }
    }
}

/* ################################# Main Function ################################# */

fn main() {
    panic::set_hook(Box::new(|_| {}));

    let mut input: String = String::new();
    let mut output: String = String::new();
    let mut macro_hash: HashMap<String, String> = HashMap::new();

    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        strip_comments(&mut input, None);
    } else {
        for i in (1..args.len()).rev() {
            strip_comments(&mut input, Some(&args[i]));
        }
    }

    state_machine(&mut input, &mut output, &mut macro_hash);

    print!("{}", output);
}
