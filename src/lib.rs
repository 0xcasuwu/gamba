pub mod precompiled;

// Re-export alkanes module
pub mod alkanes;

#[cfg(test)]
pub mod tests {
    pub mod std;
    pub mod orbital_wand_integration_test;
    pub mod test_basic_forge_clean;
    pub mod orbital_forge_verification_test;
    pub mod orbital_integration_test;
    pub mod test_comprehensive_integration;
    pub mod test_dust_bonus_calculations;
    pub mod test_forge_edge_cases;
    pub mod test_forge_performance;
    pub mod test_multi_forge_scenarios;
    pub mod test_xor_randomness_analysis;
    pub mod debug_minimal_test;
    pub mod wand_utils;
    // pub mod mod; // Invalid - mod is a keyword
}
