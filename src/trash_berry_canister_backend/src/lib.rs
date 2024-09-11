use candid::{candid_method, CandidType, Deserialize, Principal};
use ic_cdk_macros::*;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(CandidType, Serialize, Deserialize, Clone)]
struct GarbageReport {
    id: u64,
    image_hash: [u8; 32],
    latitude: f64,
    longitude: f64,
    reporter: Principal,
    timestamp: u64,
}

#[derive(Default)]
struct TrashBerryStorage {
    reports: HashMap<u64, GarbageReport>,
    next_id: u64,
}

thread_local! {
    static STATE: std::cell::RefCell<TrashBerryStorage> = std::cell::RefCell::default();
}

#[update]
#[candid_method(update)]
fn submit_garbage_report(
    image_data: Vec<u8>,
    latitude: f64,
    longitude: f64,
) -> Result<u64, String> {
    if image_data.is_empty() {
        return Err("Image data cannot be empty".to_string());
    }
    //ici nous dédions de la mémoire a la variable hasher pour puis l'initialisons  avec avec l'image convertie sous forme binaire

    let mut hasher = Sha256::new();
    hasher.update(&image_data);
    let image_hash: [u8; 32] = hasher.finalize().into();

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let id = state.next_id;
        state.next_id += 1;

        let report = GarbageReport {
            id,
            image_hash,
            latitude,
            longitude,
            reporter: ic_cdk::caller(),
            timestamp: ic_cdk::api::time(),
        };

        state.reports.insert(id, report);
        Ok(id)
    })
}

#[query]
#[candid_method(query)]
fn get_garbage_report(id: u64) -> Option<GarbageReport> {
    STATE.with(|state| state.borrow().reports.get(&id).cloned())
}

#[query]
#[candid_method(query)]
fn get_all_garbage_reports() -> Vec<GarbageReport> {
    STATE.with(|state| state.borrow().reports.values().cloned().collect())
}

#[query]
#[candid_method(query)]
fn get_report_locations() -> Vec<(u64, f64, f64)> {
    STATE.with(|state| {
        state
            .borrow()
            .reports
            .values()
            .map(|report| (report.id, report.latitude, report.longitude))
            .collect()
    })
}

ic_cdk::export_candid!();
