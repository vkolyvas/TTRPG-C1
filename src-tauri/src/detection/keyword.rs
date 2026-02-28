//! Keyword detection module

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Keyword match result
#[derive(Debug, Clone)]
pub struct KeywordMatch {
    pub keyword: String,
    pub category: String,
    pub confidence: f32,
    pub start_index: usize,
    pub end_index: usize,
}

/// Keyword definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyword {
    pub word: String,
    pub category: String,
    pub variations: Vec<String>,
    pub mood: Option<String>,
    pub priority: u8,
}

impl Keyword {
    /// Create a new keyword
    pub fn new(word: String, category: String) -> Self {
        Self {
            word: word.clone(),
            category,
            variations: vec![word],
            mood: None,
            priority: 0,
        }
    }

    /// Add a variation
    pub fn with_variation(mut self, variation: String) -> Self {
        self.variations.push(variation);
        self
    }

    /// Set the mood
    pub fn with_mood(mut self, mood: String) -> Self {
        self.mood = Some(mood);
        self
    }
}

/// Keyword vocabulary
#[derive(Debug, Clone)]
pub struct KeywordVocabulary {
    keywords: HashMap<String, Keyword>,
    categories: HashMap<String, Vec<String>>,
    version: u64,
}

impl KeywordVocabulary {
    /// Create a new vocabulary
    pub fn new() -> Self {
        Self {
            keywords: HashMap::new(),
            categories: HashMap::new(),
            version: 0,
        }
    }

    /// Add a keyword
    pub fn add_keyword(&mut self, keyword: Keyword) {
        for variation in &keyword.variations {
            self.keywords.insert(variation.to_lowercase(), keyword.clone());
        }

        self.categories
            .entry(keyword.category.clone())
            .or_default()
            .push(keyword.word.clone());

        self.version += 1;
    }

    /// Remove a keyword
    pub fn remove_keyword(&mut self, word: &str) {
        if let Some(keyword) = self.keywords.remove(&word.to_lowercase()) {
            if let Some(cat_keywords) = self.categories.get_mut(&keyword.category) {
                cat_keywords.retain(|k| k != &keyword.word);
            }
            self.version += 1;
        }
    }

    /// Get a keyword by exact match
    pub fn get(&self, word: &str) -> Option<&Keyword> {
        self.keywords.get(&word.to_lowercase())
    }

    /// Search for keywords in text (fuzzy matching)
    pub fn search(&self, text: &str) -> Vec<KeywordMatch> {
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower.split_whitespace().collect();
        let mut matches = Vec::new();

        for (i, word) in words.iter().enumerate() {
            // Exact match
            if let Some(keyword) = self.keywords.get(*word) {
                matches.push(KeywordMatch {
                    keyword: keyword.word.clone(),
                    category: keyword.category.clone(),
                    confidence: 1.0,
                    start_index: i,
                    end_index: i,
                });
                continue;
            }

            // Fuzzy match (substring)
            for (kw, keyword) in &self.keywords {
                if kw.contains(word) || word.contains(kw) {
                    let confidence = (kw.len() as f32) / (word.len().max(kw.len()) as f32);
                    if confidence > 0.5 {
                        matches.push(KeywordMatch {
                            keyword: keyword.word.clone(),
                            category: keyword.category.clone(),
                            confidence,
                            start_index: i,
                            end_index: i,
                        });
                    }
                }
            }
        }

        // Sort by priority and confidence
        matches.sort_by(|a, b| {
            let keyword_a = self.keywords.get(&a.keyword.to_lowercase());
            let keyword_b = self.keywords.get(&b.keyword.to_lowercase());

            let priority_a = keyword_a.map(|k| k.priority).unwrap_or(0);
            let priority_b = keyword_b.map(|k| k.priority).unwrap_or(0);

            priority_b
                .cmp(&priority_a)
                .then(b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal))
        });

        matches
    }

    /// Get keywords by category
    pub fn get_by_category(&self, category: &str) -> Vec<&Keyword> {
        self.keywords
            .values()
            .filter(|k| k.category == category)
            .collect()
    }

    /// Get vocabulary version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Load from JSON
    pub fn from_json(json: &str) -> Result<Self, AppError> {
        let keywords: Vec<Keyword> = serde_json::from_str(json)
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        let mut vocab = Self::new();
        for keyword in keywords {
            vocab.add_keyword(keyword);
        }

        Ok(vocab)
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, AppError> {
        let keywords: Vec<&Keyword> = self.keywords.values().collect();
        serde_json::to_string_pretty(&keywords)
            .map_err(|e| AppError::Serialization(e.to_string()))
    }
}

impl Default for KeywordVocabulary {
    fn default() -> Self {
        Self::new()
    }
}

/// Keyword detector
pub struct KeywordDetector {
    vocabulary: KeywordVocabulary,
    fuzzy_threshold: f32,
}

impl KeywordDetector {
    /// Create a new keyword detector
    pub fn new() -> Self {
        Self {
            vocabulary: KeywordVocabulary::new(),
            fuzzy_threshold: 0.7,
        }
    }

    /// Set vocabulary
    pub fn set_vocabulary(&mut self, vocabulary: KeywordVocabulary) {
        self.vocabulary = vocabulary;
    }

    /// Load vocabulary from file
    pub fn load_vocabulary(&mut self, path: &str) -> Result<(), AppError> {
        let content = std::fs::read_to_string(path)?;
        let vocab = KeywordVocabulary::from_json(&content)?;
        self.vocabulary = vocab;
        tracing::info!("Loaded keyword vocabulary v{}", self.vocabulary.version());
        Ok(())
    }

    /// Detect keywords in text
    pub fn detect(&self, text: &str) -> Vec<KeywordMatch> {
        self.vocabulary.search(text)
    }

    /// Get vocabulary version
    pub fn version(&self) -> u64 {
        self.vocabulary.version()
    }

    /// Reload vocabulary if changed
    pub fn reload_if_changed(&mut self, path: &str) -> Result<bool, AppError> {
        let content = std::fs::read_to_string(path)?;
        let new_vocab = KeywordVocabulary::from_json(&content)?;

        if new_vocab.version() > self.vocabulary.version() {
            self.vocabulary = new_vocab;
            tracing::info!("Reloaded keyword vocabulary v{}", self.vocabulary.version());
            return Ok(true);
        }

        Ok(false)
    }
}

impl Default for KeywordDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Default keyword vocabulary for TTRPG
pub fn default_ttrpg_vocabulary() -> KeywordVocabulary {
    let mut vocab = KeywordVocabulary::new();

    // Combat keywords
    vocab.add_keyword(Keyword::new("battle".to_string(), "combat".to_string())
        .with_variation("fight".to_string())
        .with_variation("attack".to_string())
        .with_mood("angry".to_string())
        .with_variation("combat".to_string()));

    vocab.add_keyword(Keyword::new("dragon".to_string(), "creature".to_string())
        .with_mood("fearful".to_string()));

    vocab.add_keyword(Keyword::new("slain".to_string(), "combat".to_string())
        .with_variation("killed".to_string())
        .with_variation("defeated".to_string())
        .with_mood("sad".to_string()));

    // Exploration keywords
    vocab.add_keyword(Keyword::new("enter".to_string(), "exploration".to_string())
        .with_variation("enter".to_string())
        .with_variation("go into".to_string()));

    vocab.add_keyword(Keyword::new("treasure".to_string(), "loot".to_string())
        .with_variation("gold".to_string())
        .with_variation("riches".to_string())
        .with_mood("happy".to_string()));

    // Mystery keywords
    vocab.add_keyword(Keyword::new("secret".to_string(), "mystery".to_string())
        .with_variation("hidden".to_string())
        .with_variation("mysterious".to_string()));

    vocab.add_keyword(Keyword::new("clue".to_string(), "mystery".to_string())
        .with_variation("evidence".to_string()));

    // Social keywords
    vocab.add_keyword(Keyword::new("merchant".to_string(), "social".to_string())
        .with_variation("shopkeeper".to_string())
        .with_variation("trader".to_string()));

    vocab.add_keyword(Keyword::new("king".to_string(), "social".to_string())
        .with_variation("queen".to_string())
        .with_variation("lord".to_string())
        .with_variation("lady".to_string()));

    // Danger keywords
    vocab.add_keyword(Keyword::new("trap".to_string(), "danger".to_string())
        .with_variation("danger".to_string())
        .with_variation("warning".to_string())
        .with_mood("fearful".to_string()));

    vocab.add_keyword(Keyword::new("poison".to_string(), "danger".to_string())
        .with_mood("disgusted".to_string()));

    // Emotional keywords
    vocab.add_keyword(Keyword::new("laugh".to_string(), "emotion".to_string())
        .with_variation("hilarious".to_string())
        .with_mood("happy".to_string()));

    vocab.add_keyword(Keyword::new("cry".to_string(), "emotion".to_string())
        .with_variation("tears".to_string())
        .with_mood("sad".to_string()));

    vocab
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_detection() {
        let mut detector = KeywordDetector::new();
        let vocab = default_ttrpg_vocabulary();
        detector.set_vocabulary(vocab);

        let matches = detector.detect("You enter the cave and find a dragon!");
        assert!(!matches.is_empty());

        // Should detect "enter", "dragon"
        let categories: Vec<_> = matches.iter().map(|m| m.category.clone()).collect();
        assert!(categories.contains(&"exploration".to_string()));
        assert!(categories.contains(&"creature".to_string()));
    }
}
