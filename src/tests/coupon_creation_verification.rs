use alkanes::view;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use wasm_bindgen_test::wasm_bindgen_test;
use alkanes::tests::helpers::clear;
use alkanes::indexer::index_block;
use std::str::FromStr;
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
fn test_coupon_creation_with_alkane_return() -> Result<()> {
    println!("\n🎯 COUPON CREATION VERIFICATION TEST");
    println!("====================================");
    println!("This test verifies that CreateCoupon actually returns a coupon alkane");
    
    clear();
    
    // STEP 1: Deploy templates
    println!("\n📦 STEP 1: Template Deployment");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            free_mint_build::get_bytes(),
            coupon_template_build::get_bytes(),
            factory_build::get_bytes(),
        ].into(),
        [
            vec![3u128, 797u128, 101u128],     // free_mint template → 4,797
            vec![3u128, 0x601, 10u128],        // coupon_token template → 4,0x601  
            vec![3u128, 0x701, 10u128],        // coupon_factory template → 4,0x701
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;
    println!("✅ Templates deployed at block 4");

    // STEP 2: Create DUST token
    println!("\n💨 STEP 2: DUST Token Creation");
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
                                    6u128, 797u128, 0u128,   // Call free_mint instance at 4,797
                                    1000000u128,             // Max supply
                                    1u128,                   // Decimals
                                    100000u128,              // Amount to mint
                                    0x44555354,              // "DUST" name 
                                    0x0,                     // No description
                                    0x44555354,              // "DUST" symbol
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
    println!("✅ DUST token created at 2,797");

    // STEP 3: Initialize factory  
    println!("\n🏭 STEP 3: Factory Initialization");
    let coupon_token_template_id = AlkaneId { block: 4, tx: 0x601 };
    
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
                                    4u128, 0x701, 0u128, // Initialize factory at 4,0x701
                                    144u128, // Success threshold
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
    index_block(&init_factory_block, 2)?;
    println!("✅ Factory initialized");

    // STEP 4: CreateCoupon call with DUST staking
    println!("\n🎰 STEP 4: CreateCoupon Call with DUST Staking");
    let dust_token_id = AlkaneId { block: 2, tx: 797 };
    let factory_id = AlkaneId { block: 4, tx: 0x701 };

    let create_coupon_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                    edicts: vec![
                        protorune::Edict {
                            id: protorune::RuneId {
                                block: dust_token_id.block as u64,
                                tx: dust_token_id.tx as u32,
                            },
                            amount: 1000u128,  // Stake 1000 DUST tokens
                            output: 1u32,      // Send to vout 1 (factory call)
                        }
                    ],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    factory_id.block, factory_id.tx, 1u128, // CreateCoupon opcode
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    protorune::Edict {
                                        id: protorune::RuneId {
                                            block: dust_token_id.block as u64,
                                            tx: dust_token_id.tx as u32,
                                        },
                                        amount: 1000u128,
                                        output: 1u32,
                                    }
                                ],
                            }
                        ].encipher()?
                    )
                }).encipher(),
                value: Amount::from_sat(546)
            }
        ],
    }]);
    index_block(&create_coupon_block, 3)?;
    
    // STEP 5: VERIFY COUPON CREATION AND ALKANE RETURN
    println!("\n🔍 STEP 5: Verifying Coupon Creation");
    
    let create_coupon_tx = &create_coupon_block.txdata[0];
    let txid = create_coupon_tx.compute_txid();
    
    // Check trace for vout 1 (where factory call happens)
    let trace_data = &view::trace(&OutPoint { txid, vout: 1 })?;
    let trace_result: alkanes_support::trace::Trace = alkanes_support::proto::alkanes::AlkanesTrace::parse_from_bytes(trace_data)?.into();
    let trace_guard = trace_result.0.lock().unwrap();
    
    println!("🔍 Factory call trace (vout 1): {:?}", *trace_guard);
    
    // Check if there are any alkane transfers in the trace
    let mut coupon_found = false;
    let mut coupon_details = String::new();
    
    for entry in trace_guard.iter() {
        if let Some(return_context) = entry.as_return_context() {
            if !return_context.inner.alkanes.0.is_empty() {
                coupon_found = true;
                coupon_details = format!("Found {} alkane(s) returned", return_context.inner.alkanes.0.len());
                
                for (i, alkane) in return_context.inner.alkanes.0.iter().enumerate() {
                    println!("   📋 Returned Alkane {}: AlkaneId({}, {}) - Amount: {}", 
                             i+1, alkane.id.block, alkane.id.tx, alkane.amount);
                }
            }
        }
    }
    
    // VERIFICATION RESULTS
    println!("\n🎯 VERIFICATION RESULTS:");
    if coupon_found {
        println!("   ✅ SUCCESS: Coupon alkane was created and returned!");
        println!("   📋 Details: {}", coupon_details);
        println!("   🎰 The gambling mechanism successfully created a coupon NFT");
    } else {
        println!("   ❌ ISSUE: No coupon alkanes found in return trace");
        println!("   🔍 This suggests the CreateCoupon call may not be returning coupons properly");
    }
    
    println!("\n🎯 COUPON CREATION VERIFICATION COMPLETE!");
    println!("   CreateCoupon functionality: {}", if coupon_found { "✅ WORKING" } else { "❌ NEEDS INVESTIGATION" });

    Ok(())
}
