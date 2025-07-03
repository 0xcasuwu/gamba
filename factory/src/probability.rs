use anyhow::Result;

pub struct ProbabilityCalculator;

impl ProbabilityCalculator {
    /// Calculate the probability of winning given a dust amount
    pub fn calculate_win_probability(dust_amount: u128) -> f64 {
        let dust_bonus = Self::calculate_dust_bonus(dust_amount);
        let effective_threshold = 141u8.saturating_sub(dust_bonus);
        
        // Probability = (256 - effective_threshold) / 256
        (256.0 - effective_threshold as f64) / 256.0
    }
    
    /// Calculate dust bonus points for a given dust amount
    pub fn calculate_dust_bonus(dust_amount: u128) -> u8 {
        const DUST_BONUS_THRESHOLD: u128 = 2000;
        const DUST_BONUS_INCREMENT: u128 = 1000;
        const DUST_BONUS_POINTS: u8 = 10;
        
        if dust_amount < DUST_BONUS_THRESHOLD {
            return 0;
        }
        
        let bonus_increments = (dust_amount - DUST_BONUS_THRESHOLD) / DUST_BONUS_INCREMENT;
        let bonus = bonus_increments * (DUST_BONUS_POINTS as u128);
        
        // Cap at 255 to prevent overflow
        std::cmp::min(bonus, 255) as u8
    }
    
    /// Calculate expected value for a given dust amount
    pub fn calculate_expected_value(dust_amount: u128, position_token_value: u128) -> f64 {
        let win_probability = Self::calculate_win_probability(dust_amount);
        let total_stake = position_token_value + dust_amount;
        
        // Expected value = (win_probability * wand_value) - (loss_probability * stake)
        // Assuming wand value equals stake for simplicity
        let wand_value = total_stake as f64;
        let loss_probability = 1.0 - win_probability;
        
        (win_probability * wand_value) - (loss_probability * total_stake as f64)
    }
    
    /// Get optimal dust amount for maximum expected value
    pub fn get_optimal_dust_amount(position_token_value: u128, max_dust: u128) -> u128 {
        let mut best_dust = 1000u128; // minimum
        let mut best_ev = Self::calculate_expected_value(best_dust, position_token_value);
        
        // Test dust amounts in increments of 1000
        let mut dust = 2000u128;
        while dust <= max_dust {
            let ev = Self::calculate_expected_value(dust, position_token_value);
            if ev > best_ev {
                best_ev = ev;
                best_dust = dust;
            }
            dust += 1000;
        }
        
        best_dust
    }
    
    /// Calculate the break-even dust amount
    pub fn calculate_break_even_dust(position_token_value: u128) -> u128 {
        // Find dust amount where expected value = 0
        let mut dust = 1000u128;
        let max_dust = 100_000u128;
        
        while dust <= max_dust {
            let ev = Self::calculate_expected_value(dust, position_token_value);
            if ev >= 0.0 {
                return dust;
            }
            dust += 100;
        }
        
        max_dust
    }
    
    /// Get win probability for different dust tiers
    pub fn get_dust_tier_probabilities() -> Vec<(u128, f64)> {
        vec![
            (1000, Self::calculate_win_probability(1000)),   // Base odds
            (2000, Self::calculate_win_probability(2000)),   // +10 bonus
            (3000, Self::calculate_win_probability(3000)),   // +20 bonus
            (5000, Self::calculate_win_probability(5000)),   // +40 bonus
            (10000, Self::calculate_win_probability(10000)), // +90 bonus
            (20000, Self::calculate_win_probability(20000)), // +190 bonus
        ]
    }
    
    /// Simulate gambling outcomes for statistical analysis
    pub fn simulate_outcomes(dust_amount: u128, num_simulations: u32) -> (u32, u32, f64) {
        let dust_bonus = Self::calculate_dust_bonus(dust_amount);
        let mut wins = 0u32;
        let mut losses = 0u32;
        
        // Simple simulation using deterministic XOR patterns
        for i in 0..num_simulations {
            // Simulate base XOR (0-255)
            let base_xor = (i % 256) as u8;
            let final_xor = base_xor.saturating_add(dust_bonus);
            
            if final_xor >= 141 {
                wins += 1;
            } else {
                losses += 1;
            }
        }
        
        let actual_win_rate = wins as f64 / num_simulations as f64;
        (wins, losses, actual_win_rate)
    }
    
    /// Calculate house edge for different dust amounts
    pub fn calculate_house_edge(dust_amount: u128) -> f64 {
        let win_probability = Self::calculate_win_probability(dust_amount);
        
        // House edge = 1 - (2 * win_probability)
        // This assumes 1:1 payout ratio
        1.0 - (2.0 * win_probability)
    }
    
    /// Get recommended dust amounts for different risk profiles
    pub fn get_risk_recommendations(position_token_value: u128) -> Vec<(String, u128, f64)> {
        vec![
            ("Conservative".to_string(), 1000, Self::calculate_win_probability(1000)),
            ("Moderate".to_string(), 3000, Self::calculate_win_probability(3000)),
            ("Aggressive".to_string(), 5000, Self::calculate_win_probability(5000)),
            ("High Roller".to_string(), 10000, Self::calculate_win_probability(10000)),
        ]
    }
    
    /// Calculate variance for gambling outcomes
    pub fn calculate_variance(dust_amount: u128, position_token_value: u128) -> f64 {
        let win_probability = Self::calculate_win_probability(dust_amount);
        let total_stake = (position_token_value + dust_amount) as f64;
        
        // Variance = p * (1-p) * (win_amount - loss_amount)^2
        let win_amount = total_stake; // Assuming 1:1 payout
        let loss_amount = -total_stake;
        let outcome_difference = win_amount - loss_amount;
        
        win_probability * (1.0 - win_probability) * outcome_difference.powi(2)
    }
    
    /// Calculate standard deviation
    pub fn calculate_standard_deviation(dust_amount: u128, position_token_value: u128) -> f64 {
        Self::calculate_variance(dust_amount, position_token_value).sqrt()
    }
    
    /// Calculate Kelly criterion for optimal bet sizing
    pub fn calculate_kelly_criterion(dust_amount: u128, position_token_value: u128) -> f64 {
        let win_probability = Self::calculate_win_probability(dust_amount);
        let loss_probability = 1.0 - win_probability;
        
        // Kelly = (bp - q) / b
        // where b = odds received (1:1 = 1), p = win probability, q = loss probability
        let b = 1.0; // 1:1 payout
        
        (b * win_probability - loss_probability) / b
    }
}