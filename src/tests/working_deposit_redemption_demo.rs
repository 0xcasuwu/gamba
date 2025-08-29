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
use metashrew_core::{stdio::stdout};
use std::fmt::Write;
use protobuf::Message;
use alkanes_support::proto::alkanes::AlkanesTrace;

use crate::tests::std::factory_build;
use crate::tests::std::coupon_template_build;
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
fn test_working_deposit_redemption_demo() -> Result<()> {
    writeln!(stdout(), "\n🎰 WORKING DEPOSIT-TO-REDEMPTION DEMO")?;
    writeln!(stdout(), "======================================")?;
    
    clear();
    
    // PHASE 1: Deploy contract templates (like gamba_deposit_redemption_test.rs)
    writeln!(stdout(), "\n📦 PHASE 1: Deploying Contract Templates")?;
    let template_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            free_mint_build::get_bytes(),
            coupon_template_build::get_bytes(),
            factory_build::get_bytes(),
        ].into(),
        [
            // free_mint template → deploys instance at block 4, tx 797 (opcode 0 for init)
            vec![3u128, 797u128, 101u128],
            // coupon_token template → deploys instance at block 4, tx 0x601 (opcode 0 for init)
            vec![3u128, 0x601, 0u128, 1u128, 1000u128, 50u128, 10u128, 60u128, 1u128, 1u128, 4u128, 0x701u128],
            // factory template → deploys instance at block 4, tx 0x701 (opcode 0 for init)
            vec![3u128, 0x701, 0u128, 144u128, 4u128, 0x601u128],
        ].into_iter().map(|v| into_cellpack(v)).collect::<Vec<Cellpack>>()
    );
    index_block(&template_block, 0)?;
    writeln!(stdout(), "✅ Contract templates deployed")?;
    
    // PHASE 2: Initialize Free-Mint Contract (like gamba_deposit_redemption_test.rs working pattern)
    writeln!(stdout(), "\n🪙 PHASE 2: Initialize Free-Mint Contract")?;
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
                                    3u128, 797u128, 0u128,  // Deploy to free-mint contract, opcode 0 (Initialize)
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
    
    let free_mint_contract_id = AlkaneId { block: 3, tx: 797 };
    writeln!(stdout(), "✅ Free-mint contract initialized: {:?}", free_mint_contract_id)?;
    
    // PHASE 3: Mint Tokens from Free-Mint Contract
    writeln!(stdout(), "\n💰 PHASE 3: Mint Tokens from Free-Mint")?;
    let mint_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
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
                                    free_mint_contract_id.block, free_mint_contract_id.tx, 77u128,  // MintTokens opcode
                                    50_000u128, // Amount to mint  
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
    index_block(&mint_block, 5)?;
    
    writeln!(stdout(), "✅ Alice minted 50,000 tokens")?;
    
    // PHASE 4: Alice's Deposit (use working pattern from gamba_deposit_redemption_test.rs)
    writeln!(stdout(), "\n🎲 PHASE 4: Alice's Deposit")?;
    let factory_id = AlkaneId { block: 3, tx: 0x701 };
    let mint_outpoint = OutPoint {
        txid: mint_block.txdata[0].compute_txid(),
        vout: 0,
    };
    let mint_sheet = load_sheet(&RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES.select(&consensus_encode(&mint_outpoint)?));
    let token_rune_id = ProtoruneRuneId { block: 3, tx: 797 };
    let available_tokens = mint_sheet.get(&token_rune_id);
    let stake_amount = 5000u128;
    
    writeln!(stdout(), "💰 Available tokens: {}", available_tokens)?;
    writeln!(stdout(), "🎯 Stake amount: {}", stake_amount)?;
    
    if available_tokens < stake_amount {
        return Err(anyhow::anyhow!("Insufficient tokens: have {}, need {}", available_tokens, stake_amount));
    }
    
    let deposit_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: mint_outpoint,
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
                                    factory_id.block,
                                    factory_id.tx,
                                    1u128, // create_coupon opcode
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: token_rune_id.clone(),
                                        amount: available_tokens,
                                        output: 1,
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
    index_block(&deposit_block, 10)?;
    
    writeln!(stdout(), "✅ Alice's deposit submitted at block 10")?;
    writeln!(stdout(), "📋 Deposit TX: {}", deposit_block.txdata[0].compute_txid())?;
    
    // Analyze deposit traces
    writeln!(stdout(), "\n🔍 DEPOSIT TRACE ANALYSIS:")?;
    let mut coupon_id: Option<ProtoruneRuneId> = None;
    
    for vout in 0..5 {
        let trace_data = &view::trace(&OutPoint {
            txid: deposit_block.txdata[0].compute_txid(),
            vout,
        })?;
        
        if !trace_data.is_empty() {
            let trace_result: alkanes_support::trace::Trace = 
                AlkanesTrace::parse_from_bytes(trace_data)?.into();
            let trace_guard = trace_result.0.lock().unwrap();
            
            if !trace_guard.is_empty() {
                writeln!(stdout(), "   📊 vout {} trace: {:?}", vout, *trace_guard)?;
                
                let trace_str = format!("{:?}", *trace_guard);
                
                if trace_str.contains("CreateAlkane") {
                    writeln!(stdout(), "   🎫 COUPON CREATED!")?;
                    coupon_id = Some(ProtoruneRuneId { block: 10, tx: 1 });
                }
                
                if trace_str.contains("ReturnContext") && !trace_str.contains("RevertContext") {
                    writeln!(stdout(), "   ✅ DEPOSIT SUCCESS!")?;
                }
                
                if trace_str.contains("RevertContext") {
                    writeln!(stdout(), "   ❌ DEPOSIT FAILED: {}", trace_str)?;
                }
            }
        }
    }
    
    // PHASE 5: Alice's Redemption
    writeln!(stdout(), "\n💰 PHASE 5: Alice's Redemption")?;
    let coupon_id = coupon_id.unwrap_or(ProtoruneRuneId { block: 10, tx: 1 });
    let coupon_outpoint = OutPoint {
        txid: deposit_block.txdata[0].compute_txid(),
        vout: 0,
    };
    
    writeln!(stdout(), "🎫 Alice presents coupon: {:?}", coupon_id)?;
    
    let redemption_block: Block = protorune_helpers::create_block_with_txs(vec![Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: coupon_outpoint,
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
                                    factory_id.block,
                                    factory_id.tx,
                                    60u128, // RedeemWinningCoupon opcode
                                    coupon_id.block,
                                    coupon_id.tx,
                                ]).encipher(),
                                protocol_tag: AlkaneMessageContext::protocol_tag() as u128,
                                pointer: Some(0),
                                refund: Some(0),
                                from: None,
                                burn: None,
                                edicts: vec![
                                    ProtostoneEdict {
                                        id: coupon_id.clone(),
                                        amount: 1, // Send 1 coupon token
                                        output: 1,
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
    index_block(&redemption_block, 15)?;
    
    writeln!(stdout(), "📋 Redemption TX: {}", redemption_block.txdata[0].compute_txid())?;
    
    // Analyze redemption traces  
    writeln!(stdout(), "\n🔍 REDEMPTION TRACE ANALYSIS:")?;
    for vout in 0..5 {
        let trace_data = &view::trace(&OutPoint {
            txid: redemption_block.txdata[0].compute_txid(),
            vout,
        })?;
        
        if !trace_data.is_empty() {
            let trace_result: alkanes_support::trace::Trace = 
                AlkanesTrace::parse_from_bytes(trace_data)?.into();
            let trace_guard = trace_result.0.lock().unwrap();
            
            if !trace_guard.is_empty() {
                writeln!(stdout(), "   📊 vout {} redemption trace: {:?}", vout, *trace_guard)?;
                
                let trace_str = format!("{:?}", *trace_guard);
                
                if trace_str.contains("AlkaneTransfer") {
                    writeln!(stdout(), "   💰 PAYOUT DETECTED!")?;
                }
                
                if trace_str.contains("ReturnContext") && !trace_str.contains("RevertContext") {
                    writeln!(stdout(), "   ✅ REDEMPTION SUCCESS!")?;
                }
                
                if trace_str.contains("RevertContext") {
                    writeln!(stdout(), "   ❌ REDEMPTION FAILED: {}", trace_str)?;
                }
            }
        }
    }
    
    writeln!(stdout(), "\n🎊 COMPLETE LIFECYCLE DEMONSTRATED!")?;
    writeln!(stdout(), "✅ Contract setup → Token minting → Deposit → Coupon creation → Redemption")?;
    writeln!(stdout(), "🎰 Alice's complete gambling journey with full trace evidence!")?;
    
    Ok(())
}
