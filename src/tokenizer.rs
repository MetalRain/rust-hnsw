extern crate unicode_normalization;

use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines};
use std::convert::AsRef;
use std::path::Path;
use std::vec::Vec;
use std::collections::{HashMap};

use unicode_normalization::UnicodeNormalization;


fn read_lines<P>(path: P) -> io::Result<Lines<BufReader<File>>>
    where P: AsRef<Path>
{
    let file = File::open(path)?;
    Ok(BufReader::new(file).lines())
}

fn normalize(phrase: String) -> String {
    phrase
        // unicode normalization
        .as_str().nfc().to_string()
        //TODO: accent normalization
        // case normalization
        .to_lowercase()
}

fn pre_tokenize(phrase: String) -> Vec<String> {
    // TODO: how can this return Iterator of &str?
    phrase.split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
}


fn load_corpus_file(name: &str) -> impl Iterator<Item = Vec<String>> {
    let filename = "./data/".to_owned() + name;

    read_lines(filename)
        .expect("Could not find the file")
        .map(|l| l.unwrap())
        .map(|phrase| normalize(phrase))
        .map(|phrase| pre_tokenize(phrase))
}

struct TokenizerModel {
    corpus: HashMap<String, u64>,
    vocabulary: HashMap<String, u32>,
}

impl TokenizerModel {
    fn new(phrases: impl Iterator<Item = Vec<String>>, size: u32) -> TokenizerModel {
        let mut corpus: HashMap<String, u64> = HashMap::new();
        for phrase in phrases {
            for token in phrase {
                let count = corpus.entry(token)
                    .or_insert(0);
                *count += 1;
            }
        };
        let vocabulary = TokenizerModel::initialize_vocabulary(corpus.clone());
        let mut model = TokenizerModel{
            corpus: corpus,
            vocabulary: vocabulary,
        };
        let mut pieces_remaining = size - model.vocabulary.len() as u32;
        loop {
            pieces_remaining -= 1;
            model.increase_vocabulary();
            if pieces_remaining <= 0 {
                break;
            }
        }
        model
    }

    /// Initialize small vocabulary from corpus
    /// vocabulary will be later increased by increase_vocabulary
    fn initialize_vocabulary(corpus: HashMap<String, u64>) -> HashMap<String, u32> {
        let mut vocabulary = HashMap::new();
        let pieces = corpus.keys()
            .flat_map(|token| token.chars());
        for (i, piece) in pieces.enumerate() {
            vocabulary.insert(piece.to_string(), i as u32);
        }
        vocabulary
    }

    fn add_piece(&mut self, piece: String){
        self.vocabulary.insert(piece, self.vocabulary.len() as u32 + 1u32);
    }

    fn increase_vocabulary(&self) {
        // TODO: implement merges
         
    }

    fn tokenize(&self, phrase: String) -> Vec<u32> {
        let mut tokens = Vec::new();
        let normalized = pre_tokenize(normalize(phrase));
        println!("Normalized: {:?}", &normalized);

        for token in normalized {
            let mut slice = token.as_str();
            // Find matching vocabulary pieces until
            // token is filled
            loop {
                for (piece, id) in self.vocabulary.iter() {
                    let l = piece.len();
                    let piece_slice = piece.as_str();
                    if &slice[0..l] == piece_slice {
                        slice = &slice[l..];
                        tokens.push(id.clone());
                        break;
                    }
                }
                // TODO: special case for characters not found in vocabulary
                if slice.len() == 0 {
                    break;
                }
            }
        }
        tokens
    }
}


pub fn update_tokenizer() {
    let phrases = load_corpus_file("t8.shakespeare.txt");
    let model = TokenizerModel::new(phrases, 120u32);
    println!("Tokens: {}, Vocabulary size: {}", model.corpus.len(), model.vocabulary.len());
    println!("Vocabulary: {:?}", model.vocabulary);

    let phrase = "You are dirt";
    let tokens = model.tokenize(phrase.clone().to_string());
    println!("{}, tokens: {:?}", phrase, tokens);
}