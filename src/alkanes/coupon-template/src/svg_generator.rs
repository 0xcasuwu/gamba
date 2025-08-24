use anyhow::Result;

#[derive(Debug, Clone)]
pub struct CouponData {
    pub coupon_id: u128,
    pub stake_amount: u128,
    pub base_xor: u8,
    pub stake_bonus: u8,
    pub final_result: u8,
    pub creation_block: u128,
    pub current_block: u128,
    pub coupon_type: String,
    pub is_winner: bool,
}

pub struct SvgGenerator;

impl SvgGenerator {
    /// Generate an SVG representation of a coupon based on its properties
    pub fn generate_svg(data: CouponData) -> Result<String> {
        let CouponData {
            coupon_id,
            stake_amount,
            base_xor,
            stake_bonus,
            final_result,
            creation_block,
            current_block,
            coupon_type,
            is_winner,
        } = data;

        // Calculate colors based on coupon properties
        let (primary_color, secondary_color, accent_color) = Self::calculate_colors(final_result, base_xor, is_winner);
        let ticket_width = Self::calculate_ticket_width(&coupon_type);
        let badge_size = Self::calculate_badge_size(final_result);
        let decoration_count = Self::calculate_decoration_count(stake_amount);

        let status_text = if is_winner { "WINNER" } else { "BETTER LUCK NEXT TIME" };
        let status_color = if is_winner { "#10b981" } else { "#ef4444" };

        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 600" width="400" height="600">
  <defs>
    <linearGradient id="ticketGradient" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:{primary_color};stop-opacity:1" />
      <stop offset="100%" style="stop-color:{secondary_color};stop-opacity:1" />
    </linearGradient>
    <radialGradient id="badgeGradient" cx="50%" cy="30%" r="50%">
      <stop offset="0%" style="stop-color:{accent_color};stop-opacity:0.9" />
      <stop offset="70%" style="stop-color:{accent_color};stop-opacity:0.7" />
      <stop offset="100%" style="stop-color:{black_color};stop-opacity:0.3" />
    </radialGradient>
    <filter id="glow">
      <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
      <feMerge>
        <feMergeNode in="coloredBlur"/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
  </defs>
  
  <!-- Background -->
  <rect width="400" height="600" fill="{bg_color}"/>
  
  <!-- Gambling background pattern -->
  <g opacity="0.1">
    {background_pattern}
  </g>
  
  <!-- Main Ticket Body -->
  <rect x="{ticket_x}" y="120" width="{ticket_width}" height="400"
        fill="url(#ticketGradient)" rx="15" filter="url(#glow)"
        stroke="white" stroke-width="2" stroke-dasharray="10,5"/>
  
  <!-- Ticket perforations -->
  {ticket_perforations}
  
  <!-- Status Badge -->
  <circle cx="200" cy="180" r="{badge_size}"
          fill="url(#badgeGradient)" filter="url(#glow)"/>
  
  <!-- Inner badge detail -->
  <circle cx="195" cy="175" r="{inner_badge_size}"
          fill="{accent_color}" opacity="0.8"/>
  
  <!-- Decorative elements -->
  {decorations}
  
  <!-- Badge glow -->
  <circle cx="200" cy="180" r="{glow_size}"
          fill="{accent_color}" opacity="0.3" filter="url(#glow)"/>
  
  <!-- Title -->
  <text x="200" y="50" text-anchor="middle" font-family="serif" font-size="24"
        fill="{primary_color}" font-weight="bold">{coupon_type} Ticket</text>
  
  <!-- Coupon ID -->
  <text x="200" y="75" text-anchor="middle" font-family="monospace" font-size="16"
        fill="{gray_color}">#{coupon_id}</text>
  
  <!-- Status -->
  <text x="200" y="105" text-anchor="middle" font-family="serif" font-size="18"
        fill="{status_color}" font-weight="bold">{status_text}</text>
  
  <!-- Stats Panel -->
  <rect x="50" y="520" width="300" height="70" fill="rgba(0,0,0,0.7)"
        stroke="{primary_color}" stroke-width="1" rx="5"/>
  
  <!-- Stats Text -->
  <text x="70" y="540" font-family="monospace" font-size="12" fill="{light_gray_color}">
    Result: {final_result} | Stake: {stake_amount}
  </text>
  <text x="70" y="555" font-family="monospace" font-size="12" fill="{light_gray_color}">
    XOR: {base_xor} | Bonus: +{stake_bonus}
  </text>
  <text x="70" y="570" font-family="monospace" font-size="10" fill="{gray_color}">
    Block: {creation_block}
  </text>
</svg>"#,
            primary_color = primary_color,
            secondary_color = secondary_color,
            accent_color = accent_color,
            black_color = "#000000",
            bg_color = "#0a0a0a",
            gray_color = "#888888",
            light_gray_color = "#cccccc",
            status_color = status_color,
            status_text = status_text,
            ticket_x = 200 - ticket_width / 2,
            ticket_width = ticket_width,
            badge_size = badge_size,
            inner_badge_size = badge_size - 5,
            glow_size = badge_size + 10,
            decorations = Self::generate_decorations(decoration_count, &accent_color),
            ticket_perforations = Self::generate_ticket_perforations(ticket_width, &primary_color),
            background_pattern = Self::generate_background_pattern(&coupon_type),
            coupon_type = coupon_type,
            coupon_id = coupon_id,
            final_result = final_result,
            stake_amount = stake_amount,
            base_xor = base_xor,
            stake_bonus = stake_bonus,
            creation_block = creation_block,
        );

        Ok(svg)
    }

    /// Calculate colors based on coupon properties
    fn calculate_colors(final_result: u8, base_xor: u8, is_winner: bool) -> (String, String, String) {
        // Winner tickets get brighter, more appealing colors
        let primary = if is_winner {
            match final_result {
                250..=255 => "#fbbf24", // Gold for legendary winners
                230..=249 => "#10b981", // Green for epic winners
                200..=229 => "#3b82f6", // Blue for rare winners
                170..=199 => "#8b5cf6", // Purple for uncommon winners
                _ => "#6b7280", // Gray for common winners
            }
        } else {
            match final_result {
                250..=255 => "#dc2626", // Red for legendary losers
                230..=249 => "#7c2d12", // Dark orange for epic losers
                200..=229 => "#1e40af", // Dark blue for rare losers
                170..=199 => "#374151", // Dark gray for uncommon losers
                _ => "#1f2937", // Very dark for common losers
            }
        };

        let secondary = if is_winner {
            match final_result {
                250..=255 => "#f59e0b",
                230..=249 => "#059669",
                200..=229 => "#1d4ed8",
                170..=199 => "#7c3aed",
                _ => "#4b5563",
            }
        } else {
            match final_result {
                250..=255 => "#991b1b",
                230..=249 => "#92400e",
                200..=229 => "#1e3a8a",
                170..=199 => "#1f2937",
                _ => "#111827",
            }
        };

        // Accent color varies based on XOR value for uniqueness
        let accent = match base_xor % 6 {
            0 => if is_winner { "#fbbf24" } else { "#ef4444" }, // Gold/Red
            1 => if is_winner { "#3b82f6" } else { "#1d4ed8" }, // Blue
            2 => if is_winner { "#10b981" } else { "#059669" }, // Green
            3 => if is_winner { "#f59e0b" } else { "#d97706" }, // Orange
            4 => if is_winner { "#a855f7" } else { "#7c3aed" }, // Purple
            _ => if is_winner { "#06b6d4" } else { "#0891b2" }, // Cyan
        };

        (primary.to_string(), secondary.to_string(), accent.to_string())
    }

    /// Calculate ticket width based on coupon type
    fn calculate_ticket_width(coupon_type: &str) -> u32 {
        match coupon_type {
            "Jackpot" => 280,
            "Big Win" => 260,
            "Win" => 240,
            "Small Win" => 220,
            "Loss" => 200,
            _ => 180,
        }
    }

    /// Calculate badge size based on final result
    fn calculate_badge_size(final_result: u8) -> u32 {
        match final_result {
            250..=255 => 35,
            230..=249 => 30,
            200..=229 => 25,
            170..=199 => 22,
            145..=169 => 18,
            _ => 15,
        }
    }

    /// Calculate number of decorations based on stake amount
    fn calculate_decoration_count(stake_amount: u128) -> u32 {
        match stake_amount {
            10000.. => 12,
            5000..=9999 => 10,
            1000..=4999 => 8,
            100..=999 => 6,
            _ => 4,
        }
    }

    /// Generate decorative effects around the ticket
    fn generate_decorations(count: u32, accent_color: &str) -> String {
        let mut decorations = String::new();
        
        for i in 0..count {
            let angle = (i as f32 * 360.0 / count as f32).to_radians();
            let radius = 90.0 + (i as f32 * 15.0) % 50.0;
            let x = 200.0 + radius * angle.cos();
            let y = 300.0 + radius * angle.sin();
            let size = 3 + (i % 4);
            let opacity = 0.5 + (i as f32 * 0.1) % 0.4;
            
            decorations.push_str(&format!(
                r#"<polygon points="{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}"
                    fill="{}" opacity="{:.2}">
    <animate attributeName="opacity" values="{:.2};0.1;{:.2}" dur="3s" repeatCount="indefinite"/>
  </polygon>"#,
                x, y - size as f32,
                x + size as f32, y,
                x, y + size as f32,
                x - size as f32, y,
                accent_color, opacity, opacity, opacity
            ));
        }
        
        decorations
    }

    /// Generate ticket perforations
    fn generate_ticket_perforations(ticket_width: u32, primary_color: &str) -> String {
        let ticket_center = 200;
        let perforation_spacing = 15;
        let mut perforations = String::new();
        
        // Top perforations
        for i in 0..(ticket_width / perforation_spacing) {
            let x = ticket_center - (ticket_width / 2) as i32 + (i * perforation_spacing) as i32;
            
            perforations.push_str(&format!(
                r#"<circle cx="{}" cy="120" r="2" fill="{}" opacity="0.6"/>"#,
                x, primary_color
            ));
        }
        
        // Bottom perforations
        for i in 0..(ticket_width / perforation_spacing) {
            let x = ticket_center - (ticket_width / 2) as i32 + (i * perforation_spacing) as i32;
            
            perforations.push_str(&format!(
                r#"<circle cx="{}" cy="520" r="2" fill="{}" opacity="0.6"/>"#,
                x, primary_color
            ));
        }
        
        perforations
    }

    /// Generate background pattern based on coupon type
    fn generate_background_pattern(coupon_type: &str) -> String {
        match coupon_type {
            "Jackpot" => {
                format!(r#"<rect x="80" y="140" width="25" height="25" fill="{}" transform="rotate(45 92 152)"/>
                   <rect x="280" y="240" width="20" height="20" fill="{}" transform="rotate(45 290 250)"/>
                   <rect x="60" y="380" width="15" height="15" fill="{}" transform="rotate(45 67 387)"/>
                   <rect x="320" y="430" width="30" height="30" fill="{}" transform="rotate(45 335 445)"/>"#,
                   "#fbbf24", "#f59e0b", "#d97706", "#92400e")
            }
            "Big Win" => {
                format!(r#"<polygon points="100,120 115,150 85,150" fill="{}"/>
                   <polygon points="300,220 315,250 285,250" fill="{}"/>
                   <polygon points="120,420 135,450 105,450" fill="{}"/>"#,
                   "#10b981", "#059669", "#047857")
            }
            "Win" => {
                format!(r#"<circle cx="90" cy="160" r="12" fill="{}"/>
                   <circle cx="310" cy="260" r="10" fill="{}"/>
                   <circle cx="110" cy="440" r="15" fill="{}"/>"#,
                   "#3b82f6", "#1d4ed8", "#1e40af")
            }
            _ => {
                format!(r#"<circle cx="120" cy="200" r="8" fill="{}"/>
                   <circle cx="280" cy="350" r="6" fill="{}"/>
                   <circle cx="150" cy="480" r="10" fill="{}"/>"#,
                   "#6b7280", "#4b5563", "#374151")
            }
        }
    }

    /// Generate JSON attributes for the coupon token
    pub fn get_attributes(data: CouponData) -> Result<String> {
        let CouponData {
            coupon_id,
            stake_amount,
            base_xor,
            stake_bonus,
            final_result,
            creation_block,
            current_block,
            coupon_type,
            is_winner,
        } = data;

        let age = current_block.saturating_sub(creation_block);
        let rarity_score = Self::calculate_rarity_score(final_result, stake_amount);

        let attributes = format!(
            r#"{{
  "name": "{} Coupon #{}",
  "description": "A gambling coupon generated from blockchain entropy. Each coupon represents a unique bet with win/loss determined by cryptographic randomness and XOR calculations.",
  "image": "data:image/svg+xml;base64,{{SVG_DATA}}",
  "attributes": [
    {{
      "trait_type": "Type",
      "value": "{}"
    }},
    {{
      "trait_type": "Result",
      "value": "{}"
    }},
    {{
      "trait_type": "Final Score",
      "value": {}
    }},
    {{
      "trait_type": "Stake Amount",
      "value": {}
    }},
    {{
      "trait_type": "Base XOR",
      "value": {}
    }},
    {{
      "trait_type": "Stake Bonus",
      "value": {}
    }},
    {{
      "trait_type": "Final Result",
      "value": {}
    }},
    {{
      "trait_type": "Creation Block",
      "value": {}
    }},
    {{
      "trait_type": "Age (Blocks)",
      "value": {}
    }},
    {{
      "trait_type": "Rarity Score",
      "value": {}
    }}
  ]
}}"#,
            coupon_type,
            coupon_id,
            coupon_type,
            if is_winner { "Winner" } else { "Loser" },
            final_result,
            stake_amount,
            base_xor,
            stake_bonus,
            final_result,
            creation_block,
            age,
            rarity_score
        );

        Ok(attributes)
    }

    /// Calculate a rarity score based on various factors
    fn calculate_rarity_score(final_result: u8, stake_amount: u128) -> u32 {
        let mut score = final_result as u32;
        
        // Add bonus for stake amount
        if stake_amount >= 10000 {
            score += 50;
        } else if stake_amount >= 5000 {
            score += 30;
        } else if stake_amount >= 1000 {
            score += 15;
        } else if stake_amount >= 100 {
            score += 5;
        }
        
        // Add bonus for high scores
        match final_result {
            250..=255 => score += 100, // Jackpot bonus
            230..=249 => score += 50,  // Big win bonus
            200..=229 => score += 25,  // Win bonus
            170..=199 => score += 10,  // Small win bonus
            _ => {}
        }
        
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_generation() {
        let data = CouponData {
            coupon_id: 1,
            stake_amount: 5000,
            base_xor: 200,
            stake_bonus: 25,
            final_result: 225,
            creation_block: 1000,
            current_block: 1100,
            coupon_type: "Win".to_string(),
            is_winner: true,
        };

        let svg = SvgGenerator::generate_svg(data).unwrap();
        assert!(svg.contains("svg"));
        assert!(svg.contains("Win Ticket"));
        assert!(svg.contains("#1"));
        assert!(svg.contains("WINNER"));
    }

    #[test]
    fn test_attributes_generation() {
        let data = CouponData {
            coupon_id: 1,
            stake_amount: 5000,
            base_xor: 200,
            stake_bonus: 25,
            final_result: 225,
            creation_block: 1000,
            current_block: 1100,
            coupon_type: "Win".to_string(),
            is_winner: true,
        };

        let attributes = SvgGenerator::get_attributes(data).unwrap();
        assert!(attributes.contains("Win Coupon #1"));
        assert!(attributes.contains("\"value\": 225"));
        assert!(attributes.contains("\"value\": 5000"));
        assert!(attributes.contains("Winner"));
    }

    #[test]
    fn test_color_calculation() {
        let (primary, secondary, accent) = SvgGenerator::calculate_colors(255, 100, true);
        assert_eq!(primary, "#fbbf24"); // Gold for winner
        
        let (primary, secondary, accent) = SvgGenerator::calculate_colors(255, 100, false);
        assert_eq!(primary, "#dc2626"); // Red for loser
    }

    #[test]
    fn test_rarity_score() {
        let score = SvgGenerator::calculate_rarity_score(255, 10000);
        assert!(score > 300); // Should be high for jackpot with large stake
        
        let score = SvgGenerator::calculate_rarity_score(150, 0);
        assert!(score < 200); // Should be lower for loss with no stake
    }
}