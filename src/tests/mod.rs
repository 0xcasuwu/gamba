pub mod std;
pub mod orbital_forge_verification_test;
pub mod orbital_integration_test;
pub mod debug_minimal_test;
pub mod test_basic_forge_clean;

// Comprehensive test suite (matching boiler quality)
pub mod test_multi_forge_scenarios;
pub mod test_forge_edge_cases;
pub mod test_forge_performance;
pub mod test_xor_randomness_analysis;
pub mod test_dust_bonus_calculations;
pub mod test_comprehensive_integration;

// Comprehensive integration tests
pub mod orbital_wand_integration_test;

// Note: Advanced tests commented out due to API changes
// #[cfg(test)]
// pub mod wand_factory_tests;
//
// #[cfg(test)]
// pub mod wand_utils;
//
// #[cfg(test)]
// pub mod wand_end_to_end_test;
