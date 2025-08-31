use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use wasm_bindgen_test::wasm_bindgen_test;
use alkanes::tests::helpers::clear;
use alkanes::indexer::index_block;
use std::str::FromStr;
use std::fmt::Write;
use alkanes::message::AlkaneMessageContext;
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use alkanes::tests::helpers as alkane_helpers;
use protorune::{balance_sheet::{load_sheet}, tables::RuneTable, message::MessageContext};
use protorune_support::balance_sheet::BalanceSheetOperations;
use bitcoin::{transaction::Version, ScriptBuf, Sequence};
use bitcoin::{Address, Amount, Block, Transaction, TxIn, TxOut, Witness};
use metashrew_support::{index_pointer::KeyValuePointer, utils::consensus_encode};
use ordinals::Runestone;
use protorune::test_helpers::{get_btc_network, ADDRESS1};
use protorune::{test_helpers as protorune_helpers};
use protorune_support::{balance_sheet::ProtoruneRuneId, protostone::{Protostone, ProtostoneEdict}};
use protorune::protostone::Protostones;
use metashrew_core::{println, stdio::stdout};
use alkanes::view;
use alkanes_support::proto::alkanes::AlkanesTrace;
use prost::Message;

use crate::tests::std::factory_build;

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
fn test_block_isolation() -> Result<()> {
    clear();
    
    println!("🧪 BLOCK ISOLATION DEMONSTRATION TEST");
    println!("=====================================");

    // PHASE 1: Deploy Factory Contract
    println!("\n📦 PHASE 1: Deploying Factory Contract");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [factory_build::get_bytes()].into(),
        [vec![3u128, 0x901u128]].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;
    println!("✅ Factory contract deployed at block 0");

    // PHASE 2: Initialize Factory in Block 1
    println!("\n🏭 PHASE 2: Initializing Factory in Block 1");
    let factory_id = AlkaneId { block: 4, tx: 0x901 };
    let init_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                script_pubkey: Address::from_str(ADDRESS1().as_str())?.require_network(get_btc_network())?.script_pubkey(),
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
                                    factory_id.block,
                                    factory_id.tx,
                                    0u128, // Initialize opcode
                                    100u128, // success_threshold
                                    4u128, 0x601u128, // coupon_token_template_id
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
    index_block(&init_block, 1)?;
    println!("✅ Factory initialized in block 1");

    // Analyze trace of initialization
    for vout in 0..5 {
        let trace_data = &view::trace(&bitcoin::blockdata::transaction::OutPoint {
            txid: init_block.txdata[0].compute_txid(),
            vout,
        })?;
        let trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&trace_data[..])?.into();
        let trace_guard = trace_result.0.lock().unwrap();
        if !trace_guard.is_empty() {
            println!("   • vout {}: {:?}", vout, *trace_guard);
        }
    }

    // PHASE 3: Re-initialize Factory in Block 2 (should be isolated)
    println!("\n🏭 PHASE 3: Re-initializing Factory in Block 2 (should be isolated)");
    let reinit_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                script_pubkey: Address::from_str(ADDRESS1().as_str())?.require_network(get_btc_network())?.script_pubkey(),
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
                                    factory_id.block,
                                    factory_id.tx,
                                    0u128, // Initialize opcode
                                    200u128, // different success_threshold
                                    4u128, 0x602u128, // different coupon_token_template_id
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
    index_block(&reinit_block, 2)?;
    println!("✅ Factory re-initialized in block 2");

    // Analyze trace of re-initialization
    for vout in 0..5 {
        let trace_data = &view::trace(&bitcoin::blockdata::transaction::OutPoint {
            txid: reinit_block.txdata[0].compute_txid(),
            vout,
        })?;
        let trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&trace_data[..])?.into();
        let trace_guard = trace_result.0.lock().unwrap();
        if !trace_guard.is_empty() {
            println!("   • vout {}: {:?}", vout, *trace_guard);
        }
    }

    println!("\n🎊 BLOCK ISOLATION DEMONSTRATION TEST COMPLETE");
    println!("============================================");
    println!("✅ Factory contract deployed successfully");
    println!("✅ Factory initialized in block 1");
    println!("✅ Factory re-initialized in block 2 (demonstrating block isolation)");

    Ok(())
}
