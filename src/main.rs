/// get_start_pos(), get_mapped_file(), print_buf(), encode(), read_the_rest() functions are:

/// MIT License

/// Copyright (c) 2018 nao-kobayashi

/// Permission is hereby granted, free of charge, to any person obtaining a copy
/// of this software and associated documentation files (the "Software"), to deal
/// in the Software without restriction, including without limitation the rights
/// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
/// copies of the Software, and to permit persons to whom the Software is
/// furnished to do so, subject to the following conditions:

/// The above copyright notice and this permission notice shall be included in all
/// copies or substantial portions of the Software.

/// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
/// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
/// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
/// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
/// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
/// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
/// SOFTWARE.
extern crate clap;
extern crate getopts;
extern crate memmap;
extern crate termcolor;
use clap::{App, Arg};
use getopts::Options;
use itertools::Itertools;
use memmap::MmapOptions;
use std::fs::File;
use std::io::Write;
use std::{env, thread, time};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

struct ReadBufResult {
    buf: Vec<u8>,
    length: usize,
}

fn main() {
    let matches = App::new("dragontail")
        .author("wataru-script")
        .about("Dress up tail command")
        .arg(
            Arg::with_name("file")
                .value_name("FILE")
                .help("File name")
                .required(true),
        )
        .arg(
            Arg::with_name("keyword")
                .value_name("KEYWORD")
                .help("Keyword")
                .required(false),
        )
        .arg(
            Arg::with_name("fn")
                .help("Follow change of file")
                .short("fn")
                .long("follow"),
        )
        .arg(
            Arg::with_name("n")
                .help("Number of lines")
                .short("n")
                .long("number"),
        )
        .get_matches();
    // let args: Vec<String> = env::args().collect();
    // let mut opts = Options::new();
    // opts.optflag("f", "", "follow　change of file");
    // opts.optopt("n", "number", "number of lines", "NUMBER");

    // let arguments = match opts.parse(&args[1..]) {
    //     Ok(m) => m,
    //     Err(why) => panic!("Error: {}", why),
    // };
    // let line: i32 = if arguments.opt_present("n") {
    //     let l = match arguments.opt_str("n") {
    //         Some(num) => num,
    //         None => panic!("check your number of lines."),
    //     };
    //     l.parse().unwrap()
    // } else {
    //     10
    // };
    // let keywords = get_correct_keywords(&arguments.free[1..].to_vec());
    // if !arguments.opt_present("f") {
    //     tail(&arguments.free[0], line, &keywords)
    // } else {
    //     tail_follow(&args[1], line, &keywords);
    // }
}

fn get_start_pos(mmap: &memmap::Mmap, character_num: usize, line: i32) -> usize {
    let mut i = character_num - 1;
    let mut newline_num = line;
    loop {
        if i <= 0 {
            break;
        }
        if &mmap[i..(i + 1)] == b"\n" {
            newline_num -= 1;
            if newline_num <= 0 {
                break;
            }
        }
        i -= 1;
    }
    i
}

fn get_correct_keywords(keywords: &Vec<String>) -> Vec<String> {
    let mut res: Vec<String> = vec![];
    for (index, value) in keywords.iter().enumerate() {
        let mut flag = false;
        for i in 0..(keywords.len()) {
            if keywords[i].find(value).is_some() && index != i {
                res.push(keywords[i].to_string());
                flag = true;
                break;
            }
            if value.find(&keywords[i]).is_some() && index != i {
                res.push(value.to_string());
                flag = true;
                break;
            }
        }
        if !flag {
            res.push(value.to_string());
        }
    }
    res.into_iter().unique().collect()
}

fn get_mapped_file(file_name: &String) -> memmap::Mmap {
    let file = match File::open(file_name) {
        Ok(result) => result,
        Err(e) => panic!("error: {}", e),
    };
    let mmap = unsafe {
        match MmapOptions::new().map(&file) {
            Ok(result) => result,
            Err(e) => panic!("error: {}", e),
        }
    };
    mmap
}

fn print_buf(buf: Vec<u8>, keywords: &Vec<String>) {
    for line in buf.split(|x| *x == b'\n') {
        match encode(line) {
            Some(encoded) => {
                let splited_line = split_line_by_keywords(&encoded, keywords);
                print_colored_line(splited_line, keywords);
            }
            None => panic!("encode error."),
        }
    }
}

fn split_line_by_keywords<'a>(line: &'a String, keywords: &Vec<String>) -> Vec<&'a str> {
    let mut matches: Vec<(usize, &str)> = vec![];
    for kw in keywords {
        let mut m: Vec<_> = line.match_indices(kw).collect();
        if m.len() > 0 {
            matches.append(&mut m);
        }
    }
    matches.sort_by_key(|k| k.0);
    let mut result: Vec<&str> = vec![];
    let mut count: usize = 0;
    for m in matches {
        result.push(&line[count..m.0]);
        result.push(&line[m.0..(m.0 + m.1.len())]);
        count = m.0 + m.1.len();
    }
    if count != line.len() {
        result.push(&line[count..]);
    }
    result
}

fn print_colored_line(splited_line: Vec<&str>, keywords: &Vec<String>) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let len = splited_line.len();
    for i in 0..len {
        for (ki, kw) in keywords.iter().enumerate() {
            if splited_line[i] == kw {
                let color = get_colors(ki);
                stdout.set_color(ColorSpec::new().set_fg(color)).unwrap();
                break;
            } else {
                stdout
                    .set_color(ColorSpec::new().set_fg(Some(Color::White)))
                    .unwrap();
            }
        }
        if i != (len - 1) {
            write!(&mut stdout, "{}", splited_line[i]).unwrap();
        } else {
            writeln!(&mut stdout, "{}", splited_line[i]).unwrap();
        }
    }
}

fn get_colors(index: usize) -> Option<termcolor::Color> {
    let color_val: usize = 6;
    match index % color_val {
        0 => return Some(Color::Magenta),
        1 => return Some(Color::Cyan),
        2 => return Some(Color::Green),
        3 => return Some(Color::Red),
        4 => return Some(Color::Yellow),
        5 => return Some(Color::Blue),
        _ => return None,
    }
}

fn encode(buf: &[u8]) -> Option<String> {
    if let Ok(res) = String::from_utf8(buf.to_vec()) {
        return Some(res);
    }
    None
}

fn tail(file_name: &String, line: i32, keywords: &Vec<String>) {
    let mmap = get_mapped_file(file_name);
    let char_length = mmap.len() as usize;
    let start: usize = get_start_pos(&mmap, char_length, line);
    let buf = mmap[start..char_length].to_vec();
    print_buf(buf, keywords);
}

fn tail_follow(file_name: &String, line: i32, keywords: &Vec<String>) {
    let mmap = get_mapped_file(file_name);
    let mut char_length = mmap.len() as usize;
    let start: usize = get_start_pos(&mmap, char_length, line);
    let buf = mmap[start..char_length].to_vec();
    print_buf(buf, keywords);
    loop {
        match read_the_rest(file_name, char_length) {
            Some(result) => {
                print_buf(result.buf, keywords);
                char_length = result.length;
            }
            None => {
                thread::sleep(time::Duration::from_secs(1));
            }
        }
    }
}

fn read_the_rest(file_name: &String, start_pos: usize) -> Option<ReadBufResult> {
    let file = match File::open(file_name) {
        Ok(result) => result,
        Err(e) => panic!("error: {}", e),
    };
    let mmap = unsafe {
        match MmapOptions::new().map(&file) {
            Ok(result) => result,
            Err(e) => panic!("error: {}", e),
        }
    };
    let length = mmap.len() as usize;
    if length <= start_pos {
        return None;
    }
    Some(ReadBufResult {
        buf: mmap[start_pos..length].to_vec(),
        length,
    })
}
