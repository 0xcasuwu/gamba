use alkanes_runtime::{
  declare_alkane, message::MessageDispatch, storage::StoragePointer,
  runtime::AlkaneResponder
};

use alkanes_support::{
  id::AlkaneId,
  response::CallResponse
};

use metashrew_support::index_pointer::KeyValuePointer;
use metashrew_support::compat::to_arraybuffer_layout;
use bitcoin::hashes::Hash;

use anyhow::{anyhow, Result};
use std::sync::Arc;

use crate::coupon_svg::{CouponData, CouponSvgGenerator};

#[derive(Default)]
pub struct CouponTemplate(());

impl AlkaneResponder for CouponTemplate {}

#[derive(MessageDispatch)]
enum CouponTemplateMessage {
  #[opcode(0)]
  #[returns(CallResponse)]
  Initialize {
    coupon_id: u128,
    final_xor_result: u128,
    base_xor_result: u128,
    token_bonus: u128,
    total_token_amount: u128,
    block_height: u128,
    uniqueness: u128,
    is_winner: u128,
  },

  #[opcode(1000)]
  #[returns(CallResponse)]
  GetData,

  #[opcode(1001)]
  #[returns(CallResponse)]
  GetContentType,

  #[opcode(1002)]
  #[returns(CallResponse)]
  GetAttributes,

  #[opcode(2000)]
  #[returns(CallResponse)]
  GetRarity,

  #[opcode(2001)]
  #[returns(CallResponse)]
  GetCouponType,

  #[opcode(2002)]
  #[returns(CallResponse)]
  GetFinalXor,

  #[opcode(2003)]
  #[returns(CallResponse)]
  GetTokenAmount,

  #[opcode(2004)]
  #[returns(CallResponse)]
  IsWinner,

  #[opcode(2005)]
  #[returns(CallResponse)]
  GetPowerLevel,
}

#[derive(Clone)]
pub struct CouponState {
    pub coupon_id: u128,
    pub final_xor_result: u8,
    pub base_xor_result: u8,
    pub token_bonus: u8,
    pub total_token_amount: u128,
    pub block_height: u32,
    pub uniqueness: u8,
    pub is_winner: bool,
}

impl CouponState {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(64);
        
        // Coupon ID (16 bytes)
        bytes.extend_from_slice(&self.coupon_id.to_le_bytes());
        
        // XOR results and bonus (3 bytes)
        bytes.push(self.final_xor_result);
        bytes.push(self.base_xor_result);
        bytes.push(self.token_bonus);
        
        // Winner flag (1 byte)
        bytes.push(if self.is_winner { 1 } else { 0 });
        
        // Token amount (16 bytes)
        bytes.extend_from_slice(&self.total_token_amount.to_le_bytes());
        
        // Block height (4 bytes)
        bytes.extend_from_slice(&self.block_height.to_le_bytes());
        
        // Uniqueness (1 byte)
        bytes.push(self.uniqueness);
        
        // Pad to 64 bytes
        while bytes.len() < 64 {
            bytes.push(0);
        }
        
        bytes
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 41 {
            return Err(anyhow!("Invalid coupon state length: expected at least 41, got {}", bytes.len()));
        }
        
        let coupon_id = u128::from_le_bytes(bytes[0..16].try_into().unwrap());
        let final_xor_result = bytes[16];
        let base_xor_result = bytes[17];
        let token_bonus = bytes[18];
        let is_winner = bytes[19] == 1;
        let total_token_amount = u128::from_le_bytes(bytes[20..36].try_into().unwrap());
        let block_height = u32::from_le_bytes(bytes[36..40].try_into().unwrap());
        let uniqueness = bytes[40];
        
        Ok(CouponState {
            coupon_id,
            final_xor_result,
            base_xor_result,
            token_bonus,
            total_token_amount,
            block_height,
            uniqueness,
            is_winner,
        })
    }
}

impl CouponTemplate {
  fn initialize(
    &self,
    coupon_id: u128,
    final_xor_result: u128,
    base_xor_result: u128,
    token_bonus: u128,
    total_token_amount: u128,
    block_height: u128,
    uniqueness: u128,
    is_winner: u128,
  ) -> Result<CallResponse> {
    self.observe_initialization()?;
    let context = self.context()?;

    // Store the imprinted state
    let coupon_state = CouponState {
      coupon_id,
      final_xor_result: final_xor_result as u8,
      base_xor_result: base_xor_result as u8,
      token_bonus: token_bonus as u8,
      total_token_amount,
      block_height: block_height as u32,
      uniqueness: uniqueness as u8,
      is_winner: is_winner != 0,
    };

    self.store_coupon_state(&coupon_state)?;

    // Return the coupon NFT to the caller
    let response = CallResponse::forward(&context.incoming_alkanes);
    Ok(response)
  }

  fn get_data(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    // Read our imprinted state
    let coupon_state = self.get_coupon_state()?;

    // Create CouponData for SVG generation
    let coupon_data = CouponData {
      coupon_id: coupon_state.coupon_id,
      txid: bitcoin::Txid::from_raw_hash(bitcoin::hashes::sha256d::Hash::from_byte_array([coupon_state.uniqueness; 32])), // Simplified for display
      merkle_root: bitcoin::blockdata::block::TxMerkleNode::from_raw_hash(bitcoin::hashes::sha256d::Hash::from_byte_array([coupon_state.base_xor_result; 32])), // Simplified
      base_xor_result: coupon_state.base_xor_result,
      token_bonus: coupon_state.token_bonus,
      final_xor_result: coupon_state.final_xor_result,
      token_amount: coupon_state.total_token_amount,
      is_winner: coupon_state.is_winner,
    };

    // Generate SVG using our imprinted state
    let svg = CouponSvgGenerator::generate_svg(coupon_data)?;
    response.data = svg.into_bytes();

    Ok(response)
  }

  fn get_content_type(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    response.data = b"image/svg+xml".to_vec();
    Ok(response)
  }

  fn get_attributes(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let coupon_state = self.get_coupon_state()?;
    let attributes = self.generate_coupon_attributes(&coupon_state)?;
    response.data = attributes.into_bytes();

    Ok(response)
  }

  fn get_rarity(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let coupon_state = self.get_coupon_state()?;
    let rarity = self.calculate_rarity(coupon_state.final_xor_result);
    response.data = rarity.into_bytes();

    Ok(response)
  }

  fn get_coupon_type(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let coupon_state = self.get_coupon_state()?;
    let coupon_type = self.calculate_coupon_type(coupon_state.final_xor_result);
    response.data = coupon_type.into_bytes();

    Ok(response)
  }

  fn get_final_xor(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let coupon_state = self.get_coupon_state()?;
    response.data = (coupon_state.final_xor_result as u128).to_le_bytes().to_vec();

    Ok(response)
  }

  fn get_token_amount(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let coupon_state = self.get_coupon_state()?;
    response.data = coupon_state.total_token_amount.to_le_bytes().to_vec();

    Ok(response)
  }

  fn is_winner(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let coupon_state = self.get_coupon_state()?;
    response.data = vec![if coupon_state.is_winner { 1 } else { 0 }];

    Ok(response)
  }

  fn get_power_level(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let coupon_state = self.get_coupon_state()?;
    let power_level = self.calculate_power_level(coupon_state.final_xor_result);
    response.data = power_level.into_bytes();

    Ok(response)
  }

  // Helper functions for rarity/type calculation
  fn calculate_rarity(&self, final_xor_result: u8) -> String {
    match final_xor_result {
      150..=170 => "Common".to_string(),
      171..=190 => "Rare".to_string(),
      191..=210 => "Epic".to_string(),
      211..=230 => "Legendary".to_string(),
      231..=250 => "Mythic".to_string(),
      251..=255 => "Cosmic".to_string(),
      _ => "Losing".to_string(), // Below 150 = losing coupon
    }
  }

  fn calculate_coupon_type(&self, final_xor_result: u8) -> String {
    if final_xor_result < 150 {
      return "Losing Ticket".to_string();
    }
    
    match final_xor_result % 7 {
      0 => "Lucky Number".to_string(),
      1 => "Golden Ticket".to_string(),
      2 => "Diamond Draw".to_string(),
      3 => "Platinum Prize".to_string(),
      4 => "Silver Strike".to_string(),
      5 => "Bronze Bonus".to_string(),
      6 => "Jackpot Joy".to_string(),
      _ => "Mystery Prize".to_string(),
    }
  }

  fn calculate_power_level(&self, final_xor_result: u8) -> String {
    match final_xor_result {
      0..=149 => "No Prize".to_string(),
      150..=170 => "Small Win".to_string(),
      171..=190 => "Good Win".to_string(),
      191..=210 => "Great Win".to_string(),
      211..=230 => "Excellent Win".to_string(),
      231..=250 => "Amazing Win".to_string(),
      251..=255 => "Jackpot Win".to_string(),
    }
  }

  fn generate_coupon_attributes(&self, coupon_state: &CouponState) -> Result<String> {
    let rarity = self.calculate_rarity(coupon_state.final_xor_result);
    let coupon_type = self.calculate_coupon_type(coupon_state.final_xor_result);
    let power_level = self.calculate_power_level(coupon_state.final_xor_result);
    
    let status = if coupon_state.is_winner { "Winner" } else { "Loser" };
    let description = format!("A gambling coupon created from {} tokens. Result: {}", coupon_state.total_token_amount, status);
    
    let attributes = format!(r#"{{
  "name": "Gambling Coupon #{}",
  "description": "{}",
  "attributes": [
    {{"trait_type": "Coupon ID", "value": "{}"}},
    {{"trait_type": "Status", "value": "{}"}},
    {{"trait_type": "Rarity", "value": "{}"}},
    {{"trait_type": "Prize Level", "value": "{}"}},
    {{"trait_type": "Coupon Type", "value": "{}"}},
    {{"trait_type": "Token Amount", "value": "{}"}},
    {{"trait_type": "Token Bonus", "value": "{}"}},
    {{"trait_type": "Base XOR", "value": "{}"}},
    {{"trait_type": "Final XOR", "value": "{}"}},
    {{"trait_type": "Block Height", "value": "{}"}},
    {{"trait_type": "Uniqueness", "value": "{}"}}
  ]
}}"#,
      coupon_state.coupon_id,
      description,
      coupon_state.coupon_id,
      status,
      rarity,
      power_level,
      coupon_type,
      coupon_state.total_token_amount,
      coupon_state.token_bonus,
      coupon_state.base_xor_result,
      coupon_state.final_xor_result,
      coupon_state.block_height,
      coupon_state.uniqueness
    );
    
    Ok(attributes)
  }

  // Storage functions
  fn coupon_state_pointer(&self) -> StoragePointer {
    StoragePointer::keyword(&StoragePointer::default(), "/coupon_state")
  }

  fn store_coupon_state(&self, state: &CouponState) -> Result<()> {
    let bytes = state.to_bytes();
    let mut pointer = self.coupon_state_pointer();
    pointer.set(Arc::new(bytes));
    Ok(())
  }

  fn get_coupon_state(&self) -> Result<CouponState> {
    let bytes = self.coupon_state_pointer().get();
    if bytes.is_empty() {
      return Err(anyhow!("Coupon not initialized"));
    }
    CouponState::from_bytes(&bytes)
  }
}

declare_alkane! {
  impl AlkaneResponder for CouponTemplate {
    type Message = CouponTemplateMessage;
  }
}