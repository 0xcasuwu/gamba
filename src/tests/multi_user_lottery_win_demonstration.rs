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
fn test_multi_user_lottery_win_demonstration() -> Result<()> {
    println!("\n🎰 MULTI-USER LOTTERY WIN DEMONSTRATION");
    println!("=======================================");
    println!("🎯 OBJECTIVE: Show winner getting MORE than deposit by winning from losing pot");
    
    clear();
    
    // PHASE 1: Deploy all contract templates (EXACT WORKING PATTERN)
    println!("\n📦 PHASE 1: Deploying All Contract Templates");
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
    println!("✅ All contract templates deployed at block 0");

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
                                    6u128, 797u128, 0u128,
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
    index_block(&free_mint_block, 1)?;
    
    let free_mint_contract_id = AlkaneId { block: 2, tx: 1 };
    println!("✅ Free-mint contract initialized at {:?}", free_mint_contract_id);

    // PHASE 3: Initialize Factory 
    println!("\n🏭 PHASE 3: Initializing Factory");
    let factory_init_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
    index_block(&factory_init_block, 2)?;
    
    let factory_contract_id = AlkaneId { block: 4, tx: 1793 };
    println!("✅ Factory contract initialized at {:?}", factory_contract_id);

    // PHASE 4: Create multiple deposits at SAME BLOCK (critical for pot sharing)
    println!("\n👥 PHASE 4: Creating Multiple Deposits at Same Block (Block 10)");
    
    // Both users will deposit from the same free-mint tokens - use multi-tx block
    let multi_user_deposit_block: Block = protorune_helpers::create_block_with_txs(vec![
        // TX 0: USER A (LOSER) deposits 200,000 tokens
        Transaction {
            version: Version::ONE,
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: free_mint_block.txdata[0].compute_txid(),
                    vout: 0, // Token from free-mint initialization
                },
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
                            // Split tokens: 200k to USER A deposit, 100k to USER B
                            ordinals::Edict {
                                id: ordinals::RuneId::new(2, 1).unwrap(),
                                amount: 200000, // USER A gets 200k
                                output: 0,
                            },
                            ordinals::Edict {
                                id: ordinals::RuneId::new(2, 1).unwrap(), 
                                amount: 100000, // Remaining 100k for USER B
                                output: 1,
                            },
                        ],
                        etching: None,
                        mint: None,
                        pointer: None,
                        protocol: Some(
                            vec![
                                Protostone {
                                    message: into_cellpack(vec![
                                        4u128, 1793u128, 1u128, // CreateCoupon at factory
                                    ]).encipher(),
                                    protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                    pointer: Some(0),
                                    refund: Some(0),
                                    from: None,
                                    burn: None,
                                    edicts: vec![
                                        ProtostoneEdict {
                                            id: ProtoruneRuneId { block: 2, tx: 1 },
                                            amount: 200000, // 200k tokens (USER A - LOSER)
                                            output: 0,
                                        }
                                    ],
                                }
                            ].encipher()?
                        )
                    }).encipher(),
                    value: Amount::from_sat(546)
                },
                TxOut {
                    script_pubkey: Address::from_str(ADDRESS1().as_str())
                        .unwrap()
                        .require_network(get_btc_network())
                        .unwrap()
                        .script_pubkey(),
                    value: Amount::from_sat(546),
                }
            ],
        },
        // TX 1: USER B (WINNER) deposits 100,000 tokens  
        Transaction {
            version: Version::ONE,
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: bitcoin::Txid::all_zeros(), // Will be set to TX 0 of this block
                    vout: 1, // From output 1 of TX 0 (USER B's 100k tokens)
                },
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
                                        4u128, 1793u128, 1u128, // CreateCoupon at factory
                                    ]).encipher(),
                                    protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                    pointer: Some(0),
                                    refund: Some(0),
                                    from: None,
                                    burn: None,
                                    edicts: vec![
                                        ProtostoneEdict {
                                            id: ProtoruneRuneId { block: 2, tx: 1 },
                                            amount: 100000, // 100k tokens (USER B - WINNER)
                                            output: 0,
                                        }
                                    ],
                                }
                            ].encipher()?
                        )
                    }).encipher(),
                    value: Amount::from_sat(546)
                }
            ],
        }
    ]);
    
    // Fix TX 1 input to reference TX 0 properly
    let tx0_txid = multi_user_deposit_block.txdata[0].compute_txid();
    let mut fixed_block = multi_user_deposit_block.clone();
    fixed_block.txdata[1].input[0].previous_output.txid = tx0_txid;
    
    index_block(&fixed_block, 10)?; // Both deposits at block 10

    println!("✅ USER A deposited 200,000 tokens (LOSER)");
    println!("✅ USER B deposited 100,000 tokens (WINNER)");
    println!("✅ Both deposits at block 10 (same lottery round)");

    // PHASE 5: Extract USER B's coupon token (the winner)
    println!("\n🔍 PHASE 5: Extracting USER B's Coupon Token (Winner)");
    let mut user_b_coupon_id: Option<AlkaneId> = None;
    
    // Look at TX 1 (USER B's transaction) 
    for vout in 0..6 {
        let trace_data = &view::trace(&OutPoint {
            txid: fixed_block.txdata[1].compute_txid(), // TX 1 is USER B
            vout,
        })?;
        let trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&trace_data[..])?.into();
        let trace_guard = trace_result.0.lock().unwrap();

        if !trace_guard.is_empty() {
            println!("     🔍 USER B TX 1 vout {} trace: {:?}", vout, *trace_guard);
            for entry in trace_guard.iter() {
                match entry {
                    alkanes_support::trace::TraceEvent::CreateAlkane(alkane_id) => {
                        user_b_coupon_id = Some(alkane_id.clone());
                        println!("     ✅ CAPTURED USER B COUPON TOKEN: ({}, {})", alkane_id.block, alkane_id.tx);
                    },
                    _ => {}
                }
            }
        }
    }

    let coupon_id = user_b_coupon_id.ok_or_else(|| anyhow::anyhow!("No USER B coupon token created!"))?;
    println!("✅ USER B COUPON TOKEN FOUND: {:?}", coupon_id);

    // PHASE 6: REDEMPTION - USER B wins and should get MORE than 100k
    println!("\n💰 PHASE 6: USER B REDEMPTION (Should get 100k + share of 200k from USER A)");
    
    let coupon_outpoint = OutPoint {
        txid: fixed_block.txdata[1].compute_txid(), // TX 1 is USER B
        vout: 0,
    };
    
    println!("🔍 Using coupon outpoint: {:?}", coupon_outpoint);
    println!("🎫 USER B bringing in coupon token: ({}, {})", coupon_id.block, coupon_id.tx);
    
    let redemption_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: coupon_outpoint, // CRITICAL: Bring in USER B's coupon
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
                                    4u128, 1793u128, 2u128, // RedeemCoupon opcode
                                    coupon_id.block, coupon_id.tx, // Pass coupon ID
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: ProtoruneRuneId { block: coupon_id.block, tx: coupon_id.tx },
                                        amount: 1, // Bring the coupon token
                                        output: 0, // Send to factory call
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
    index_block(&redemption_block, 20)?; // Block 20 satisfies timing constraint

    // PHASE 7: ANALYZE REDEMPTION TRACES FOR MULTI-USER PAYOUT
    println!("\n🔍 PHASE 7: ANALYZING MULTI-USER REDEMPTION TRACES:");
    let mut total_payout_received = 0u128;

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

    // PHASE 8: BALANCE SHEET VERIFICATION  
    println!("\n💰 PHASE 8: BALANCE SHEET VERIFICATION");
    let redemption_outpoint = OutPoint {
        txid: redemption_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    let redemption_sheet = load_sheet(
        &RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
            .OUTPOINT_TO_RUNES
            .select(&consensus_encode(&redemption_outpoint)?)
    );
    
    let mut total_balance_received = 0u128;
    for (id, amount) in redemption_sheet.balances().iter() {
        println!("   💰 Balance received - Token ID: {:?}, Amount: {}", id, amount);
        total_balance_received += amount;
    }

    println!("\n🎊 MULTI-USER LOTTERY RESULTS:");
    println!("==============================");
    println!("📊 USER A: 200,000 tokens deposited (LOSER)");
    println!("📊 USER B: 100,000 tokens deposited (WINNER)"); 
    println!("📊 Total pot: 300,000 tokens");
    println!("📊 Expected USER B payout: 100,000 + 200,000 = 300,000 tokens");
    
    let max_payout = std::cmp::max(total_payout_received, total_balance_received);
    println!("✅ ACTUAL USER B PAYOUT: {} tokens", max_payout);
    
    if max_payout > 100000 {
        let profit = max_payout - 100000;
        println!("✅ PROFIT FROM WINNING: {} tokens", profit);
        println!("🎉 🎉 🎉 SUCCESS: USER B WON {} FROM USER A'S LOSING DEPOSIT! 🎉 🎉 🎉", profit);
        
        if max_payout == 300000 {
            println!("🏆 PERFECT: USER B got entire pot (100k + 200k = 300k)!");
        } else if profit == 200000 {
            println!("🏆 PERFECT: USER B won all of USER A's losing deposit!");
        }
    } else if max_payout == 100000 {
        println!("❌ PROBLEM: USER B only got original deposit back - pot distribution not working");
        return Err(anyhow::anyhow!("Multi-user pot distribution failed - winner should get more than deposit"));
    } else {
        println!("❌ PROBLEM: USER B got less than original deposit");
        return Err(anyhow::anyhow!("Redemption failed - got less than deposit"));
    }

    Ok(())
}
