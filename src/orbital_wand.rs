use metashrew_support::index_pointer::KeyValuePointer;
use metashrew_support::compat::to_arraybuffer_layout;
use metashrew_support::utils::consensus_decode;

use alkanes_runtime::{
  declare_alkane, message::MessageDispatch, storage::StoragePointer, token::Token,
  runtime::AlkaneResponder
};

use alkanes_support::{
  id::AlkaneId,
  parcel::AlkaneTransfer, response::CallResponse,
  utils::overflow_error
};

use bitcoin::hashes::Hash;
use bitcoin::{Txid, Transaction, blockdata::block::TxMerkleNode};

use anyhow::{anyhow, Result};
use std::sync::Arc;

use crate::wand_svg::{WandData, WandSvgGenerator};
use crate::probability::ProbabilityCalculator;

const DUST_TOKEN_BLOCK: u128 = 0x2;
const MIN_DUST_STAKE: u128 = 1000;
const DUST_BONUS_THRESHOLD: u128 = 2000;
const DUST_BONUS_INCREMENT: u128 = 1000;
const DUST_BONUS_POINTS: u8 = 10;
const WIN_THRESHOLD: u8 = 141;

#[derive(Default)]
pub struct OrbitalWand(());

impl AlkaneResponder for OrbitalWand {}

#[derive(MessageDispatch)]
enum OrbitalWandMessage {
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
}

impl Token for OrbitalWand {
  fn name(&self) -> String {
    return String::from("Orbital Wand")
  }

  fn symbol(&self) -> String {
    return String::from("WAND");
  }
}

#[derive(Clone)]
pub struct WandMetadata {
    pub wand_id: u128,
    pub position_token_id: AlkaneId,
    pub txid: Txid,
    pub merkle_root: TxMerkleNode,
    pub base_xor_result: u8,
    pub dust_bonus: u8,
    pub final_xor_result: u8,
    pub dust_amount: u128,
    pub block_height: u32,
}

impl WandMetadata {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(115);
        
        // Wand ID (16 bytes)
        bytes.extend_from_slice(&self.wand_id.to_le_bytes());
        
        // Position token ID (32 bytes)
        bytes.extend_from_slice(&self.position_token_id.block.to_le_bytes());
        bytes.extend_from_slice(&self.position_token_id.tx.to_le_bytes());
        
        // Transaction ID (32 bytes)
        bytes.extend_from_slice(self.txid.as_byte_array());
        
        // Merkle root (32 bytes)
        bytes.extend_from_slice(self.merkle_root.as_byte_array());
        
        // XOR results (3 bytes)
        bytes.push(self.base_xor_result);
        bytes.push(self.dust_bonus);
        bytes.push(self.final_xor_result);
        
        // Dust amount (16 bytes)
        bytes.extend_from_slice(&self.dust_amount.to_le_bytes());
        
        // Block height (4 bytes)
        bytes.extend_from_slice(&self.block_height.to_le_bytes());
        
        bytes
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 115 {
            return Err(anyhow!("Invalid wand metadata length"));
        }
        
        let wand_id = u128::from_le_bytes(bytes[0..16].try_into().unwrap());
        
        let position_block = u128::from_le_bytes(bytes[16..32].try_into().unwrap());
        let position_tx = u128::from_le_bytes(bytes[32..48].try_into().unwrap());
        let position_token_id = AlkaneId { block: position_block, tx: position_tx };
        
        let txid = Txid::from_byte_array(bytes[48..80].try_into().unwrap());
        let merkle_root = TxMerkleNode::from_byte_array(bytes[80..112].try_into().unwrap());
        
        let base_xor_result = bytes[112];
        let dust_bonus = bytes[113];
        let final_xor_result = bytes[114];
        
        let dust_amount = u128::from_le_bytes(bytes[115..131].try_into().unwrap());
        let block_height = u32::from_le_bytes(bytes[131..135].try_into().unwrap());
        
        Ok(WandMetadata {
            wand_id,
            position_token_id,
            txid,
            merkle_root,
            base_xor_result,
            dust_bonus,
            final_xor_result,
            dust_amount,
            block_height,
        })
    }
}

impl OrbitalWand {
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

    // Validate inputs: must have exactly 1 position token + dust
    if context.incoming_alkanes.0.len() != 2 {
      return Err(anyhow!("Must send exactly 1 position token + dust"));
    }

    // Find position token and dust
    let mut position_token: Option<AlkaneTransfer> = None;
    let mut dust_transfer: Option<AlkaneTransfer> = None;

    for alkane in &context.incoming_alkanes.0 {
      if alkane.id.block == DUST_TOKEN_BLOCK {
        if dust_transfer.is_some() {
          return Err(anyhow!("Multiple dust transfers not allowed"));
        }
        dust_transfer = Some(alkane.clone());
      } else {
        if position_token.is_some() {
          return Err(anyhow!("Multiple position tokens not allowed"));
        }
        position_token = Some(alkane.clone());
      }
    }

    let position_token = position_token.ok_or_else(|| anyhow!("No position token provided"))?;
    let dust_transfer = dust_transfer.ok_or_else(|| anyhow!("No dust provided"))?;

    // Validate dust amount
    if dust_transfer.value < MIN_DUST_STAKE {
      return Err(anyhow!("Minimum {} dust required", MIN_DUST_STAKE));
    }

    // Calculate dust bonus
    let dust_bonus = self.calculate_dust_bonus(dust_transfer.value);

    // Get randomness sources
    let merkle_root = self.merkle_root()?;
    let base_xor_result = self.calculate_base_xor(&txid, &merkle_root)?;
    let final_xor_result = base_xor_result.saturating_add(dust_bonus);

    // Check if player wins
    if final_xor_result < WIN_THRESHOLD {
      // Player loses - consume their stake
      self.add_tx_hash(&txid)?;
      self.record_loss(dust_transfer.value)?;
      return Err(anyhow!("XOR result {} < {}. Better luck next time!", final_xor_result, WIN_THRESHOLD));
    }

    // Player wins! Create orbital wand NFT
    let wand_id = self.get_next_wand_id()?;
    let block_height = self.block_height()?;

    let wand_metadata = WandMetadata {
      wand_id,
      position_token_id: position_token.id.clone(),
      txid,
      merkle_root,
      base_xor_result,
      dust_bonus,
      final_xor_result,
      dust_amount: dust_transfer.value,
      block_height,
    };

    // Store wand metadata
    self.store_wand_metadata(&wand_metadata)?;
    self.record_win(dust_transfer.value)?;
    self.add_tx_hash(&txid)?;

    // Create response with new wand NFT
    let mut response = CallResponse::default();
    response.alkanes.0.push(AlkaneTransfer {
      id: context.myself.clone(),
      value: 1u128,
    });

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
      position_token_id: latest_metadata.position_token_id,
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
      wand_list.push(format!(
        "{{\"id\":{},\"position\":\"{}:{}\",\"xor\":{},\"dust\":{},\"power\":\"{}\"}}",
        metadata.wand_id,
        metadata.position_token_id.block,
        metadata.position_token_id.tx,
        metadata.final_xor_result,
        metadata.dust_amount,
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

  fn calculate_base_xor(&self, txid: &Txid, merkle_root: &TxMerkleNode) -> Result<u8> {
    let txid_bytes = txid.as_byte_array();
    let merkle_bytes = merkle_root.as_byte_array();
    
    let txid_last = txid_bytes[31];
    let merkle_last = merkle_bytes[31];
    
    Ok(txid_last ^ merkle_last)
  }

  fn calculate_wand_power(&self, final_xor_result: u8) -> String {
    match final_xor_result {
      141..=160 => "Apprentice".to_string(),
      161..=180 => "Adept".to_string(),
      181..=200 => "Expert".to_string(),
      201..=220 => "Master".to_string(),
      221..=240 => "Grandmaster".to_string(),
      241..=255 => "Cosmic".to_string(),
      _ => "Unknown".to_string(),
    }
  }

  fn generate_wand_attributes(&self, metadata: &WandMetadata) -> Result<String> {
    let power = self.calculate_wand_power(metadata.final_xor_result);
    let wand_type = self.get_wand_type(metadata.final_xor_result);
    
    let attributes = format!(r#"{{
  "name": "Orbital Wand #{}",
  "description": "A magical orbital wand forged from position token {}:{} with {} dust enhancement",
  "attributes": [
    {{"trait_type": "Wand ID", "value": "{}"}},
    {{"trait_type": "Power Level", "value": "{}"}},
    {{"trait_type": "Wand Type", "value": "{}"}},
    {{"trait_type": "Position Token", "value": "{}:{}"}},
    {{"trait_type": "Dust Amount", "value": "{}"}},
    {{"trait_type": "Dust Bonus", "value": "{}"}},
    {{"trait_type": "Base XOR", "value": "{}"}},
    {{"trait_type": "Final XOR", "value": "{}"}},
    {{"trait_type": "Block Height", "value": "{}"}},
    {{"trait_type": "Transaction", "value": "{}"}},
    {{"trait_type": "Merkle Root", "value": "{}"}}
  ]
}}"#,
      metadata.wand_id,
      metadata.position_token_id.block,
      metadata.position_token_id.tx,
      metadata.dust_amount,
      metadata.wand_id,
      power,
      wand_type,
      metadata.position_token_id.block,
      metadata.position_token_id.tx,
      metadata.dust_amount,
      metadata.dust_bonus,
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

  fn record_win(&self, dust_amount: u128) -> Result<()> {
    self.add_dust_consumed(dust_amount)?;
    self.add_position_consumed()?;
    self.add_game()?;
    Ok(())
  }

  fn record_loss(&self, dust_amount: u128) -> Result<()> {
    self.add_dust_consumed(dust_amount)?;
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
    // In a real implementation, this would get the current block's merkle root
    // For testing, we'll use a deterministic value based on block height
    let block_height = self.block_height()?;
    let mut bytes = [0u8; 32];
    bytes[0..4].copy_from_slice(&block_height.to_le_bytes());
    Ok(TxMerkleNode::from_byte_array(bytes))
  }

  fn block_height(&self) -> Result<u32> {
    // In a real implementation, this would get the current block height
    // For testing, we'll use a mock value
    Ok(1000)
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
}

declare_alkane! {
  impl AlkaneResponder for OrbitalWand {
    type Message = OrbitalWandMessage;
  }
}