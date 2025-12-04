use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

// Embed the dictionary at compile time
const DICTIONARY: &str = include_str!("../dict.txt");

#[derive(Serialize, Deserialize)]
pub struct AnagramResults {
    single: Vec<String>,
    multi: Vec<String>,
}

/// Compute letter frequency map for the target phrase
fn compute_frequency(s: &str) -> [u8; 26] {
    let mut freq = [0u8; 26];
    for b in s.bytes() {
        if b.is_ascii_alphabetic() {
            freq[(b.to_ascii_lowercase() - b'a') as usize] += 1;
        }
    }
    freq
}

/// Check if word can be formed from available letters
#[inline]
fn can_use_word(word_freq: &[u8; 26], available: &[u8; 26]) -> bool {
    word_freq
        .iter()
        .zip(available.iter())
        .all(|(need, have)| need <= have)
}

/// Subtract word letters from available letters
#[inline]
fn subtract_letters(available: &[u8; 26], word_freq: &[u8; 26]) -> [u8; 26] {
    let mut result = *available;
    for i in 0..26 {
        result[i] -= word_freq[i];
    }
    result
}

/// Check if all letters are used
#[inline]
fn is_complete(freq: &[u8; 26]) -> bool {
    freq.iter().all(|&c| c == 0)
}

/// Count total letters remaining
#[inline]
fn count_remaining(freq: &[u8; 26]) -> usize {
    freq.iter().map(|&c| c as usize).sum()
}

/// Recursively find all anagram combinations
fn find_anagrams_recursive(
    dict_words: &[(String, [u8; 26], usize)],
    available: &[u8; 26],
    current: &mut Vec<String>,
    results: &mut Vec<Vec<String>>,
    start_idx: usize,
    remaining_letters: usize,
) {
    if remaining_letters == 0 {
        results.push(current.clone());
        return;
    }

    // Safety limit for web environment
    if results.len() > 50_000 {
        return;
    }

    for i in start_idx..dict_words.len() {
        let (word, word_freq, word_len) = &dict_words[i];

        if *word_len > remaining_letters {
            continue;
        }

        if can_use_word(word_freq, available) {
            let new_available = subtract_letters(available, word_freq);
            let new_remaining = remaining_letters - word_len;
            
            current.push(word.clone());
            find_anagrams_recursive(
                dict_words,
                &new_available,
                current,
                results,
                i,
                new_remaining,
            );
            current.pop();
        }
    }
}

#[wasm_bindgen]
pub fn solve_anagrams(target: &str) -> JsValue {
    // Parse embedded dictionary
    let dictionary: Vec<String> = DICTIONARY
        .lines()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphabetic()))
        .collect();

    // Compute target frequency
    let target_freq = compute_frequency(target);
    let target_len = count_remaining(&target_freq);

    // Pre-process dictionary
    let mut dict_words: Vec<(String, [u8; 26], usize)> = dictionary
        .into_iter()
        .map(|w| {
            let freq = compute_frequency(&w);
            let len = count_remaining(&freq);
            (w, freq, len)
        })
        .filter(|(_, freq, len)| *len <= target_len && can_use_word(freq, &target_freq))
        .collect();

    dict_words.sort_by(|a, b| b.2.cmp(&a.2));

    // Find all anagrams (both single-word and multi-word)
    let mut results = Vec::new();
    let mut current = Vec::new();
    find_anagrams_recursive(&dict_words, &target_freq, &mut current, &mut results, 0, target_len);

    // Convert all results to space-delimited strings
    let mut all_anagrams: Vec<String> = results
        .into_iter()
        .map(|words| words.join(" "))
        .collect();

    // Sort and deduplicate
    all_anagrams.sort();
    all_anagrams.dedup();

    // Sort by number of words, then by total length
    all_anagrams.sort_by(|a, b| {
        let a_words = a.split_whitespace().count();
        let b_words = b.split_whitespace().count();
        let words_cmp = a_words.cmp(&b_words);
        if words_cmp != std::cmp::Ordering::Equal {
            return words_cmp;
        }
        b.len().cmp(&a.len())
    });

    // Return in format expected by index.html (which still expects single/multi structure)
    // but now both contain space-delimited strings
    let anagram_results = AnagramResults {
        single: Vec::new(),  // Empty since we're combining everything
        multi: all_anagrams,
    };

    serde_wasm_bindgen::to_value(&anagram_results).unwrap()
}
