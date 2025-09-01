// Debug test to verify threshold logic
fn test_threshold_logic() {
    let final_result_1: u8 = 255;
    let final_result_2: u8 = 176; 
    let threshold: u8 = 250;
    
    let winner_1 = (final_result_1 as u128) > (threshold as u128);
    let winner_2 = (final_result_2 as u128) > (threshold as u128);
    
    println!("USER 1: {} > {} = {}", final_result_1, threshold, winner_1);
    println!("USER 2: {} > {} = {}", final_result_2, threshold, winner_2);
    
    // Expected: USER 1 = true, USER 2 = false
}
