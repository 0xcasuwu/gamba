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

use anyhow::{anyhow, Result};
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

impl AlkaneResponder for CouponToken {}

#[derive(MessageDispatch)]
enum CouponTokenMessage {
    #[opcode(0)]
    #[returns(CallResponse)]
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
                let is_winner = self.is_winner();
                format!("{} Coupon #{}", if is_winner { "WINNING" } else { "LOSING" }, coupon_id)
            })
    }

    fn symbol(&self) -> String {
        String::from_utf8(symbol_pointer().get().as_ref().clone())
            .unwrap_or_else(|_| {
                // Safe fallback that doesn't rely on storage that might not be initialized
                let coupon_id = self.coupon_id_pointer().get_value::<u128>();
                let is_winner = self.is_winner();
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

        self.observe_initialization()?;

        // Set name and symbol based on coupon properties
        let is_winner_bool = is_winner != 0;
        let coupon_type = if is_winner_bool { "WINNING" } else { "LOSING" };
        let name_string = format!("{} Gambling Coupon #{}", coupon_type, coupon_id);
        let symbol_string = format!("{}-{}", if is_winner_bool { "WIN" } else { "LOSE" }, coupon_id);
        
        name_pointer().set(Arc::new(name_string.as_bytes().to_vec()));
        symbol_pointer().set(Arc::new(symbol_string.as_bytes().to_vec()));

        // Store immutable coupon details
        let factory_id = AlkaneId { block: factory_block, tx: factory_tx };
        self.set_factory_id(&factory_id);
        self.set_coupon_id(coupon_id);
        self.set_stake_amount(stake_amount);
        self.set_base_xor(base_xor as u8);
        self.set_stake_bonus(stake_bonus as u8);
        self.set_final_result(final_result as u8);
        self.set_is_winner(is_winner_bool);
        self.set_creation_block(creation_block);

        // Return exactly 1 coupon token
        response.alkanes.0.push(AlkaneTransfer {
            id: context.myself.clone(),
            value: 1u128,
        });

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

    // Getter functions
    fn get_coupon_id(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.coupon_id().to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_stake_amount(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.stake_amount().to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_base_xor(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = (self.base_xor() as u128).to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_stake_bonus(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = (self.stake_bonus() as u128).to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_final_result(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = (self.final_result() as u128).to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_creation_block(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.creation_block().to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_factory_id(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let factory_id = self.factory_ref();
        
        // Pack AlkaneId into response (32 bytes: 16 for block, 16 for tx)
        let mut data = Vec::with_capacity(32);
        data.extend_from_slice(&factory_id.block.to_le_bytes());
        data.extend_from_slice(&factory_id.tx.to_le_bytes());
        
        response.data = data;
        Ok(response)
    }

    fn get_all_coupon_details(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Return core coupon details
        let coupon_id = self.coupon_id();
        let stake_amount = self.stake_amount();
        let base_xor = self.base_xor() as u128;
        let stake_bonus = self.stake_bonus() as u128;
        let final_result = self.final_result() as u128;
        let creation_block = self.creation_block();
        let is_winner = if self.is_winner() { 1u128 } else { 0u128 };

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

        let coupon_type = self.determine_coupon_type(self.final_result(), self.is_winner());
        response.data = coupon_type.as_bytes().to_vec();

        Ok(response)
    }

    fn is_winner_response(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let is_winner = if self.is_winner() { 1u128 } else { 0u128 };
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

    fn is_winner(&self) -> bool {
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
            coupon_type: self.determine_coupon_type(self.final_result(), self.is_winner()),
            is_winner: self.is_winner(),
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
            coupon_type: self.determine_coupon_type(self.final_result(), self.is_winner()),
            is_winner: self.is_winner(),
        };

        // Generate the attributes JSON
        let attributes = SvgGenerator::get_attributes(coupon_data)?;
        response.data = attributes.into_bytes();

        Ok(response)
    }
}

declare_alkane! {
  impl AlkaneResponder for CouponToken {
    type Message = CouponTokenMessage;
  }
}
