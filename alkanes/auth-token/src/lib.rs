use alkanes_support::context::Context;
use metashrew_support::compat::to_arraybuffer_layout;

use alkanes_runtime::{
    declare_alkane, message::MessageDispatch, runtime::AlkaneResponder, token::Token,
};

use alkanes_support::{
    id::AlkaneId,
    parcel::AlkaneTransfer,
    response::CallResponse,
};

use anyhow::{anyhow, Result};

#[derive(Default)]
pub struct AuthToken(());

impl AlkaneResponder for AuthToken {}

#[derive(MessageDispatch)]
enum AuthTokenMessage {
    #[opcode(0)]
    Initialize {
        auth_level: u128,
        issuer_id: AlkaneId,
        expiry_block: u128,
    },

    #[opcode(10)]
    #[returns(u128)]
    GetAuthLevel,

    #[opcode(11)]
    #[returns(AlkaneId)]
    GetIssuer,

    #[opcode(12)]
    #[returns(u128)]
    GetExpiryBlock,

    #[opcode(13)]
    #[returns(u128)]
    IsValid,

    #[opcode(20)]
    #[returns(Vec<u8>)]
    GetAuthDetails,
}

impl Token for AuthToken {
    fn name(&self) -> String {
        let auth_level = self.auth_level();
        format!("Auth Token Level {}", auth_level)
    }

    fn symbol(&self) -> String {
        let auth_level = self.auth_level();
        format!("AUTH{}", auth_level)
    }
}

impl AuthToken {
    fn initialize(
        &self,
        auth_level: u128,
        issuer_id: AlkaneId,
        expiry_block: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::default();

        self.observe_initialization()?;

        // Store auth token parameters
        self.set_auth_level(auth_level);
        self.set_issuer_id(&issuer_id);
        self.set_expiry_block(expiry_block);
        self.set_creation_block(u128::from(self.height()));

        // Return exactly 1 auth token
        response.alkanes.0.push(AlkaneTransfer {
            id: context.myself.clone(),
            value: 1u128,
        });

        Ok(response)
    }

    fn get_auth_level(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.auth_level().to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_issuer(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let issuer_id = self.issuer_id()?;
        
        // Pack AlkaneId into response (32 bytes: 16 for block, 16 for tx)
        let mut data = Vec::with_capacity(32);
        data.extend_from_slice(&issuer_id.block.to_le_bytes());
        data.extend_from_slice(&issuer_id.tx.to_le_bytes());
        
        response.data = data;
        Ok(response)
    }

    fn get_expiry_block(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.expiry_block().to_le_bytes().to_vec();
        Ok(response)
    }

    fn is_valid(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let current_block = u128::from(self.height());
        let expiry_block = self.expiry_block();
        let is_valid = current_block < expiry_block;
        
        response.data = (if is_valid { 1u128 } else { 0u128 }).to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_auth_details(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let issuer_id = self.issuer_id()?;
        
        // Format: [auth_level (16)] + [issuer_block (16)] + [issuer_tx (16)] + [expiry_block (16)] + [creation_block (16)]
        // Total: 80 bytes
        let mut data = Vec::with_capacity(80);
        
        data.extend_from_slice(&self.auth_level().to_le_bytes());
        data.extend_from_slice(&issuer_id.block.to_le_bytes());
        data.extend_from_slice(&issuer_id.tx.to_le_bytes());
        data.extend_from_slice(&self.expiry_block().to_le_bytes());
        data.extend_from_slice(&self.creation_block().to_le_bytes());
        
        response.data = data;
        Ok(response)
    }

    // Storage operations
    fn auth_level(&self) -> u128 {
        self.load_u128("/auth_level")
    }

    fn set_auth_level(&self, level: u128) {
        self.store(
            "/auth_level".as_bytes().to_vec(),
            level.to_le_bytes().to_vec(),
        );
    }

    fn issuer_id(&self) -> Result<AlkaneId> {
        let bytes = self.load("/issuer_id".as_bytes().to_vec());

        if bytes.len() < 32 {
            return Err(anyhow!("Issuer ID not set"));
        }

        Ok(AlkaneId {
            block: u128::from_le_bytes(bytes[0..16].try_into().map_err(|_| {
                anyhow!("Failed to parse issuer block ID from storage")
            })?),
            tx: u128::from_le_bytes(bytes[16..32].try_into().map_err(|_| {
                anyhow!("Failed to parse issuer tx ID from storage")
            })?),
        })
    }

    fn set_issuer_id(&self, id: &AlkaneId) {
        let mut bytes = Vec::with_capacity(32);
        bytes.extend_from_slice(&id.block.to_le_bytes());
        bytes.extend_from_slice(&id.tx.to_le_bytes());

        self.store("/issuer_id".as_bytes().to_vec(), bytes);
    }

    fn expiry_block(&self) -> u128 {
        self.load_u128("/expiry_block")
    }

    fn set_expiry_block(&self, block: u128) {
        self.store(
            "/expiry_block".as_bytes().to_vec(),
            block.to_le_bytes().to_vec(),
        );
    }

    fn creation_block(&self) -> u128 {
        self.load_u128("/creation_block")
    }

    fn set_creation_block(&self, block: u128) {
        self.store(
            "/creation_block".as_bytes().to_vec(),
            block.to_le_bytes().to_vec(),
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
}

declare_alkane! {
  impl AlkaneResponder for AuthToken {
    type Message = AuthTokenMessage;
  }
}