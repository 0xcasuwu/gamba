use anyhow::Result;
use wasm_bindgen_test::wasm_bindgen_test;
use std::{
    fmt::Write,
    str::FromStr,
};

use bitcoin::{
    Address, Amount, Block,
    blockdata::transaction::OutPoint,
    transaction::Version,
    ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness,
};

use ordinals::Runestone;

use protobuf::Message;

use metashrew_core::{println, stdio::stdout};
use metashrew_support::index_pointer::KeyValuePointer;

use alkanes::{
    indexer::index_block,
    message::AlkaneMessageContext,
    tests::helpers::{self as alkane_helpers, clear},
    view,
};
use alkanes_support::{
    cellpack::Cellpack,
    id::AlkaneId,
    proto::alkanes::AlkanesTrace,
    trace::Trace,
};

use protorune::{
    balance_sheet::load_sheet,
    message::MessageContext,
    protostone::Protostones,
    tables::RuneTable,
    test_helpers::{self as protorune_helpers, get_btc_network, ADDRESS1},
};
use protorune_support::{
    balance_sheet::{BalanceSheetOperations, ProtoruneRuneId},
    protostone::Protostone,
    utils::consensus_encode,
};

use crate::precompiled::{
    auth_token_build, coupon_template_build, factory_build, free_mint_build,
};

pub fn into_cellpack(v: Vec<u128>) -> Cellpack {
    Cellpack {
        target: AlkaneId {
            block: v[0],
            tx: v[1]
        },
        inputs: v[2..].into()
    }
}

// Helper to mint tokens from the free-mint contract
fn mint_tokens_from_free_mint_contract(free_mint_contract_id: &AlkaneId, block_height: u32) -> Result<Block> {
    let mint_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::from_height(block_height as u16),
            witness: Witness::new()
        }],
        output: vec![
            TxOut {
                script_pubkey: Address::from_str(ADDRESS1().as_str())
                    .unwrap()
                    .require_network(get_btc_network())
                    .unwrap()
                    .script_pubkey(),
                value: Amount::from_sat(546),
            },
            TxOut {
                script_pubkey: (Runestone {
                    edicts: vec![],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    free_mint_contract_id.block,
                                    free_mint_contract_id.tx,
                                    77u128 // MintTokens opcode
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![],
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    index_block(&mint_block, block_height)?;
    
    println!("✅ Minted tokens from free-mint contract at block {}", block_height);
    Ok(mint_block)
}

#[wasm_bindgen_test]
fn test_minimal_debug_factory_deployment() -> Result<()> {
    println!("\n🔍 MINIMAL DEBUG: Factory Deployment Only");
    println!("=========================================");
    
    clear();
    
    // STEP 1: Deploy templates only
    println!("\n📦 STEP 1: Template Deployment");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [free_mint_build::get_bytes()].into(),
        [vec![3u128, 797u128, 101u128]].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;
    
    // TRACE: Template block deployment
    println!("🔍 TRACE: Template block deployment at block 0");
    for (i, tx) in template_block.txdata.iter().enumerate() {
        println!("   • TX {} traces:", i);
        for vout in 0..5 {
            let trace_data = &view::trace(&OutPoint {
                txid: tx.compute_txid(),
                vout,
            })?;
            let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
            let trace_guard = trace_result.0.lock().unwrap();
            if !trace_guard.is_empty() {
                println!("     - vout {}: {:?}", vout, *trace_guard);
            }
        }
    }
    println!("✅ Templates deployed successfully");
    
    // PHASE 2: Initialize Free-Mint Contract
    println!("\n🪙 PHASE 2: Initializing Free-Mint Contract");
    let free_mint_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
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
                value: Amount::from_sat(546),
            },
            TxOut {
                script_pubkey: (Runestone {
                    edicts: vec![],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    2u128, 0u128, 0u128,  // Target template at AlkaneId { block: 2, tx: 0 }, opcode 0 (Initialize)
                                    1000000u128,            // token_units (initial supply)
                                    100000u128,             // value_per_mint  
                                    1000000000u128,         // cap (high cap for testing)
                                    0x54455354,             // name_part1 ("TEST")
                                    0x434f494e,             // name_part2 ("COIN")
                                    0x545354,               // symbol ("TST")
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![],
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    index_block(&free_mint_block, 1)?;
    
    let free_mint_contract_id = AlkaneId { block: 2, tx: 1 };
    println!("✅ Free-mint contract initialized at {:?}", free_mint_contract_id);

    // PHASE 3: Mint tokens from the Free-Mint Contract
    println!("\n💰 PHASE 3: Minting Tokens");
    let mint_block_height = 5;
    let minted_block = mint_tokens_from_free_mint_contract(&free_mint_contract_id, mint_block_height)?;
    
    // Verify minted tokens
    let mint_outpoint = OutPoint {
        txid: minted_block.txdata[0].compute_txid(),
        vout: 0z,
    };
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let minted_token_id = ProtoruneRuneId { block: 2, tx: 1 }; // Reverted to original
    let minted_amount = mint_sheet.get(&minted_token_id);

    println!("🔍 Minted token ID: {:?}", minted_token_id);
    println!("🔍 Minted amount: {}", minted_amount);

    assert!(minted_amount > 0, "Expected minted amount to be greater than 0");
    println!("✅ Tokens successfully minted and verified.");

    // TRACE: Minted block data
    println!("🔍 TRACE: Minted block data at block {}", mint_block_height);
    for (i, tx) in minted_block.txdata.iter().enumerate() {
        println!("   • TX {} traces:", i);
        for vout in 0..5 {
            let trace_data = &view::trace(&OutPoint {
                txid: tx.compute_txid(),
                vout,
            })?;
            let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
            let trace_guard = trace_result.0.lock().unwrap();
            if !trace_guard.is_empty() {
                println!("     - vout {}: {:?}", vout, *trace_guard);
            }
        }
    }

    

    // TRACE: Minted block data
    println!("🔍 TRACE: Minted block data at block {}", mint_block_height);
    for (i, tx) in minted_block.txdata.iter().enumerate() {
        println!("   • TX {} traces:", i);
        for vout in 0..5 {
            let trace_data = &view::trace(&OutPoint {
                txid: tx.compute_txid(),
                vout,
            })?;
            let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
            let trace_guard = trace_result.0.lock().unwrap();
            if !trace_guard.is_empty() {
                println!("     - vout {}: {:?}", vout, *trace_guard);
            }
        }
    }

    
    

    Ok(())
}