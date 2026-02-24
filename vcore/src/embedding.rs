// VecBase — embedding.rs
// Vector normalization, format parsing, and similarity metrics.
// Author: d65v <https://github.com/d65v>

/// Supported similarity metrics.
#[derive(Debug, Clone, PartialEq)]
pub enum Metric {
    /// Cosine similarity (assumes pre-normalized vectors → dot product)
    Cosine,
    /// Euclidean (L2) distance (lower = closer)
    Euclidean,
    /// Raw dot product (higher = closer)
    DotProduct,
}

// ── Normalization ─────────────────────────────────────────────────────────────

/// L2-normalize a vector (in-place copy). Returns a unit vector.
/// If the vector is all-zero, it is returned unchanged.
pub fn normalize(v: &[f32]) -> Vec<f32> {
    let mag = magnitude(v);
    if mag < 1e-10 {
        return v.to_vec();
    }
    v.iter().map(|x| x / mag).collect()
}

/// Compute the L2 (Euclidean) magnitude (norm) of a vector.
#[inline]
pub fn magnitude(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

// ── Similarity / Distance ─────────────────────────────────────────────────────

/// Dot product of two equal-length vectors.
///
/// # Panics
/// Does not panic; if lengths differ the shorter one is the limit (zip).
#[inline]
pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Cosine similarity between two vectors.
/// Pre-normalize both for speed if calling many times.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let na = normalize(a);
    let nb = normalize(b);
    dot(&na, &nb).clamp(-1.0, 1.0)
}

/// Squared Euclidean distance (cheaper — avoids sqrt when only ranking).
pub fn euclidean_distance_sq(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let d = x - y;
            d * d
        })
        .sum()
}

/// Euclidean distance.
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    euclidean_distance_sq(a, b).sqrt()
}

/// Generic score function: higher score = better match.
pub fn score(metric: &Metric, query: &[f32], candidate: &[f32]) -> f32 {
    match metric {
        Metric::Cosine => dot(query, candidate), // assumes pre-normalized
        Metric::DotProduct => dot(query, candidate),
        Metric::Euclidean => -euclidean_distance(query, candidate), // negate: lower dist = higher score
    }
}

// ── Embedding Parsing ─────────────────────────────────────────────────────────

/// Parse a JSON array of floats into a Vec<f32>.
///
/// # Errors
/// Returns `None` if parsing fails or the array is empty.
pub fn parse_json_embedding(json: &str) -> Option<Vec<f32>> {
    // Minimal parser — avoids pulling in a full JSON lib for a hot path.
    let trimmed = json.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return None;
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    let values: Option<Vec<f32>> = inner
        .split(',')
        .map(|s| s.trim().parse::<f32>().ok())
        .collect();
    let v = values?;
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

/// Parse a whitespace-separated string of floats.
pub fn parse_text_embedding(text: &str) -> Option<Vec<f32>> {
    let v: Option<Vec<f32>> = text
        .split_whitespace()
        .map(|s| s.parse::<f32>().ok())
        .collect();
    let v = v?;
    if v.is_empty() { None } else { Some(v) }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_unit_vector() {
        let v = vec![3.0f32, 4.0];
        let n = normalize(&v);
        let mag = magnitude(&n);
        assert!((mag - 1.0).abs() < 1e-6, "magnitude should be ~1.0");
    }

    #[test]
    fn test_normalize_zero_vector() {
        let v = vec![0.0f32, 0.0, 0.0];
        let n = normalize(&v);
        assert_eq!(n, v, "zero vector should be returned unchanged");
    }

    #[test]
    fn test_cosine_same_vector() {
        let v = vec![1.0, 2.0, 3.0];
        let s = cosine_similarity(&v, &v);
        assert!((s - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_cosine_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let s = cosine_similarity(&a, &b);
        assert!(s.abs() < 1e-6);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        assert!((euclidean_distance(&a, &b) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_parse_json_embedding() {
        let json = "[0.1, 0.2, 0.3]";
        let v = parse_json_embedding(json).unwrap();
        assert_eq!(v.len(), 3);
        assert!((v[0] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_parse_json_invalid() {
        assert!(parse_json_embedding("not json").is_none());
        assert!(parse_json_embedding("[]").is_none());
    }

    #[test]
    fn test_parse_text_embedding() {
        let text = "1.0 2.0 3.0 4.0";
        let v = parse_text_embedding(text).unwrap();
        assert_eq!(v.len(), 4);
    }
}
