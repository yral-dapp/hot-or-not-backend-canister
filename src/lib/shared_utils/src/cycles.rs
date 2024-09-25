use crate::constant::{
    ASSUMED_BYTES_PER_INGRESS_CALL, ASSUMED_NUMBER_OF_INGRESS_CALL_PER_SEC,
    ASSUMED_NUMBER_OF_INSTRUCTIONS_PER_INGRESS_CALL, BASE_COST_FOR_EXECUTION,
    BASE_COST_FOR_INGRESS_MESSAGE, COST_PER_BILLION_INSTRUCTION_EXECUTED,
    COST_PER_BYTE_FOR_INGRESS_MESSAGE, DEFAULT_FREEZING_THRESHOLD,
    MAX_NUMBER_OF_DAYS_TO_KEEP_CANISTER_RUNNING, RESERVED_NUMBER_OF_INSTRUCTIONS_FOR_INSTALL_CODE,
    THRESHOLD_NUMBER_OF_DAYS_TO_KEEP_CANISTER_RUNNING,
};

pub fn get_cycles_reserved_in_freezing_threshold(
    idle_cycles_burned_per_day: u128,
    freezing_threshold_in_days: Option<u128>,
) -> u128 {
    let freezing_threshold_in_days =
        freezing_threshold_in_days.unwrap_or(DEFAULT_FREEZING_THRESHOLD);
    idle_cycles_burned_per_day * freezing_threshold_in_days
}

pub fn get_execution_cost_per_ingress_message() -> u128 {
    let res = BASE_COST_FOR_EXECUTION
        + ((ASSUMED_NUMBER_OF_INSTRUCTIONS_PER_INGRESS_CALL
            * COST_PER_BILLION_INSTRUCTION_EXECUTED)
            / 1_000_000_000);

    res
}

pub fn get_cycles_required_per_ingress_message_reception() -> u128 {
    let res = BASE_COST_FOR_INGRESS_MESSAGE
        + (ASSUMED_BYTES_PER_INGRESS_CALL * COST_PER_BYTE_FOR_INGRESS_MESSAGE);

    res
}

pub fn calculate_cost_for_canister_per_day(idle_cycles_burned_per_day: u128) -> u128 {
    let ingress_message_cycles_cost_per_day = (24 * 60 * 60)
        * get_cycles_required_per_ingress_message_reception()
        * ASSUMED_NUMBER_OF_INGRESS_CALL_PER_SEC;
    let storage_cost_per_day = idle_cycles_burned_per_day;
    let execution_cost_per_day = (24 * 60 * 60)
        * get_execution_cost_per_ingress_message()
        * ASSUMED_NUMBER_OF_INGRESS_CALL_PER_SEC;

    ingress_message_cycles_cost_per_day + storage_cost_per_day + execution_cost_per_day
}

pub fn calculate_recharge_and_threshold_cycles_for_canister(
    idle_cycles_burned_per_day: u128,
    freezing_threshold_in_days: Option<u128>,
) -> (u128, u128) {
    let freezing_threshold_cycles = get_cycles_reserved_in_freezing_threshold(
        idle_cycles_burned_per_day,
        freezing_threshold_in_days,
    );
    let canister_cost_per_day = calculate_cost_for_canister_per_day(idle_cycles_burned_per_day);
    let threshold_cycles_to_keep_canister_running: u128 = freezing_threshold_cycles
        + (THRESHOLD_NUMBER_OF_DAYS_TO_KEEP_CANISTER_RUNNING * canister_cost_per_day);
    let mut recharge_amount_for_canister: u128 = freezing_threshold_cycles
        + (MAX_NUMBER_OF_DAYS_TO_KEEP_CANISTER_RUNNING * canister_cost_per_day);
    recharge_amount_for_canister = recharge_amount_for_canister.min(5_000_000_000_000); //maximum 5T cycles alloted
    (
        threshold_cycles_to_keep_canister_running,
        recharge_amount_for_canister,
    )
}

pub fn calulate_required_cycles_for_upgrading(
    idle_cycles_burned_per_day: u128,
    freezing_threshold_in_days: Option<u128>,
) -> u128 {
    let freezing_threshold_cycles = get_cycles_reserved_in_freezing_threshold(
        idle_cycles_burned_per_day,
        freezing_threshold_in_days,
    );
    let cycles_required_for_upgrade_execution = BASE_COST_FOR_EXECUTION
        + ((RESERVED_NUMBER_OF_INSTRUCTIONS_FOR_INSTALL_CODE
            * COST_PER_BILLION_INSTRUCTION_EXECUTED)
            / 1_000_000_000);
    freezing_threshold_cycles + cycles_required_for_upgrade_execution
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cycles_required_for_upgrade_for_idle_canister() {
        let (threshold, recharge) =
            calculate_recharge_and_threshold_cycles_for_canister(27_000_000, None);
        let cycles_required_for_upgrade = calulate_required_cycles_for_upgrading(27_000_000, None);
        assert!(threshold > cycles_required_for_upgrade);
        assert!(recharge > cycles_required_for_upgrade);
        assert!(recharge < 1_000_000_000_000);
    }

    #[test]
    fn test_threshold_and_recharge_for_filled_canister() {
        let idle_cycles_burned_per_day: u128 = 5_000_000_000;
        let (threshold, recharge) =
            calculate_recharge_and_threshold_cycles_for_canister(idle_cycles_burned_per_day, None);
        let cycles_required_for_upgrade =
            calulate_required_cycles_for_upgrading(idle_cycles_burned_per_day, None);
        assert!(threshold > cycles_required_for_upgrade);
        assert!(recharge > cycles_required_for_upgrade);
        assert!(recharge < 1_000_000_000_000);
    }
}
