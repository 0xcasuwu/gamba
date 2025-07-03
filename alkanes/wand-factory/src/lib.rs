use alkanes_support::context::Context;
use metashrew_support::compat::to_arraybuffer_layout;

use alkanes_runtime::{
    declare_alkane, message::MessageDispatch, runtime::AlkaneResponder, token::Token,
};

use alkanes_support::{
    cellpack::Cellpack,
    id::AlkaneId,
    parcel::{AlkaneTransfer, AlkaneTransferParcel},
    response::CallResponse,
};

use anyhow::{anyhow, Result};

/// Orbital token template ID
const ORBITAL_TOKEN_TEMPLATE_ID: u128 = 0x601;

/// DUST token ID (assumed to be block 2, tx 35270)
const DUST_TOKEN_ID: AlkaneId = AlkaneId { block: 2, tx: 35270 };

#[derive(Default)]
pub struct WandFactory(());

impl AlkaneResponder for WandFactory {}

#[derive(MessageDispatch)]
enum WandFactoryMessage {
    #[opcode(0)]
    Initialize {
        dust_token_id: AlkaneId,           // DUST token for enhancement
        success_threshold: u128,           // XOR threshold for success (e.g., 144)
        dust_bonus_rate: u128,             // DUST bonus rate (e.g., 5 per 1000 DUST)
        orbital_token_template_id: AlkaneId, // Template for creating orbital tokens
    },

    #[opcode(1)]
    ForgeOrbital,

    #[opcode(10)]
    #[returns(u128)]
    GetSuccessfulForges,

    #[opcode(11)]
    #[returns(u128)]
    GetFailedForges,

    #[opcode(12)]
    #[returns(u128)]
    GetTotalForges,

    #[opcode(20)]
    #[returns(AlkaneId)]
    GetDustTokenId,

    #[opcode(21)]
    #[returns(u128)]
    GetSuccessThreshold,

    #[opcode(22)]
    #[returns(u128)]
    GetDustBonusRate,

    #[opcode(23)]
    #[returns(AlkaneId)]
    GetOrbitalTokenTemplateId,

    #[opcode(30)]
    #[returns(Vec<u8>)]
    GetAllRegisteredOrbitals,

    #[opcode(31)]
    #[returns(u128)]
    IsRegisteredOrbital {
        orbital_id: AlkaneId,
    },

    #[opcode(40)]
    #[returns(Vec<u8>)]
    GetFactoryInfo,

    #[opcode(50)]
    #[returns(u128)]
    CalculateBaseXor,

    #[opcode(51)]
    #[returns(u128)]
    CalculateDustBonus {
        dust_amount: u128,
    },
}

impl Token for WandFactory {
    fn name(&self) -> String {
        String::from("Orbital Factory")
    }

    fn symbol(&self) -> String {
        String::from("FORGE")
    }
}

impl WandFactory {
    fn initialize(
        &self,
        dust_token_id: AlkaneId,
        success_threshold: u128,
        dust_bonus_rate: u128,
        orbital_token_template_id: AlkaneId,
    ) -> Result<CallResponse> {
        let _context = self.context()?;
        let response = CallResponse::default();

        self.observe_initialization()?;

        // Store all parameters
        self.set_dust_token_id(&dust_token_id)?;
        self.set_success_threshold(success_threshold as u8);
        self.set_dust_bonus_rate(dust_bonus_rate);
        self.set_orbital_token_template_id(&orbital_token_template_id)?;

        // Initialize counters
        self.set_successful_forges(0);
        self.set_failed_forges(0);

        Ok(response)
    }

    fn forge_orbital(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::default();

        // Calculate base XOR from blockchain data
        let base_xor = self.calculate_base_xor_internal()?;

        // Get DUST tokens and calculate bonus
        let dust_amount = self.get_dust_input_amount(&context)?;
        let dust_bonus = self.calculate_dust_bonus_internal(dust_amount)?;
        let final_result = base_xor.saturating_add(dust_bonus);

        // Check success threshold
        let success_threshold = self.success_threshold();
        if final_result > success_threshold {
            // Successful forge - create orbital token
            let orbital_token = self.create_orbital_token(
                dust_amount,
                base_xor,
                dust_bonus,
                final_result,
            )?;

            // Register the orbital token as our child
            self.register_orbital(&orbital_token.id);

            // Increment successful forges
            let new_successful = self.successful_forges().checked_add(1).unwrap_or(0);
            self.set_successful_forges(new_successful);

            // Return the orbital token to the user
            response.alkanes.0.push(orbital_token);
        } else {
            // Failed forge - increment failed counter but no token
            let new_failed = self.failed_forges().checked_add(1).unwrap_or(0);
            self.set_failed_forges(new_failed);
        }

        // DUST is consumed regardless of success/failure
        // (This is automatic as DUST tokens are not returned in response)

        Ok(response)
    }

    fn calculate_base_xor_internal(&self) -> Result<u8> {
        // Get current block height and use it as source of randomness
        let current_height = self.height();
        
        // Get current transaction data - use input data as source
        let context = self.context()?;
        let myself = &context.myself;
        
        // XOR block height with transaction components
        let height_byte = (current_height % 256) as u8;
        let block_byte = (myself.block % 256) as u8;
        let tx_byte = (myself.tx % 256) as u8;
        
        let base_xor = height_byte ^ block_byte ^ tx_byte;

        Ok(base_xor)
    }

    fn get_dust_input_amount(&self, context: &Context) -> Result<u128> {
        let dust_token_id = self.dust_token_id()?;
        let mut total_dust = 0u128;

        // Check all incoming tokens for DUST
        for transfer in &context.incoming_alkanes.0 {
            if transfer.id == dust_token_id {
                total_dust = total_dust.checked_add(transfer.value).unwrap_or(total_dust);
            }
        }

        Ok(total_dust)
    }

    fn calculate_dust_bonus_internal(&self, dust_amount: u128) -> Result<u8> {
        let dust_bonus_rate = self.dust_bonus_rate();
        let bonus = ((dust_amount / 1000) * dust_bonus_rate).min(255) as u8;
        Ok(bonus)
    }

    fn create_orbital_token(
        &self,
        dust_amount: u128,
        base_xor: u8,
        dust_bonus: u8,
        final_result: u8,
    ) -> Result<AlkaneTransfer> {
        let context = self.context()?;
        let orbital_template_id = self.orbital_token_template_id()?;
        let current_block = u128::from(self.height());
        let forge_id = self.total_forges();

        // Create cellpack for orbital token creation
        let cellpack = Cellpack {
            target: orbital_template_id,
            inputs: vec![
                0x0,           // Initialize opcode
                forge_id,      // Unique forge ID
                dust_amount,   // DUST amount used
                base_xor as u128, // Base XOR result
                dust_bonus as u128, // DUST bonus applied
                final_result as u128, // Final XOR result
                current_block, // Block of creation
                context.myself.block, // Factory block ID
                context.myself.tx,    // Factory tx ID
            ],
        };

        // No tokens sent to orbital (it's created with forge state only)
        let orbital_parcel = AlkaneTransferParcel::default();

        let create_response = self.call(&cellpack, &orbital_parcel, self.fuel())?;

        if create_response.alkanes.0.is_empty() {
            return Err(anyhow!("Orbital token not returned by template"));
        }

        Ok(create_response.alkanes.0[0].clone())
    }

    // Storage operations following boiler patterns

    fn dust_token_id(&self) -> Result<AlkaneId> {
        let bytes = self.load("/dust_token_id".as_bytes().to_vec());

        if bytes.len() < 32 {
            return Err(anyhow!("DUST token ID not set"));
        }

        Ok(AlkaneId {
            block: u128::from_le_bytes(bytes[0..16].try_into().map_err(|_| {
                anyhow!("Failed to parse DUST token block ID from storage")
            })?),
            tx: u128::from_le_bytes(bytes[16..32].try_into().map_err(|_| {
                anyhow!("Failed to parse DUST token tx ID from storage")
            })?),
        })
    }

    fn set_dust_token_id(&self, id: &AlkaneId) -> Result<()> {
        let mut bytes = Vec::with_capacity(32);
        bytes.extend_from_slice(&id.block.to_le_bytes());
        bytes.extend_from_slice(&id.tx.to_le_bytes());

        self.store("/dust_token_id".as_bytes().to_vec(), bytes);
        Ok(())
    }

    fn orbital_token_template_id(&self) -> Result<AlkaneId> {
        let bytes = self.load("/orbital_token_template_id".as_bytes().to_vec());

        if bytes.len() < 32 {
            return Err(anyhow!("Orbital token template ID not set"));
        }

        Ok(AlkaneId {
            block: u128::from_le_bytes(bytes[0..16].try_into().map_err(|_| {
                anyhow!("Failed to parse orbital template block ID from storage")
            })?),
            tx: u128::from_le_bytes(bytes[16..32].try_into().map_err(|_| {
                anyhow!("Failed to parse orbital template tx ID from storage")
            })?),
        })
    }

    fn set_orbital_token_template_id(&self, id: &AlkaneId) -> Result<()> {
        let mut bytes = Vec::with_capacity(32);
        bytes.extend_from_slice(&id.block.to_le_bytes());
        bytes.extend_from_slice(&id.tx.to_le_bytes());

        self.store("/orbital_token_template_id".as_bytes().to_vec(), bytes);
        Ok(())
    }

    fn success_threshold(&self) -> u8 {
        let bytes = self.load("/success_threshold".as_bytes().to_vec());
        if !bytes.is_empty() {
            bytes[0]
        } else {
            144 // Default threshold
        }
    }

    fn set_success_threshold(&self, threshold: u8) {
        self.store("/success_threshold".as_bytes().to_vec(), vec![threshold]);
    }

    fn dust_bonus_rate(&self) -> u128 {
        self.load_u128("/dust_bonus_rate")
    }

    fn set_dust_bonus_rate(&self, rate: u128) {
        self.store(
            "/dust_bonus_rate".as_bytes().to_vec(),
            rate.to_le_bytes().to_vec(),
        );
    }

    fn successful_forges(&self) -> u128 {
        self.load_u128("/successful_forges")
    }

    fn set_successful_forges(&self, count: u128) {
        self.store(
            "/successful_forges".as_bytes().to_vec(),
            count.to_le_bytes().to_vec(),
        );
    }

    fn failed_forges(&self) -> u128 {
        self.load_u128("/failed_forges")
    }

    fn set_failed_forges(&self, count: u128) {
        self.store(
            "/failed_forges".as_bytes().to_vec(),
            count.to_le_bytes().to_vec(),
        );
    }

    fn total_forges(&self) -> u128 {
        self.successful_forges().saturating_add(self.failed_forges())
    }

    // Registry operations following boiler patterns

    fn is_registered_orbital_internal(&self, orbital_id: &AlkaneId) -> bool {
        let key = format!("/registered_orbitals/{}_{}", orbital_id.block, orbital_id.tx).into_bytes();
        let bytes = self.load(key);
        !bytes.is_empty() && bytes[0] == 1
    }

    fn register_orbital(&self, orbital_id: &AlkaneId) {
        // Store individual registration for O(1) lookup
        let key = format!("/registered_orbitals/{}_{}", orbital_id.block, orbital_id.tx).into_bytes();
        self.store(key, vec![1u8]);

        // Add to centralized list for enumeration
        let mut orbitals_list = self.registered_orbitals_list();
        orbitals_list.push(orbital_id.clone());
        self.set_registered_orbitals_list(orbitals_list);

        // Update count
        let new_count = self.registered_orbitals_count().checked_add(1).unwrap_or(0);
        self.set_registered_orbitals_count(new_count);
    }

    fn registered_orbitals_list(&self) -> Vec<AlkaneId> {
        let bytes = self.load("/registered_orbitals_list".as_bytes().to_vec());
        if bytes.is_empty() {
            return Vec::new();
        }

        let mut orbitals = Vec::new();
        let mut offset = 0;

        // Each AlkaneId is 32 bytes (16 bytes block + 16 bytes tx)
        while offset + 32 <= bytes.len() {
            let block_bytes: [u8; 16] = bytes[offset..offset+16].try_into().unwrap_or([0; 16]);
            let tx_bytes: [u8; 16] = bytes[offset+16..offset+32].try_into().unwrap_or([0; 16]);
            
            orbitals.push(AlkaneId {
                block: u128::from_le_bytes(block_bytes),
                tx: u128::from_le_bytes(tx_bytes),
            });
            
            offset += 32;
        }

        orbitals
    }

    fn set_registered_orbitals_list(&self, orbitals: Vec<AlkaneId>) {
        let mut bytes = Vec::new();
        
        for orbital in orbitals {
            bytes.extend_from_slice(&orbital.block.to_le_bytes());
            bytes.extend_from_slice(&orbital.tx.to_le_bytes());
        }
        
        self.store("/registered_orbitals_list".as_bytes().to_vec(), bytes);
    }

    fn registered_orbitals_count(&self) -> u128 {
        self.load_u128("/registered_orbitals_count")
    }

    fn set_registered_orbitals_count(&self, count: u128) {
        self.store(
            "/registered_orbitals_count".as_bytes().to_vec(),
            count.to_le_bytes().to_vec(),
        );
    }

    // Helper function to load u128 values from storage
    fn load_u128(&self, key_str: &str) -> u128 {
        let key = key_str.as_bytes().to_vec();
        let bytes = self.load(key);
        if bytes.len() >= 16 {
            let bytes_array: [u8; 16] = bytes[0..16].try_into().unwrap_or([0; 16]);
            u128::from_le_bytes(bytes_array)
        } else {
            0
        }
    }

    // Getter functions for frontend/external consumption

    fn get_successful_forges(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.successful_forges().to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_failed_forges(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.failed_forges().to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_total_forges(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.total_forges().to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_dust_token_id(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        let dust_token_id = self.dust_token_id()?;
        
        let mut data = Vec::with_capacity(32);
        data.extend_from_slice(&dust_token_id.block.to_le_bytes());
        data.extend_from_slice(&dust_token_id.tx.to_le_bytes());
        
        response.data = data;
        Ok(response)
    }

    fn get_success_threshold(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = (self.success_threshold() as u128).to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_dust_bonus_rate(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.dust_bonus_rate().to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_orbital_token_template_id(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        let template_id = self.orbital_token_template_id()?;
        
        let mut data = Vec::with_capacity(32);
        data.extend_from_slice(&template_id.block.to_le_bytes());
        data.extend_from_slice(&template_id.tx.to_le_bytes());
        
        response.data = data;
        Ok(response)
    }

    fn get_all_registered_orbitals(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let orbitals_list = self.registered_orbitals_list();
        let orbitals_count = orbitals_list.len();

        // Format: [count (8 bytes)] + [AlkaneId_1 (32 bytes)] + [AlkaneId_2 (32 bytes)] + ...
        let mut data = Vec::new();

        // Add count of registered orbitals (as u64 for compatibility)
        data.extend_from_slice(&(orbitals_count as u64).to_le_bytes());

        // Add each registered orbital AlkaneId
        for orbital in orbitals_list {
            data.extend_from_slice(&orbital.block.to_le_bytes()); // 16 bytes
            data.extend_from_slice(&orbital.tx.to_le_bytes());    // 16 bytes
        }

        response.data = data;
        Ok(response)
    }

    fn is_registered_orbital(&self, orbital_id: AlkaneId) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        let is_registered = self.is_registered_orbital_internal(&orbital_id);
        response.data = (if is_registered { 1u128 } else { 0u128 }).to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_factory_info(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let dust_token_id = self.dust_token_id()?;
        let template_id = self.orbital_token_template_id()?;
        
        // Format: [dust_token_id (32)] + [template_id (32)] + [success_threshold (1)] +
        //         [dust_bonus_rate (16)] + [successful_forges (16)] + [failed_forges (16)]
        // Total: 113 bytes
        let mut data = Vec::with_capacity(113);
        
        // DUST token ID (32 bytes)
        data.extend_from_slice(&dust_token_id.block.to_le_bytes());
        data.extend_from_slice(&dust_token_id.tx.to_le_bytes());
        
        // Template ID (32 bytes)
        data.extend_from_slice(&template_id.block.to_le_bytes());
        data.extend_from_slice(&template_id.tx.to_le_bytes());
        
        // Configuration values
        data.push(self.success_threshold()); // 1 byte
        data.extend_from_slice(&self.dust_bonus_rate().to_le_bytes()); // 16 bytes
        
        // Statistics
        data.extend_from_slice(&self.successful_forges().to_le_bytes()); // 16 bytes
        data.extend_from_slice(&self.failed_forges().to_le_bytes());     // 16 bytes
        
        response.data = data;
        Ok(response)
    }

    fn calculate_base_xor(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        let base_xor = self.calculate_base_xor_internal()?;
        response.data = (base_xor as u128).to_le_bytes().to_vec();
        Ok(response)
    }

    fn calculate_dust_bonus(&self, dust_amount: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        let dust_bonus = self.calculate_dust_bonus_internal(dust_amount)?;
        response.data = (dust_bonus as u128).to_le_bytes().to_vec();
        Ok(response)
    }
}

declare_alkane! {
  impl AlkaneResponder for WandFactory {
    type Message = WandFactoryMessage;
  }
}
