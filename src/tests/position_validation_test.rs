//! Position Validation Test
//! 
//! Tests the new position validation logic that checks for specific
//! alkamist and dust positions similar to boiler's approach.

use crate::{DustSwap, orbital_wand::OrbitalWand};
use alkanes_support::{id::AlkaneId, parcel::AlkaneTransfer};
use anyhow::Result;

#[cfg(test)]
mod tests {
    use super::*;

    const ALKAMIST_BLOCK: u128 = 0x2;
    const ALKAMIST_TX: u128 = 25720;
    const DUST_BLOCK: u128 = 0x2;
    const DUST_TX: u128 = 35275;

    #[test]
    fn test_dust_swap_specific_positions() -> Result<()> {
        let dust_swap = DustSwap::default();
        
        // Test specific valid alkamist position
        let alkamist_id = AlkaneId {
            block: ALKAMIST_BLOCK,
            tx: ALKAMIST_TX,
        };
        assert!(dust_swap.is_valid_alkamist_or_dust(&alkamist_id)?, 
                "Specific alkamist position 2:25720 should be valid");

        // Test specific valid dust position
        let dust_id = AlkaneId {
            block: DUST_BLOCK,
            tx: DUST_TX,
        };
        assert!(dust_swap.is_valid_alkamist_or_dust(&dust_id)?, 
                "Specific dust position 2:35275 should be valid");

        // Test other dust tokens from block 2 (backward compatibility)
        let other_dust_id = AlkaneId {
            block: DUST_BLOCK,
            tx: 12345,
        };
        assert!(dust_swap.is_valid_alkamist_or_dust(&other_dust_id)?, 
                "Other dust tokens from block 2 should be valid");

        // Test invalid tokens
        let invalid_id = AlkaneId {
            block: 0x99,
            tx: 0x7777,
        };
        assert!(!dust_swap.is_valid_alkamist_or_dust(&invalid_id)?, 
                "Invalid tokens should be rejected");

        Ok(())
    }

    #[test]
    fn test_orbital_wand_specific_positions() -> Result<()> {
        let wand = OrbitalWand::default();
        
        // Test specific valid alkamist position
        let alkamist_id = AlkaneId {
            block: ALKAMIST_BLOCK,
            tx: ALKAMIST_TX,
        };
        assert!(wand.is_alkamist_token(&alkamist_id), 
                "Specific alkamist position 2:25720 should be valid");

        // Test specific valid dust position
        let dust_id = AlkaneId {
            block: DUST_BLOCK,
            tx: DUST_TX,
        };
        assert!(wand.is_dust_token(&dust_id), 
                "Specific dust position 2:35275 should be valid");

        // Test other dust tokens from block 2 (backward compatibility)
        let other_dust_id = AlkaneId {
            block: DUST_BLOCK,
            tx: 12345,
        };
        assert!(wand.is_dust_token(&other_dust_id), 
                "Other dust tokens from block 2 should be valid");

        // Test invalid tokens
        let invalid_id = AlkaneId {
            block: 0x99,
            tx: 0x7777,
        };
        assert!(!wand.is_valid_alkamist_or_dust(&invalid_id), 
                "Invalid tokens should be rejected");

        Ok(())
    }

    #[test]
    fn test_incoming_alkanes_validation() -> Result<()> {
        let dust_swap = DustSwap::default();
        
        // Test valid incoming alkanes
        let valid_transfers = vec![
            AlkaneTransfer {
                id: AlkaneId { block: ALKAMIST_BLOCK, tx: ALKAMIST_TX },
                value: 5,
            },
            AlkaneTransfer {
                id: AlkaneId { block: DUST_BLOCK, tx: DUST_TX },
                value: 1000,
            },
        ];
        
        assert!(dust_swap.validate_incoming_alkanes(&valid_transfers).is_ok(),
                "Valid incoming alkanes should pass validation");

        // Test empty incoming alkanes
        let empty_transfers = vec![];
        assert!(dust_swap.validate_incoming_alkanes(&empty_transfers).is_err(),
                "Empty incoming alkanes should fail validation");

        // Test zero value transfer
        let zero_value_transfers = vec![
            AlkaneTransfer {
                id: AlkaneId { block: ALKAMIST_BLOCK, tx: ALKAMIST_TX },
                value: 0,
            },
        ];
        assert!(dust_swap.validate_incoming_alkanes(&zero_value_transfers).is_err(),
                "Zero value transfers should fail validation");

        // Test invalid token ID
        let invalid_transfers = vec![
            AlkaneTransfer {
                id: AlkaneId { block: 0x99, tx: 0x7777 },
                value: 1,
            },
        ];
        assert!(dust_swap.validate_incoming_alkanes(&invalid_transfers).is_err(),
                "Invalid token IDs should fail validation");

        Ok(())
    }

    #[test]
    fn test_alkamist_position_exclusion() -> Result<()> {
        let dust_swap = DustSwap::default();
        
        // Test that alkamist position is NOT valid when used as dust
        // This prevents double-counting the same position
        let alkamist_as_dust = AlkaneId {
            block: DUST_BLOCK, // Same block as dust
            tx: ALKAMIST_TX,   // But alkamist tx - should be rejected
        };

        // When checking specifically for alkamist position, it should be valid
        assert!(dust_swap.is_valid_alkamist_or_dust(&alkamist_as_dust)?, 
                "Alkamist position should be valid when checked as alkamist");

        // But when we have the logic to prevent double-counting, 
        // the alkamist position should be treated as alkamist, not dust
        let wand = OrbitalWand::default();
        assert!(wand.is_alkamist_token(&alkamist_as_dust), 
                "Alkamist position should be identified as alkamist token");
        assert!(!wand.is_dust_token(&alkamist_as_dust), 
                "Alkamist position should NOT be identified as dust token");

        Ok(())
    }
}