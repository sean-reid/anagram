use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// Embed the dictionary at compile time
const DICTIONARY: &str = include_str!("../dict.txt");

// Debug logging macro - only compiles when debug feature is enabled
#[cfg(feature = "debug")]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        web_sys::console::log_1(&format!($($arg)*).into());
    };
}

#[cfg(not(feature = "debug"))]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        // No-op when debug feature is disabled
    };
}

#[derive(Serialize, Deserialize)]
pub struct AnagramResults {
    results: Vec<String>,
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

/// Count total letters remaining
#[inline]
fn count_remaining(freq: &[u8; 26]) -> usize {
    freq.iter().map(|&c| c as usize).sum()
}

/// Calculate a quality score for an anagram phrase
/// Higher scores = better (fewer words, longer words, more balanced)
fn calculate_quality_score(words: &[String]) -> i32 {
    let num_words = words.len() as i32;
    let total_len: i32 = words.iter().map(|w| w.len() as i32).sum();
    let avg_word_len = if num_words > 0 { total_len / num_words } else { 0 };
    
    // Strongly prefer fewer words
    let word_count_penalty = num_words * 1000;
    
    // Reward average word length
    let length_bonus = avg_word_len * 100;
    
    // Small penalty for variance (prefer balanced word lengths)
    let variance_penalty = if num_words > 1 {
        let variance: i32 = words.iter()
            .map(|w| {
                let len = w.len() as i32;
                (len - avg_word_len).abs()
            })
            .sum();
        variance * 5
    } else {
        0
    };
    
    length_bonus - word_count_penalty - variance_penalty
}

/// Create a canonical signature for a word set to detect redundancy
fn create_signature(words: &[String]) -> String {
    let mut substantial: Vec<&String> = words.iter()
        .filter(|w| w.len() >= 4)
        .collect();
    substantial.sort();
    substantial.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("|")
}

/// Check if adding this word would create a redundant path
fn would_be_redundant(
    current: &[String],
    new_word: &str,
    seen_signatures: &HashSet<String>,
) -> bool {
    let mut test_words = current.to_vec();
    test_words.push(new_word.to_string());
    
    // Create signature from substantial words (4+ letters)
    let sig = create_signature(&test_words);
    
    // If signature is empty (no substantial words yet), not redundant
    if sig.is_empty() {
        return false;
    }
    
    // Check if we've seen this combination of substantial words
    seen_signatures.contains(&sig)
}

/// Recursively find all anagram combinations, filtering redundancy during search
fn find_anagrams_recursive(
    dict_words: &[(String, [u8; 26], usize)],
    available: &[u8; 26],
    current: &mut Vec<String>,
    results: &mut Vec<(i32, Vec<String>)>,
    seen_signatures: &mut HashSet<String>,
    start_idx: usize,
    remaining_letters: usize,
    max_results: usize,
) {
    if remaining_letters == 0 {
        // Calculate quality score for this solution
        let score = calculate_quality_score(current);
        results.push((score, current.clone()));
        
        // Record signature to prevent redundant searches
        let sig = create_signature(current);
        if !sig.is_empty() {
            seen_signatures.insert(sig);
        }
        return;
    }

    // Stop if we've found enough results
    if results.len() >= max_results {
        return;
    }

    // Dynamic minimum word length based on depth and remaining letters
    let depth = current.len();
    let min_word_len = if depth == 0 {
        // First word: try everything, but prioritize longer words
        1
    } else if depth == 1 {
        // Second word: prefer words that use at least 40% of remaining
        (remaining_letters * 4 / 10).max(2).min(remaining_letters)
    } else if depth == 2 {
        // Third word: prefer words that use at least 50% of remaining
        (remaining_letters * 5 / 10).max(3).min(remaining_letters)
    } else {
        // Fourth+ word: strongly prefer longer words
        (remaining_letters * 6 / 10).max(3).min(remaining_letters)
    };

    // Try words in order (already sorted by length descending)
    for i in start_idx..dict_words.len() {
        let (word, word_freq, word_len) = &dict_words[i];

        if *word_len > remaining_letters {
            continue;
        }

        // Apply minimum word length filter with gradual pruning
        if *word_len < min_word_len {
            // Only explore smaller words if we haven't found many results yet
            // or if remaining letters is very small
            if results.len() > max_results / 10 && remaining_letters > 5 {
                break; // Skip rest since they're even shorter
            }
        }

        if !can_use_word(word_freq, available) {
            continue;
        }

        // Skip if this would create a redundant path
        if would_be_redundant(current, word, seen_signatures) {
            continue;
        }

        let new_available = subtract_letters(available, word_freq);
        let new_remaining = remaining_letters - word_len;
        
        current.push(word.clone());
        
        // Recurse with priority: longer words are tried first
        find_anagrams_recursive(
            dict_words,
            &new_available,
            current,
            results,
            seen_signatures,
            i,
            new_remaining,
            max_results,
        );
        
        current.pop();
        
        // Early exit if we've found enough results
        if results.len() >= max_results {
            return;
        }
    }
}

#[wasm_bindgen]
pub fn test_logging() {
    debug_log!("Debug logging is ENABLED");
    #[cfg(feature = "debug")]
    web_sys::console::log_1(&"Direct console.log test".into());
}

#[wasm_bindgen]
pub fn solve_anagrams(target: &str) -> Result<JsValue, JsValue> {
    debug_log!("=== Starting anagram solver ===");
    debug_log!("Target phrase: '{}'", target);
    
    // Validate input
    if target.trim().is_empty() {
        debug_log!("ERROR: Empty target phrase");
        return Err(JsValue::from_str("Target phrase cannot be empty"));
    }
    
    debug_log!("Parsing dictionary...");
    // Parse embedded dictionary
    let dictionary: Vec<String> = DICTIONARY
        .lines()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphabetic()))
        .collect();
    
    debug_log!("Dictionary size: {} words", dictionary.len());
    
    if dictionary.is_empty() {
        debug_log!("ERROR: Dictionary is empty");
        return Err(JsValue::from_str("Dictionary is empty - check dict.txt"));
    }

    // Compute target frequency
    debug_log!("Computing target frequency...");
    let target_freq = compute_frequency(target);
    let target_len = count_remaining(&target_freq);
    debug_log!("Target length: {} letters", target_len);

    // Pre-process dictionary
    debug_log!("Pre-processing dictionary...");
    let mut dict_words: Vec<(String, [u8; 26], usize)> = dictionary
        .into_iter()
        .map(|w| {
            let freq = compute_frequency(&w);
            let len = count_remaining(&freq);
            (w, freq, len)
        })
        .filter(|(_, freq, len)| *len <= target_len && can_use_word(freq, &target_freq))
        .collect();

    debug_log!("Filtered dictionary: {} valid words", dict_words.len());

    // Sort by length descending - ensures we try longer words first
    dict_words.sort_by(|a, b| b.2.cmp(&a.2));
    
    if !dict_words.is_empty() {
        debug_log!("Longest word: '{}' ({} letters)", dict_words[0].0, dict_words[0].2);
        debug_log!("Shortest word: '{}' ({} letters)", 
                   dict_words[dict_words.len()-1].0, 
                   dict_words[dict_words.len()-1].2);
    }

    // Find anagrams with inline redundancy filtering
    debug_log!("Starting recursive search...");
    let mut results = Vec::new();
    let mut current = Vec::new();
    let mut seen_signatures = HashSet::new();
    
    find_anagrams_recursive(
        &dict_words,
        &target_freq,
        &mut current,
        &mut results,
        &mut seen_signatures,
        0,
        target_len,
        50_000,
    );

    debug_log!("Search complete. Found {} solutions", results.len());

    // Sort by quality score (highest first)
    debug_log!("Sorting by quality score...");
    results.sort_by(|a, b| b.0.cmp(&a.0));

    // Convert to strings with final deduplication
    debug_log!("Converting to strings and deduplicating...");
    let mut seen_phrases = HashSet::new();
    let all_anagrams: Vec<String> = results
        .into_iter()
        .filter_map(|(_, words)| {
            let phrase = words.join(" ");
            if seen_phrases.insert(phrase.clone()) {
                Some(phrase)
            } else {
                None
            }
        })
        .take(10_000)
        .collect();

    debug_log!("Final result count: {}", all_anagrams.len());
    if !all_anagrams.is_empty() {
        debug_log!("Best result: '{}'", all_anagrams[0]);
    }

    let anagram_results = AnagramResults {
        results: all_anagrams,
    };

    debug_log!("Serializing results...");
    let result = serde_wasm_bindgen::to_value(&anagram_results)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)));
    
    debug_log!("=== Anagram solver complete ===");
    result
}
