use serde_json::{json, Value};

/// Shared AI billing settlement rules for synchronous gateway calls and async jobs.
///
/// Keep reservation capture/release math here so product-specific AI billing changes do
/// not diverge between request paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ChargeSettlement {
    pub(crate) requested_minor: i64,
    pub(crate) captured_minor: i64,
    pub(crate) released_minor: i64,
    pub(crate) additional_minor: i64,
    pub(crate) shortfall_minor: i64,
}

pub(crate) fn settle_charge(
    requested_minor: i64,
    reservation_held_minor: i64,
    wallet_balance_minor: i64,
    wallet_held_minor: i64,
) -> ChargeSettlement {
    let requested_minor = requested_minor.max(0);
    let reservation_held_minor = reservation_held_minor.max(0);
    let available_minor = wallet_balance_minor
        .saturating_sub(wallet_held_minor)
        .max(0);
    let max_capturable_minor = reservation_held_minor.saturating_add(available_minor);
    let captured_minor = requested_minor.min(max_capturable_minor);

    ChargeSettlement {
        requested_minor,
        captured_minor,
        released_minor: (reservation_held_minor - captured_minor).max(0),
        additional_minor: (captured_minor - reservation_held_minor).max(0),
        shortfall_minor: (requested_minor - captured_minor).max(0),
    }
}

pub(crate) fn settlement_metadata(settlement: ChargeSettlement) -> Value {
    json!({
        "billing_settlement": {
            "requested_minor": settlement.requested_minor,
            "captured_minor": settlement.captured_minor,
            "released_minor": settlement.released_minor,
            "additional_minor": settlement.additional_minor,
            "shortfall_minor": settlement.shortfall_minor,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{settle_charge, settlement_metadata};

    #[test]
    fn captures_reserved_amount_and_releases_unused_hold() {
        let settlement = settle_charge(80, 100, 1000, 100);

        assert_eq!(settlement.requested_minor, 80);
        assert_eq!(settlement.captured_minor, 80);
        assert_eq!(settlement.released_minor, 20);
        assert_eq!(settlement.additional_minor, 0);
        assert_eq!(settlement.shortfall_minor, 0);
    }

    #[test]
    fn can_capture_more_than_reserved_when_free_balance_exists() {
        let settlement = settle_charge(200, 137, 1000, 137);

        assert_eq!(settlement.captured_minor, 200);
        assert_eq!(settlement.additional_minor, 63);
        assert_eq!(settlement.released_minor, 0);
        assert_eq!(settlement.shortfall_minor, 0);
    }

    #[test]
    fn does_not_consume_other_active_holds() {
        let settlement = settle_charge(500, 100, 500, 300);

        assert_eq!(settlement.captured_minor, 300);
        assert_eq!(settlement.additional_minor, 200);
        assert_eq!(settlement.released_minor, 0);
        assert_eq!(settlement.shortfall_minor, 200);
    }

    #[test]
    fn clamps_negative_inputs_to_zero() {
        let settlement = settle_charge(-1, -1, -1, 10);

        assert_eq!(settlement.requested_minor, 0);
        assert_eq!(settlement.captured_minor, 0);
        assert_eq!(settlement.released_minor, 0);
        assert_eq!(settlement.additional_minor, 0);
        assert_eq!(settlement.shortfall_minor, 0);
    }

    #[test]
    fn metadata_uses_stable_billing_settlement_shape() {
        let metadata = settlement_metadata(settle_charge(120, 100, 300, 100));

        assert_eq!(metadata["billing_settlement"]["requested_minor"], 120);
        assert_eq!(metadata["billing_settlement"]["captured_minor"], 120);
        assert_eq!(metadata["billing_settlement"]["additional_minor"], 20);
    }
}
