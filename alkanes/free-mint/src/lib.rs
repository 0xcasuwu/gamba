use alkanes_support::context::Context;
use metashrew_support::compat::to_arraybuffer_layout;

use alkanes_runtime::{
    declare_alkane, message::MessageDispatch, runtime::AlkaneResponder, token::Token,
};

use alkanes_support::{
    parcel::AlkaneTransfer,
    response::CallResponse,
};

use anyhow::{anyhow, Result};

#[derive(Default)]
pub struct FreeMint(());

impl AlkaneResponder for FreeMint {}

#[derive(MessageDispatch)]
enum FreeMintMessage {
    #[opcode(0)]
    Initialize,

    #[opcode(1)]
    Mint {
        amount: u128,
    },

    #[opcode(10)]
    #[returns(u128)]
    GetTotalSupply,

    #[opcode(11)]
    #[returns(u128)]
    GetBalance,
}

impl Token for FreeMint {
    fn name(&self) -> String {
        String::from("DUST Token")
    }

    fn symbol(&self) -> String {
        String::from("DUST")
    }
}

impl FreeMint {
    fn initialize(&self) -> Result<CallResponse> {
        let _context = self.context()?;
        let response = CallResponse::default();

        self.observe_initialization()?;

        // Initialize total supply
        self.set_total_supply(0);

        Ok(response)
    }

    fn mint(&self, amount: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::default();

        // Create tokens and send to caller
        response.alkanes.0.push(AlkaneTransfer {
            id: context.myself.clone(),
            value: amount,
        });

        // Update total supply
        let current_supply = self.total_supply();
        let new_supply = current_supply.saturating_add(amount);
        self.set_total_supply(new_supply);

        Ok(response)
    }

    fn get_total_supply(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.total_supply().to_le_bytes().to_vec();
        Ok(response)
    }

    fn get_balance(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        // Calculate total incoming balance
        let mut total_balance = 0u128;
        for transfer in &context.incoming_alkanes.0 {
            if transfer.id == context.myself {
                total_balance = total_balance.saturating_add(transfer.value);
            }
        }
        
        response.data = total_balance.to_le_bytes().to_vec();
        Ok(response)
    }

    // Storage operations
    fn total_supply(&self) -> u128 {
        let bytes = self.load("/total_supply".as_bytes().to_vec());
        if bytes.len() >= 16 {
            let bytes_array: [u8; 16] = bytes[0..16].try_into().unwrap_or([0; 16]);
            u128::from_le_bytes(bytes_array)
        } else {
            0
        }
    }

    fn set_total_supply(&self, supply: u128) {
        self.store(
            "/total_supply".as_bytes().to_vec(),
            supply.to_le_bytes().to_vec(),
        );
    }
}

declare_alkane! {
  impl AlkaneResponder for FreeMint {
    type Message = FreeMintMessage;
  }
}