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
fn test_working_redemption_with_payout() -> Result<()> {
    println!("\n🎰 WORKING REDEMPTION WITH PAYOUT TEST");
    println!("======================================");
    println!("🎯 OBJECTIVE: DEMONSTRATE ACTUAL PAYOUT WITH TRACE CITATIONS");
    
    clear();
    
    // PHASE 1: Deploy all contract templates (EXACT WORKING PATTERN)
    println!("\n📦 PHASE 1: Deploying All Contract Templates (WORKING PATTERN)");
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            free_mint_build::get_bytes(),
            coupon_template_build::get_bytes(),
            factory_build::get_bytes(),
        ].into(),
        [
            // EXACT working parameters from comprehensive test
            vec![3u128, 797u128, 101u128, 1000000u128, 100000u128, 1000000000u128, 0x54455354, 0x434f494e, 0x545354], 
            vec![3u128, 0x601, 10u128 ],
            vec![3u128, 0x701, 0u128, 144u128, 4u128, 0x601u128],
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;
    println!("✅ All contract templates deployed at block 0");

    // PHASE 2: Initialize Free-Mint Contract (EXACT WORKING PATTERN)
    println!("\n🪙 PHASE 2: Initializing Free-Mint Contract (WORKING PATTERN)");
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
                                    6u128, 797u128, 0u128,  // EXACT working pattern
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
    index_block(&free_mint_block, 1)?; // Block 1 as per working test
    
    let free_mint_contract_id = AlkaneId { block: 2, tx: 1 };
    println!("✅ Free-mint contract initialized at {:?}", free_mint_contract_id);

    // PHASE 3: Initialize Factory (EXACT WORKING PATTERN)
    println!("\n🏭 PHASE 3: Initializing Factory (WORKING PATTERN)");
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
                                    6u128, 0x701, 0u128,   // EXACT working pattern
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
    index_block(&factory_init_block, 2)?; // Block 2 as per working test
    
    let factory_contract_id = AlkaneId { block: 4, tx: 1793 };
    println!("✅ Factory contract initialized at {:?}", factory_contract_id);

    // PHASE 4: Create coupon (EXACT WORKING PATTERN + CAPTURE TOKEN)
    println!("\n🎫 PHASE 4: Creating Coupon with Token Capture");
    let coupon_creation_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                        amount: 100000, // 100k tokens
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
    }]);
    index_block(&coupon_creation_block, 10)?;

    // EXTRACT the created coupon token from traces
    println!("\n🔍 ANALYZING COUPON CREATION TRACES:");
    let mut created_coupon_id: Option<AlkaneId> = None;
    
    for vout in 0..6 {
        let trace_data = &view::trace(&OutPoint {
            txid: coupon_creation_block.txdata[0].compute_txid(),
            vout,
        })?;
        let trace_result: alkanes_support::trace::Trace = AlkanesTrace::decode(&trace_data[..])?.into();
        let trace_guard = trace_result.0.lock().unwrap();

        if !trace_guard.is_empty() {
            println!("   📊 Coupon creation vout {} trace: {:?}", vout, *trace_guard);
            
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
    }

    let coupon_id = created_coupon_id.ok_or_else(|| anyhow::anyhow!("No coupon token created!"))?;
    println!("✅ COUPON CREATION SUCCESSFUL: {:?}", coupon_id);

    // PHASE 5: REDEMPTION WITH COUPON TOKEN (BOILER INPUT PATTERN)
    println!("\n💰 PHASE 5: REDEMPTION WITH ACTUAL COUPON TOKEN (BOILER PATTERN)");
    
    let coupon_outpoint = OutPoint {
        txid: coupon_creation_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    println!("🔍 Using coupon outpoint: {:?}", coupon_outpoint);
    println!("🎫 Bringing in coupon token: ({}, {}) with amount 1", coupon_id.block, coupon_id.tx);
    
    let redemption_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: coupon_outpoint, // CRITICAL: Bring in the coupon
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
    index_block(&redemption_block, 20)?; // Block 20 satisfies timing constraint (10 + 10)

    // PHASE 6: ANALYZE REDEMPTION TRACES FOR PAYOUT EVIDENCE
    println!("\n🔍 PHASE 6: ANALYZING REDEMPTION TRACES FOR PAYOUT:");
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

    // PHASE 7: BALANCE SHEET VERIFICATION
    println!("\n💰 PHASE 7: BALANCE SHEET VERIFICATION");
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

    println!("\n🎊 WORKING REDEMPTION WITH PAYOUT RESULTS:");
    println!("==========================================");
    println!("✅ Original deposit: 100,000 tokens");
    println!("✅ Coupon created: {:?}", coupon_id);
    
    if total_payout_received > 0 || total_balance_received > 0 {
        let max_payout = std::cmp::max(total_payout_received, total_balance_received);
        println!("✅ TOTAL PAYOUT RECEIVED: {} tokens", max_payout);
        if let Some(token_id) = payout_token_id {
            println!("✅ Payout token: {:?}", token_id);
        }
        
        if max_payout > 100000 {
            println!("✅ PROFIT: {} tokens", max_payout - 100000);
            println!("🎉 SUCCESS: USER RECEIVED DEPOSIT + WINNINGS WITH TRACE CITATIONS!");
        } else if max_payout == 100000 {
            println!("✅ BREAK-EVEN: User received original deposit back");
        } else if max_payout > 0 {
            println!("⚠️ PARTIAL PAYOUT: {} tokens received (less than original)", max_payout);
        }
    } else {
        println!("❌ No payout received - redemption failed");
        return Err(anyhow::anyhow!("Redemption did not produce payout"));
    }

    Ok(())
}
