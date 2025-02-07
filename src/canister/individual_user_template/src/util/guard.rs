use crate::api::profile::get_profile_details_v2::get_profile_details_v2;

pub fn is_caller_profile_owner() -> Result<(), String> {

    if ic_cdk::caller() != get_profile_details_v2().principal_id{
        return Err("Unauthorize".to_string());
    }
    Ok(())
}