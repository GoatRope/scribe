#![feature(btreemap_remove_entry)]
#![feature(vec_remove_item)]

use shrust::{ExecError, Shell, ShellIO};
use std::collections::BTreeSet;
use std::io::{stdin, stdout, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

mod resource;
mod state;

use resource::Resource;
use state::State;
use std::io;

pub fn run() {
    let mut state = state::State::new(String::from("/Users/adamm/.scribe-rs"));
    state.initialize().unwrap();
    let mut shell = Shell::new(state);
    shell.new_command_noargs("tag", "Tag lookup", |_io, state| tag_command(state));
    shell.new_command("index", "Index lookup", 1, |_io, state, args| {
        index_command(state, args)
    });
    shell.new_command_noargs("search", "Search", |_io, state| search_command(state));
    shell.new_command_noargs("new", "New Resource", |_io, state| new_command(state));
    shell.new_command_noargs("ls", "List tags", |_io, state| list_command(state));
    shell.new_command("rm", "Remove tag", 1, |_io, state, args| {
        remove_command(state, args)
    });
    shell.run_loop(&mut ShellIO::default())
}

fn write_color(color: Color) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(color)))?;
    Ok(())
}

fn print_separator() -> () {
    write_color(Color::Red).unwrap();
    println!(
        "~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-"
    );
    write_color(Color::White).unwrap();
}

pub fn new_command(state: &mut State) -> Result<(), ExecError> {
    let tags_list = prompt("tags");
    let mut tags = BTreeSet::new();
    for t in tags_list.split_whitespace().map(String::from) {
        tags.insert(t);
    }
    let content = long_prompt("content", 2);
    let mut resource = Resource::new(tags, content);
    match state.add_resource(&mut resource, true) {
        Ok(x) => Ok(x),
        Err(_e) => Err(ExecError::Quit),
    }
}

pub fn tag_command(state: &mut State) -> Result<(), ExecError> {
    let tags_list = prompt("tags");
    let mut tags = BTreeSet::new();
    for t in tags_list.split_whitespace().map(String::from) {
        tags.insert(t);
    }
    for tag in tags.iter() {
        match state.tag_cache.borrow().get(tag) {
            None => println!("go fish"),
            Some(set) => {
                print_separator();
                for hash in set.iter() {
                    println!(
                        "{}",
                        state.resource_lookup.borrow().get(hash).unwrap().content
                    );
                    print_separator();
                }
                stdout().flush().unwrap();
            }
        }
    }
    Ok(())
}

pub fn index_command(state: &mut State, args: &[&str]) -> Result<(), ExecError> {
    match state.search_indices.borrow().get(args[0]) {
        None => println!("go fish"),
        Some(resources) => {
            print_separator();
            for resource in resources.iter() {
                println!("{}", resource);
            }
            print_separator();
            stdout().flush().unwrap();
        }
    };
    Ok(())
}

pub fn search_command(state: &State) -> Result<(), ExecError> {
    let terms_list = prompt("terms");
    let terms: Vec<_> = terms_list.split_whitespace().map(String::from).collect();
    let mut term_sets = Vec::<&BTreeSet<String>>::new();
    let indices = state.search_indices.borrow();
    terms.iter().for_each(|t| match indices.get(t) {
        None => {}
        Some(s) => {
            term_sets.push(s);
        }
    });
    if !term_sets.is_empty() {
        let mut acc = term_sets.pop().unwrap().clone();
        for x in term_sets {
            let intersection: Vec<_> = acc.intersection(x).cloned().collect();
            acc.clear();
            for x in intersection.into_iter() {
                acc.insert(x);
            }
        }
        print_separator();
        for sha in acc {
            let lookup = state.resource_lookup.borrow();
            let resource_cell = lookup.get(&sha).unwrap();
            println!("{}", resource_cell.content);
            print_separator();
        }
    }
    Ok(())
}

pub fn list_command(state: &mut State) -> Result<(), ExecError> {
    for key in state.tag_cache.borrow().keys() {
        println!("{}", key);
    }
    stdout().flush().unwrap();
    Ok(())
}

pub fn remove_command(state: &mut State, args: &[&str]) -> Result<(), ExecError> {
    let arg = args[0].to_string();
    state.rm_tag(&arg);
    Ok(())
}

fn prompt(p: &str) -> String {
    println!("{} =>", p);
    let mut s = String::new();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    String::from(s.trim())
}

fn long_prompt(p: &str, num_blank_lines: isize) -> String {
    println!("{} =>", p);
    let mut result = String::new();
    let mut s = String::new();
    let mut blank_line_count: isize = 0;
    loop {
        s.clear();
        let bytes = stdin()
            .read_line(&mut s)
            .expect("Did not enter a correct string");
        if bytes < 2 {
            blank_line_count += 1;
            if blank_line_count >= num_blank_lines {
                return String::from(result.trim());
            }
            result = format!("{}{}", result, s);
        } else {
            blank_line_count = 0;
            result = format!("{}{}", result, s);
        }
    }
}
