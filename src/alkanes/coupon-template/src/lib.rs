use metashrew_support::compat::to_arraybuffer_layout;
use metashrew_support::index_pointer::KeyValuePointer;

use alkanes_runtime::{
    declare_alkane, message::MessageDispatch, runtime::AlkaneResponder, token::Token,
    storage::StoragePointer,
};

use alkanes_support::{
    id::AlkaneId,
    parcel::AlkaneTransfer,
    response::CallResponse,
};

use anyhow::Result;
use std::sync::Arc;

mod svg_generator;
use svg_generator::{SvgGenerator, CouponData};

/// Trims a u128 value to a String by removing trailing zeros
pub fn trim(v: u128) -> String {
    let bytes: Vec<u8> = v.to_le_bytes()
        .into_iter()
        .take_while(|&b| b != 0)
        .collect();
    
    // Only attempt UTF-8 conversion if we have valid bytes
    if bytes.is_empty() {
        String::new()
    } else {
        String::from_utf8(bytes).unwrap_or_else(|_| {
            // Simple fallback: just use the numeric value as string
            v.to_string()
        })
    }
}

/// TokenName struct to hold two u128 values for the name
#[derive(Default, Clone, Copy)]
pub struct TokenName {
    pub part1: u128,
    pub part2: u128,
}

impl From<TokenName> for String {
    fn from(name: TokenName) -> Self {
        // Trim both parts and concatenate them
        format!("{}{}", trim(name.part1), trim(name.part2)) 
    }
}

impl TokenName {
    pub fn new(part1: u128, part2: u128) -> Self {
        Self { part1, part2 }
    }
}

/// Returns a StoragePointer for the token name
fn name_pointer() -> StoragePointer {
    StoragePointer::from_keyword("/name")
}

/// Returns a StoragePointer for the token symbol
fn symbol_pointer() -> StoragePointer {
    StoragePointer::from_keyword("/symbol")
}

/// Convert string to u128 for name encoding
fn string_to_u128(s: &str) -> u128 {
    let bytes = s.as_bytes();
    let mut result = 0u128;
    for (i, &byte) in bytes.iter().enumerate() {
        if i >= 16 { break; } // u128 can only hold 16 bytes
        result |= (byte as u128) << (i * 8);
    }
    result
}

#[derive(Default)]
pub struct CouponToken(());

// Manual AlkaneResponder implementation (required before declare_alkane! macro)
impl AlkaneResponder for CouponToken {}

#[derive(MessageDispatch)]
enum CouponTokenMessage {
    #[opcode(0)]
    Initialize {
        coupon_id: u128,
        stake_amount: u128,
        base_xor: u128,
        stake_bonus: u128,
        final_result: u128,
        is_winner: u128,
        creation_block: u128,
        factory_block: u128,
        factory_tx: u128,
    },

    #[opcode(10)]
    #[returns(CallResponse)]
    GetCouponId,

    #[opcode(11)]
    #[returns(CallResponse)]
    GetStakeAmount,

    #[opcode(12)]
    #[returns(CallResponse)]
    GetBaseXor,

    #[opcode(13)]
    #[returns(CallResponse)]
    GetStakeBonus,

    #[opcode(14)]
    #[returns(CallResponse)]
    GetFinalResult,

    #[opcode(15)]
    #[returns(CallResponse)]
    GetCreationBlock,

    #[opcode(16)]
    #[returns(CallResponse)]
    GetFactoryId,

    #[opcode(17)]
    #[returns(CallResponse)]
    GetAllCouponDetails,

    #[opcode(18)]
    #[returns(CallResponse)]
    GetCouponType,

    #[opcode(19)]
    #[returns(CallResponse)]
    IsWinner,

    /// Get the token name
    #[opcode(99)]
    #[returns(CallResponse)]
    GetName,

    /// Get the token symbol
    #[opcode(100)]
    #[returns(CallResponse)]
    GetSymbol,

    /// Get the SVG data
    #[opcode(1000)]
    #[returns(CallResponse)]
    GetData,

    /// Get the content type
    #[opcode(1001)]
    #[returns(CallResponse)]
    GetContentType,

    /// Get the attributes (metadata)
    #[opcode(1002)]
    #[returns(CallResponse)]
    GetAttributes,
}

impl Token for CouponToken {
    fn name(&self) -> String {
        String::from_utf8(name_pointer().get().as_ref().clone())
            .unwrap_or_else(|_| {
                // Safe fallback that doesn't rely on storage that might not be initialized
                let coupon_id = self.coupon_id_pointer().get_value::<u128>();
                let is_winner = self.is_winner_internal();
                format!("{} Coupon #{}", if is_winner { "WINNING" } else { "LOSING" }, coupon_id)
            })
    }

    fn symbol(&self) -> String {
        String::from_utf8(symbol_pointer().get().as_ref().clone())
            .unwrap_or_else(|_| {
                // Safe fallback that doesn't rely on storage that might not be initialized
                let coupon_id = self.coupon_id_pointer().get_value::<u128>();
                let is_winner = self.is_winner_internal();
                format!("{}-{}", if is_winner { "WIN" } else { "LOSE" }, coupon_id)
            })
    }
}

impl CouponToken {
    fn initialize(
        &self,
        coupon_id: u128,
        stake_amount: u128,
        base_xor: u128,
        stake_bonus: u128,
        final_result: u128,
        is_winner: u128,
        creation_block: u128,
        factory_block: u128,
        factory_tx: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::default();

        println!("🚀 COUPON TEMPLATE: Initialize called with coupon_id: {}, stake_amount: {}", coupon_id, stake_amount);
        println!("🚀 COUPON TEMPLATE: Context myself: {:?}", context.myself);

        self.observe_initialization()?;

        // Set basic name and symbol
        let name_string = format!("Coupon #{}", coupon_id);
        let symbol_string = format!("CPN-{}", coupon_id);
        
        name_pointer().set(Arc::new(name_string.as_bytes().to_vec()));
        symbol_pointer().set(Arc::new(symbol_string.as_bytes().to_vec()));

        // Store all coupon details
        self.set_factory_id(&context.caller);
        self.set_coupon_id(coupon_id);
        self.set_stake_amount(stake_amount);
        self.set_base_xor(base_xor as u8);
        self.set_stake_bonus(stake_bonus as u8);
        self.set_final_result(final_result as u8);
        self.set_is_winner(is_winner != 0);
        self.set_creation_block(creation_block);

        // Return exactly 1 coupon token
        println!("🚀 COUPON TEMPLATE: About to return token with id: {:?}, value: 1", context.myself);
        response.alkanes.0.push(AlkaneTransfer {
            id: context.myself.clone(),
            value: 1u128,
        });
        println!("🚀 COUPON TEMPLATE: Response alkanes count after push: {}", response.alkanes.0.len());
        println!("🚀 COUPON TEMPLATE: Response alkanes: {:?}", response.alkanes.0);

        Ok(response)
    }

    fn determine_coupon_type(&self, final_result: u8, is_winner: bool) -> String {
        if is_winner {
            match final_result {
                250..=255 => "JACKPOT",
                230..=249 => "BIG WIN",
                200..=229 => "WIN",
                170..=199 => "SMALL WIN",
                _ => "WIN",
            }
        } else {
            "LOSE"
        }.to_string()
    }

    /// Set the token name and symbol (following free-mint pattern)
    fn set_name_and_symbol(&self, name: TokenName, symbol: u128) {
        let name_string: String = name.into();
        name_pointer()
            .set(Arc::new(name_string.as_bytes().to_vec()));
        self.set_string_field(symbol_pointer(), symbol);
    }

    /// Set a string field in storage (following free-mint pattern)
    fn set_string_field(&self, mut pointer: StoragePointer, v: u128) {
        pointer.set(Arc::new(trim(v).as_bytes().to_vec()));
    }

    // Private getter helper methods for internal use
    fn get_coupon_id_internal(&self) -> u128 {
        self.coupon_id()
    }

    fn get_stake_amount_internal(&self) -> u128 {
        self.stake_amount()
    }

    fn get_base_xor_internal(&self) -> u8 {
        self.base_xor()
    }

    fn get_stake_bonus_internal(&self) -> u8 {
        self.stake_bonus()
    }

    fn get_final_result_internal(&self) -> u8 {
        self.final_result()
    }

    fn get_creation_block_internal(&self) -> u128 {
        self.creation_block()
    }

    fn get_factory_id_internal(&self) -> AlkaneId {
        self.factory_ref()
    }

    fn get_all_coupon_details(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Return core coupon details using internal getters
        let coupon_id = self.get_coupon_id_internal();
        let stake_amount = self.get_stake_amount_internal();
        let base_xor = self.get_base_xor_internal() as u128;
        let stake_bonus = self.get_stake_bonus_internal() as u128;
        let final_result = self.get_final_result_internal() as u128;
        let creation_block = self.get_creation_block_internal();
        let is_winner = if self.is_winner_internal() { 1u128 } else { 0u128 };

        // Pack all values into a single byte array
        // Each value is 16 bytes (128 bits) - 7 values total
        let mut data = Vec::with_capacity(16 * 7);
        data.extend_from_slice(&coupon_id.to_le_bytes());
        data.extend_from_slice(&stake_amount.to_le_bytes());
        data.extend_from_slice(&base_xor.to_le_bytes());
        data.extend_from_slice(&stake_bonus.to_le_bytes());
        data.extend_from_slice(&final_result.to_le_bytes());
        data.extend_from_slice(&creation_block.to_le_bytes());
        data.extend_from_slice(&is_winner.to_le_bytes());

        response.data = data;
        Ok(response)
    }

    fn get_coupon_type(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let coupon_type = self.determine_coupon_type(self.final_result(), self.is_winner_internal());
        response.data = coupon_type.as_bytes().to_vec();

        Ok(response)
    }

    fn is_winner(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let is_winner = if self.is_winner_internal() { 1u128 } else { 0u128 };
        response.data = is_winner.to_le_bytes().to_vec();

        Ok(response)
    }

    // Storage operations using StoragePointer pattern

    fn factory_alkane_id_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/factory-alkane-id")
    }

    fn factory_ref(&self) -> AlkaneId {
        let data = self.factory_alkane_id_pointer().get();
        if data.len() == 0 {
            panic!("Factory reference not found");
        }
        
        let bytes = data.as_ref();
        AlkaneId {
            block: u128::from_le_bytes(bytes[0..16].try_into().unwrap()),
            tx: u128::from_le_bytes(bytes[16..32].try_into().unwrap()),
        }
    }

    fn set_factory_id(&self, id: &AlkaneId) {
        let mut bytes = Vec::with_capacity(32);
        bytes.extend_from_slice(&id.block.to_le_bytes());
        bytes.extend_from_slice(&id.tx.to_le_bytes());
        
        self.factory_alkane_id_pointer().set(Arc::new(bytes));
    }

    fn coupon_id_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/coupon_id")
    }

    fn coupon_id(&self) -> u128 {
        self.coupon_id_pointer().get_value::<u128>()
    }

    fn set_coupon_id(&self, coupon_id: u128) {
        self.coupon_id_pointer().set_value::<u128>(coupon_id);
    }

    // Coupon data storage functions
    fn stake_amount_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/stake_amount")
    }

    fn stake_amount(&self) -> u128 {
        self.stake_amount_pointer().get_value::<u128>()
    }

    fn set_stake_amount(&self, stake_amount: u128) {
        self.stake_amount_pointer().set_value::<u128>(stake_amount);
    }

    fn base_xor_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/base_xor")
    }

    fn base_xor(&self) -> u8 {
        self.base_xor_pointer().get_value::<u8>()
    }

    fn set_base_xor(&self, base_xor: u8) {
        self.base_xor_pointer().set_value::<u8>(base_xor);
    }

    fn stake_bonus_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/stake_bonus")
    }

    fn stake_bonus(&self) -> u8 {
        self.stake_bonus_pointer().get_value::<u8>()
    }

    fn set_stake_bonus(&self, stake_bonus: u8) {
        self.stake_bonus_pointer().set_value::<u8>(stake_bonus);
    }

    fn final_result_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/final_result")
    }

    fn final_result(&self) -> u8 {
        self.final_result_pointer().get_value::<u8>()
    }

    fn set_final_result(&self, final_result: u8) {
        self.final_result_pointer().set_value::<u8>(final_result);
    }

    fn is_winner_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/is_winner")
    }

    fn is_winner_internal(&self) -> bool {
        self.is_winner_pointer().get_value::<u8>() != 0
    }

    fn set_is_winner(&self, is_winner: bool) {
        self.is_winner_pointer().set_value::<u8>(if is_winner { 1 } else { 0 });
    }

    fn creation_block_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/creation_block")
    }

    fn creation_block(&self) -> u128 {
        self.creation_block_pointer().get_value::<u128>()
    }

    fn set_creation_block(&self, creation_block: u128) {
        self.creation_block_pointer().set_value::<u128>(creation_block);
    }


    /// Get the token name (following free-mint pattern)
    fn get_name(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.name().into_bytes().to_vec();

        Ok(response)
    }

    /// Get the token symbol (following free-mint pattern)
    fn get_symbol(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.symbol().into_bytes().to_vec();

        Ok(response)
    }

    /// Get the SVG data for this coupon token
    fn get_data(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Prepare coupon data for SVG generation
        let coupon_data = CouponData {
            coupon_id: self.coupon_id(),
            stake_amount: self.stake_amount(),
            base_xor: self.base_xor(),
            stake_bonus: self.stake_bonus(),
            final_result: self.final_result(),
            creation_block: self.creation_block(),
            current_block: u128::from(self.height()),
            coupon_type: self.determine_coupon_type(self.final_result(), self.is_winner_internal()),
            is_winner: self.is_winner_internal(),
        };

        // Generate the SVG
        let svg = SvgGenerator::generate_svg(coupon_data)?;
        response.data = svg.into_bytes();

        Ok(response)
    }

    /// Get the content type for the SVG
    fn get_content_type(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = String::from("image/svg+xml").into_bytes();

        Ok(response)
    }

    /// Get the attributes (metadata) for this coupon token
    fn get_attributes(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Prepare coupon data for attributes generation
        let coupon_data = CouponData {
            coupon_id: self.coupon_id(),
            stake_amount: self.stake_amount(),
            base_xor: self.base_xor(),
            stake_bonus: self.stake_bonus(),
            final_result: self.final_result(),
            creation_block: self.creation_block(),
            current_block: u128::from(self.height()),
            coupon_type: self.determine_coupon_type(self.final_result(), self.is_winner_internal()),
            is_winner: self.is_winner_internal(),
        };

        // Generate the attributes JSON
        let attributes = SvgGenerator::get_attributes(coupon_data)?;
        response.data = attributes.into_bytes();

        Ok(response)
    }
}



impl CouponToken {
    /// Handle GetCouponId opcode (method name matches enum variant in snake_case)
    fn get_coupon_id(&self) -> Result<CallResponse> {
        println!("🔍 GETTER CALLED: GetCouponId (opcode 10)");
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        let coupon_id = self.get_coupon_id_internal();
        response.data = coupon_id.to_le_bytes().to_vec();
        println!("🔍 GETTER RESULT: Coupon ID = {}", coupon_id);
        Ok(response)
    }
    
    /// Handle GetStakeAmount opcode
    fn get_stake_amount(&self) -> Result<CallResponse> {
        println!("🔍 GETTER CALLED: GetStakeAmount (opcode 11)");
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        let stake_amount = self.get_stake_amount_internal();
        response.data = stake_amount.to_le_bytes().to_vec();
        println!("🔍 GETTER RESULT: Stake Amount = {}", stake_amount);
        Ok(response)
    }
    
    /// Handle GetBaseXor opcode
    fn get_base_xor(&self) -> Result<CallResponse> {
        println!("🔍 GETTER CALLED: GetBaseXor (opcode 12)");
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        let base_xor = self.get_base_xor_internal();
        response.data = (base_xor as u128).to_le_bytes().to_vec();
        println!("🔍 GETTER RESULT: Base XOR = {}", base_xor);
        Ok(response)
    }
    
    /// Handle GetStakeBonus opcode
    fn get_stake_bonus(&self) -> Result<CallResponse> {
        println!("🔍 GETTER CALLED: GetStakeBonus (opcode 13)");
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        let stake_bonus = self.get_stake_bonus_internal();
        response.data = (stake_bonus as u128).to_le_bytes().to_vec();
        println!("🔍 GETTER RESULT: Stake Bonus = {}", stake_bonus);
        Ok(response)
    }
    
    /// Handle GetFinalResult opcode
    fn get_final_result(&self) -> Result<CallResponse> {
        println!("🔍 GETTER CALLED: GetFinalResult (opcode 14)");
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        let final_result = self.get_final_result_internal();
        response.data = (final_result as u128).to_le_bytes().to_vec();
        println!("🔍 GETTER RESULT: Final Result = {}", final_result);
        Ok(response)
    }
    
    /// Handle GetCreationBlock opcode
    fn get_creation_block(&self) -> Result<CallResponse> {
        println!("🔍 GETTER CALLED: GetCreationBlock (opcode 15)");
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        let creation_block = self.get_creation_block_internal();
        response.data = creation_block.to_le_bytes().to_vec();
        println!("🔍 GETTER RESULT: Creation Block = {}", creation_block);
        Ok(response)
    }
    
    /// Handle GetFactoryId opcode
    fn get_factory_id(&self) -> Result<CallResponse> {
        println!("🔍 GETTER CALLED: GetFactoryId (opcode 16)");
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let factory_id = self.get_factory_id_internal();
        
        // Pack AlkaneId into response (32 bytes: 16 for block, 16 for tx)
        let mut data = Vec::with_capacity(32);
        data.extend_from_slice(&factory_id.block.to_le_bytes());
        data.extend_from_slice(&factory_id.tx.to_le_bytes());
        
        response.data = data;
        println!("🔍 GETTER RESULT: Factory ID = {:?}", factory_id);
        Ok(response)
    }
}

declare_alkane! {
  impl AlkaneResponder for CouponToken {
    type Message = CouponTokenMessage;
  }
}
