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
use bitcoin::{Txid, Transaction};

use anyhow::{anyhow, Result};
use std::sync::Arc;

pub mod orbital_wand;
pub use orbital_wand::OrbitalWand;

pub mod probability;
pub use probability::ProbabilityCalculator;

pub mod wand_svg;

pub mod tests {
    pub mod std;
    pub mod comprehensive_gambling_test;
    pub mod xor_logic_verification_test;
    pub mod probability_calculation_test;
    pub mod dust_bonus_verification_test;
    pub mod orbital_wand_integration_test;
    pub mod position_validation_test;
}

const DUST_BLOCK: u128 = 0x2;

const DUST_PER_POSITION: u128 = 10_000_000_000_000;
const POSITION_SUPPLY: u128 = 10_000;
const DUST_CAP: u128 = POSITION_SUPPLY * DUST_PER_POSITION;

#[derive(Default)]
pub struct DustSwap(());

impl AlkaneResponder for DustSwap {}

#[derive(MessageDispatch)]
enum DustSwapMessage {
  #[opcode(0)]
  Initialize,

  #[opcode(42)]
  PositionToDust,

  #[opcode(69)]
  DustToPosition,

  #[opcode(77)]
  MintTokens,

  #[opcode(99)]
  #[returns(String)]
  GetName,

  #[opcode(100)]
  #[returns(String)]
  GetSymbol,

  #[opcode(101)]
  #[returns(u128)]
  GetTotalSupply,
  
  #[opcode(102)]
  #[returns(u128)]
  GetCap,

  #[opcode(103)]
  #[returns(u128)]
  GetMinted,

  #[opcode(104)]
  #[returns(u128)]
  GetValuePerMint,

  #[opcode(1000)]
  #[returns(Vec<u8>)]
  GetData,

  #[opcode(2000)]
  #[returns(u128)]
  GetPositionStackCount,

  #[opcode(2001)]
  #[returns(Vec<Vec<u8>>)]
  GetPositionStack,

  #[opcode(2002)]
  #[returns(String)]
  GetPositionStackJson,
}

impl Token for DustSwap {
  fn name(&self) -> String {
    return String::from("DUST")
  }

  fn symbol(&self) -> String {
    return String::from("DUST");
  }
}

impl DustSwap {
  fn initialize(&self) -> Result<CallResponse> {
    self.observe_initialization()?;
    let context = self.context()?;

    let response = CallResponse::forward(&context.incoming_alkanes);
    Ok(response)
  }

  fn get_name(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    response.data = self.name().into_bytes();

    Ok(response)
  }

  fn get_symbol(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    response.data = self.symbol().into_bytes();

    Ok(response)
  }

  fn get_total_supply(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    response.data = self.total_supply().to_le_bytes().to_vec();

    Ok(response)
  }

  fn get_cap(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    response.data = DUST_CAP.to_le_bytes().to_vec();

    Ok(response)
  }

  fn get_minted(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    response.data = self.instances_count().to_le_bytes().to_vec();

    Ok(response)
  }

  fn get_value_per_mint(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    response.data = DUST_PER_POSITION.to_le_bytes().to_vec();

    Ok(response)
  }

  fn get_data(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    // Return empty data for now - could add dust particle animation
    response.data = Vec::new();

    Ok(response)
  }

  fn total_supply_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/total_supply")
  }

  fn total_supply(&self) -> u128 {
    self.total_supply_pointer().get_value::<u128>()
  }

  fn set_total_supply(&self, v: u128) {
    self.total_supply_pointer().set_value::<u128>(v);
  }

  fn increase_total_supply(&self, v: u128) -> Result<()> {
    self.set_total_supply(overflow_error(self.total_supply().checked_add(v))?);
    Ok(())
  }

  fn decrease_total_supply(&self, v: u128) -> Result<()> {
    self.set_total_supply(overflow_error(self.total_supply().checked_sub(v))?);
    Ok(())
  }

  fn is_valid_alkamist_or_dust(&self, id: &AlkaneId) -> Result<bool> {
    // Accept specific valid positions:
    // - Alkamist position at 2:25720
    // - Dust position at 2:35275
    // - Any Dust tokens from block 2 (for backward compatibility)
    
    const ALKAMIST_BLOCK: u128 = 0x2;
    const ALKAMIST_TX: u128 = 25720;
    const DUST_TX: u128 = 35275;
    
    // Check for specific valid alkamist position
    if id.block == ALKAMIST_BLOCK && id.tx == ALKAMIST_TX {
      return Ok(true);
    }
    
    // Check for specific valid dust position
    if id.block == DUST_BLOCK && id.tx == DUST_TX {
      return Ok(true);
    }
    
    // For backward compatibility, accept any token from DUST_BLOCK (block 2)
    // but exclude the specific alkamist position to avoid double-counting
    if id.block == DUST_BLOCK && id.tx != ALKAMIST_TX {
      return Ok(true);
    }
    
    Ok(false)
  }

  /// Validate incoming alkanes similar to boiler's authenticate_position
  fn validate_incoming_alkanes(&self, incoming_alkanes: &[AlkaneTransfer]) -> Result<()> {
    // Validate incoming alkanes structure
    if incoming_alkanes.is_empty() {
      return Err(anyhow!("No incoming alkanes for validation"));
    }

    for transfer in incoming_alkanes {
      // The value should be at least 1
      if transfer.value < 1 {
        return Err(anyhow!("Less than 1 unit of token supplied for alkane {}:{}",
                          transfer.id.block, transfer.id.tx));
      }

      // Validate that this is a valid alkamist or dust token
      if !self.is_valid_alkamist_or_dust(&transfer.id)? {
        return Err(anyhow!(
          "Invalid token ID {}:{} - must be valid Alkamist position (2:25720), Dust position (2:35275), or other Dust token from block 2",
          transfer.id.block, transfer.id.tx
        ));
      }
    }

    Ok(())
  }

  fn position_to_dust(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let txid = self.transaction_id()?;

    // Enforce one swap per transaction
    if self.has_tx_hash(&txid) {
      return Err(anyhow!("Transaction already used for swap"));
    }
    
    if context.incoming_alkanes.0.is_empty() {
      return Err(anyhow!("Must send at least 1 Position token to swap"));
    }

    self.add_tx_hash(&txid)?;

    let mut response = CallResponse::default();
    let mut total_dust = 0u128;

    // Validate all incoming alkanes first
    self.validate_incoming_alkanes(&context.incoming_alkanes.0)?;

    for alkane in context.incoming_alkanes.0.iter() {
      self.add_instance(&alkane.id)?;

      total_dust = total_dust.checked_add(DUST_PER_POSITION)
        .ok_or_else(|| anyhow!("Dust amount overflow"))?;
    }

    self.increase_total_supply(total_dust)?;

    response.alkanes.0.push(AlkaneTransfer {
      id: context.myself.clone(),
      value: total_dust,
    }); 

    Ok(response)
  }

  fn dust_to_position(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let txid = self.transaction_id()?;

    // Enforce one swap per transaction
    if self.has_tx_hash(&txid) {
      return Err(anyhow!("Transaction already used for swap"));
    }
    
    if context.incoming_alkanes.0.len() != 1 {
      return Err(anyhow!("Must send $DUST to swap"));
    }

    let transfer = context.incoming_alkanes.0[0].clone();
    if transfer.id != context.myself.clone() {
      return Err(anyhow!("Supplied alkane is not $DUST"));
    }

    if transfer.value < DUST_PER_POSITION {
      return Err(anyhow!(
        "Not enough $DUST supplied to swap"
      ));
    }

    let position_count = transfer.value / DUST_PER_POSITION;
    let dust_used = position_count * DUST_PER_POSITION;
    let dust_change = transfer.value % DUST_PER_POSITION;

    let count = self.instances_count();
    if count < position_count {
      return Err(anyhow!("Not enough Position tokens available to swap"));
    }

    self.add_tx_hash(&txid)?;

    let mut response = CallResponse::default();

    self.decrease_total_supply(dust_used)?;
  
    // Position tokens
    for _ in 0..position_count {
      response.alkanes.0.push(AlkaneTransfer {
        id: self.pop_instance()?,
        value: 1u128,
      });
    }

    // Change
    if dust_change > 0 {
      response.alkanes.0.push(AlkaneTransfer {
        id: context.myself.clone(),
        value: dust_change,
      });
    }

    Ok(response)
  }

  fn mint_tokens(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let txid = self.transaction_id()?;

    // Enforce one mint per transaction
    if self.has_tx_hash(&txid) {
      return Err(anyhow!("Transaction already used for minting"));
    }

    // Check if we have reached the dust cap
    let current_supply = self.total_supply();
    if current_supply >= DUST_CAP {
      return Err(anyhow!("Dust cap of {} reached", DUST_CAP));
    }

    // Validate that caller sent valid Alkamist or position tokens
    if context.incoming_alkanes.0.is_empty() {
      return Err(anyhow!("Must send tokens to mint dust"));
    }

    let mut total_mint_amount = 0u128;
    let mut response = CallResponse::default();

    // Validate all incoming alkanes first
    self.validate_incoming_alkanes(&context.incoming_alkanes.0)?;

    for alkane in context.incoming_alkanes.0.iter() {
      // Calculate mint amount based on token value
      let mint_amount = alkane.value.checked_mul(DUST_PER_POSITION)
        .ok_or_else(|| anyhow!("Mint amount overflow"))?;

      // Check if minting this amount would exceed cap
      let new_supply = current_supply.checked_add(total_mint_amount).and_then(|s| s.checked_add(mint_amount));
      if new_supply.is_none() || new_supply.unwrap() > DUST_CAP {
        return Err(anyhow!("Minting would exceed dust cap"));
      }

      total_mint_amount = total_mint_amount.checked_add(mint_amount)
        .ok_or_else(|| anyhow!("Total mint amount overflow"))?;
    }

    // Record transaction to prevent replay
    self.add_tx_hash(&txid)?;

    // Update total supply
    self.increase_total_supply(total_mint_amount)?;

    // Return minted dust tokens
    response.alkanes.0.push(AlkaneTransfer {
      id: context.myself.clone(),
      value: total_mint_amount,
    });

    Ok(response)
  }

  fn get_position_stack_count(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    response.data = self.instances_count().to_le_bytes().to_vec();

    Ok(response)
  }

  fn get_position_stack(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let count = self.instances_count();
    let mut position_ids = Vec::new();

    for i in 0..count {
      let instance_id = self.lookup_instance(i)?;
      let mut bytes = Vec::with_capacity(32);
      bytes.extend_from_slice(&instance_id.block.to_le_bytes());
      bytes.extend_from_slice(&instance_id.tx.to_le_bytes());
      position_ids.push(bytes);
    }

    let mut flattened = Vec::new();
    for bytes in position_ids {
      flattened.extend(bytes);
    }

    response.data = flattened;
    Ok(response)
  }

  fn get_position_stack_json(&self) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);

    let count = self.instances_count();
    let mut position_ids = Vec::new();

    for i in 0..count {
      let instance_id = self.lookup_instance(i)?;
      position_ids.push(format!("{}:{}", instance_id.block, instance_id.tx));
    }

    response.data = serde_json::to_string(&position_ids)?.into_bytes();
    Ok(response)
  }

  fn instances_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/instances")
  }

  fn instances_count(&self) -> u128 {
    self.instances_pointer().get_value::<u128>()
  }

  fn set_instances_count(&self, count: u128) {
    self.instances_pointer().set_value::<u128>(count);
  }

  fn add_instance(&self, instance_id: &AlkaneId) -> Result<u128> {
    let count = self.instances_count();
    let new_count = count.checked_add(1)
      .ok_or_else(|| anyhow!("instances count overflow"))?;

    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(&instance_id.block.to_le_bytes());
    bytes.extend_from_slice(&instance_id.tx.to_le_bytes());

    let bytes_vec = new_count.to_le_bytes().to_vec();
    let mut instance_pointer = self.instances_pointer().select(&bytes_vec);
    instance_pointer.set(Arc::new(bytes));
    
    self.set_instances_count(new_count);
    
    Ok(new_count)
  }

  fn pop_instance(&self) -> Result<AlkaneId> {
    let count = self.instances_count();

    let new_count = count.checked_sub(1)
      .ok_or_else(|| anyhow!("instances count underflow"))?;

    let instance_id = self.lookup_instance(count - 1)?;
    
    // Remove the instance by setting it to empty
    let bytes_vec = count.to_le_bytes().to_vec();
    let mut instance_pointer = self.instances_pointer().select(&bytes_vec);
    instance_pointer.set(Arc::new(Vec::new()));
    
    self.set_instances_count(new_count);
    
    Ok(instance_id)
  }

  fn lookup_instance(&self, index: u128) -> Result<AlkaneId> {
    let bytes_vec = (index + 1).to_le_bytes().to_vec();
    let instance_pointer = self.instances_pointer().select(&bytes_vec);
    
    let bytes = instance_pointer.get();
    if bytes.len() != 32 {
      return Err(anyhow!("Invalid instance data length"));
    }

    let block_bytes = &bytes[..16];
    let tx_bytes = &bytes[16..];

    let block = u128::from_le_bytes(block_bytes.try_into().unwrap());
    let tx = u128::from_le_bytes(tx_bytes.try_into().unwrap());

    Ok(AlkaneId { block, tx })
  }

  fn transaction_id(&self) -> Result<Txid> {
    Ok(
      consensus_decode::<Transaction>(&mut std::io::Cursor::new(self.transaction()))?
        .compute_txid(),
    )
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
  impl AlkaneResponder for DustSwap {
    type Message = DustSwapMessage;
  }
}
