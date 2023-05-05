use std::io;
use std::io::Write;
use std::process::{Command, Stdio, Child, exit};
use std::fs::OpenOptions;
use std::path::Path;
use std::env;

fn read_command() -> Vec<String> {
    let mut input = String::new();
    print!("rush$ ");

    io::stdout().flush().expect("flush error");
    io::stdin().read_line(&mut input).expect("read error");

    input.trim().split_whitespace().map(str::to_string).collect()
}

fn amp_checker(process: &mut Vec<Child>) {
    let mut i:usize = 0;

    while i < process.len() {
        match process[i].try_wait() {
	    Ok(Some(_)) => {
		process.remove(i);
	    },
	    Ok(None) => {
		i += 1;
	    },
	    Err(err) => {
		println!("error in try_wait: {}", err);
		exit(1);
	    }
        }
    }
}

fn red_operators(input: &mut Vec<String>) -> Result<(Stdio, Stdio), &str> {
    let mut stdin = Stdio::inherit();
    let mut stdout = Stdio::inherit();
    let mut red_input = false;
    let mut red_output = false;
    let mut i:usize = 0;

    while i < input.len() {
	if input[i] == "<" {
	    if red_input {
		return Err("Only one < is allowed");
	    }
	    if input.len() <= i+1 {
		return Err("File required after <");
	    }

	    let output = OpenOptions::new().read(true).open(&input[i+1]);	    
	    stdin = match output {
		Err(_) => {
		    return Err("no such file/directory");
		},
		Ok(file) => {
		    red_input = true;
		    input.remove(i);
		    input.remove(i);
		    file.into()
		}
	    }
	}
	else if input[i] == ">" {
	    if red_output {
		return Err("Only one > or >> is allowed");
	    }
	    if input.len() <= i+1 {
		return Err("File required after >");
	    }

	    let output = OpenOptions::new().write(true).create(true).open(&input[i+1]);
	    stdout = match output {
		Err(_) => {
		    return Err("no such file/directory");
		},
		Ok(file) => {
		    red_output = true;
		    input.remove(i);
		    input.remove(i);
		    file.into()
		}
	    }
	}
	else if input[i] == ">>" {
	    if red_output {
		return Err("Only one > or >> is allowed");
	    }
	    if input.len() <= i+1 {
		return Err("File required after >>");
	    }

	    let output = OpenOptions::new().append(true).create(true).open(&input[i+1]);
	    stdout = match output {
		Err(_) => {
		    return Err("no such file/directory");
		},
		Ok(file) => {
		    red_output = true;
		    input.remove(i);
		    input.remove(i);
		    file.into()
		}
	    }
	}
	else {
	    i += 1;
	}
    }
    Ok((stdin, stdout))
}

fn exec_command(input: &mut Vec<String>, b_proc: &mut Vec<Child>) {
    let command = input.remove(0);
    let amp = match input.last() {
	Some(last) => {
	    if last == "&" {
		input.pop();
		true
	    }
	    else {
		false
	    }
	},
	None => false
    };

    let (stdin, stdout) = match red_operators(input) {
	Err(err) => {
	    println!("{}", err);
	    return;
	},
	Ok((stdin, stdout)) => (stdin, stdout)
    };

    let proc = Command::new(command).args(input).stdin(stdin).stdout(stdout).spawn();
    let mut proc = match proc {
	Err(err) => {
	    println!("{}", err);
	    return;
	},
	Ok(proc) => proc
    };

    if !amp {
	proc.wait().expect("error in wait");
    }
    else {
	b_proc.push(proc);
    }
}

fn main() {
    let mut proc: Vec<Child> = Vec::new();
    loop {
	let mut input = read_command();
	amp_checker(&mut proc);

	if input.len() == 0 {
	    continue;
	}
	if input[0].to_lowercase() == "exit" {
	    break;
	}
	if input[0].to_lowercase() == "cd" {
	    let new_dir = &input[1];
	    let path = Path::new(&new_dir);
	    if let Err(e) = env::set_current_dir(path) {
	        eprintln!("{}", e);
	    }
	    continue;
	}
	exec_command(&mut input, &mut proc);
    }
}
