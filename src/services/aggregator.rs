/// Aggregator: validates and normalizes proof-of-life scores from the mobile app.
///
/// In the MVP, the aggregator simply validates the range.
/// In production, this would cross-reference multiple data sources,
/// apply temporal consistency checks, and detect anomalies.

/// Validated score ready for on-chain publishing.
#[derive(Debug, Clone)]
pub struct ValidatedScore {
    pub user_stellar_address: String,
    pub score: u32,
    pub source: String,
}

/// Validate a perceptron output from the mobile app.
///
/// Rules:
/// - Score must be in range [0, 10000] (represents 0.00% - 100.00%)
/// - Source must not be empty
/// - User address must be a valid Stellar address (G...)
pub fn validate_score(
    user_stellar_address: &str,
    perceptron_output: i32,
    source: &str,
) -> Result<ValidatedScore, String> {
    // Range check
    if perceptron_output < 0 || perceptron_output > 10000 {
        return Err(format!(
            "perceptron_output must be in [0, 10000], got {}",
            perceptron_output
        ));
    }

    // Source check
    if source.is_empty() {
        return Err("source must not be empty".into());
    }

    // Address format check
    if !user_stellar_address.starts_with('G') || user_stellar_address.len() != 56 {
        return Err(format!(
            "Invalid Stellar address: {}",
            user_stellar_address
        ));
    }

    Ok(ValidatedScore {
        user_stellar_address: user_stellar_address.to_string(),
        score: perceptron_output as u32,
        source: source.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_ADDR: &str = "GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN7";

    #[test]
    fn test_valid_score() {
        let result = validate_score(VALID_ADDR, 8500, "perceptron_v1");
        assert!(result.is_ok());
        let v = result.unwrap();
        assert_eq!(v.score, 8500);
        assert_eq!(v.source, "perceptron_v1");
    }

    #[test]
    fn test_score_too_low() {
        assert!(validate_score(VALID_ADDR, -1, "test").is_err());
    }

    #[test]
    fn test_score_too_high() {
        assert!(validate_score(VALID_ADDR, 10001, "test").is_err());
    }

    #[test]
    fn test_empty_source() {
        assert!(validate_score(VALID_ADDR, 5000, "").is_err());
    }

    #[test]
    fn test_invalid_address() {
        assert!(validate_score("INVALID", 5000, "test").is_err());
    }

    #[test]
    fn test_boundary_scores() {
        assert!(validate_score(VALID_ADDR, 0, "test").is_ok());
        assert!(validate_score(VALID_ADDR, 10000, "test").is_ok());
    }
}
