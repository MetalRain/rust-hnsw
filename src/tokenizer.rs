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
    corpus: HashMap<Vec<String>, u64>,
    vocabulary: HashMap<String, u32>,
    merge_rules: Vec<(String, String, String)>,
}

impl TokenizerModel {
    fn new(phrases: impl Iterator<Item = Vec<String>>, size: usize) -> TokenizerModel {
        let mut word_counts: HashMap<String, u64> = HashMap::new();
        for phrase in phrases {
            for token in phrase {
                let count = word_counts.entry(token)
                    .or_insert(0);
                *count += 1;
            }
        };
        let mut corpus: HashMap<Vec<String>, u64> = HashMap::new();
        let mut vocabulary: HashMap<String, u32> = HashMap::new();
        let mut vocabulary_index: u32 = 0;
        for (word, count) in word_counts.iter() {
            let word_tokens: Vec<String> = word.chars()
                .map(|c| c.to_string())
                .collect();
            corpus.insert(word_tokens.clone(), *count);
            for token in word_tokens {
                if !vocabulary.contains_key(&token){
                    vocabulary.insert(token.to_string(), vocabulary_index);
                    vocabulary_index += 1;
                }
            }
        }
    
        let mut model = TokenizerModel{
            corpus: corpus,
            vocabulary: vocabulary,
            merge_rules: Vec::new(),
        };

        // Find best tokens to increase vocabulary to size
        let mut pieces_remaining = size - model.vocabulary.len();
        loop {
            println!("Increasing vocabulary.. remaining {}", pieces_remaining);
            // NOTE:
            // While big batches are faster to calculate
            // smaller batch size
            // adds longer and more interesting compound words
            if pieces_remaining > 1000 {
                pieces_remaining -= 500;
                if !model.increase_vocabulary(500) {
                    break;
                }
            } if pieces_remaining > 300 {
                pieces_remaining -= 300;
                if !model.increase_vocabulary(300) {
                    break;
                }
            } else if pieces_remaining > 100 {
                pieces_remaining -= 100;
                if !model.increase_vocabulary(100) {
                    break;
                }
            } else if pieces_remaining > 1 {
                pieces_remaining -= pieces_remaining;
                if !model.increase_vocabulary(pieces_remaining) {
                    break;
                }
            } else {
                break;
            }
        }
        model
    }

    fn increase_vocabulary(&mut self, amount: usize) -> bool {
        // find most used token pairs
        let amount_added = self.most_frequent_pairs(amount, 5);
    
        // update corpus
        self.update_corpus();

        amount_added >= amount
    }

    fn most_frequent_pairs(&mut self, amount: usize, low_limit: u32) -> usize {
        let mut frequencies: HashMap<(&String, &String), u32> = HashMap::new();
        
        for word_tokens in self.corpus.keys() {
            let token_count = word_tokens.len();
            if token_count < 2 {
                continue;
            }
            let last_index = word_tokens.len() - 2;
            for i in 0..last_index {
                let token_a = &word_tokens[i];
                let token_b = &word_tokens[i+1];

                let vocabulary_pair: (&String, &String) = (token_a, token_b);
                let count = frequencies.entry(vocabulary_pair)
                    .or_insert(0);
                *count += 1;
            }
        }

        let mut counts: Vec<(&(&String, &String), &u32)> = frequencies.iter().collect();
        counts.sort_by(|&(_, a_val), &(_, b_val)| b_val.cmp(a_val));

        let pairs_count = counts.len();
        if pairs_count == 0 {
            return 0;
        }

        let mut limited_amount = amount;
        if pairs_count < limited_amount {
            limited_amount = pairs_count - 1;
        }

        let mut count_added = 0;
        for ((a, b), occurrencies) in &counts[0..limited_amount] {
            if *occurrencies < &low_limit {
                continue
            }
            count_added +=1;
            //println!("Most used pair: {} {}, occurrencies: {}", a, b, occurrencies);
            let new_token: String = a.to_string() + b.as_str();

            // add to merge rules
            self.merge_rules.push((a.to_string(), b.to_string(), new_token.clone()));
            
            // add to vocabulary
            self.vocabulary.insert(new_token, self.vocabulary.len() as u32);
        }

        count_added
    }

    fn update_corpus(&mut self) {
        let mut new_corpus: HashMap<Vec<String>, u64> = HashMap::new();
        for (word_tokens, count) in self.corpus.iter() {
            new_corpus.insert(self.apply_merge_rules(word_tokens.clone()), *count);
        }
        self.corpus = new_corpus;
    }


    fn apply_merge_rules(&self, word_tokens: Vec<String>) -> Vec<String> {
        //println!("Old vec {:?}", word_tokens);
        let mut new_tokens: Vec<String> = Vec::new();
        let token_count = word_tokens.len();
        if token_count < 2 {
            return word_tokens;
        }
        let last_index = token_count - 2;

        let mut i = 0;
        loop {
            if i > last_index {
                break;
            }

            let old_token_a = &word_tokens[i];
            let old_token_b = &word_tokens[i+1];
            let mut token_to_push: &String = old_token_a;

            for (token_a, token_b, new_token) in self.merge_rules.iter() {
                if old_token_a == token_a && old_token_b == token_b {
                    token_to_push = new_token;
                    i += 1;
                    break;
                }
            }

            new_tokens.push(token_to_push.to_string());

            i += 1;
        }

        // last token
        i = token_count - 1;
        let old_token_a = &word_tokens[i-1];
        let old_token_b = &word_tokens[i];

        for (token_a, token_b, _new_token) in self.merge_rules.iter() {
            if old_token_a == token_a && old_token_b == token_b {
                // no more tokens to push
                return new_tokens;
            }
        }
        new_tokens.push(old_token_b.to_string());
        //println!("New vec {:?}", new_tokens);
       
        new_tokens
    }

    fn tokenize(&self, phrase: String) -> Vec<u32> {
        let mut tokens = Vec::new();
        let normalized = pre_tokenize(normalize(phrase));
        println!("Normalized: {:?}", &normalized);

        for token in normalized {
            let mut slice = token.as_str();
            let mut slice_len = slice.len();
            // Find matching vocabulary pieces until
            // token is filled
            loop {
                for (piece, id) in self.vocabulary.iter() {
                    let l = piece.len();
                    let piece_slice = piece.as_str();
                    if l <= slice_len && &slice[0..l] == piece_slice {
                        slice = &slice[l..];
                        slice_len = slice.len();
                        tokens.push(id.clone());
                        break;
                    }
                }
                // TODO: special case for characters not found in vocabulary
                if slice_len == 0 {
                    break;
                }
            }
        }
        tokens
    }
}


pub fn update_tokenizer() {
    let phrases = load_corpus_file("t8.shakespeare.txt");
    let model = TokenizerModel::new(phrases, 5000);
    println!("Tokens: {}, Vocabulary size: {}", model.corpus.len(), model.vocabulary.len());
    println!("Vocabulary: {:?}", model.vocabulary);

    let phrase = "You are dirt";
    let tokens = model.tokenize(phrase.clone().to_string());
    println!("{}, tokens: {:?}", phrase, tokens);
}