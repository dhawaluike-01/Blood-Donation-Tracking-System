#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, symbol_short, Address};

// Structure to track blood donation details
#[contracttype]
#[derive(Clone)]
pub struct BloodDonation {
    pub donation_id: u64,
    pub donor_address: Address,
    pub blood_type: String,
    pub donation_time: u64,
    pub storage_temp: i32,      // Temperature in Celsius
    pub is_contaminated: bool,
    pub recipient_address: Address,
    pub is_delivered: bool,
}

// Structure to track overall statistics
#[contracttype]
#[derive(Clone)]
pub struct DonationStats {
    pub total_donations: u64,
    pub active_donations: u64,
    pub delivered_donations: u64,
    pub contaminated_donations: u64,
}

// Symbol for donation counter
const DONATION_COUNT: Symbol = symbol_short!("D_COUNT");

// Symbol for overall statistics
const STATS: Symbol = symbol_short!("STATS");

// Enum for mapping donation ID to donation data
#[contracttype]
pub enum DonationBook {
    Donation(u64),
}

#[contract]
pub struct BloodDonationContract;

#[contractimpl]
impl BloodDonationContract {
    
    // Function to register a new blood donation
    pub fn register_donation(
        env: Env,
        donor: Address,
        blood_type: String,
        storage_temp: i32,
    ) -> u64 {
        // Verify donor authorization
        donor.require_auth();
        
        // Get and increment donation counter
        let mut count: u64 = env.storage().instance().get(&DONATION_COUNT).unwrap_or(0);
        count += 1;
        
        let time = env.ledger().timestamp();
        
        // Create new donation record
        let donation = BloodDonation {
            donation_id: count,
            donor_address: donor.clone(),
            blood_type: blood_type.clone(),
            donation_time: time,
            storage_temp,
            is_contaminated: false,
            recipient_address: Address::from_string(&String::from_str(&env, "none")),
            is_delivered: false,
        };
        
        // Update statistics
        let mut stats = Self::view_stats(env.clone());
        stats.total_donations += 1;
        stats.active_donations += 1;
        
        // Store donation data
        env.storage().instance().set(&DonationBook::Donation(count), &donation);
        env.storage().instance().set(&DONATION_COUNT, &count);
        env.storage().instance().set(&STATS, &stats);
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Blood Donation Registered: ID {}", count);
        
        count
    }
    
    // Function to update storage conditions and check for contamination
    pub fn update_storage_condition(
        env: Env,
        donation_id: u64,
        new_temp: i32,
    ) {
        let mut donation = Self::view_donation(env.clone(), donation_id);
        
        // Ensure donation exists
        if donation.donation_id == 0 {
            log!(&env, "Donation not found");
            panic!("Donation not found");
        }
        
        // Check if already delivered
        if donation.is_delivered {
            log!(&env, "Cannot update - donation already delivered");
            panic!("Cannot update delivered donation");
        }
        
        // Update temperature
        donation.storage_temp = new_temp;
        
        // Check for contamination (safe range: 2°C to 6°C)
        if new_temp < 2 || new_temp > 6 {
            if !donation.is_contaminated {
                donation.is_contaminated = true;
                
                let mut stats = Self::view_stats(env.clone());
                stats.contaminated_donations += 1;
                stats.active_donations -= 1;
                env.storage().instance().set(&STATS, &stats);
                
                log!(&env, "WARNING: Donation {} marked as contaminated due to improper temperature", donation_id);
            }
        }
        
        env.storage().instance().set(&DonationBook::Donation(donation_id), &donation);
        env.storage().instance().extend_ttl(5000, 5000);
    }
    
    // Function to transfer blood to recipient
    pub fn transfer_to_recipient(
        env: Env,
        donation_id: u64,
        recipient: Address,
    ) {
        // Verify recipient authorization
        recipient.require_auth();
        
        let mut donation = Self::view_donation(env.clone(), donation_id);
        
        // Validation checks
        if donation.donation_id == 0 {
            log!(&env, "Donation not found");
            panic!("Donation not found");
        }
        
        if donation.is_contaminated {
            log!(&env, "Cannot transfer contaminated blood");
            panic!("Blood is contaminated");
        }
        
        if donation.is_delivered {
            log!(&env, "Donation already delivered");
            panic!("Already delivered");
        }
        
        // Update donation record
        donation.recipient_address = recipient.clone();
        donation.is_delivered = true;
        
        // Update statistics
        let mut stats = Self::view_stats(env.clone());
        stats.active_donations -= 1;
        stats.delivered_donations += 1;
        
        env.storage().instance().set(&DonationBook::Donation(donation_id), &donation);
        env.storage().instance().set(&STATS, &stats);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Blood Donation {} transferred to recipient", donation_id);
    }
    
    // Function to view a specific donation record
    pub fn view_donation(env: Env, donation_id: u64) -> BloodDonation {
        let key = DonationBook::Donation(donation_id);
        
        env.storage().instance().get(&key).unwrap_or(BloodDonation {
            donation_id: 0,
            donor_address: Address::from_string(&String::from_str(&env, "none")),
            blood_type: String::from_str(&env, "Unknown"),
            donation_time: 0,
            storage_temp: 0,
            is_contaminated: false,
            recipient_address: Address::from_string(&String::from_str(&env, "none")),
            is_delivered: false,
        })
    }
    
    // Function to view overall donation statistics
    pub fn view_stats(env: Env) -> DonationStats {
        env.storage().instance().get(&STATS).unwrap_or(DonationStats {
            total_donations: 0,
            active_donations: 0,
            delivered_donations: 0,
            contaminated_donations: 0,
        })
    }
}

#[cfg(test)]
mod test;