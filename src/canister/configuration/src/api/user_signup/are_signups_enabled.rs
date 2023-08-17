use crate::{data::CanisterData, CANISTER_DATA};

#[ic_cdk::query]
#[candid::candid_method(query)]
fn are_signups_enabled() -> bool {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data = canister_data_ref_cell.borrow();
        are_signups_enabled_impl(&canister_data)
    })
}

fn are_signups_enabled_impl(canister_data: &CanisterData) -> bool {
    canister_data.signups_enabled
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_are_signups_enabled_impl() {
        let mut canister_data = CanisterData {
            signups_enabled: true,
            ..Default::default()
        };

        assert!(are_signups_enabled_impl(&canister_data));

        canister_data.signups_enabled = false;
        assert!(!are_signups_enabled_impl(&canister_data));
    }
}
