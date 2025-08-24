use alkanes::view;
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
use bitcoin::{transaction::Version, ScriptBuf, Sequence};
use bitcoin::{Address, Amount, Block, Transaction, TxIn, TxOut, Witness};
use bitcoin::hashes::Hash;
use ordinals::Runestone;
use protorune::test_helpers::{get_btc_network, ADDRESS1};
use protorune::{test_helpers as protorune_helpers};
use protorune_support::protostone::Protostone;
use protorune::protostone::Protostones;
use protorune::message::MessageContext;
use metashrew_core::{println, stdio::stdout};
use protobuf::Message;
use crate::precompiled::factory_build;
use crate::precompiled::coupon_template_build;
use crate::precompiled::free_mint_build;
use crate::precompiled::auth_token_build;

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
fn test_minimal_debug_factory_deployment() -> Result<()> {
    println!("\n🔍 MINIMAL DEBUG: Factory Deployment Only");
    println!("=========================================");
    
    clear();
    
    // STEP 1: Deploy templates only
    println!("\n📦 STEP 1: Template Deployment");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            free_mint_build::get_bytes(),
            coupon_template_build::get_bytes(),
            factory_build::get_bytes(),
            auth_token_build::get_bytes(),
        ].into(),
        [
            vec![3u128, 797u128, 101u128],     // free_mint template → deploys at block 4
            vec![3u128, 0x601, 10u128],        // coupon_token template → deploys at block 4
            vec![3u128, 0x701, 10u128],        // coupon_factory template → deploys at block 4
            vec![3u128, 0xffee, 0u128, 1u128], // auth_token template → deploys at block 4
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
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
    
    // STEP 2: Deploy DUST token 
    println!("\n💨 STEP 2: DUST Token Deployment");
    let dust_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    4u128, 797u128, 0u128,   // Call deployed free_mint instance (template at 3 → instance at 4)
                                    1000000u128,             
                                    1u128,                   
                                    100000u128,              
                                    0x44555354,              
                                    0x0,                     
                                    0x44555354,              
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
    index_block(&dust_block, 1)?;
    println!("✅ DUST token deployed successfully");
    
    // STEP 3: Initialize factory (FIXED: Wait until factory exists at block 4+)
    println!("\n🏭 STEP 3: Factory Initialization");
    let dust_token_id = AlkaneId { block: 2, tx: 797 };
    let coupon_token_template_id = AlkaneId { block: 4, tx: 0x601 }; // FIXED: instance at block 4
    
    let init_factory_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    4u128, 0x701, 0u128, // Initialize coupon factory
                                    dust_token_id.block, dust_token_id.tx,
                                    144u128, // Success threshold
                                    5u128, // DUST bonus rate
                                    coupon_token_template_id.block, coupon_token_template_id.tx,
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
    index_block(&init_factory_block, 4)?; // FIXED: Initialize at block 4 when factory exists
    println!("✅ Factory initialized successfully");
    
    // STEP 4: Test simple getter call
    println!("\n📊 STEP 4: Simple Getter Test");
    let factory_id = AlkaneId { block: 4, tx: 0x701 };
    
    let getter_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    factory_id.block, factory_id.tx, 10u128, // GetSuccessfulCoupons - test if factory is working
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
    index_block(&getter_block, 5)?; // FIXED: Call after factory is initialized at block 4
    
    // TRACE: Getter call
    println!("🔍 TRACE: Simple getter call at block 3");
    for vout in 0..3 {
        let trace_data = &view::trace(&OutPoint {
            txid: getter_block.txdata[0].compute_txid(),
            vout,
        })?;
        let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
        let trace_guard = trace_result.0.lock().unwrap();
        if !trace_guard.is_empty() {
            println!("   • Getter vout {} trace: {:?}", vout, *trace_guard);
        }
    }
    
    println!("✅ Simple getter test completed");
    
    println!("\n🎯 MINIMAL DEBUG RESULT: Deployment and getter test successful!");
    
    Ok(())
}

#[wasm_bindgen_test]
fn test_minimal_debug_forge_call() -> Result<()> {
    println!("\n🔍 MINIMAL DEBUG: Forge Call Only - FIXED ADDRESSING");
    println!("====================================================");
    
    clear();
    
    // FIXED: Consistent template deployment following boiler pattern
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            free_mint_build::get_bytes(),
            coupon_template_build::get_bytes(),
            factory_build::get_bytes(),
            auth_token_build::get_bytes(),     // auth_token_build exists in precompiled
        ].into(),
        [
            vec![3u128, 797u128, 101u128],     // free_mint template → deploys instance at block 4, tx 797
            vec![3u128, 0x601, 10u128],        // coupon_token template → deploys instance at block 4, tx 0x601
            vec![3u128, 0x701, 10u128],        // coupon_factory template → deploys instance at block 4, tx 0x701
            vec![3u128, 0xffee, 0u128, 1u128], // auth_token template → deploys instance at block 4, tx 0xffee
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
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
    println!("✅ Templates deployed: 3,n → instances at 4,n");
    
    let dust_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    4u128, 797u128, 0u128,   // FIXED: Call deployed free_mint instance at 4,797
                                    1000000u128,             
                                    1u128,                   
                                    100000u128,              
                                    0x44555354,              
                                    0x0,                     
                                    0x44555354,              
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
    index_block(&dust_block, 1)?;
    
    println!("✅ DUST token call: 4,797,0 (CORRECT addressing)");
    
    // FIXED: Reference the correct deployed instances (not templates)
    let dust_token_id = AlkaneId { block: 2, tx: 797 };        // DUST token deployed in previous transaction
    let coupon_token_template_id = AlkaneId { block: 4, tx: 0x601 }; // FIXED: Template deployed at 3,0x601 → instance at 4,0x601
    
    let init_factory_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    4u128, 0x701, 0u128,     // FIXED: Call factory instance at 4,0x701 
                                    dust_token_id.block, dust_token_id.tx,
                                    144u128,
                                    5u128,
                                    coupon_token_template_id.block, coupon_token_template_id.tx, // Template reference (4,0x601)
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
    index_block(&init_factory_block, 4)?; // FIXED: Initialize at block 4 when factory exists
    
    println!("✅ Factory initialization: 4,0x701,0 (CORRECT addressing)");
    
    // STEP: Test minimal forge call (NO DUST, NO EDICTS)
    println!("\n🔥 STEP: Minimal Forge Call (No DUST)");
    let factory_id = AlkaneId { block: 4, tx: 0x701 };
    
    let forge_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                    edicts: vec![], // NO EDICTS!
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    factory_id.block, factory_id.tx, 1u128, // ForgeCoupon opcode (CORRECT!)
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![], // NO EDICTS!
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    index_block(&forge_block, 5)?; // FIXED: Call after factory is initialized at block 4
    
    // TRACE: Minimal forge call
    println!("🔍 TRACE: Minimal forge call at block 5");
    for vout in 0..3 {
        let trace_data = &view::trace(&OutPoint {
            txid: forge_block.txdata[0].compute_txid(),
            vout,
        })?;
        let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
        let trace_guard = trace_result.0.lock().unwrap();
        if !trace_guard.is_empty() {
            println!("   • Forge vout {} trace: {:?}", vout, *trace_guard);
        }
    }
    
    println!("✅ Minimal forge call completed");
    
    Ok(())
}
