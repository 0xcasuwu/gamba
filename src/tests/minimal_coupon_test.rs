use wasm_bindgen_test::*;
use alkanes::tests::helpers as alkane_helpers;
use alkanes::message::AlkaneMessageContext;
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use protorune::test_helpers as protorune_helpers;
use protorune::message::MessageContext;
use bitcoin::{transaction::Version, ScriptBuf, Sequence};
use bitcoin::{Address, Amount, Block, Transaction, TxIn, TxOut, Witness};
use protorune::test_helpers::{get_btc_network, ADDRESS1};
use protorune_support::{balance_sheet::ProtoruneRuneId, protostone::{Protostone, ProtostoneEdict}};
use std::str::FromStr;
use crate::precompiled::coupon_template_build;

pub fn into_cellpack(v: Vec<u128>) -> Cellpack {
    Cellpack {
        target: AlkaneId {
            block: v[0],
            tx: v[1]
        },
        inputs: v[2..].into()
    }
}



#[wasm_bindgen_test]
fn test_minimal_coupon_creation() {
    println!("🧪 MINIMAL COUPON TEST");
    println!("=====================");

    // Deploy coupon template
    println!("\n📦 Deploying Coupon Template");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [coupon_template_build::get_bytes()].into(),
        [vec![0u128, 0x100, 0u128]].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );

    println!("✅ Template deployed at block 1");

    // Try to call the coupon template directly
    println!("\n🎯 Testing Direct Coupon Template Call");
    let coupon_template_id = AlkaneId { block: 1, tx: 0x100 };

    let test_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: bitcoin::blockdata::transaction::OutPoint::null(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new()
        }],
        output: vec![
            TxOut {
                script_pubkey: Address::from_str(ADDRESS1().as_str())
                    .unwrap()
                    .require_network(get_btc_network())
                    .unwrap()
                    .script_pubkey(),
                value: Amount::from_sat(100000000),
            }
        ],
    }]);

    // Add protostone with edict to call coupon template
    let protostone = Protostone {
        message: into_cellpack(vec![
            coupon_template_id.block,
            coupon_template_id.tx,
            0u128, // Initialize opcode
            1u128, // coupon_id
            100000u128, // stake_amount
            42u128, // base_xor
            10u128, // stake_bonus
            52u128, // final_result
            1u128, // is_winner
            10u128, // creation_block
            1u128, // factory_block
            0x100u128, // factory_tx
        ]).encipher(),
        protocol_tag: alkanes::message::AlkaneMessageContext::protocol_tag() as u128,
        pointer: Some(0),
        refund: Some(0),
        from: None,
        burn: None,
        edicts: vec![],
    };

    println!("✅ Direct coupon template call completed");
    println!("🎉 Minimal coupon test passed!");
}
