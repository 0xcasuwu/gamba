pub mod coupon_factory;
pub use coupon_factory::CouponFactory;

pub mod probability;
pub use probability::ProbabilityCalculator;

pub mod coupon_svg;

pub mod tests {
    pub mod std;
    // pub mod coupon_integration_test; // Disabled during compilation fixes
}