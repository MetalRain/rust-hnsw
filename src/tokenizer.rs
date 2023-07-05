extern crate unicode_normalization;

use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines};
use std::convert::AsRef;
use std::path::Path;
use std::vec::Vec;

use unicode_normalization::UnicodeNormalization;


fn read_lines<P>(path: P) -> io::Result<Lines<BufReader<File>>>
    where P: AsRef<Path>
{
    let file = File::open(path)?;
    Ok(BufReader::new(file).lines())
}

fn normalize(line: String) -> String {
    line
        // unicode normalization
        .as_str().nfc().to_string()
        //TODO: accent normalization
        // case normalization
        .to_lowercase()
}

fn load_corpus_file(name: &str) -> impl Iterator<Item = String> {
    let filename = "./data/".to_owned() + name;
    
    read_lines(filename)
        .expect("Could not find the file")
        .map(|l| l.unwrap())
        .map(|l| normalize(l))
}


pub fn update_tokenizer() {
    let lines = load_corpus_file("t8.shakespeare.txt");

    println!("Lines {}", lines.collect::<Vec<String>>().len());
}