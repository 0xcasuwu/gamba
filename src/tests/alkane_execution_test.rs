use anyhow::Result;
use wasm_bindgen_test::wasm_bindgen_test;
use std::str::FromStr;
use bitcoin::{transaction::Version, ScriptBuf, Sequence};
use bitcoin::{Address, Amount, Block, Transaction, TxIn, TxOut, Witness};
use bitcoin::blockdata::transaction::OutPoint;
use ordinals::Runestone;
use alkanes::tests::helpers::clear;
use alkanes::indexer::index_block;
use alkanes::message::AlkaneMessageContext;
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use alkanes::tests::helpers as alkane_helpers;
use alkanes::view;
use alkanes_support::proto::alkanes::AlkanesTrace;
use protorune::test_helpers::{get_btc_network, ADDRESS1};
use protorune::{test_helpers as protorune_helpers, message::MessageContext};
use protorune_support::{balance_sheet::{BalanceSheetOperations, ProtoruneRuneId}, protostone::{Protostone, ProtostoneEdict}};
use protorune::{balance_sheet::load_sheet, tables::RuneTable};
use protorune::protostone::Protostones;
use metashrew_support::{index_pointer::KeyValuePointer, utils::consensus_encode};
use prost::Message;
use metashrew_core::{println, stdio::stdout};
use std::fmt::Write;
use crate::precompiled::{factory_build, coupon_template_build};
use alkanes::precompiled::free_mint_build;

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
fn test_alkane_execution_with_traces() -> Result<()> {
    println!("🔥 ALKANE EXECUTION TEST WITH ROBUST TRACES");
    println!("===========================================");
    
    clear();
    
    // PHASE 1: Deploy contracts to generate alkane creation traces
    println!("\n📦 PHASE 1: Template Deployment");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            free_mint_build::get_bytes(),
            coupon_template_build::get_bytes(),
            factory_build::get_bytes(),
        ].into(),
        [
            vec![3u128, 797u128, 101u128, 1000000u128, 100000u128, 1000000000u128, 0x54455354, 0x434f494e, 0x545354], 
            vec![3u128, 0x601, 10u128 ],
            vec![3u128, 0x701, 0u128, 144u128, 4u128, 0x601u128],
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;

    // Check every single transaction and vout for trace data
    println!("\n🔍 COMPREHENSIVE TRACE ANALYSIS:");
    for (i, tx) in template_block.txdata.iter().enumerate() {
        println!("   📋 TX {} ({})", i, tx.compute_txid());
        for vout in 0..tx.output.len() {
            let outpoint = OutPoint {
                txid: tx.compute_txid(),
                vout: vout as u32,
            };
            
            let trace_data = view::trace(&outpoint)?;
            println!("      • vout {}: raw_len={}", vout, trace_data.len());
            
            if trace_data.len() > 0 {
                // Try to decode the trace data
                match AlkanesTrace::decode(&trace_data[..]) {
                    Ok(alkanes_trace) => {
                        let trace_result: alkanes_support::trace::Trace = alkanes_trace.into();
                        let trace_guard = trace_result.0.lock().unwrap();
                        if !trace_guard.is_empty() {
                            println!("        TRACE EVENTS: {:?}", *trace_guard);
                        } else {
                            println!("        TRACE: decoded but empty");
                        }
                    }
                    Err(e) => {
                        println!("        TRACE DECODE ERROR: {:?}", e);
                        println!("        RAW BYTES: {:?}", trace_data);
                    }
                }
            }
        }
    }

    // PHASE 2: Try to trigger actual alkane execution
    println!("\n🪙 PHASE 2: Free-Mint Call to Trigger Execution");
    let free_mint_call = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    6u128, 797u128, 0u128,  // Call the free-mint
                                    1000000u128,            
                                    100000u128,             
                                    1000000000u128,         
                                    0x54455354,             
                                    0x434f494e,             
                                    0x545354,               
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
    index_block(&free_mint_call, 1)?;

    // Check traces from the alkane call
    println!("\n🔍 ALKANE CALL TRACE ANALYSIS:");
    let tx = &free_mint_call.txdata[0];
    println!("   📋 Call TX ({})", tx.compute_txid());
    for vout in 0..tx.output.len() {
        let outpoint = OutPoint {
            txid: tx.compute_txid(),
            vout: vout as u32,
        };
        
        let trace_data = view::trace(&outpoint)?;
        println!("      • vout {}: raw_len={}", vout, trace_data.len());
        
        if trace_data.len() > 0 {
            match AlkanesTrace::decode(&trace_data[..]) {
                Ok(alkanes_trace) => {
                    let trace_result: alkanes_support::trace::Trace = alkanes_trace.into();
                    let trace_guard = trace_result.0.lock().unwrap();
                    if !trace_guard.is_empty() {
                        println!("        🎯 ALKANE EXECUTION TRACE: {:?}", *trace_guard);
                    } else {
                        println!("        TRACE: decoded but empty");
                    }
                }
                Err(e) => {
                    println!("        TRACE DECODE ERROR: {:?}", e);
                    println!("        RAW BYTES (first 50): {:?}", &trace_data[..trace_data.len().min(50)]);
                }
            }
        }
    }

    println!("\n🎯 TRACE SUMMARY:");
    println!("   📊 Template deployment traces examined");
    println!("   📊 Alkane execution call traces examined");  
    println!("   📊 Raw trace data lengths reported");
    println!("   📊 Decode attempts made on all non-empty traces");

    Ok(())
}
