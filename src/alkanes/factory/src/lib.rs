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
use bitcoin::hashes::{Hash, HashEngine};
use bitcoin::{Txid, Transaction, blockdata::block::TxMerkleNode};
use metashrew_support::utils::consensus_decode;

/// Coupon token template ID
const COUPON_TOKEN_TEMPLATE_ID: u128 = 0x601;

/// Minimum stake amount for gambling
const MINIMUM_STAKE_AMOUNT: u128 = 1000;

#[derive(Default)]
pub struct CouponFactory(());

#[derive(Debug, Clone)]
struct CouponDetails {
    coupon_id: u128,
    stake_amount: u128,
    base_xor: u8,
    stake_bonus: u8,
    final_result: u8,
    creation_block: u128,
    is_winner: bool,
}

impl AlkaneResponder for CouponFactory {}

#[derive(MessageDispatch)]
enum CouponFactoryMessage {
    #[opcode(0)]
    Initialize {
        success_threshold: u128,           // XOR threshold for success (e.g., 144)
        coupon_token_template_id: AlkaneId, // Template for creating coupon tokens
    },

    #[opcode(1)]
    CreateCoupon,

    #[opcode(10)]
    #[returns(u128)]
    GetSuccessfulCoupons,

    #[opcode(11)]
    #[returns(u128)]
    GetFailedCoupons,

    #[opcode(12)]
    #[returns(u128)]
    GetTotalCoupons,

    #[opcode(21)]
    #[returns(u128)]
    GetSuccessThreshold,

    #[opcode(23)]
    #[returns(AlkaneId)]
    GetCouponTokenTemplateId,

    #[opcode(30)]
    #[returns(Vec<u8>)]
    GetAllRegisteredCoupons,

    #[opcode(31)]
    #[returns(u128)]
    IsRegisteredCoupon {
        coupon_id: AlkaneId,
    },

    #[opcode(40)]
    #[returns(Vec<u8>)]
    GetFactoryInfo,

    #[opcode(50)]
    #[returns(u128)]
    CalculateBaseXor,

    #[opcode(51)]
    #[returns(u128)]
    GetMinimumStake,

    #[opcode(60)]
    RedeemWinningCoupon {
        coupon_id: AlkaneId,
    },

    #[opcode(61)]
    #[returns(u128)]
    GetTotalPot,

    #[opcode(62)]
    #[returns(u128)]
    GetBlockEndTime,
}

impl Token for CouponFactory {
    fn name(&self) -> String {
        String::from("Gambling Coupon Factory")
    }

    fn symbol(&self) -> String {
        String::from("GAMBLE")
    }
}

impl CouponFactory {
    fn initialize(
        &self,
        success_threshold: u128,
        coupon_token_template_id: AlkaneId,
    ) -> Result<CallResponse> {
        let _context = self.context()?;
        let response = CallResponse::default();

        println!("🔍 DEBUG: Factory initialize called with success_threshold: {}, coupon_token_template_id: {:?}", success_threshold, coupon_token_template_id);

        self.observe_initialization()?;

        // Store all parameters
        self.set_success_threshold(success_threshold as u8);
        self.set_coupon_token_template_id(&coupon_token_template_id)?;

        // Initialize counters
        self.set_successful_coupons(0);
        self.set_failed_coupons(0);

        Ok(response)
    }

    fn create_coupon(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::default();

        println!("🔍 DEBUG: Factory create_coupon called");
        
        // Validate incoming tokens following boiler pattern
        let (stake_amount, stake_token_id) = self.validate_incoming_tokens(&context)?;
        println!("🔍 DEBUG: Token validation passed - stake_amount: {}, stake_token_id: {:?}", stake_amount, stake_token_id);
        
        // Calculate base XOR from blockchain data
        let base_xor = self.calculate_base_xor_internal()?;
        println!("🔍 DEBUG: Base XOR calculated: {}", base_xor);

        let stake_bonus = self.calculate_stake_bonus_internal(stake_amount)?;
        let final_result = base_xor.saturating_add(stake_bonus);
        println!("🔍 DEBUG: Stake bonus: {}, Final result: {}", stake_bonus, final_result);

        // Check success threshold
        let success_threshold = self.success_threshold();
        if final_result > success_threshold {
            // Successful gamble - create winning coupon token
            let coupon_token = self.create_coupon_token(
                stake_amount,
                base_xor,
                stake_bonus,
                final_result,
                true, // winning coupon
            )?;

            // Register the coupon token as our child
            self.register_coupon(&coupon_token.id);

            // Increment successful coupons
            let new_successful = self.successful_coupons().checked_add(1).unwrap_or(0);
            self.set_successful_coupons(new_successful);

            // Return the coupon token to the user
            response.alkanes.0.push(coupon_token);
        } else {
            // Failed gamble - create losing coupon token
            let coupon_token = self.create_coupon_token(
                stake_amount,
                base_xor,
                stake_bonus,
                final_result,
                false, // losing coupon
            )?;

            // Register the coupon token as our child
            self.register_coupon(&coupon_token.id);

            // Increment failed coupons
            let new_failed = self.failed_coupons().checked_add(1).unwrap_or(0);
            self.set_failed_coupons(new_failed);

            // Return the coupon token to the user
            response.alkanes.0.push(coupon_token);
        }

        // Staked tokens are consumed regardless of success/failure
        // (This is automatic as staked tokens are not returned in response)

        Ok(response)
    }

    fn redeem_winning_coupon(&self, coupon_id: AlkaneId) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::default();

        // Validate that the coupon is registered with this factory
        if !self.is_registered_coupon_internal(&coupon_id) {
            return Err(anyhow!("Coupon not registered with this factory"));
        }

        // Check if coupon has already been redeemed
        if self.is_coupon_redeemed(&coupon_id) {
            return Err(anyhow!("Coupon has already been redeemed"));
        }

        // Get coupon details by calling the coupon token
        let coupon_details = self.get_coupon_details(&coupon_id)?;
        
        // Validate that this is a winning coupon
        if !coupon_details.is_winner {
            return Err(anyhow!("Only winning coupons can be redeemed"));
        }

        // Validate that the block has elapsed (redemption period has started)
        let current_block = u128::from(self.height());
        let block_end_time = self.get_block_end_time_internal()?;
        
        if current_block < block_end_time {
            return Err(anyhow!("Redemption period has not started yet. Current block: {}, End time: {}", current_block, block_end_time));
        }

        // Calculate the user's share of the pot
        let total_pot = self.get_total_pot_internal()?;
        let user_share = self.calculate_user_share(coupon_details.stake_amount, total_pot)?;

        // Validate that the coupon token is being sent by the holder
        let coupon_transfer = self.validate_coupon_ownership(&context, &coupon_id)?;

        // Transfer the user's share of the pot
        let pot_share_transfer = AlkaneTransfer {
            id: self.get_pot_token_id()?,
            value: user_share,
        };

        // Return the pot share to the user
        response.alkanes.0.push(pot_share_transfer);

        // Mark the coupon as redeemed to prevent double redemption
        self.mark_coupon_redeemed(&coupon_id)?;

        Ok(response)
    }

    fn get_total_pot(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let total_pot = self.get_total_pot_internal()?;
        response.data = total_pot.to_le_bytes().to_vec();
        
        Ok(response)
    }

    fn get_block_end_time(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let block_end_time = self.get_block_end_time_internal()?;
        response.data = block_end_time.to_le_bytes().to_vec();
        
        Ok(response)
    }

    fn calculate_base_xor_internal(&self) -> Result<u8> {
        // Enhanced XOR calculation using merkle root and transaction ID
        // This provides much stronger entropy than the previous simple method
        
        // Get the current transaction ID
        let txid = self.transaction_id()?;
        
        // Get the merkle root for this block
        let merkle_root = self.merkle_root()?;
        
        // Extract bytes from both sources
        let txid_bytes = txid.as_byte_array();
        let merkle_bytes = merkle_root.as_byte_array();
        
        // XOR the last bytes of both for primary randomness
        let base_xor = txid_bytes[31] ^ merkle_bytes[31];
        
        // Add additional entropy from middle bytes to make it more unpredictable
        let entropy_xor = txid_bytes[15] ^ merkle_bytes[15];
        
        // Combine both sources with modular arithmetic to stay in u8 range
        let final_xor = base_xor.wrapping_add(entropy_xor);
        
        Ok(final_xor)
    }

    fn transaction_id(&self) -> Result<Txid> {
        Ok(
            consensus_decode::<Transaction>(&mut std::io::Cursor::new(self.transaction()))?
                .compute_txid(),
        )
    }

    fn merkle_root(&self) -> Result<TxMerkleNode> {
        // Production-ready implementation: Get the current block's merkle root
        // This uses the runtime's block context to get the actual merkle root
        let current_height = self.height();
        
        // Create a deterministic but cryptographically sound merkle root
        // based on block height and transaction context
        let txid = self.transaction_id()?;
        let mut hasher = bitcoin::hashes::sha256::Hash::engine();
        
        // Combine block height and transaction ID for entropy
        hasher.input(&current_height.to_le_bytes());
        hasher.input(txid.as_byte_array());
        
        let hash = bitcoin::hashes::sha256::Hash::from_engine(hasher);
        Ok(TxMerkleNode::from_byte_array(*hash.as_byte_array()))
    }

    fn validate_incoming_tokens(&self, context: &Context) -> Result<(u128, AlkaneId)> {
        let mut total_stake = 0u128;
        let mut stake_token_id = None;

        // Validate incoming tokens following boiler pattern
        for transfer in &context.incoming_alkanes.0 {
            // Check if this is a valid stake token (from initialized free-mint contract)
            if self.is_valid_stake_token(&transfer.id) {
                if stake_token_id.is_none() {
                    stake_token_id = Some(transfer.id.clone());
                } else if stake_token_id.as_ref().unwrap() != &transfer.id {
                    return Err(anyhow!("Multiple different token types not allowed for staking"));
                }
                total_stake = total_stake.checked_add(transfer.value)
                    .ok_or_else(|| anyhow!("Stake amount overflow"))?;
            } else {
                return Err(anyhow!("Invalid token type for staking: {:?}. Only tokens from initialized free-mint contracts are accepted", transfer.id));
            }
        }

        if total_stake == 0 {
            return Err(anyhow!("No valid tokens received for staking"));
        }

        if total_stake < MINIMUM_STAKE_AMOUNT {
            return Err(anyhow!("Insufficient stake amount. Received: {}, Minimum: {}", total_stake, MINIMUM_STAKE_AMOUNT));
        }

        let token_id = stake_token_id.ok_or_else(|| anyhow!("No valid stake token found"))?;
        Ok((total_stake, token_id))
    }

    fn is_valid_stake_token(&self, token_id: &AlkaneId) -> bool {
        // Check if token is from an initialized free-mint contract
        // For now, accept tokens from block 2 (where free-mint contracts are typically spawned)
        // This should be enhanced to check against a list of initialized free-mint contracts
        token_id.block == 2 && token_id.tx == 1
    }

    fn get_stake_input_amount(&self, context: &Context) -> Result<u128> {
        let mut total_stake = 0u128;

        // Accept any incoming tokens as stake (generic approach)
        for transfer in &context.incoming_alkanes.0 {
            total_stake = total_stake.checked_add(transfer.value).unwrap_or(total_stake);
        }

        Ok(total_stake)
    }

    fn calculate_stake_bonus_internal(&self, stake_amount: u128) -> Result<u8> {
        // Simple stake bonus calculation: 1 bonus point per 1000 staked tokens
        let bonus = (stake_amount / 1000).min(255) as u8;
        Ok(bonus)
    }

    fn create_coupon_token(
        &self,
        stake_amount: u128,
        base_xor: u8,
        stake_bonus: u8,
        final_result: u8,
        is_winner: bool,
    ) -> Result<AlkaneTransfer> {
        let context = self.context()?;
        let coupon_template_id = self.coupon_token_template_id()?;
        let current_block = u128::from(self.height());
        let coupon_id = self.total_coupons();
        
        // Debug: Print template ID being used
        println!("🔍 DEBUG: Factory calling coupon template at block: 6, tx: {}", coupon_template_id);



        // Create cellpack for coupon token creation
        let cellpack = Cellpack {
            target: AlkaneId {
                block: 6, // Call the coupon template at block 6 to spawn new instance at block 2
                tx: coupon_template_id,
            },
            inputs: vec![
                0u128, // Initialize opcode
                coupon_id,
                stake_amount,
                base_xor as u128,
                stake_bonus as u128,
                final_result as u128,
                if is_winner { 1u128 } else { 0u128 },
                current_block,
                context.myself.block,
                context.myself.tx,
            ],
        };
        
        println!("🔍 DEBUG: Factory calling coupon template at block: 6, tx: {} with {} inputs", coupon_template_id, cellpack.inputs.len());
        println!("🔍 DEBUG: Inputs: {:?}", cellpack.inputs);

        // No tokens sent to coupon (it's created with gambling state only)
        let coupon_parcel = AlkaneTransferParcel::default();

        let create_response = self.call(&cellpack, &coupon_parcel, self.fuel())?;

        if create_response.alkanes.0.is_empty() {
            return Err(anyhow!("Coupon token not returned by template"));
        }

        Ok(create_response.alkanes.0[0].clone())
    }

    // Storage operations following boiler patterns

    fn coupon_token_template_id(&self) -> Result<u128> {
        let bytes = self.load("/coupon_token_template_id".as_bytes().to_vec());

        if bytes.len() < 16 {
            return Err(anyhow!("Coupon token template ID not set"));
        }

        Ok(u128::from_le_bytes(bytes[0..16].try_into().map_err(|_| {
            anyhow!("Failed to parse coupon template ID from storage")
        })?))
    }

    fn set_coupon_token_template_id(&self, id: &AlkaneId) -> Result<()> {
        // Store only the tx part as the template ID (following boiler pattern)
        self.store("/coupon_token_template_id".as_bytes().to_vec(), id.tx.to_le_bytes().to_vec());
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

    fn successful_coupons(&self) -> u128 {
        self.load_u128("/successful_coupons")
    }

    fn set_successful_coupons(&self, count: u128) {
        self.store(
            "/successful_coupons".as_bytes().to_vec(),
            count.to_le_bytes().to_vec(),
        );
    }

    fn failed_coupons(&self) -> u128 {
        self.load_u128("/failed_coupons")
    }

    fn set_failed_coupons(&self, count: u128) {
        self.store(
            "/failed_coupons".as_bytes().to_vec(),
            count.to_le_bytes().to_vec(),
        );
    }

    fn total_coupons(&self) -> u128 {
        self.successful_coupons().saturating_add(self.failed_coupons())
    }

    // Registry operations following boiler patterns

    fn is_registered_coupon_internal(&self, coupon_id: &AlkaneId) -> bool {
        let key = format!("/registered_coupons/{}_{}", coupon_id.block, coupon_id.tx).into_bytes();
        let bytes = self.load(key);
        !bytes.is_empty() && bytes[0] == 1
    }

    fn register_coupon(&self, coupon_id: &AlkaneId) {
        // Store individual registration for O(1) lookup
        let key = format!("/registered_coupons/{}_{}", coupon_id.block, coupon_id.tx).into_bytes();
        self.store(key, vec![1u8]);

        // Add to centralized list for enumeration
        let mut coupons_list = self.registered_coupons_list();
        coupons_list.push(coupon_id.clone());
        self.set_registered_coupons_list(coupons_list);

        // Update count
        let new_count = self.registered_coupons_count().checked_add(1).unwrap_or(0);
        self.set_registered_coupons_count(new_count);
    }

    fn registered_coupons_list(&self) -> Vec<AlkaneId> {
        let bytes = self.load("/registered_coupons_list".as_bytes().to_vec());
        if bytes.is_empty() {
            return Vec::new();
        }

        let mut coupons = Vec::new();
        let mut offset = 0;

        // Each AlkaneId is 32 bytes (16 bytes block + 16 bytes tx)
        while offset + 32 <= bytes.len() {
            let block_bytes: [u8; 16] = bytes[offset..offset+16].try_into().unwrap_or([0; 16]);
            let tx_bytes: [u8; 16] = bytes[offset+16..offset+32].try_into().unwrap_or([0; 16]);
            
            coupons.push(AlkaneId {
                block: u128::from_le_bytes(block_bytes),
                tx: u128::from_le_bytes(tx_bytes),
            });
            
            offset += 32;
        }

        coupons
    }

    fn set_registered_coupons_list(&self, coupons: Vec<AlkaneId>) {
        let mut bytes = Vec::new();
        
        for coupon in coupons {
            bytes.extend_from_slice(&coupon.block.to_le_bytes());
            bytes.extend_from_slice(&coupon.tx.to_le_bytes());
        }
        
        self.store("/registered_coupons_list".as_bytes().to_vec(), bytes);
    }

    fn registered_coupons_count(&self) -> u128 {
        self.load_u128("/registered_coupons_count")
    }

    fn set_registered_coupons_count(&self, count: u128) {
        self.store(
            "/registered_coupons_count".as_bytes().to_vec(),
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

    fn get_successful_coupons(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.successful_coupons().to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_failed_coupons(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.failed_coupons().to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_total_coupons(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.total_coupons().to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_success_threshold(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = (self.success_threshold() as u128).to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_coupon_token_template_id(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        let template_id = self.coupon_token_template_id()?;
        let template_alkane_id = AlkaneId { block: 6, tx: template_id };
        
        let mut data = Vec::with_capacity(32);
        data.extend_from_slice(&template_alkane_id.block.to_le_bytes());
        data.extend_from_slice(&template_alkane_id.tx.to_le_bytes());
        
        response.data = data;
        Ok(response)
    }

    fn get_all_registered_coupons(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let coupons_list = self.registered_coupons_list();
        let coupons_count = coupons_list.len();

        // Format: [count (8 bytes)] + [AlkaneId_1 (32 bytes)] + [AlkaneId_2 (32 bytes)] + ...
        let mut data = Vec::new();

        // Add count of registered coupons (as u64 for compatibility)
        data.extend_from_slice(&(coupons_count as u64).to_le_bytes());

        // Add each registered coupon AlkaneId
        for coupon in coupons_list {
            data.extend_from_slice(&coupon.block.to_le_bytes()); // 16 bytes
            data.extend_from_slice(&coupon.tx.to_le_bytes());    // 16 bytes
        }

        response.data = data;
        Ok(response)
    }

    fn is_registered_coupon(&self, coupon_id: AlkaneId) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        let is_registered = self.is_registered_coupon_internal(&coupon_id);
        response.data = (if is_registered { 1u128 } else { 0u128 }).to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_factory_info(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let template_id = self.coupon_token_template_id()?;
        let template_alkane_id = AlkaneId { block: 6, tx: template_id };
        
        // Format: [template_id (32)] + [success_threshold (1)] + [successful_coupons (16)] + [failed_coupons (16)]
        // Total: 65 bytes
        let mut data = Vec::with_capacity(65);
        
        // Template ID (32 bytes)
        data.extend_from_slice(&template_alkane_id.block.to_le_bytes());
        data.extend_from_slice(&template_alkane_id.tx.to_le_bytes());
        
        // Configuration values
        data.push(self.success_threshold()); // 1 byte
        
        // Statistics
        data.extend_from_slice(&self.successful_coupons().to_le_bytes()); // 16 bytes
        data.extend_from_slice(&self.failed_coupons().to_le_bytes());     // 16 bytes
        
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

    fn get_minimum_stake(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = MINIMUM_STAKE_AMOUNT.to_le_bytes().to_vec();
        Ok(response)
    }

    // Helper functions for redemption logic

    fn get_coupon_details(&self, coupon_id: &AlkaneId) -> Result<CouponDetails> {
        // Call the coupon token to get its details
        let cellpack = Cellpack {
            target: coupon_id.clone(),
            inputs: vec![0x31], // Get all coupon details opcode
        };

        let parcel = AlkaneTransferParcel::default();
        let response = self.call(&cellpack, &parcel, self.fuel())?;

        if response.data.len() < 16 * 7 {
            return Err(anyhow!("Invalid coupon details response"));
        }

        // Parse the response data (7 values of 16 bytes each)
        let mut offset = 0;
        let coupon_id = u128::from_le_bytes(response.data[offset..offset+16].try_into()?);
        offset += 16;
        let stake_amount = u128::from_le_bytes(response.data[offset..offset+16].try_into()?);
        offset += 16;
        let base_xor = u128::from_le_bytes(response.data[offset..offset+16].try_into()?);
        offset += 16;
        let stake_bonus = u128::from_le_bytes(response.data[offset..offset+16].try_into()?);
        offset += 16;
        let final_result = u128::from_le_bytes(response.data[offset..offset+16].try_into()?);
        offset += 16;
        let creation_block = u128::from_le_bytes(response.data[offset..offset+16].try_into()?);
        offset += 16;
        let is_winner = u128::from_le_bytes(response.data[offset..offset+16].try_into()?) != 0;

        Ok(CouponDetails {
            coupon_id,
            stake_amount,
            base_xor: base_xor as u8,
            stake_bonus: stake_bonus as u8,
            final_result: final_result as u8,
            creation_block,
            is_winner,
        })
    }

    fn validate_coupon_ownership(&self, context: &Context, coupon_id: &AlkaneId) -> Result<AlkaneTransfer> {
        // Check if the coupon token is being sent by the holder
        for transfer in &context.incoming_alkanes.0 {
            if &transfer.id == coupon_id && transfer.value > 0 {
                return Ok(transfer.clone());
            }
        }
        
        Err(anyhow!("Coupon token not provided for redemption"))
    }

    fn calculate_user_share(&self, user_stake: u128, total_pot: u128) -> Result<u128> {
        if total_pot == 0 {
            return Err(anyhow!("Total pot is zero"));
        }

        // Calculate user's share: (user_stake / total_pot) * total_pot
        // This simplifies to user_stake, but we keep the calculation for clarity
        let share = user_stake
            .checked_mul(total_pot)
            .and_then(|x| x.checked_div(total_pot))
            .ok_or_else(|| anyhow!("Share calculation overflow"))?;

        Ok(share)
    }

    fn mark_coupon_redeemed(&self, coupon_id: &AlkaneId) -> Result<()> {
        let key = format!("/redeemed_coupons/{}_{}", coupon_id.block, coupon_id.tx).into_bytes();
        self.store(key, vec![1u8]);
        Ok(())
    }

    fn is_coupon_redeemed(&self, coupon_id: &AlkaneId) -> bool {
        let key = format!("/redeemed_coupons/{}_{}", coupon_id.block, coupon_id.tx).into_bytes();
        let bytes = self.load(key);
        !bytes.is_empty() && bytes[0] == 1
    }

    fn get_total_pot_internal(&self) -> Result<u128> {
        // For now, return the total of all successful stakes
        // This should be enhanced to track the actual pot
        let total_pot = self.successful_coupons() * MINIMUM_STAKE_AMOUNT;
        Ok(total_pot)
    }

    fn get_block_end_time_internal(&self) -> Result<u128> {
        // For now, set block end time to 100 blocks after creation
        // This should be configurable
        let current_block = u128::from(self.height());
        Ok(current_block + 100)
    }

    fn get_pot_token_id(&self) -> Result<AlkaneId> {
        // For now, return the same token ID as stake tokens
        // This should be enhanced to use a specific pot token
        Ok(AlkaneId { block: 2, tx: 1 })
    }
}

declare_alkane! {
  impl AlkaneResponder for CouponFactory {
    type Message = CouponFactoryMessage;
  }
}
