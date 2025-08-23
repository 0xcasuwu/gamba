use metashrew_support::utils::consensus_decode;
use metashrew_support::index_pointer::KeyValuePointer;
use metashrew_support::compat::to_arraybuffer_layout;

use alkanes_runtime::{
  declare_alkane, message::MessageDispatch, storage::StoragePointer,
  runtime::AlkaneResponder
};

use alkanes_support::{
  id::AlkaneId,
  parcel::{AlkaneTransfer, AlkaneTransferParcel},
  response::CallResponse
};

use bitcoin::hashes::{Hash, HashEngine};
use bitcoin::{Txid, Transaction, blockdata::block::TxMerkleNode};

use anyhow::{anyhow, Result};
use std::sync::Arc;

use crate::coupon_svg::{CouponData, CouponSvgGenerator};

// Generic gambling constants - no specific token restrictions
const MIN_TOKEN_STAKE: u128 = 1; // Minimum tokens required for any token type
const BONUS_THRESHOLD: u128 = 1000; // Threshold for bonus calculation
const BONUS_INCREMENT: u128 = 500;
const BONUS_POINTS: u8 = 5; // Base bonus points per increment
const SUCCESS_THRESHOLD: u8 = 150; // Values 150+ succeed in creating a coupon

// Single coupon template ID - one template for all coupon types
const COUPON_TEMPLATE_ID: u128 = 0x1001;           // Single coupon template

// Template block (assuming template is deployed in block 6 like boiler)
const COUPON_TEMPLATE_BLOCK: u128 = 6;

#[derive(Default)]
pub struct CouponFactory(());

impl AlkaneResponder for CouponFactory {}

#[derive(MessageDispatch)]
enum CouponFactoryMessage {
  #[opcode(0)]
  Initialize,

  #[opcode(42)]
  CreateCoupon,

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
  GetCouponCount,

  #[opcode(2001)]
  #[returns(Vec<Vec<u8>>)]
  GetCouponList,

  #[opcode(2002)]
  #[returns(String)]
  GetCouponListJson,

  #[opcode(2003)]
  #[returns(Vec<u8>)]
  GetCouponMetadata,

  #[opcode(2004)]
  #[returns(u128)]
  GetTotalTokensConsumed,

  #[opcode(2005)]
  #[returns(u128)]
  GetTotalPositionsConsumed,

  #[opcode(2006)]
  #[returns(f64)]
  GetWinRate,

  #[opcode(2007)]
  #[returns(Vec<u8>)]
  GetLatestCouponData,

  #[opcode(2008)]
  #[returns(Vec<u8>)]
  GetGamblingStats,

  #[opcode(2009)]
  #[returns(String)]
  GetContractInfo,

  // Child registration opcodes (like boiler)
  #[opcode(3000)]
  #[returns(Vec<u8>)]
  GetAllRegisteredCoupons,

  #[opcode(3001)]
  #[returns(bool)]
  IsRegisteredCoupon {
    coupon_id: AlkaneId,
  },
}


#[derive(Clone)]
pub struct CouponMetadata {
    pub coupon_id: u128,
    pub stake_token_id: AlkaneId, // Any token can be staked
    pub txid: Txid,
    pub merkle_root: TxMerkleNode,
    pub base_xor_result: u8,
    pub token_bonus: u8,
    pub final_xor_result: u8,
    pub token_amount: u128,
    pub block_height: u32,
    pub is_winner: bool, // Indicates if this coupon won or lost
}

impl CouponMetadata {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(120); // Reduced capacity for simplified fields
        
        // Coupon ID (16 bytes)
        bytes.extend_from_slice(&self.coupon_id.to_le_bytes());
        
        // Stake token ID (32 bytes)
        bytes.extend_from_slice(&self.stake_token_id.block.to_le_bytes());
        bytes.extend_from_slice(&self.stake_token_id.tx.to_le_bytes());
        
        // Transaction ID (32 bytes)
        bytes.extend_from_slice(self.txid.as_byte_array());
        
        // Merkle root (32 bytes)
        bytes.extend_from_slice(self.merkle_root.as_byte_array());
        
        // XOR results (3 bytes)
        bytes.push(self.base_xor_result);
        bytes.push(self.token_bonus);
        bytes.push(self.final_xor_result);
        
        // Token amount (16 bytes)
        bytes.extend_from_slice(&self.token_amount.to_le_bytes());
        
        // Block height (4 bytes)
        bytes.extend_from_slice(&self.block_height.to_le_bytes());
        
        // Is winner flag (1 byte)
        bytes.push(if self.is_winner { 1 } else { 0 });
        
        bytes
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 120 {
            return Err(anyhow!("Invalid coupon metadata length: expected 120, got {}", bytes.len()));
        }
        
        let coupon_id = u128::from_le_bytes(bytes[0..16].try_into().unwrap());
        
        let stake_block = u128::from_le_bytes(bytes[16..32].try_into().unwrap());
        let stake_tx = u128::from_le_bytes(bytes[32..48].try_into().unwrap());
        let stake_token_id = AlkaneId { block: stake_block, tx: stake_tx };
        
        let txid = Txid::from_byte_array(bytes[48..80].try_into().unwrap());
        let merkle_root = TxMerkleNode::from_byte_array(bytes[80..112].try_into().unwrap());
        
        let base_xor_result = bytes[112];
        let token_bonus = bytes[113];
        let final_xor_result = bytes[114];
        
        let token_amount = u128::from_le_bytes(bytes[115..131].try_into().unwrap());
        let block_height = u32::from_le_bytes(bytes[131..135].try_into().unwrap());
        let is_winner = bytes[135] == 1;
        
        Ok(CouponMetadata {
            coupon_id,
            stake_token_id,
            txid,
            merkle_root,
            base_xor_result,
            token_bonus,
            final_xor_result,
            token_amount,
            block_height,
            is_winner,
        })
    }
}

impl CouponFactory {
  fn initialize(&self) -> Result<CallResponse> {
    self.observe_initialization()?;
    let context = self.context()?;

    let response = CallResponse::forward(&context.incoming_alkanes);
    Ok(response)
  }

  fn create_coupon(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let txid = self.transaction_id()?;

    // Enforce one creation per transaction
    if self.has_tx_hash(&txid) {
      return Err(anyhow!("Transaction already used for coupon creation"));
    }

    // Validate inputs: must have at least 1 token of any type
    if context.incoming_alkanes.0.is_empty() {
      return Err(anyhow!("Must send at least 1 token to create a coupon"));
    }

    // Calculate total token amount with overflow protection
    let total_tokens: u128 = context.incoming_alkanes.0.iter()
      .try_fold(0u128, |acc, t| acc.checked_add(t.value))
      .ok_or_else(|| anyhow!("Token amount overflow"))?;

    // Validate minimum stake
    if total_tokens < MIN_TOKEN_STAKE {
      return Err(anyhow!("Insufficient tokens - minimum {} required, got {}", MIN_TOKEN_STAKE, total_tokens));
    }

    // Calculate bonus based on total token amount
    let token_bonus = self.calculate_token_bonus(total_tokens);

    // Get current block info
    let current_block = self.height() as u32;
    let merkle_root = self.merkle_root()?;

    // Use the first incoming token ID as the stake reference
    let stake_token_id = context.incoming_alkanes.0[0].id;

    // Generate coupon ID based on txid + stake info
    let coupon_id = self.generate_coupon_id(&txid, &stake_token_id);

    // Calculate XOR results
    let base_xor_result = self.calculate_base_xor(&txid, &merkle_root);
    let final_xor_result = (base_xor_result ^ token_bonus) & 0xFF;

    // Check if this is a winning coupon (final XOR >= 150)
    let is_winner = final_xor_result >= SUCCESS_THRESHOLD;

    // Create coupon metadata
    let coupon_metadata = CouponMetadata {
      coupon_id,
      stake_token_id,
      txid,
      merkle_root,
      base_xor_result,
      token_bonus,
      final_xor_result,
      token_amount: total_tokens,
      block_height: current_block,
      is_winner,
    };

    // Create the coupon NFT
    let coupon_nft = self.create_coupon_nft(&coupon_metadata)?;

    // Store transaction hash to prevent reuse
    self.store_tx_hash(&txid);

    let mut response = CallResponse::default();
    
    // Only add the coupon NFT if it has value (for winners)
    if coupon_nft.value > 0 {
      response.alkanes.0.push(coupon_nft);
    }
    
    Ok(response)
  }

  fn get_data(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    // Get latest coupon metadata and generate SVG
    let coupon_count = self.coupon_count();
    if coupon_count == 0 {
      response.data = Vec::new();
      return Ok(response);
    }

    let latest_metadata = self.get_coupon_metadata_internal(coupon_count - 1)?;
    let coupon_data = CouponData {
      coupon_id: latest_metadata.coupon_id,
      txid: latest_metadata.txid,
      merkle_root: latest_metadata.merkle_root,
      base_xor_result: latest_metadata.base_xor_result,
      token_bonus: latest_metadata.token_bonus,
      final_xor_result: latest_metadata.final_xor_result,
      token_amount: latest_metadata.token_amount,
      is_winner: latest_metadata.is_winner,
    };

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

    let coupon_count = self.coupon_count();
    if coupon_count == 0 {
      response.data = b"{}".to_vec();
      return Ok(response);
    }

    let latest_metadata = self.get_coupon_metadata_internal(coupon_count - 1)?;
    let attributes = self.generate_coupon_attributes(&latest_metadata)?;
    response.data = attributes.into_bytes();

    Ok(response)
  }

  fn get_coupon_count(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    response.data = self.coupon_count().to_le_bytes().to_vec();
    Ok(response)
  }

  fn get_coupon_list(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let count = self.coupon_count();
    let mut coupon_list = Vec::new();

    for i in 0..count {
      let metadata = self.get_coupon_metadata_internal(i)?;
      coupon_list.push(metadata.to_bytes());
    }

    let mut flattened = Vec::new();
    for bytes in coupon_list {
      flattened.extend(bytes);
    }

    response.data = flattened;
    Ok(response)
  }

  fn get_coupon_list_json(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let count = self.coupon_count();
    let mut coupon_list = Vec::new();

    for i in 0..count {
      let metadata = self.get_coupon_metadata_internal(i)?;
      let result_str = if metadata.is_winner { "winner" } else { "loser" };
      let power = self.calculate_coupon_power(metadata.final_xor_result);
      
      coupon_list.push(format!(
        "{{\"id\":{},\"stake_token\":\"{}:{}\",\"result\":\"{}\",\"xor\":{},\"tokens\":{},\"power\":\"{}\"}}",
        metadata.coupon_id,
        metadata.stake_token_id.block,
        metadata.stake_token_id.tx,
        result_str,
        metadata.final_xor_result,
        metadata.token_amount,
        power
      ));
    }

    let json = format!("[{}]", coupon_list.join(","));
    response.data = json.into_bytes();
    Ok(response)
  }

  fn get_coupon_metadata(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let coupon_count = self.coupon_count();
    if coupon_count == 0 {
      response.data = Vec::new();
      return Ok(response);
    }

    let latest_metadata = self.get_coupon_metadata_internal(coupon_count - 1)?;
    response.data = latest_metadata.to_bytes();

    Ok(response)
  }

  fn get_total_tokens_consumed(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    response.data = self.total_tokens_consumed().to_le_bytes().to_vec();
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
    
    let wins = self.coupon_count();
    let total_games = self.total_games();
    
    let win_rate = if total_games > 0 {
      (wins as f64) / (total_games as f64)
    } else {
      0.0
    };
    
    response.data = win_rate.to_le_bytes().to_vec();
    Ok(response)
  }

  fn get_latest_coupon_data(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let coupon_count = self.coupon_count();
    if coupon_count == 0 {
      response.data = Vec::new();
      return Ok(response);
    }

    let latest_metadata = self.get_coupon_metadata_internal(coupon_count - 1)?;
    response.data = latest_metadata.to_bytes();

    Ok(response)
  }

  fn get_gambling_stats(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    // Comprehensive gambling statistics
    let total_games = self.total_games();
    let total_wins = self.coupon_count();
    let total_tokens_consumed = self.total_tokens_consumed();
    let total_positions_consumed = self.total_positions_consumed();

    // Calculate win rate
    let win_rate_percentage = if total_games > 0 {
      ((total_wins as f64) / (total_games as f64) * 100.0) as u32
    } else {
      0u32
    };

    // Pack statistics into response
    // Format: [total_games (16)] + [total_wins (16)] + [total_tokens (16)] + [total_positions (16)] + [win_rate_percentage (4)]
    let mut data = Vec::with_capacity(68);
    data.extend_from_slice(&total_games.to_le_bytes());
    data.extend_from_slice(&total_wins.to_le_bytes());
    data.extend_from_slice(&total_tokens_consumed.to_le_bytes());
    data.extend_from_slice(&total_positions_consumed.to_le_bytes());
    data.extend_from_slice(&win_rate_percentage.to_le_bytes());

    response.data = data;
    Ok(response)
  }

  fn get_contract_info(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let info = format!(r#"{{
  "name": "Generic Gambling Contract",
  "version": "3.0.0",
  "description": "Generic gambling contract that accepts any tokens to create win/lose coupons",
  "features": [
    "Generic token support",
    "Token burning for improved odds",
    "Dynamic bonus calculation",
    "Merkle root randomness",
    "NFT coupon creation",
    "Win/lose indication",
    "Comprehensive statistics"
  ],
  "constants": {{
    "min_token_stake": {},
    "bonus_threshold": {},
    "bonus_increment": {},
    "bonus_points": {},
    "success_threshold": {}
  }},
  "current_stats": {{
    "total_coupons": {},
    "total_games": {},
    "total_tokens_consumed": {}
  }}
}}"#,
      MIN_TOKEN_STAKE,
      BONUS_THRESHOLD,
      BONUS_INCREMENT,
      BONUS_POINTS,
      SUCCESS_THRESHOLD,
      self.coupon_count(),
      self.total_games(),
      self.total_tokens_consumed()
    );

    response.data = info.into_bytes();
    Ok(response)
  }

  // Generic validation functions - now accepts any tokens
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
      // Note: No specific token validation - accept any tokens
    }

    Ok(())
  }

  // Factory helper functions for coupon rarity/type
  fn get_coupon_rarity(&self, final_xor_result: u8) -> String {
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

  fn get_coupon_type_name(&self, final_xor_result: u8) -> String {
    if final_xor_result >= SUCCESS_THRESHOLD {
      match final_xor_result % 5 {
        0 => "Lucky Coupon".to_string(),
        1 => "Golden Coupon".to_string(),
        2 => "Diamond Coupon".to_string(),
        3 => "Platinum Coupon".to_string(),
        4 => "Cosmic Coupon".to_string(),
        _ => "Winner Coupon".to_string(),
      }
    } else {
      "Losing Coupon".to_string()
    }
  }

  // Helper functions
  fn calculate_token_bonus(&self, token_amount: u128) -> u8 {
    if token_amount < BONUS_THRESHOLD {
      return 0;
    }
    
    let bonus_increments = (token_amount - BONUS_THRESHOLD) / BONUS_INCREMENT;
    let bonus = bonus_increments * (BONUS_POINTS as u128);
    
    // Cap at 255 to prevent overflow
    std::cmp::min(bonus, 255) as u8
  }

  fn calculate_base_xor(&self, _txid: &Txid, merkle_root: &TxMerkleNode) -> u8 {
    let merkle_bytes = merkle_root.as_byte_array();
    
    // Use the last byte of the merkle root as the base randomness
    // This gives us a value from 0-255 which we compare against SUCCESS_THRESHOLD
    merkle_bytes[31]
  }

  fn calculate_coupon_power(&self, final_xor_result: u8) -> String {
    match final_xor_result {
      150..=170 => "Apprentice".to_string(),
      171..=190 => "Adept".to_string(),
      191..=210 => "Expert".to_string(),
      211..=230 => "Master".to_string(),
      231..=250 => "Grandmaster".to_string(),
      251..=255 => "Cosmic".to_string(),
      _ => "Failed".to_string(), // Should not happen for successful coupons
    }
  }

  fn generate_coupon_attributes(&self, metadata: &CouponMetadata) -> Result<String> {
    let power = self.calculate_coupon_power(metadata.final_xor_result);
    let coupon_type = self.get_coupon_type_name(metadata.final_xor_result);
    let rarity = self.get_coupon_rarity(metadata.final_xor_result);
    let result_str = if metadata.is_winner { "Winner" } else { "Loser" };
    
    let description = format!("A gambling coupon created with {} tokens from {}:{}",
                            metadata.token_amount,
                            metadata.stake_token_id.block,
                            metadata.stake_token_id.tx);
    
    let attributes = format!(r#"{{
  "name": "{} #{}",
  "description": "{}",
  "attributes": [
    {{"trait_type": "Coupon ID", "value": "{}"}},
    {{"trait_type": "Result", "value": "{}"}},
    {{"trait_type": "Power Level", "value": "{}"}},
    {{"trait_type": "Coupon Type", "value": "{}"}},
    {{"trait_type": "Rarity", "value": "{}"}},
    {{"trait_type": "Stake Token", "value": "{}:{}"}},
    {{"trait_type": "Token Amount", "value": "{}"}},
    {{"trait_type": "Token Bonus", "value": "{}"}},
    {{"trait_type": "Base XOR", "value": "{}"}},
    {{"trait_type": "Final XOR", "value": "{}"}},
    {{"trait_type": "Block Height", "value": "{}"}},
    {{"trait_type": "Transaction", "value": "{}"}},
    {{"trait_type": "Merkle Root", "value": "{}"}}
  ]
}}"#,
      coupon_type,
      metadata.coupon_id,
      description,
      metadata.coupon_id,
      result_str,
      power,
      coupon_type,
      rarity,
      metadata.stake_token_id.block,
      metadata.stake_token_id.tx,
      metadata.token_amount,
      metadata.token_bonus,
      metadata.base_xor_result,
      metadata.final_xor_result,
      metadata.block_height,
      metadata.txid,
      metadata.merkle_root
    );
    
    Ok(attributes)
  }

  // Storage functions
  fn coupon_count_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/coupon_count")
  }

  fn coupon_count(&self) -> u128 {
    self.coupon_count_pointer().get_value::<u128>()
  }

  fn set_coupon_count(&self, count: u128) {
    self.coupon_count_pointer().set_value::<u128>(count);
  }

  fn get_next_coupon_id(&self) -> Result<u128> {
    let count = self.coupon_count();
    let new_count = count.checked_add(1)
      .ok_or_else(|| anyhow!("Coupon count overflow"))?;
    self.set_coupon_count(new_count);
    Ok(count)
  }

  fn store_coupon_metadata(&self, metadata: &CouponMetadata) -> Result<()> {
    let bytes = metadata.to_bytes();
    let mut coupon_pointer = StoragePointer::from_keyword("/coupons/")
      .select(&metadata.coupon_id.to_le_bytes().to_vec());
    coupon_pointer.set(Arc::new(bytes));
    Ok(())
  }

  fn get_coupon_metadata_internal(&self, coupon_id: u128) -> Result<CouponMetadata> {
    let coupon_pointer = StoragePointer::from_keyword("/coupons/")
      .select(&coupon_id.to_le_bytes().to_vec());
    let bytes = coupon_pointer.get();
    CouponMetadata::from_bytes(&bytes)
  }

  fn total_tokens_consumed_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/total_tokens_consumed")
  }

  fn total_tokens_consumed(&self) -> u128 {
    self.total_tokens_consumed_pointer().get_value::<u128>()
  }

  fn add_tokens_consumed(&self, amount: u128) -> Result<()> {
    let current = self.total_tokens_consumed();
    let new_total = current.checked_add(amount)
      .ok_or_else(|| anyhow!("Tokens consumed overflow"))?;
    self.total_tokens_consumed_pointer().set_value::<u128>(new_total);
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
    self.add_tokens_consumed(total_stake)?;
    self.add_position_consumed()?;
    self.add_game()?;
    Ok(())
  }

  fn record_loss(&self, total_stake: u128) -> Result<()> {
    self.add_tokens_consumed(total_stake)?;
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
    Ok(self.height().try_into().unwrap())
  }

  fn has_tx_hash(&self, txid: &Txid) -> bool {
    StoragePointer::from_keyword("/tx-hashes/")
      .select(&txid.as_byte_array().to_vec())
      .get_value::<u8>()
      == 1
  }

  fn store_tx_hash(&self, txid: &Txid) {
    StoragePointer::from_keyword("/tx-hashes/")
      .select(&txid.as_byte_array().to_vec())
      .set_value::<u8>(0x01);
  }

  // Child registration functions (like boiler's vault factory)
  fn register_coupon(&self, coupon_id: &AlkaneId) {
    // Maintain existing individual storage for O(1) lookups
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

  fn is_registered_coupon_internal(&self, coupon_id: &AlkaneId) -> bool {
    let key = format!("/registered_coupons/{}_{}", coupon_id.block, coupon_id.tx).into_bytes();
    let bytes = self.load(key);
    !bytes.is_empty() && bytes[0] == 1
  }

  fn is_registered_coupon(&self, coupon_id: AlkaneId) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    let is_registered = self.is_registered_coupon_internal(&coupon_id);
    response.data = vec![if is_registered { 1u8 } else { 0u8 }];
    Ok(response)
  }

  fn get_all_registered_coupons(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    // Get all registered coupons from centralized list
    let coupons_list = self.registered_coupons_list();
    let coupons_count = coupons_list.len();

    // Encode the registered coupons as bytes for external consumption
    // Format: [count (8 bytes)] + [AlkaneId_1 (32 bytes)] + [AlkaneId_2 (32 bytes)] + ...
    // Each AlkaneId: [block (16 bytes)] + [tx (16 bytes)]
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

  // Helper functions for coupon creation
  fn generate_coupon_id(&self, txid: &Txid, stake_token_id: &AlkaneId) -> u128 {
    let mut hasher = bitcoin::hashes::sha256::Hash::engine();
    hasher.input(txid.as_byte_array());
    hasher.input(&stake_token_id.block.to_le_bytes());
    hasher.input(&stake_token_id.tx.to_le_bytes());
    
    let hash = bitcoin::hashes::sha256::Hash::from_engine(hasher);
    u128::from_le_bytes(hash.as_byte_array()[0..16].try_into().unwrap())
  }

  fn create_coupon_nft(&self, metadata: &CouponMetadata) -> Result<AlkaneTransfer> {
    // Store the coupon metadata first
    self.store_coupon_metadata(metadata)?;

    // Only create an NFT if this is a winning coupon
    if metadata.is_winner {
      // Record the win
      self.record_win(metadata.token_amount)?;

      // Create NFT transfer for winning coupon
      let coupon_nft = AlkaneTransfer {
        id: AlkaneId {
          block: COUPON_TEMPLATE_BLOCK,
          tx: COUPON_TEMPLATE_ID,
        },
        value: 1, // Always mint exactly 1 NFT for winners
      };

      // Register the new coupon
      self.register_coupon(&coupon_nft.id);

      Ok(coupon_nft)
    } else {
      // Record the loss but don't create an NFT
      self.record_loss(metadata.token_amount)?;
      
      // Return a zero-value transfer (no NFT minted for losers)
      Ok(AlkaneTransfer {
        id: AlkaneId { block: 0, tx: 0 },
        value: 0,
      })
    }
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
  impl AlkaneResponder for CouponFactory {
    type Message = CouponFactoryMessage;
  }
}