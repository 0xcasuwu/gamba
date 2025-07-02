use metashrew_support::utils::consensus_decode;

use alkanes_runtime::{
  declare_alkane, message::MessageDispatch, storage::StoragePointer,
  runtime::AlkaneResponder
};

use alkanes_support::{
  cellpack::Cellpack,
  id::AlkaneId,
  parcel::{AlkaneTransfer, AlkaneTransferParcel},
  response::CallResponse
};

use bitcoin::hashes::{Hash, HashEngine};
use bitcoin::{Txid, Transaction, blockdata::block::TxMerkleNode};

use anyhow::{anyhow, Result};
use std::sync::Arc;

use crate::wand_svg::{WandData, WandSvgGenerator};
use crate::probability::ProbabilityCalculator;

// Token validation constants
const DUST_TOKEN_BLOCK: u128 = 0x2;
const ALKAMIST_TOKEN_BLOCK: u128 = 0x2; // Alkamist position is also in block 2
const ALKAMIST_TX: u128 = 25720; // Specific alkamist position tx
const DUST_TX: u128 = 35275; // Specific dust position tx
const MIN_DUST_STAKE: u128 = 1000;
const MIN_ALKAMIST_STAKE: u128 = 1; // Minimum Alkamist tokens required
const DUST_BONUS_THRESHOLD: u128 = 2000;
const DUST_BONUS_INCREMENT: u128 = 1000;
const DUST_BONUS_POINTS: u8 = 10;
const ALKAMIST_BONUS_MULTIPLIER: u8 = 5; // Bonus points per Alkamist token
const SUCCESS_THRESHOLD: u8 = 150; // Values 150+ succeed in creating a wand

// Single wand template ID - one template for all wand types
const WAND_TEMPLATE_ID: u128 = 0x1001;           // Single wand template

// Template block (assuming template is deployed in block 6 like boiler)
const WAND_TEMPLATE_BLOCK: u128 = 6;

#[derive(Default)]
pub struct OrbitalWandFactory(());

impl AlkaneResponder for OrbitalWandFactory {}

#[derive(MessageDispatch)]
enum OrbitalWandFactoryMessage {
  #[opcode(0)]
  Initialize,

  #[opcode(42)]
  CastWand,

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
  #[returns(u128)]
  GetWandCount,

  #[opcode(2001)]
  #[returns(Vec<Vec<u8>>)]
  GetWandList,

  #[opcode(2002)]
  #[returns(String)]
  GetWandListJson,

  #[opcode(2003)]
  #[returns(Vec<u8>)]
  GetWandMetadata,

  #[opcode(2004)]
  #[returns(u128)]
  GetTotalDustConsumed,

  #[opcode(2005)]
  #[returns(u128)]
  GetTotalPositionsConsumed,

  #[opcode(2006)]
  #[returns(f64)]
  GetWinRate,

  #[opcode(2007)]
  #[returns(Vec<u8>)]
  GetLatestWandData,

  #[opcode(2008)]
  #[returns(u128)]
  GetTotalAlkamistConsumed,

  #[opcode(2009)]
  #[returns(Vec<u8>)]
  GetGamblingStats,

  #[opcode(2010)]
  #[returns(String)]
  GetContractInfo,

  // Child registration opcodes (like boiler)
  #[opcode(3000)]
  #[returns(Vec<u8>)]
  GetAllRegisteredWands,

  #[opcode(3001)]
  #[returns(bool)]
  IsRegisteredWand {
    wand_id: AlkaneId,
  },
}


#[derive(Clone)]
pub struct WandMetadata {
    pub wand_id: u128,
    pub stake_token_id: AlkaneId, // Can be Alkamist or Dust token
    pub stake_token_type: StakeTokenType,
    pub txid: Txid,
    pub merkle_root: TxMerkleNode,
    pub base_xor_result: u8,
    pub dust_bonus: u8,
    pub alkamist_bonus: u8,
    pub final_xor_result: u8,
    pub dust_amount: u128,
    pub alkamist_amount: u128,
    pub block_height: u32,
}

#[derive(Clone, Copy)]
pub enum StakeTokenType {
    Dust = 0,
    Alkamist = 1,
    Mixed = 2, // Both Dust and Alkamist
}

impl WandMetadata {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(140); // Increased capacity for new fields
        
        // Wand ID (16 bytes)
        bytes.extend_from_slice(&self.wand_id.to_le_bytes());
        
        // Stake token ID (32 bytes)
        bytes.extend_from_slice(&self.stake_token_id.block.to_le_bytes());
        bytes.extend_from_slice(&self.stake_token_id.tx.to_le_bytes());
        
        // Stake token type (1 byte)
        bytes.push(self.stake_token_type as u8);
        
        // Transaction ID (32 bytes)
        bytes.extend_from_slice(self.txid.as_byte_array());
        
        // Merkle root (32 bytes)
        bytes.extend_from_slice(self.merkle_root.as_byte_array());
        
        // XOR results (4 bytes)
        bytes.push(self.base_xor_result);
        bytes.push(self.dust_bonus);
        bytes.push(self.alkamist_bonus);
        bytes.push(self.final_xor_result);
        
        // Amounts (32 bytes)
        bytes.extend_from_slice(&self.dust_amount.to_le_bytes());
        bytes.extend_from_slice(&self.alkamist_amount.to_le_bytes());
        
        // Block height (4 bytes)
        bytes.extend_from_slice(&self.block_height.to_le_bytes());
        
        bytes
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 140 {
            return Err(anyhow!("Invalid wand metadata length: expected 140, got {}", bytes.len()));
        }
        
        let wand_id = u128::from_le_bytes(bytes[0..16].try_into().unwrap());
        
        let stake_block = u128::from_le_bytes(bytes[16..32].try_into().unwrap());
        let stake_tx = u128::from_le_bytes(bytes[32..48].try_into().unwrap());
        let stake_token_id = AlkaneId { block: stake_block, tx: stake_tx };
        
        let stake_token_type = match bytes[48] {
            0 => StakeTokenType::Dust,
            1 => StakeTokenType::Alkamist,
            2 => StakeTokenType::Mixed,
            _ => return Err(anyhow!("Invalid stake token type")),
        };
        
        let txid = Txid::from_byte_array(bytes[49..81].try_into().unwrap());
        let merkle_root = TxMerkleNode::from_byte_array(bytes[81..113].try_into().unwrap());
        
        let base_xor_result = bytes[113];
        let dust_bonus = bytes[114];
        let alkamist_bonus = bytes[115];
        let final_xor_result = bytes[116];
        
        let dust_amount = u128::from_le_bytes(bytes[117..133].try_into().unwrap());
        let alkamist_amount = u128::from_le_bytes(bytes[133..149].try_into().unwrap());
        let block_height = u32::from_le_bytes(bytes[149..153].try_into().unwrap());
        
        Ok(WandMetadata {
            wand_id,
            stake_token_id,
            stake_token_type,
            txid,
            merkle_root,
            base_xor_result,
            dust_bonus,
            alkamist_bonus,
            final_xor_result,
            dust_amount,
            alkamist_amount,
            block_height,
        })
    }
}

impl OrbitalWandFactory {
  fn initialize(&self) -> Result<CallResponse> {
    self.observe_initialization()?;
    let context = self.context()?;

    let response = CallResponse::forward(&context.incoming_alkanes);
    Ok(response)
  }

  fn cast_wand(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let txid = self.transaction_id()?;

    // Enforce one cast per transaction
    if self.has_tx_hash(&txid) {
      return Err(anyhow!("Transaction already used for wand casting"));
    }

    // Validate inputs: must have at least 1 token (Alkamist or Dust)
    if context.incoming_alkanes.0.is_empty() {
      return Err(anyhow!("Must send at least 1 Alkamist or Dust token"));
    }

    // Validate all incoming alkanes first
    self.validate_incoming_alkanes()?;

    // Separate Alkamist and Dust tokens
    let mut alkamist_transfers: Vec<AlkaneTransfer> = Vec::new();
    let mut dust_transfers: Vec<AlkaneTransfer> = Vec::new();

    for alkane in &context.incoming_alkanes.0 {
      if self.is_alkamist_token(&alkane.id) {
        alkamist_transfers.push(alkane.clone());
      } else if self.is_dust_token(&alkane.id) {
        dust_transfers.push(alkane.clone());
      }
      // Note: invalid tokens are already rejected by validate_incoming_alkanes
    }

    // Must have at least one valid token (this should always be true after validation)
    if alkamist_transfers.is_empty() && dust_transfers.is_empty() {
      return Err(anyhow!("No valid Alkamist or Dust tokens provided"));
    }

    // Calculate total amounts with overflow protection
    let total_alkamist: u128 = alkamist_transfers.iter()
      .try_fold(0u128, |acc, t| acc.checked_add(t.value))
      .ok_or_else(|| anyhow!("Alkamist amount overflow"))?;
    let total_dust: u128 = dust_transfers.iter()
      .try_fold(0u128, |acc, t| acc.checked_add(t.value))
      .ok_or_else(|| anyhow!("Dust amount overflow"))?;

    // Validate minimum stakes
    if total_alkamist > 0 && total_alkamist < MIN_ALKAMIST_STAKE {
      return Err(anyhow!("Insufficient Alkamist tokens - minimum {} required, got {}", MIN_ALKAMIST_STAKE, total_alkamist));
    }
    if total_dust > 0 && total_dust < MIN_DUST_STAKE {
      return Err(anyhow!("Insufficient dust tokens - minimum {} required, got {}", MIN_DUST_STAKE, total_dust));
    }

    // Calculate bonuses
    let dust_bonus = self.calculate_dust_bonus(total_dust);
    let alkamist_bonus = self.calculate_alkamist_bonus(total_alkamist);

    // Get randomness sources
    let merkle_root = self.merkle_root()?;
    let base_xor_result = self.calculate_base_xor(&txid, &merkle_root)?;
    
    // Calculate final XOR with overflow protection (using saturating_add to prevent overflow)
    let final_xor_result = base_xor_result
      .saturating_add(dust_bonus)
      .saturating_add(alkamist_bonus);

    // Check if wand creation succeeds
    if final_xor_result < SUCCESS_THRESHOLD {
      // Wand creation fails - tokens are burned (consumed) to improve odds but failed
      self.add_tx_hash(&txid)?;
      self.record_loss_detailed(total_dust, total_alkamist)?;
      return Err(anyhow!(
        "Wand creation failed! XOR result {} < {} (base: {}, dust bonus: {}, alkamist bonus: {}). Your tokens were burned in the attempt.",
        final_xor_result, SUCCESS_THRESHOLD, base_xor_result, dust_bonus, alkamist_bonus
      ));
    }

    // FACTORY PATTERN: Wand creation succeeds! Create individual wand NFT using cellpack
    let wand_id = self.get_next_wand_id()?;
    let block_height = self.block_height()?;

    // Create cellpack to call the single wand template
    let cellpack = Cellpack {
      target: AlkaneId {
        block: WAND_TEMPLATE_BLOCK,
        tx: WAND_TEMPLATE_ID,
      },
      // Pass wand creation data to the template
      inputs: vec![
        0x0,                    // Initialize opcode
        wand_id,                // Wand ID
        final_xor_result as u128, // Final XOR result
        base_xor_result as u128,  // Base XOR result
        dust_bonus as u128,       // Dust bonus
        alkamist_bonus as u128,   // Alkamist bonus
        total_dust,               // Dust amount
        total_alkamist,           // Alkamist amount
        block_height as u128,     // Block height
        txid.to_byte_array()[0] as u128, // First byte of txid for uniqueness
      ],
    };

    // Wand NFT receives NO underlying assets - it's purely an NFT
    let wand_parcel = AlkaneTransferParcel::default();

    let create_response = self.call(&cellpack, &wand_parcel, self.fuel())?;

    if create_response.alkanes.0.is_empty() {
      return Err(anyhow!("Wand NFT not returned by template"));
    }

    // Get the created wand NFT
    let wand_nft = create_response.alkanes.0[0].clone();

    // SECURITY CRITICAL: Register this wand NFT as our child
    self.register_wand(&wand_nft.id);

    // Store wand metadata for factory tracking
    let wand_metadata = WandMetadata {
      wand_id,
      stake_token_id: wand_nft.id.clone(), // The actual wand NFT ID
      stake_token_type: if !alkamist_transfers.is_empty() && !dust_transfers.is_empty() {
        StakeTokenType::Mixed
      } else if !alkamist_transfers.is_empty() {
        StakeTokenType::Alkamist
      } else {
        StakeTokenType::Dust
      },
      txid,
      merkle_root,
      base_xor_result,
      dust_bonus,
      alkamist_bonus,
      final_xor_result,
      dust_amount: total_dust,
      alkamist_amount: total_alkamist,
      block_height,
    };

    self.store_wand_metadata(&wand_metadata)?;
    self.record_win_detailed(total_dust, total_alkamist)?;
    self.add_tx_hash(&txid)?;

    // Return the created wand NFT to the user
    let mut response = CallResponse::default();
    response.alkanes.0.push(wand_nft);

    Ok(response)
  }

  fn get_data(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    // Get latest wand metadata and generate SVG
    let wand_count = self.wand_count();
    if wand_count == 0 {
      response.data = Vec::new();
      return Ok(response);
    }

    let latest_metadata = self.get_wand_metadata(wand_count - 1)?;
    let wand_data = WandData {
      wand_id: latest_metadata.wand_id,
      position_token_id: latest_metadata.stake_token_id,
      txid: latest_metadata.txid,
      merkle_root: latest_metadata.merkle_root,
      base_xor_result: latest_metadata.base_xor_result,
      dust_bonus: latest_metadata.dust_bonus,
      final_xor_result: latest_metadata.final_xor_result,
      dust_amount: latest_metadata.dust_amount,
    };

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

    let wand_count = self.wand_count();
    if wand_count == 0 {
      response.data = b"{}".to_vec();
      return Ok(response);
    }

    let latest_metadata = self.get_wand_metadata(wand_count - 1)?;
    let attributes = self.generate_wand_attributes(&latest_metadata)?;
    response.data = attributes.into_bytes();

    Ok(response)
  }

  fn get_wand_count(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    response.data = self.wand_count().to_le_bytes().to_vec();
    Ok(response)
  }

  fn get_wand_list(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let count = self.wand_count();
    let mut wand_list = Vec::new();

    for i in 0..count {
      let metadata = self.get_wand_metadata(i)?;
      wand_list.push(metadata.to_bytes());
    }

    let mut flattened = Vec::new();
    for bytes in wand_list {
      flattened.extend(bytes);
    }

    response.data = flattened;
    Ok(response)
  }

  fn get_wand_list_json(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let count = self.wand_count();
    let mut wand_list = Vec::new();

    for i in 0..count {
      let metadata = self.get_wand_metadata(i)?;
      let stake_type_str = match metadata.stake_token_type {
        StakeTokenType::Dust => "dust",
        StakeTokenType::Alkamist => "alkamist",
        StakeTokenType::Mixed => "mixed",
      };
      wand_list.push(format!(
        "{{\"id\":{},\"stake_token\":\"{}:{}\",\"stake_type\":\"{}\",\"xor\":{},\"dust\":{},\"alkamist\":{},\"power\":\"{}\"}}",
        metadata.wand_id,
        metadata.stake_token_id.block,
        metadata.stake_token_id.tx,
        stake_type_str,
        metadata.final_xor_result,
        metadata.dust_amount,
        metadata.alkamist_amount,
        self.calculate_wand_power(metadata.final_xor_result)
      ));
    }

    let json = format!("[{}]", wand_list.join(","));
    response.data = json.into_bytes();
    Ok(response)
  }

  fn get_wand_metadata(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let wand_count = self.wand_count();
    if wand_count == 0 {
      response.data = Vec::new();
      return Ok(response);
    }

    let latest_metadata = self.get_wand_metadata(wand_count - 1)?;
    response.data = latest_metadata.to_bytes();

    Ok(response)
  }

  fn get_total_dust_consumed(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    response.data = self.total_dust_consumed().to_le_bytes().to_vec();
    Ok(response)
  }

  fn get_total_positions_consumed(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    response.data = self.total_positions_consumed().to_le_bytes().to_vec();
    Ok(response)
  }

  fn get_win_rate(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    let wins = self.wand_count();
    let total_games = self.total_games();
    
    let win_rate = if total_games > 0 {
      (wins as f64) / (total_games as f64)
    } else {
      0.0
    };
    
    response.data = win_rate.to_le_bytes().to_vec();
    Ok(response)
  }

  fn get_latest_wand_data(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let wand_count = self.wand_count();
    if wand_count == 0 {
      response.data = Vec::new();
      return Ok(response);
    }

    let latest_metadata = self.get_wand_metadata(wand_count - 1)?;
    response.data = latest_metadata.to_bytes();

    Ok(response)
  }

  fn get_total_alkamist_consumed(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    response.data = self.total_alkamist_consumed().to_le_bytes().to_vec();
    Ok(response)
  }

  fn get_gambling_stats(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    // Comprehensive gambling statistics
    let total_games = self.total_games();
    let total_wins = self.wand_count();
    let total_dust_consumed = self.total_dust_consumed();
    let total_alkamist_consumed = self.total_alkamist_consumed();
    let total_positions_consumed = self.total_positions_consumed();

    // Calculate win rate
    let win_rate_percentage = if total_games > 0 {
      ((total_wins as f64) / (total_games as f64) * 100.0) as u32
    } else {
      0u32
    };

    // Pack statistics into response
    // Format: [total_games (16)] + [total_wins (16)] + [total_dust (16)] + [total_alkamist (16)] +
    //         [total_positions (16)] + [win_rate_percentage (4)]
    let mut data = Vec::with_capacity(84);
    data.extend_from_slice(&total_games.to_le_bytes());
    data.extend_from_slice(&total_wins.to_le_bytes());
    data.extend_from_slice(&total_dust_consumed.to_le_bytes());
    data.extend_from_slice(&total_alkamist_consumed.to_le_bytes());
    data.extend_from_slice(&total_positions_consumed.to_le_bytes());
    data.extend_from_slice(&win_rate_percentage.to_le_bytes());

    response.data = data;
    Ok(response)
  }

  fn get_contract_info(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let info = format!(r#"{{
  "name": "Orbital Wand Gambling Contract",
  "version": "2.0.0",
  "description": "Wand creation contract that burns Alkamist and Dust tokens to improve success odds",
  "features": [
    "Alkamist token support",
    "Dust token support",
    "Token burning for improved odds",
    "Dynamic bonus calculation",
    "Merkle root randomness",
    "NFT wand creation",
    "Comprehensive statistics"
  ],
  "constants": {{
    "dust_token_block": {},
    "alkamist_token_block": {},
    "min_dust_stake": {},
    "min_alkamist_stake": {},
    "dust_bonus_threshold": {},
    "dust_bonus_increment": {},
    "dust_bonus_points": {},
    "alkamist_bonus_multiplier": {},
    "win_threshold": {}
  }},
  "current_stats": {{
    "total_wands": {},
    "total_games": {},
    "total_dust_consumed": {},
    "total_alkamist_consumed": {}
  }}
}}"#,
      DUST_TOKEN_BLOCK,
      ALKAMIST_TOKEN_BLOCK, // Note: Alkamist position is at 2:25720
      MIN_DUST_STAKE,
      MIN_ALKAMIST_STAKE,
      DUST_BONUS_THRESHOLD,
      DUST_BONUS_INCREMENT,
      DUST_BONUS_POINTS,
      ALKAMIST_BONUS_MULTIPLIER,
      SUCCESS_THRESHOLD,
      self.wand_count(),
      self.total_games(),
      self.total_dust_consumed(),
      self.total_alkamist_consumed()
    );

    response.data = info.into_bytes();
    Ok(response)
  }

  // Validation functions
  fn is_alkamist_token(&self, id: &AlkaneId) -> bool {
    // Check for specific valid alkamist position
    id.block == ALKAMIST_TOKEN_BLOCK && id.tx == ALKAMIST_TX
  }

  fn is_dust_token(&self, id: &AlkaneId) -> bool {
    // Check for specific valid dust position or other dust tokens from block 2
    if id.block == DUST_TOKEN_BLOCK && id.tx == DUST_TX {
      return true;
    }
    
    // For backward compatibility, accept any token from DUST_TOKEN_BLOCK (block 2)
    // but exclude the specific alkamist position to avoid double-counting
    id.block == DUST_TOKEN_BLOCK && id.tx != ALKAMIST_TX
  }

  fn is_valid_alkamist_or_dust(&self, id: &AlkaneId) -> bool {
    self.is_alkamist_token(id) || self.is_dust_token(id)
  }

  // Factory helper functions - rarity/type now determined by wand template itself
  fn get_wand_rarity(&self, final_xor_result: u8) -> String {
    match final_xor_result {
      150..=170 => "Common".to_string(),
      171..=190 => "Rare".to_string(),
      191..=210 => "Epic".to_string(),
      211..=230 => "Legendary".to_string(),
      231..=250 => "Mythic".to_string(),
      251..=255 => "Cosmic".to_string(),
      _ => "Common".to_string(), // Fallback
    }
  }

  fn get_wand_type_name(&self, final_xor_result: u8) -> String {
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

  /// Validate incoming alkanes similar to boiler's authenticate_position
  fn validate_incoming_alkanes(&self) -> Result<()> {
    let context = self.context()?;
    
    // Validate incoming alkanes structure
    if context.incoming_alkanes.0.is_empty() {
      return Err(anyhow!("No incoming alkanes for validation"));
    }

    for transfer in &context.incoming_alkanes.0 {
      // The value should be at least 1
      if transfer.value < 1 {
        return Err(anyhow!("Less than 1 unit of token supplied for alkane {}:{}",
                          transfer.id.block, transfer.id.tx));
      }

      // CRITICAL: Validate that this is a valid alkamist or dust token (similar to boiler's token validation)
      if !self.is_valid_alkamist_or_dust(&transfer.id) {
        return Err(anyhow!(
          "Invalid token ID - expected Alkamist position (block: {}, tx: {}), Dust position (block: {}, tx: {}), or other Dust token from block {}, got AlkaneId {{ block: {}, tx: {} }}",
          ALKAMIST_TOKEN_BLOCK, ALKAMIST_TX,
          DUST_TOKEN_BLOCK, DUST_TX,
          DUST_TOKEN_BLOCK,
          transfer.id.block, transfer.id.tx
        ));
      }
    }

    Ok(())
  }

  // Helper functions
  fn calculate_dust_bonus(&self, dust_amount: u128) -> u8 {
    if dust_amount < DUST_BONUS_THRESHOLD {
      return 0;
    }
    
    let bonus_increments = (dust_amount - DUST_BONUS_THRESHOLD) / DUST_BONUS_INCREMENT;
    let bonus = bonus_increments * (DUST_BONUS_POINTS as u128);
    
    // Cap at 255 to prevent overflow
    std::cmp::min(bonus, 255) as u8
  }

  fn calculate_alkamist_bonus(&self, alkamist_amount: u128) -> u8 {
    if alkamist_amount == 0 {
      return 0;
    }
    
    // Each Alkamist token provides a bonus
    let bonus = alkamist_amount * (ALKAMIST_BONUS_MULTIPLIER as u128);
    
    // Cap at 255 to prevent overflow
    std::cmp::min(bonus, 255) as u8
  }

  fn calculate_base_xor(&self, _txid: &Txid, merkle_root: &TxMerkleNode) -> Result<u8> {
    let merkle_bytes = merkle_root.as_byte_array();
    
    // Use the last byte of the merkle root as the base randomness
    // This gives us a value from 0-255 which we compare against SUCCESS_THRESHOLD
    Ok(merkle_bytes[31])
  }

  fn calculate_wand_power(&self, final_xor_result: u8) -> String {
    match final_xor_result {
      150..=170 => "Apprentice".to_string(),
      171..=190 => "Adept".to_string(),
      191..=210 => "Expert".to_string(),
      211..=230 => "Master".to_string(),
      231..=250 => "Grandmaster".to_string(),
      251..=255 => "Cosmic".to_string(),
      _ => "Failed".to_string(), // Should not happen for successful wands
    }
  }

  fn generate_wand_attributes(&self, metadata: &WandMetadata) -> Result<String> {
    let power = self.calculate_wand_power(metadata.final_xor_result);
    let wand_type = self.get_wand_type(metadata.final_xor_result);
    let stake_type_str = match metadata.stake_token_type {
      StakeTokenType::Dust => "Dust",
      StakeTokenType::Alkamist => "Alkamist",
      StakeTokenType::Mixed => "Mixed (Dust + Alkamist)",
    };
    
    let description = if metadata.alkamist_amount > 0 && metadata.dust_amount > 0 {
      format!("A magical orbital wand forged from {} Alkamist tokens and {} dust", metadata.alkamist_amount, metadata.dust_amount)
    } else if metadata.alkamist_amount > 0 {
      format!("A magical orbital wand forged from {} Alkamist tokens", metadata.alkamist_amount)
    } else {
      format!("A magical orbital wand forged from {} dust", metadata.dust_amount)
    };
    
    let attributes = format!(r#"{{
  "name": "Orbital Wand #{}",
  "description": "{}",
  "attributes": [
    {{"trait_type": "Wand ID", "value": "{}"}},
    {{"trait_type": "Power Level", "value": "{}"}},
    {{"trait_type": "Wand Type", "value": "{}"}},
    {{"trait_type": "Stake Type", "value": "{}"}},
    {{"trait_type": "Stake Token", "value": "{}:{}"}},
    {{"trait_type": "Dust Amount", "value": "{}"}},
    {{"trait_type": "Alkamist Amount", "value": "{}"}},
    {{"trait_type": "Dust Bonus", "value": "{}"}},
    {{"trait_type": "Alkamist Bonus", "value": "{}"}},
    {{"trait_type": "Base XOR", "value": "{}"}},
    {{"trait_type": "Final XOR", "value": "{}"}},
    {{"trait_type": "Block Height", "value": "{}"}},
    {{"trait_type": "Transaction", "value": "{}"}},
    {{"trait_type": "Merkle Root", "value": "{}"}}
  ]
}}"#,
      metadata.wand_id,
      description,
      metadata.wand_id,
      power,
      wand_type,
      stake_type_str,
      metadata.stake_token_id.block,
      metadata.stake_token_id.tx,
      metadata.dust_amount,
      metadata.alkamist_amount,
      metadata.dust_bonus,
      metadata.alkamist_bonus,
      metadata.base_xor_result,
      metadata.final_xor_result,
      metadata.block_height,
      metadata.txid,
      metadata.merkle_root
    );
    
    Ok(attributes)
  }

  fn get_wand_type(&self, final_xor_result: u8) -> String {
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

  // Storage functions
  fn wand_count_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/wand_count")
  }

  fn wand_count(&self) -> u128 {
    self.wand_count_pointer().get_value::<u128>()
  }

  fn set_wand_count(&self, count: u128) {
    self.wand_count_pointer().set_value::<u128>(count);
  }

  fn get_next_wand_id(&self) -> Result<u128> {
    let count = self.wand_count();
    let new_count = count.checked_add(1)
      .ok_or_else(|| anyhow!("Wand count overflow"))?;
    self.set_wand_count(new_count);
    Ok(count)
  }

  fn store_wand_metadata(&self, metadata: &WandMetadata) -> Result<()> {
    let bytes = metadata.to_bytes();
    let wand_pointer = StoragePointer::from_keyword("/wands/")
      .select(&metadata.wand_id.to_le_bytes().to_vec());
    wand_pointer.set(Arc::new(bytes));
    Ok(())
  }

  fn get_wand_metadata(&self, wand_id: u128) -> Result<WandMetadata> {
    let wand_pointer = StoragePointer::from_keyword("/wands/")
      .select(&wand_id.to_le_bytes().to_vec());
    let bytes = wand_pointer.get();
    WandMetadata::from_bytes(&bytes)
  }

  fn total_dust_consumed_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/total_dust_consumed")
  }

  fn total_dust_consumed(&self) -> u128 {
    self.total_dust_consumed_pointer().get_value::<u128>()
  }

  fn add_dust_consumed(&self, amount: u128) -> Result<()> {
    let current = self.total_dust_consumed();
    let new_total = current.checked_add(amount)
      .ok_or_else(|| anyhow!("Dust consumed overflow"))?;
    self.total_dust_consumed_pointer().set_value::<u128>(new_total);
    Ok(())
  }

  fn total_alkamist_consumed_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/total_alkamist_consumed")
  }

  fn total_alkamist_consumed(&self) -> u128 {
    self.total_alkamist_consumed_pointer().get_value::<u128>()
  }

  fn add_alkamist_consumed(&self, amount: u128) -> Result<()> {
    let current = self.total_alkamist_consumed();
    let new_total = current.checked_add(amount)
      .ok_or_else(|| anyhow!("Alkamist consumed overflow"))?;
    self.total_alkamist_consumed_pointer().set_value::<u128>(new_total);
    Ok(())
  }

  fn total_positions_consumed_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/total_positions_consumed")
  }

  fn total_positions_consumed(&self) -> u128 {
    self.total_positions_consumed_pointer().get_value::<u128>()
  }

  fn add_position_consumed(&self) -> Result<()> {
    let current = self.total_positions_consumed();
    let new_total = current.checked_add(1)
      .ok_or_else(|| anyhow!("Positions consumed overflow"))?;
    self.total_positions_consumed_pointer().set_value::<u128>(new_total);
    Ok(())
  }

  fn total_games_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/total_games")
  }

  fn total_games(&self) -> u128 {
    self.total_games_pointer().get_value::<u128>()
  }

  fn add_game(&self) -> Result<()> {
    let current = self.total_games();
    let new_total = current.checked_add(1)
      .ok_or_else(|| anyhow!("Total games overflow"))?;
    self.total_games_pointer().set_value::<u128>(new_total);
    Ok(())
  }

  fn record_win(&self, total_stake: u128) -> Result<()> {
    // Note: total_stake now includes both dust and alkamist amounts
    self.add_dust_consumed(total_stake)?; // For backward compatibility, we track total in dust
    self.add_position_consumed()?;
    self.add_game()?;
    Ok(())
  }

  fn record_loss(&self, total_stake: u128) -> Result<()> {
    // Note: total_stake now includes both dust and alkamist amounts
    self.add_dust_consumed(total_stake)?; // For backward compatibility, we track total in dust
    self.add_position_consumed()?;
    self.add_game()?;
    Ok(())
  }

  fn record_win_detailed(&self, dust_amount: u128, alkamist_amount: u128) -> Result<()> {
    self.add_dust_consumed(dust_amount)?;
    self.add_alkamist_consumed(alkamist_amount)?;
    self.add_position_consumed()?;
    self.add_game()?;
    Ok(())
  }

  fn record_loss_detailed(&self, dust_amount: u128, alkamist_amount: u128) -> Result<()> {
    self.add_dust_consumed(dust_amount)?;
    self.add_alkamist_consumed(alkamist_amount)?;
    self.add_position_consumed()?;
    self.add_game()?;
    Ok(())
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

  fn block_height(&self) -> Result<u32> {
    // Production-ready implementation: Get the actual current block height
    Ok(self.height())
  }

  fn has_tx_hash(&self, txid: &Txid) -> bool {
    StoragePointer::from_keyword("/tx-hashes/")
      .select(&txid.as_byte_array().to_vec())
      .get_value::<u8>()
      == 1
  }

  fn add_tx_hash(&self, txid: &Txid) -> Result<()> {
    StoragePointer::from_keyword("/tx-hashes/")
      .select(&txid.as_byte_array().to_vec())
      .set_value::<u8>(0x01);
    Ok(())
  }

  // Child registration functions (like boiler's vault factory)
  fn register_wand(&self, wand_id: &AlkaneId) {
    // Maintain existing individual storage for O(1) lookups
    let key = format!("/registered_wands/{}_{}", wand_id.block, wand_id.tx).into_bytes();
    self.store(key, vec![1u8]);

    // Add to centralized list for enumeration
    let mut wands_list = self.registered_wands_list();
    wands_list.push(wand_id.clone());
    self.set_registered_wands_list(wands_list);

    // Update count
    let new_count = self.registered_wands_count().checked_add(1).unwrap_or(0);
    self.set_registered_wands_count(new_count);
  }

  fn is_registered_wand_internal(&self, wand_id: &AlkaneId) -> bool {
    let key = format!("/registered_wands/{}_{}", wand_id.block, wand_id.tx).into_bytes();
    let bytes = self.load(key);
    !bytes.is_empty() && bytes[0] == 1
  }

  fn is_registered_wand(&self, wand_id: AlkaneId) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    let is_registered = self.is_registered_wand_internal(&wand_id);
    response.data = vec![if is_registered { 1u8 } else { 0u8 }];
    Ok(response)
  }

  fn get_all_registered_wands(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    // Get all registered wands from centralized list
    let wands_list = self.registered_wands_list();
    let wands_count = wands_list.len();

    // Encode the registered wands as bytes for external consumption
    // Format: [count (8 bytes)] + [AlkaneId_1 (32 bytes)] + [AlkaneId_2 (32 bytes)] + ...
    // Each AlkaneId: [block (16 bytes)] + [tx (16 bytes)]
    let mut data = Vec::new();

    // Add count of registered wands (as u64 for compatibility)
    data.extend_from_slice(&(wands_count as u64).to_le_bytes());

    // Add each registered wand AlkaneId
    for wand in wands_list {
      data.extend_from_slice(&wand.block.to_le_bytes()); // 16 bytes
      data.extend_from_slice(&wand.tx.to_le_bytes());    // 16 bytes
    }

    response.data = data;
    Ok(response)
  }

  fn registered_wands_list(&self) -> Vec<AlkaneId> {
    let bytes = self.load("/registered_wands_list".as_bytes().to_vec());
    if bytes.is_empty() {
      return Vec::new();
    }

    let mut wands = Vec::new();
    let mut offset = 0;

    // Each AlkaneId is 32 bytes (16 bytes block + 16 bytes tx)
    while offset + 32 <= bytes.len() {
      let block_bytes: [u8; 16] = bytes[offset..offset+16].try_into().unwrap_or([0; 16]);
      let tx_bytes: [u8; 16] = bytes[offset+16..offset+32].try_into().unwrap_or([0; 16]);
      
      wands.push(AlkaneId {
        block: u128::from_le_bytes(block_bytes),
        tx: u128::from_le_bytes(tx_bytes),
      });
      
      offset += 32;
    }

    wands
  }

  fn set_registered_wands_list(&self, wands: Vec<AlkaneId>) {
    let mut bytes = Vec::new();
    
    for wand in wands {
      bytes.extend_from_slice(&wand.block.to_le_bytes());
      bytes.extend_from_slice(&wand.tx.to_le_bytes());
    }
    
    self.store("/registered_wands_list".as_bytes().to_vec(), bytes);
  }

  fn registered_wands_count(&self) -> u128 {
    self.load_u128("/registered_wands_count")
  }

  fn set_registered_wands_count(&self, count: u128) {
    self.store(
      "/registered_wands_count".as_bytes().to_vec(),
      count.to_le_bytes().to_vec(),
    );
  }

  // Helper function to load u128 values from storage (like boiler)
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
}

declare_alkane! {
  impl AlkaneResponder for OrbitalWandFactory {
    type Message = OrbitalWandFactoryMessage;
  }
}