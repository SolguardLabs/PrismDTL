use serde::{Deserialize, Serialize};

use crate::{Amount, PrismError, PrismResult};

const BPS: u128 = 10_000;
const QUADRATIC_SCALE: u128 = BPS * BPS;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BondedRebatePolicy {
    pub routing_epoch: u64,
    pub maximum_rebate: Amount,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BondedRebateQuote {
    pub eligible_depth: Amount,
    pub gross_amount: Amount,
    pub depth_ratio_bps: u32,
    pub rebate: Amount,
}

pub fn quote_bonded_rebate(
    policy: BondedRebatePolicy,
    eligible_depth: Amount,
    gross_amount: Amount,
) -> PrismResult<BondedRebateQuote> {
    if gross_amount.is_zero() {
        return Err(PrismError::ZeroAmount);
    }
    let denominator = eligible_depth.checked_add(gross_amount)?.units();
    let ratio = eligible_depth
        .units()
        .checked_mul(BPS)
        .ok_or(PrismError::AmountOverflow)?
        / denominator;
    let quadratic = ratio.checked_mul(ratio).ok_or(PrismError::AmountOverflow)?;
    let rebate = policy
        .maximum_rebate
        .units()
        .checked_mul(quadratic)
        .ok_or(PrismError::AmountOverflow)?
        / QUADRATIC_SCALE;
    Ok(BondedRebateQuote {
        eligible_depth,
        gross_amount,
        depth_ratio_bps: ratio as u32,
        rebate: Amount::new(rebate)?,
    })
}
