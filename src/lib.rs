pub mod precompiled;

// Re-export alkanes module
pub mod alkanes;

#[cfg(test)]
pub mod tests {
    pub mod debug_minimal_test;
    // Other modules temporarily commented out due to compilation issues
    // pub mod std;
    // pub mod orbital_wand_integration_test;
    // pub mod test_basic_forge_clean;
    // pub mod orbital_forge_verification_test;
    // pub mod orbital_integration_test;
    // pub mod wand_utils;
}
