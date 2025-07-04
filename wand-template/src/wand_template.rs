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

use crate::wand_svg::{WandData, WandSvgGenerator};

#[derive(Default)]
pub struct WandTemplate(());

impl AlkaneResponder for WandTemplate {}

#[derive(MessageDispatch)]
enum WandTemplateMessage {
  #[opcode(0)]
  Initialize {
    wand_id: u128,
    final_xor_result: u128,
    base_xor_result: u128,
    dust_bonus: u128,
    alkamist_bonus: u128,
    total_dust: u128,
    total_alkamist: u128,
    block_height: u128,
    uniqueness: u128,
  },

  #[opcode(1000)]
  #[returns(Vec<u8>)]
  GetData,

  #[opcode(1001)]
  #[returns(String)]
  GetContentType,

  #[opcode(1002)]
  #[returns(String)]
  GetAttributes,

  #[opcode(2000)]
  #[returns(String)]
  GetRarity,

  #[opcode(2001)]
  #[returns(String)]
  GetWandType,

  #[opcode(2002)]
  #[returns(u128)]
  GetFinalXor,

  #[opcode(2003)]
  #[returns(u128)]
  GetDustAmount,

  #[opcode(2004)]
  #[returns(u128)]
  GetAlkamistAmount,

  #[opcode(2005)]
  #[returns(String)]
  GetPowerLevel,
}

#[derive(Clone)]
pub struct WandState {
    pub wand_id: u128,
    pub final_xor_result: u8,
    pub base_xor_result: u8,
    pub dust_bonus: u8,
    pub alkamist_bonus: u8,
    pub total_dust: u128,
    pub total_alkamist: u128,
    pub block_height: u32,
    pub uniqueness: u8,
}

impl WandState {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(64);
        
        // Wand ID (16 bytes)
        bytes.extend_from_slice(&self.wand_id.to_le_bytes());
        
        // XOR results and bonuses (4 bytes)
        bytes.push(self.final_xor_result);
        bytes.push(self.base_xor_result);
        bytes.push(self.dust_bonus);
        bytes.push(self.alkamist_bonus);
        
        // Token amounts (32 bytes)
        bytes.extend_from_slice(&self.total_dust.to_le_bytes());
        bytes.extend_from_slice(&self.total_alkamist.to_le_bytes());
        
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
        if bytes.len() < 57 {
            return Err(anyhow!("Invalid wand state length: expected at least 57, got {}", bytes.len()));
        }
        
        let wand_id = u128::from_le_bytes(bytes[0..16].try_into().unwrap());
        let final_xor_result = bytes[16];
        let base_xor_result = bytes[17];
        let dust_bonus = bytes[18];
        let alkamist_bonus = bytes[19];
        let total_dust = u128::from_le_bytes(bytes[20..36].try_into().unwrap());
        let total_alkamist = u128::from_le_bytes(bytes[36..52].try_into().unwrap());
        let block_height = u32::from_le_bytes(bytes[52..56].try_into().unwrap());
        let uniqueness = bytes[56];
        
        Ok(WandState {
            wand_id,
            final_xor_result,
            base_xor_result,
            dust_bonus,
            alkamist_bonus,
            total_dust,
            total_alkamist,
            block_height,
            uniqueness,
        })
    }
}

impl WandTemplate {
  fn initialize(
    &self,
    wand_id: u128,
    final_xor_result: u128,
    base_xor_result: u128,
    dust_bonus: u128,
    alkamist_bonus: u128,
    total_dust: u128,
    total_alkamist: u128,
    block_height: u128,
    uniqueness: u128,
  ) -> Result<CallResponse> {
    self.observe_initialization()?;
    let context = self.context()?;

    // Store the imprinted state
    let wand_state = WandState {
      wand_id,
      final_xor_result: final_xor_result as u8,
      base_xor_result: base_xor_result as u8,
      dust_bonus: dust_bonus as u8,
      alkamist_bonus: alkamist_bonus as u8,
      total_dust,
      total_alkamist,
      block_height: block_height as u32,
      uniqueness: uniqueness as u8,
    };

    self.store_wand_state(&wand_state)?;

    // Return the wand NFT to the caller
    let response = CallResponse::forward(&context.incoming_alkanes);
    Ok(response)
  }

  fn get_data(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    // Read our imprinted state
    let wand_state = self.get_wand_state()?;

    // Create WandData for SVG generation
    let wand_data = WandData {
      wand_id: wand_state.wand_id,
      position_token_id: AlkaneId { block: 2, tx: 35275 }, // Default dust position for display
      txid: bitcoin::Txid::from_raw_hash(bitcoin::hashes::sha256d::Hash::from_byte_array([wand_state.uniqueness; 32])), // Simplified for display
      merkle_root: bitcoin::blockdata::block::TxMerkleNode::from_raw_hash(bitcoin::hashes::sha256d::Hash::from_byte_array([wand_state.base_xor_result; 32])), // Simplified
      base_xor_result: wand_state.base_xor_result,
      dust_bonus: wand_state.dust_bonus,
      final_xor_result: wand_state.final_xor_result,
      dust_amount: wand_state.total_dust,
    };

    // Generate SVG using our imprinted state
    let svg = WandSvgGenerator::generate_svg(wand_data)?;
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

    let wand_state = self.get_wand_state()?;
    let attributes = self.generate_wand_attributes(&wand_state)?;
    response.data = attributes.into_bytes();

    Ok(response)
  }

  fn get_rarity(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let wand_state = self.get_wand_state()?;
    let rarity = self.calculate_rarity(wand_state.final_xor_result);
    response.data = rarity.into_bytes();

    Ok(response)
  }

  fn get_wand_type(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let wand_state = self.get_wand_state()?;
    let wand_type = self.calculate_wand_type(wand_state.final_xor_result);
    response.data = wand_type.into_bytes();

    Ok(response)
  }

  fn get_final_xor(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let wand_state = self.get_wand_state()?;
    response.data = (wand_state.final_xor_result as u128).to_le_bytes().to_vec();

    Ok(response)
  }

  fn get_dust_amount(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let wand_state = self.get_wand_state()?;
    response.data = wand_state.total_dust.to_le_bytes().to_vec();

    Ok(response)
  }

  fn get_alkamist_amount(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let wand_state = self.get_wand_state()?;
    response.data = wand_state.total_alkamist.to_le_bytes().to_vec();

    Ok(response)
  }

  fn get_power_level(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let wand_state = self.get_wand_state()?;
    let power_level = self.calculate_power_level(wand_state.final_xor_result);
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
      _ => "Unknown".to_string(),
    }
  }

  fn calculate_wand_type(&self, final_xor_result: u8) -> String {
    match final_xor_result % 7 {
      0 => "Stellar Wand".to_string(),
      1 => "Nebula Wand".to_string(),
      2 => "Quantum Wand".to_string(),
      3 => "Cosmic Wand".to_string(),
      4 => "Void Wand".to_string(),
      5 => "Plasma Wand".to_string(),
      6 => "Orbital Wand".to_string(),
      _ => "Mystery Wand".to_string(),
    }
  }

  fn calculate_power_level(&self, final_xor_result: u8) -> String {
    match final_xor_result {
      150..=170 => "Apprentice".to_string(),
      171..=190 => "Adept".to_string(),
      191..=210 => "Expert".to_string(),
      211..=230 => "Master".to_string(),
      231..=250 => "Grandmaster".to_string(),
      251..=255 => "Cosmic".to_string(),
      _ => "Unknown".to_string(),
    }
  }

  fn generate_wand_attributes(&self, wand_state: &WandState) -> Result<String> {
    let rarity = self.calculate_rarity(wand_state.final_xor_result);
    let wand_type = self.calculate_wand_type(wand_state.final_xor_result);
    let power_level = self.calculate_power_level(wand_state.final_xor_result);
    
    let description = if wand_state.total_alkamist > 0 && wand_state.total_dust > 0 {
      format!("A magical orbital wand forged from {} Alkamist tokens and {} dust", wand_state.total_alkamist, wand_state.total_dust)
    } else if wand_state.total_alkamist > 0 {
      format!("A magical orbital wand forged from {} Alkamist tokens", wand_state.total_alkamist)
    } else {
      format!("A magical orbital wand forged from {} dust", wand_state.total_dust)
    };
    
    let attributes = format!(r#"{{
  "name": "Orbital Wand #{}",
  "description": "{}",
  "attributes": [
    {{"trait_type": "Wand ID", "value": "{}"}},
    {{"trait_type": "Rarity", "value": "{}"}},
    {{"trait_type": "Power Level", "value": "{}"}},
    {{"trait_type": "Wand Type", "value": "{}"}},
    {{"trait_type": "Dust Amount", "value": "{}"}},
    {{"trait_type": "Alkamist Amount", "value": "{}"}},
    {{"trait_type": "Dust Bonus", "value": "{}"}},
    {{"trait_type": "Alkamist Bonus", "value": "{}"}},
    {{"trait_type": "Base XOR", "value": "{}"}},
    {{"trait_type": "Final XOR", "value": "{}"}},
    {{"trait_type": "Block Height", "value": "{}"}},
    {{"trait_type": "Uniqueness", "value": "{}"}}
  ]
}}"#,
      wand_state.wand_id,
      description,
      wand_state.wand_id,
      rarity,
      power_level,
      wand_type,
      wand_state.total_dust,
      wand_state.total_alkamist,
      wand_state.dust_bonus,
      wand_state.alkamist_bonus,
      wand_state.base_xor_result,
      wand_state.final_xor_result,
      wand_state.block_height,
      wand_state.uniqueness
    );
    
    Ok(attributes)
  }

  // Storage functions
  fn wand_state_pointer(&self) -> StoragePointer {
    StoragePointer::keyword(&StoragePointer::default(), "/wand_state")
  }

  fn store_wand_state(&self, state: &WandState) -> Result<()> {
    let bytes = state.to_bytes();
    let mut pointer = self.wand_state_pointer();
    pointer.set(Arc::new(bytes));
    Ok(())
  }

  fn get_wand_state(&self) -> Result<WandState> {
    let bytes = self.wand_state_pointer().get();
    if bytes.is_empty() {
      return Err(anyhow!("Wand not initialized"));
    }
    WandState::from_bytes(&bytes)
  }
}

declare_alkane! {
  impl AlkaneResponder for WandTemplate {
    type Message = WandTemplateMessage;
  }
}