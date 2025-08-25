use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_runtime::{auth::AuthenticatedResponder, declare_alkane, message::MessageDispatch};
#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_std_factory_support::MintableToken;
use alkanes_support::{context::Context, parcel::AlkaneTransfer, response::CallResponse};
use anyhow::{anyhow, Result};
use metashrew_support::compat::{to_arraybuffer_layout, to_passback_ptr};

#[derive(Default)]
pub struct FreeMint(()); // Renamed from OwnedToken

impl MintableToken for FreeMint {} // Renamed from OwnedToken

impl AuthenticatedResponder for FreeMint {} // Renamed from OwnedToken

#[derive(MessageDispatch)]
enum FreeMintMessage { // Renamed from OwnedTokenMessage
    #[opcode(0)]
    Initialize {
        auth_token_units: u128, // Added auth_token_units
        token_units: u128,
        value_per_mint: u128,
        cap: u128,
        name_part1: u128,
        name_part2: u128,
        symbol: u128,
    },

    #[opcode(77)] // Changed opcode for MintTokens
    MintTokens { token_units: u128 }, // Renamed from Mint

    #[opcode(88)] // Changed opcode for Burn
    Burn {},

    #[opcode(99)] // Changed opcode for GetName
    #[returns(String)]
    GetName,

    #[opcode(100)] // Changed opcode for GetSymbol
    #[returns(String)]
    GetSymbol,

    #[opcode(101)] // Changed opcode for GetTotalSupply
    #[returns(u128)]
    GetTotalSupply,

    #[opcode(1000)] // Changed opcode for GetData
    #[returns(Vec<u8>)]
    GetData,
}

impl FreeMint { // Renamed from OwnedToken
    fn initialize(
        &self,
        auth_token_units: u128, // Added auth_token_units
        token_units: u128,
        value_per_mint: u128,
        cap: u128,
        name_part1: u128,
        name_part2: u128,
        symbol: u128,
    ) -> Result<CallResponse> {
        self.observe_initialization()?;
        let context = self.context()?;
        let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes.clone());

        <Self as MintableToken>::set_name_and_symbol_str(
            self,
            format!("{:x}{:x}", name_part1, name_part2),
            format!("{:x}", symbol),
        );

        response
            .alkanes
            .0
            .push(self.deploy_auth_token(auth_token_units)?); // Call deploy_auth_token

        response.alkanes.0.push(AlkaneTransfer {
            id: context.myself.clone(),
            value: token_units,
        });

        Ok(response)
    }

    fn mint_tokens(&self, token_units: u128) -> Result<CallResponse> { // Renamed from mint
        let context = self.context()?;
        let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes.clone());

        self.only_owner()?;

        // Call the mint method from the MintableToken trait
        let transfer = <Self as MintableToken>::mint(self, &context, token_units)?;
        response.alkanes.0.push(transfer);

        Ok(response)
    }

    fn burn(&self) -> Result<CallResponse> {
        let context = self.context()?;
        if context.incoming_alkanes.0.len() != 1 {
            return Err(anyhow!("Input must be 1 alkane"));
        }
        if context.myself != context.incoming_alkanes.0[0].id {
            return Err(anyhow!("Input must be owned token"));
        }

        self.decrease_total_supply(context.incoming_alkanes.0[0].value)?;

        Ok(CallResponse::default())
    }

    fn get_name(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes.clone());

        response.data = self.name().into_bytes().to_vec();

        Ok(response)
    }

    fn get_symbol(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes.clone());

        response.data = self.symbol().into_bytes().to_vec();

        Ok(response)
    }

    fn get_total_supply(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes.clone());

        response.data = self.total_supply().to_le_bytes().to_vec();

        Ok(response)
    }

    fn get_data(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes.clone());

        response.data = self.data();

        Ok(response)
    }
}

impl AlkaneResponder for FreeMint {} // Renamed from OwnedToken

// Use the new macro format
declare_alkane! {
    impl AlkaneResponder for FreeMint { // Renamed from OwnedToken
        type Message = FreeMintMessage; // Renamed from OwnedTokenMessage
    }
}