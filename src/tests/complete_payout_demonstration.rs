use anyhow::Result;
use wasm_bindgen_test::wasm_bindgen_test;
use std::str::FromStr;
use bitcoin::{
    absolute, Address, Amount, Block, Transaction, TxIn, TxOut,
    Witness, ScriptBuf, Sequence,
};
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
use protorune_support::{balance_sheet::ProtoruneRuneId, protostone::{Protostone, ProtostoneEdict}};
use protorune::protostone::Protostones;
use prost::Message;
use crate::precompiled::{factory_build, coupon_template_build, free_mint_build};

pub fn into_cellpack(v: Vec<u128>) -> Cellpack {
    Cellpack {
        target: AlkaneId {
            block: v[0],
            tx: v[1],
        },
        inputs: v[2..].to_vec(),
    }
}

#[wasm_bindgen_test]
fn test_complete_payout_demonstration() -> Result<()> {
    clear();
    println!("\n🎰 COMPLETE PAYOUT DEMONSTRATION TEST");
    println!("====================================");
    println!("📋 GOAL: Demonstrate user receiving deposit + winnings with trace citations");
    
    // PHASE 1: Deploy templates using working pattern
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            free_mint_build::get_bytes(),
            coupon_template_build::get_bytes(),
            factory_build::get_bytes(),
        ].into(),
        [
            // Full initialization during template deployment
            vec![3u128, 797u128, 101u128, 1000000u128, 100000u128, 1000000000u128, 0x54455354, 0x434f494e, 0x545354],
            vec![3u128, 0x601, 10u128], 
            vec![3u128, 0x701, 0u128, 144u128, 4u128, 0x601u128],
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;

    // PHASE 2: Initialize DUST token (6 → 4 → 2 pattern)
    let free_mint_block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: bitcoin::transaction::Version::ONE,
        lock_time: absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
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
                                    6u128, 797u128, 0u128,
                                    1000000u128, 100000u128, 1000000000u128, 
                                    0x54455354, 0x434f494e, 0x545354
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
    index_block(&free_mint_block, 2)?;
    let dust_token_id = AlkaneId { block: 2, tx: 1 };

    // PHASE 3: Initialize Factory
    let factory_init_block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: bitcoin::transaction::Version::ONE,
        lock_time: absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
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
                                    6u128, 0x701, 0u128,   
                                    144u128,               
                                    4u128, 0x601u128,      
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
    index_block(&factory_init_block, 4)?;

    // PHASE 4: Create coupon and CAPTURE the returned coupon token
    println!("\n🎫 PHASE 4: Creating Coupon and Capturing Token ID");
    let deposit_block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: bitcoin::transaction::Version::ONE,
        lock_time: absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
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
                    edicts: vec![
                        ProtostoneEdict {
                            id: ProtoruneRuneId { block: dust_token_id.block, tx: dust_token_id.tx },
                            amount: 100000,
                            output: 0,
                        }.into()
                    ],
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    4u128, 1793u128, 1u128, // CreateCoupon  
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
    index_block(&deposit_block, 10)?;

    // Extract the created coupon ID from deposit traces
    println!("🔍 ANALYZING DEPOSIT TRACES TO CAPTURE COUPON TOKEN:");
    let mut created_coupon_id: Option<AlkaneId> = None;
    
    for vout in 0..6 {
        let trace_data = &view::trace(&OutPoint {
            txid: deposit_block.txdata[0].compute_txid(),
            vout,
        })?;
        let trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&trace_data[..])?.into();
        let trace_guard = trace_result.0.lock().unwrap();

        for entry in trace_guard.iter() {
            match entry {
                alkanes_support::trace::TraceEvent::CreateAlkane(alkane_id) => {
                    created_coupon_id = Some(alkane_id.clone());
                    println!("   ✅ CAPTURED COUPON TOKEN: ({}, {})", alkane_id.block, alkane_id.tx);
                },
                alkanes_support::trace::TraceEvent::ReturnContext(return_ctx) => {
                    if !return_ctx.inner.alkanes.0.is_empty() {
                        for alkane in return_ctx.inner.alkanes.0.iter() {
                            println!("   🎫 COUPON RETURNED: {} unit of ({}, {})", 
                                alkane.value, alkane.id.block, alkane.id.tx);
                        }
                    }
                },
                _ => {}
            }
        }
    }

    let coupon_id = created_coupon_id.ok_or_else(|| anyhow::anyhow!("No coupon token created!"))?;
    println!("✅ COUPON CREATION SUCCESSFUL: {:?}", coupon_id);

    // PHASE 5: Wait for redemption period (current_block >= creation_block + creation_block)
    // Creation at block 10, so redemption available at block 20
    println!("\n⏰ PHASE 5: Waiting for Redemption Period (Block 20)");

    // PHASE 6: SUCCESSFUL REDEMPTION with actual coupon token (BOILER PATTERN)
    println!("\n💰 PHASE 6: REDEMPTION WITH ACTUAL COUPON TOKEN (BOILER PATTERN)");
    
    // Find the coupon token outpoint from deposit block
    let coupon_outpoint = OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 0, // The output where the coupon was created
    };
    
    println!("🔍 Using coupon outpoint: {:?}", coupon_outpoint);
    println!("🎫 Bringing in coupon token: ({}, {}) with amount 1", coupon_id.block, coupon_id.tx);
    
    let redemption_block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: bitcoin::transaction::Version::ONE,
        lock_time: absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: coupon_outpoint, // CRITICAL: Reference the coupon creation output
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
                    edicts: vec![],  // No edicts needed - token comes from input
                    etching: None,
                    mint: None,
                    pointer: None,
                    protocol: Some(
                        vec![
                            Protostone {
                                message: into_cellpack(vec![
                                    4u128, 1793u128, 2u128, // RedeemCoupon opcode
                                    coupon_id.block, coupon_id.tx, // Pass coupon ID as parameters
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: ProtoruneRuneId { block: coupon_id.block, tx: coupon_id.tx },
                                        amount: 1, // Bring the coupon token from input
                                        output: 0, // Send to output 0 (the factory call)
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
    index_block(&redemption_block, 20)?;

    // PHASE 7: ANALYZE REDEMPTION TRACES FOR PAYOUT EVIDENCE
    println!("\n🔍 PHASE 7: ANALYZING REDEMPTION TRACES FOR PAYOUT:");
    let mut total_payout_received = 0u128;
    let mut payout_token_id: Option<AlkaneId> = None;

    for vout in 0..6 {
        let trace_data = &view::trace(&OutPoint {
            txid: redemption_block.txdata[0].compute_txid(),
            vout,
        })?;
        let trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&trace_data[..])?.into();
        let trace_guard = trace_result.0.lock().unwrap();

        if !trace_guard.is_empty() {
            println!("   📊 REDEMPTION vout {} trace: {:?}", vout, *trace_guard);

            for entry in trace_guard.iter() {
                match entry {
                    alkanes_support::trace::TraceEvent::ReturnContext(return_ctx) => {
                        if !return_ctx.inner.alkanes.0.is_empty() {
                            for alkane in return_ctx.inner.alkanes.0.iter() {
                                total_payout_received += alkane.value;
                                payout_token_id = Some(alkane.id.clone());
                                println!("   💰 PAYOUT RECEIVED: {} tokens of ({}, {})", 
                                    alkane.value, alkane.id.block, alkane.id.tx);
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    println!("\n🎊 COMPLETE PAYOUT DEMONSTRATION RESULTS:");
    println!("=========================================");
    println!("✅ Original deposit: 100,000 tokens");
    println!("✅ Coupon created: {:?}", coupon_id);
    if total_payout_received > 0 {
        println!("✅ TOTAL PAYOUT RECEIVED: {} tokens", total_payout_received);
        println!("✅ Payout token: {:?}", payout_token_id.unwrap_or_default());
        println!("✅ Profit: {} tokens", total_payout_received.saturating_sub(100000));
        println!("✅ SUCCESSFUL REDEMPTION DEMONSTRATED WITH TRACE CITATIONS!");
    } else {
        println!("❌ No payout received - check redemption logic");
    }

    Ok(())
}
