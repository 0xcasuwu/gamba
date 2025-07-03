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
use svg_generator::{SvgGenerator, WandData};

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
pub struct WandToken(());

impl AlkaneResponder for WandToken {}

#[derive(MessageDispatch)]
enum WandTokenMessage {
    #[opcode(0)]
    Initialize {
        forge_id: u128,
        dust_amount: u128,
        base_xor: u128,
        dust_bonus: u128,
        final_result: u128,
        creation_block: u128,
        factory_block: u128,
        factory_tx: u128,
    },

    #[opcode(10)]
    #[returns(u128)]
    GetForgeId,

    #[opcode(11)]
    #[returns(u128)]
    GetDustAmount,

    #[opcode(12)]
    #[returns(u128)]
    GetBaseXor,

    #[opcode(13)]
    #[returns(u128)]
    GetDustBonus,

    #[opcode(14)]
    #[returns(u128)]
    GetFinalResult,

    #[opcode(15)]
    #[returns(u128)]
    GetCreationBlock,

    #[opcode(16)]
    #[returns(AlkaneId)]
    GetFactoryId,

    #[opcode(17)]
    #[returns((u128, u128, u128, u128, u128, u128))]
    GetAllForgeDetails,

    #[opcode(18)]
    #[returns(String)]
    GetOrbitalType,

    #[opcode(19)]
    #[returns(String)]
    GetOrbitalPower,

    /// Get the token name
    #[opcode(99)]
    #[returns(String)]
    GetName,

    /// Get the token symbol
    #[opcode(100)]
    #[returns(String)]
    GetSymbol,

    /// Get the SVG data
    #[opcode(1000)]
    #[returns(Vec<u8>)]
    GetData,

    /// Get the content type
    #[opcode(1001)]
    #[returns(String)]
    GetContentType,

    /// Get the attributes (metadata)
    #[opcode(1002)]
    #[returns(String)]
    GetAttributes,
}

impl Token for WandToken {
    fn name(&self) -> String {
        String::from_utf8(name_pointer().get().as_ref().clone())
            .unwrap_or_else(|_| {
                // Safe fallback that doesn't rely on storage that might not be initialized
                let forge_id = self.forge_id_pointer().get_value::<u128>();
                format!("Orbital #{}", forge_id)
            })
    }

    fn symbol(&self) -> String {
        String::from_utf8(symbol_pointer().get().as_ref().clone())
            .unwrap_or_else(|_| {
                // Safe fallback that doesn't rely on storage that might not be initialized
                let forge_id = self.forge_id_pointer().get_value::<u128>();
                format!("ORB-{}", forge_id)
            })
    }
}

impl WandToken {
    fn initialize(
        &self,
        forge_id: u128,
        dust_amount: u128,
        base_xor: u128,
        dust_bonus: u128,
        final_result: u128,
        creation_block: u128,
        factory_block: u128,
        factory_tx: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::default();

        self.observe_initialization()?;

        // Set name and symbol based on orbital properties
        let orbital_type = self.determine_orbital_type(final_result as u8);
        let name_string = format!("{} Orbital #{}", orbital_type, forge_id);
        let symbol_string = format!("ORB-{}", forge_id);
        
        name_pointer().set(Arc::new(name_string.as_bytes().to_vec()));
        symbol_pointer().set(Arc::new(symbol_string.as_bytes().to_vec()));

        // Store immutable forge details - CORRECT PATTERN USING FACTORY PARAMETERS
        let factory_id = AlkaneId { block: factory_block, tx: factory_tx };
        self.set_factory_id(&factory_id);
        self.set_forge_id(forge_id);
        self.set_dust_amount(dust_amount);
        self.set_base_xor(base_xor as u8);
        self.set_dust_bonus(dust_bonus as u8);
        self.set_final_result(final_result as u8);
        self.set_creation_block(creation_block);

        // Return exactly 1 orbital token
        response.alkanes.0.push(AlkaneTransfer {
            id: context.myself.clone(),
            value: 1u128,
        });

        Ok(response)
    }

    fn determine_orbital_type(&self, final_result: u8) -> &'static str {
        match final_result {
            250..=255 => "Legendary",
            230..=249 => "Epic",
            200..=229 => "Rare",
            170..=199 => "Uncommon",
            145..=169 => "Common",
            _ => "Failed", // Should not happen for successful orbitals
        }
    }

    fn determine_orbital_power(&self, final_result: u8, dust_amount: u128) -> String {
        let base_power = match final_result {
            250..=255 => "Quantum Resonance",
            230..=249 => "Stellar Convergence",
            200..=229 => "Cosmic Alignment",
            170..=199 => "Gravitational Pull",
            145..=169 => "Orbital Drift",
            _ => "Inert",
        };
        
        // Add dust modifier
        let dust_modifier = if dust_amount >= 10000 {
            " (DUST-Amplified)"
        } else if dust_amount >= 5000 {
            " (DUST-Enhanced)"
        } else if dust_amount >= 1000 {
            " (DUST-Touched)"
        } else {
            ""
        };
        
        format!("{}{}", base_power, dust_modifier)
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
    fn get_forge_id(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.forge_id().to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_dust_amount(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.dust_amount().to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_base_xor(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = (self.base_xor() as u128).to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_dust_bonus(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = (self.dust_bonus() as u128).to_le_bytes().to_vec();
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


    fn get_all_forge_details(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Return core forge details
        let forge_id = self.forge_id();
        let dust_amount = self.dust_amount();
        let base_xor = self.base_xor() as u128;
        let dust_bonus = self.dust_bonus() as u128;
        let final_result = self.final_result() as u128;
        let creation_block = self.creation_block();

        // Pack all values into a single byte array
        // Each value is 16 bytes (128 bits) - 6 values total
        let mut data = Vec::with_capacity(16 * 6);
        data.extend_from_slice(&forge_id.to_le_bytes());
        data.extend_from_slice(&dust_amount.to_le_bytes());
        data.extend_from_slice(&base_xor.to_le_bytes());
        data.extend_from_slice(&dust_bonus.to_le_bytes());
        data.extend_from_slice(&final_result.to_le_bytes());
        data.extend_from_slice(&creation_block.to_le_bytes());

        response.data = data;
        Ok(response)
    }

    fn get_orbital_type(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let orbital_type = self.determine_orbital_type(self.final_result());
        response.data = orbital_type.as_bytes().to_vec();

        Ok(response)
    }

    fn get_orbital_power(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let orbital_power = self.determine_orbital_power(self.final_result(), self.dust_amount());
        response.data = orbital_power.as_bytes().to_vec();

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

    fn forge_id_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/forge_id")
    }

    fn forge_id(&self) -> u128 {
        self.forge_id_pointer().get_value::<u128>()
    }

    fn set_forge_id(&self, forge_id: u128) {
        self.forge_id_pointer().set_value::<u128>(forge_id);
    }

    // Forge data storage functions
    fn dust_amount_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/dust_amount")
    }

    fn dust_amount(&self) -> u128 {
        self.dust_amount_pointer().get_value::<u128>()
    }

    fn set_dust_amount(&self, dust_amount: u128) {
        self.dust_amount_pointer().set_value::<u128>(dust_amount);
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

    fn dust_bonus_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/dust_bonus")
    }

    fn dust_bonus(&self) -> u8 {
        self.dust_bonus_pointer().get_value::<u8>()
    }

    fn set_dust_bonus(&self, dust_bonus: u8) {
        self.dust_bonus_pointer().set_value::<u8>(dust_bonus);
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

    /// Get the SVG data for this wand token
    fn get_data(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Prepare orbital data for SVG generation
        let wand_data = WandData {
            wand_id: self.forge_id(),
            dust_amount: self.dust_amount(),
            base_xor: self.base_xor(),
            dust_bonus: self.dust_bonus(),
            final_result: self.final_result(),
            creation_block: self.creation_block(),
            current_block: u128::from(self.height()),
            wand_type: self.determine_orbital_type(self.final_result()).to_string(),
            wand_power: self.determine_orbital_power(self.final_result(), self.dust_amount()),
        };

        // Generate the SVG
        let svg = SvgGenerator::generate_svg(wand_data)?;
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

    /// Get the attributes (metadata) for this wand token
    fn get_attributes(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Prepare orbital data for attributes generation
        let wand_data = WandData {
            wand_id: self.forge_id(),
            dust_amount: self.dust_amount(),
            base_xor: self.base_xor(),
            dust_bonus: self.dust_bonus(),
            final_result: self.final_result(),
            creation_block: self.creation_block(),
            current_block: u128::from(self.height()),
            wand_type: self.determine_orbital_type(self.final_result()).to_string(),
            wand_power: self.determine_orbital_power(self.final_result(), self.dust_amount()),
        };

        // Generate the attributes JSON
        let attributes = SvgGenerator::get_attributes(wand_data)?;
        response.data = attributes.into_bytes();

        Ok(response)
    }
}

declare_alkane! {
  impl AlkaneResponder for WandToken {
    type Message = WandTokenMessage;
  }
}
