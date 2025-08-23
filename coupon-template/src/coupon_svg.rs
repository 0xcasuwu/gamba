use bitcoin::{Txid, blockdata::block::TxMerkleNode};
use anyhow::Result;

pub struct CouponData {
    pub coupon_id: u128,
    pub txid: Txid,
    pub merkle_root: TxMerkleNode,
    pub base_xor_result: u8,
    pub token_bonus: u8,
    pub final_xor_result: u8,
    pub token_amount: u128,
    pub is_winner: bool,
}

pub struct CouponSvgGenerator;

impl CouponSvgGenerator {
    pub fn generate_svg(data: CouponData) -> Result<String> {
        let prize_level = Self::calculate_prize_level(data.final_xor_result);
        let coupon_type = Self::get_coupon_type(data.final_xor_result);
        let theme_color = Self::get_theme_color(data.final_xor_result, data.is_winner);
        let status = if data.is_winner { "WINNER" } else { "BETTER LUCK NEXT TIME" };
        let status_color = if data.is_winner { "#10b981" } else { "#ef4444" };
        let txid_short = format!("{}...{}",
            &data.txid.to_string()[0..8],
            &data.txid.to_string()[56..64]
        );
        let merkle_short = format!("{}...{}",
            &data.merkle_root.to_string()[0..8],
            &data.merkle_root.to_string()[56..64]
        );
        let token_display = Self::format_token_amount(data.token_amount);

        let svg = format!(r##"<svg viewBox="0 0 300 450" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <!-- Gambling ticket gradient backgrounds -->
    <linearGradient id="ticketGradient" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#1f1f1f;stop-opacity:1" />
      <stop offset="25%" style="stop-color:#2a2a2a;stop-opacity:1" />
      <stop offset="50%" style="stop-color:#3a3a3a;stop-opacity:1" />
      <stop offset="75%" style="stop-color:#4a4a4a;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#2a2a2a;stop-opacity:1" />
    </linearGradient>
    
    <radialGradient id="goldAccent" cx="50%" cy="50%" r="50%">
      <stop offset="0%" style="stop-color:#fbbf24;stop-opacity:1" />
      <stop offset="30%" style="stop-color:#f59e0b;stop-opacity:1" />
      <stop offset="70%" style="stop-color:#d97706;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#92400e;stop-opacity:1" />
    </radialGradient>
    
    <linearGradient id="statusGlow" x1="0%" y1="0%" x2="100%" y2="0%">
      <stop offset="0%" style="stop-color:{};stop-opacity:1" />
      <stop offset="50%" style="stop-color:{};stop-opacity:0.8" />
      <stop offset="100%" style="stop-color:{};stop-opacity:1" />
    </linearGradient>
    
    <radialGradient id="centerGlow" cx="50%" cy="50%" r="50%">
      <stop offset="0%" style="stop-color:{};stop-opacity:1" />
      <stop offset="50%" style="stop-color:#fbbf24;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#4a4a4a;stop-opacity:1" />
    </radialGradient>
    
    <linearGradient id="prizeGlow" x1="0%" y1="0%" x2="100%" y2="0%">
      <stop offset="0%" style="stop-color:#fbbf24;stop-opacity:1" />
      <stop offset="25%" style="stop-color:#34d399;stop-opacity:1" />
      <stop offset="50%" style="stop-color:#60a5fa;stop-opacity:1" />
      <stop offset="75%" style="stop-color:#a78bfa;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#fbbf24;stop-opacity:1" />
    </linearGradient>
    
    <linearGradient id="tokenGlow" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#d4af37;stop-opacity:1" />
      <stop offset="50%" style="stop-color:#ffd700;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#b8860b;stop-opacity:1" />
    </linearGradient>
    
    <!-- Effects and filters -->
    <filter id="ticketGlow">
      <feGaussianBlur stdDeviation="8" result="coloredBlur"/>
      <feMerge>
        <feMergeNode in="coloredBlur"/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
    
    <filter id="statusShimmer">
      <feGaussianBlur stdDeviation="4" result="coloredBlur"/>
      <feDropShadow dx="2" dy="2" stdDeviation="6" flood-color="{}" flood-opacity="0.6"/>
      <feMerge>
        <feMergeNode in="coloredBlur"/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
    
    <filter id="tokenShimmer">
      <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
      <feDropShadow dx="1" dy="1" stdDeviation="4" flood-color="#ffd700" flood-opacity="0.8"/>
      <feMerge>
        <feMergeNode in="coloredBlur"/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
    
    <!-- Animated pattern for ticket background -->
    <pattern id="ticketPattern" patternUnits="userSpaceOnUse" width="40" height="40">
      <circle cx="10" cy="10" r="1" fill="#fbbf24" opacity="0.2">
        <animate attributeName="opacity" values="0.2;0.4;0.2" dur="3s" repeatCount="indefinite"/>
      </circle>
      <circle cx="30" cy="30" r="0.8" fill="#60a5fa" opacity="0.15">
        <animate attributeName="opacity" values="0.15;0.3;0.15" dur="4s" repeatCount="indefinite"/>
      </circle>
      <circle cx="20" cy="5" r="0.6" fill="#34d399" opacity="0.18">
        <animate attributeName="opacity" values="0.18;0.35;0.18" dur="2.5s" repeatCount="indefinite"/>
      </circle>
    </pattern>
  </defs>
  
  <!-- Background pattern -->
  <rect x="0" y="0" width="300" height="450" fill="url(#ticketPattern)" opacity="0.3">
    <animateTransform attributeName="transform" attributeType="XML" type="translate" values="0,0;2,1;0,0;-1,2;0,0" dur="12s" repeatCount="indefinite"/>
  </rect>
  
  <!-- Main ticket background -->
  <rect x="15" y="25" width="270" height="400" rx="20" ry="20" fill="url(#ticketGradient)" stroke="url(#goldAccent)" stroke-width="4" filter="url(#ticketGlow)">
    <animateTransform attributeName="transform" attributeType="XML" type="scale" values="1;1.003;1" dur="8s" repeatCount="indefinite" transform-origin="150 225"/>
  </rect>
  
  <!-- Perforated edge effect (top) -->
  <g fill="url(#goldAccent)" opacity="0.6">
    <circle cx="30" cy="35" r="3"/>
    <circle cx="50" cy="35" r="3"/>
    <circle cx="70" cy="35" r="3"/>
    <circle cx="90" cy="35" r="3"/>
    <circle cx="110" cy="35" r="3"/>
    <circle cx="130" cy="35" r="3"/>
    <circle cx="150" cy="35" r="3"/>
    <circle cx="170" cy="35" r="3"/>
    <circle cx="190" cy="35" r="3"/>
    <circle cx="210" cy="35" r="3"/>
    <circle cx="230" cy="35" r="3"/>
    <circle cx="250" cy="35" r="3"/>
    <circle cx="270" cy="35" r="3"/>
  </g>
  
  <!-- Perforated edge effect (bottom) -->
  <g fill="url(#goldAccent)" opacity="0.6">
    <circle cx="30" cy="415" r="3"/>
    <circle cx="50" cy="415" r="3"/>
    <circle cx="70" cy="415" r="3"/>
    <circle cx="90" cy="415" r="3"/>
    <circle cx="110" cy="415" r="3"/>
    <circle cx="130" cy="415" r="3"/>
    <circle cx="150" cy="415" r="3"/>
    <circle cx="170" cy="415" r="3"/>
    <circle cx="190" cy="415" r="3"/>
    <circle cx="210" cy="415" r="3"/>
    <circle cx="230" cy="415" r="3"/>
    <circle cx="250" cy="415" r="3"/>
    <circle cx="270" cy="415" r="3"/>
  </g>
  
  <!-- Central lottery numbers display -->
  <g transform="translate(150, 120)">
    <!-- Main numbers circle -->
    <circle cx="0" cy="0" r="45" fill="url(#centerGlow)" stroke="url(#goldAccent)" stroke-width="3" filter="url(#ticketGlow)">
      <animate attributeName="r" values="45;47;45" dur="4s" repeatCount="indefinite"/>
    </circle>
    
    <!-- Inner circle with XOR result -->
    <circle cx="0" cy="0" r="32" fill="url(#statusGlow)" opacity="0.8">
      <animate attributeName="opacity" values="0.8;1;0.8" dur="3s" repeatCount="indefinite"/>
    </circle>
    
    <!-- Lucky number display -->
    <text x="0" y="8" font-family="Arial, sans-serif" font-size="24" font-weight="bold" text-anchor="middle" fill="white">{}</text>
    
    <!-- Surrounding prize indicators -->
    <g fill="url(#prizeGlow)" opacity="0.7">
      <circle cx="-60" cy="-10" r="4">
        <animate attributeName="opacity" values="0.7;0.3;0.7" dur="2s" repeatCount="indefinite"/>
        <animateTransform attributeName="transform" type="rotate" from="0 -60 -10" to="360 -60 -10" dur="8s" repeatCount="indefinite"/>
      </circle>
      <circle cx="60" cy="-10" r="4">
        <animate attributeName="opacity" values="0.7;0.3;0.7" dur="2.2s" repeatCount="indefinite"/>
        <animateTransform attributeName="transform" type="rotate" from="360 60 -10" to="0 60 -10" dur="8s" repeatCount="indefinite"/>
      </circle>
      <circle cx="0" cy="-65" r="3">
        <animate attributeName="opacity" values="0.7;0.2;0.7" dur="1.8s" repeatCount="indefinite"/>
      </circle>
      <circle cx="0" cy="65" r="3">
        <animate attributeName="opacity" values="0.7;0.2;0.7" dur="2.5s" repeatCount="indefinite"/>
      </circle>
    </g>
  </g>
  
  <!-- Status banner -->
  <rect x="40" y="190" width="220" height="40" rx="20" ry="20" fill="url(#statusGlow)" stroke="url(#{}" stroke-width="2" filter="url(#statusShimmer)">
    <animate attributeName="opacity" values="1;0.8;1" dur="2s" repeatCount="indefinite"/>
  </rect>
  <text x="150" y="205" font-family="Arial, sans-serif" font-size="10" font-weight="bold" text-anchor="middle" fill="white" opacity="0.9">GAMBLING RESULT</text>
  <text x="150" y="220" font-family="Arial, sans-serif" font-size="16" font-weight="bold" text-anchor="middle" fill="white" filter="url(#statusShimmer)">{}</text>
  
  <!-- Coupon details section -->
  <text x="150" y="250" font-family="Arial, sans-serif" font-size="18" font-weight="bold" text-anchor="middle" fill="url(#prizeGlow)" filter="url(#ticketGlow)">Gambling Coupon #{}</text>
  <text x="150" y="270" font-family="Arial, sans-serif" font-size="12" text-anchor="middle" fill="url(#goldAccent)" opacity="0.9">{}</text>
  
  <!-- Decorative divider -->
  <g transform="translate(150, 285)">
    <line x1="-80" y1="0" x2="80" y2="0" stroke="url(#prizeGlow)" stroke-width="2" opacity="0.6"/>
    <circle cx="-60" cy="0" r="3" fill="url(#goldAccent)" opacity="0.8"/>
    <circle cx="-30" cy="0" r="2" fill="url(#centerGlow)" opacity="0.8"/>
    <circle cx="0" cy="0" r="4" fill="url(#prizeGlow)" opacity="0.8"/>
    <circle cx="30" cy="0" r="2" fill="url(#centerGlow)" opacity="0.8"/>
    <circle cx="60" cy="0" r="3" fill="url(#goldAccent)" opacity="0.8"/>
  </g>
  
  <!-- Prize level indicator -->
  <rect x="50" y="300" width="200" height="35" rx="8" ry="8" fill="url(#goldAccent)" opacity="0.1" stroke="url(#prizeGlow)" stroke-width="1.5"/>
  <text x="150" y="315" font-family="Arial, sans-serif" font-size="11" font-weight="bold" text-anchor="middle" fill="url(#goldAccent)" opacity="0.9">PRIZE LEVEL</text>
  <text x="150" y="330" font-family="Arial, sans-serif" font-size="16" font-weight="bold" text-anchor="middle" fill="url(#prizeGlow)" filter="url(#statusShimmer)">{}</text>
  
  <!-- Token stake info -->
  <rect x="50" y="345" width="200" height="30" rx="6" ry="6" fill="url(#goldAccent)" opacity="0.08" stroke="url(#tokenGlow)" stroke-width="1" opacity="0.4"/>
  <text x="150" y="358" font-family="Arial, sans-serif" font-size="9" text-anchor="middle" fill="url(#tokenGlow)" opacity="0.8">Staked Tokens</text>
  <text x="150" y="370" font-family="Arial, sans-serif" font-size="11" text-anchor="middle" fill="url(#prizeGlow)" opacity="0.9">{}</text>
  
  <!-- Token bonus -->
  <rect x="50" y="380" width="200" height="25" rx="6" ry="6" fill="url(#tokenGlow)" opacity="0.1" stroke="url(#tokenGlow)" stroke-width="1" opacity="0.6"/>
  <text x="150" y="392" font-family="Arial, sans-serif" font-size="9" text-anchor="middle" fill="url(#tokenGlow)" opacity="0.8">Token Bonus Applied</text>
  <text x="150" y="402" font-family="Arial, sans-serif" font-size="10" text-anchor="middle" fill="url(#tokenGlow)" opacity="0.9">+{}</text>
  
  <!-- Transaction details -->
  <text x="150" y="420" font-family="Arial, sans-serif" font-size="7" text-anchor="middle" fill="url(#goldAccent)" opacity="0.7">Tx: {}</text>
  <text x="150" y="432" font-family="Arial, sans-serif" font-size="7" text-anchor="middle" fill="url(#goldAccent)" opacity="0.7">Merkle: {}</text>
  <text x="150" y="444" font-family="Arial, sans-serif" font-size="7" text-anchor="middle" fill="url(#goldAccent)" opacity="0.7">XOR: {} + {} = {}</text>
</svg>"##,
            status_color, status_color, status_color,
            theme_color,
            status_color,
            data.final_xor_result,
            status_color,
            status,
            data.coupon_id,
            coupon_type,
            prize_level,
            token_display,
            data.token_bonus,
            txid_short,
            merkle_short,
            data.base_xor_result,
            data.token_bonus,
            data.final_xor_result
        );

        Ok(svg)
    }

    fn calculate_prize_level(final_xor_result: u8) -> String {
        match final_xor_result {
            150..=160 => "Consolation".to_string(),
            161..=180 => "Fourth Prize".to_string(),
            181..=200 => "Third Prize".to_string(),
            201..=220 => "Second Prize".to_string(),
            221..=240 => "First Prize".to_string(),
            241..=255 => "Jackpot".to_string(),
            _ => "Losing".to_string(),
        }
    }

    fn get_coupon_type(final_xor_result: u8) -> String {
        match final_xor_result % 7 {
            0 => "Lucky Draw".to_string(),
            1 => "Instant Win".to_string(),
            2 => "Scratch Off".to_string(),
            3 => "Lottery Pick".to_string(),
            4 => "Raffle Ticket".to_string(),
            5 => "Prize Draw".to_string(),
            6 => "Fortune Wheel".to_string(),
            _ => "Mystery Game".to_string(),
        }
    }

    fn get_theme_color(final_xor_result: u8, is_winner: bool) -> &'static str {
        if !is_winner {
            return "#ef4444"; // Red for losing tickets
        }
        
        match final_xor_result % 6 {
            0 => "#fbbf24", // Gold
            1 => "#10b981", // Green
            2 => "#3b82f6", // Blue
            3 => "#8b5cf6", // Purple
            4 => "#f59e0b", // Orange
            5 => "#06b6d4", // Cyan
            _ => "#10b981", // Green default
        }
    }

    fn format_token_amount(amount: u128) -> String {
        if amount >= 1_000_000 {
            format!("{:.1}M", amount as f64 / 1_000_000.0)
        } else if amount >= 1_000 {
            format!("{:.1}K", amount as f64 / 1_000.0)
        } else {
            amount.to_string()
        }
    }
}