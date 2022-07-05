use std::str;
use rand::Rng;

fn get_words() -> Vec<String> {
    let bytes = include_bytes!("../resources/kotus-sanalista_v1.txt");

    let all_words = match str::from_utf8(bytes) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    let split = all_words.split('\n');
    let words: Vec<String> = split.map(ToOwned::to_owned).collect();

    return words;
}

pub fn get_random_word() -> String {
    println!("Reading the word list.");
    let words: Vec<String> = get_words();
    let word_count = words.len();
    let mut rng = rand::thread_rng();
    let random_word = &words[rng.gen_range(0..word_count)];
    println!("Random word: {}", random_word);

    return random_word.clone();
}
