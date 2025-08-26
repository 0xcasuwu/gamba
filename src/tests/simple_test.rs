use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn test_basic_setup() {
    println!("🧪 BASIC SETUP TEST");
    println!("===================");
    
    // Simple test to verify the basic setup works
    let result = 2 + 2;
    assert_eq!(result, 4);
    
    println!("✅ Basic setup test passed!");
}

#[wasm_bindgen_test]
fn test_imports_work() {
    println!("🧪 IMPORTS TEST");
    println!("===============");
    
    // Test that basic imports work
    use alkanes_support::id::AlkaneId;
    
    let test_id = AlkaneId { block: 1, tx: 2 };
    assert_eq!(test_id.block, 1);
    assert_eq!(test_id.tx, 2);
    
    println!("✅ Imports test passed!");
}

