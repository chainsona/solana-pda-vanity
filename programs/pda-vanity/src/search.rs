use anchor_lang::prelude::Pubkey;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    let program_id = Pubkey::from_str("7d4pygUVej17wWKY6uiPdFSVPTDKEEAzR4YMmkc1Bss1").unwrap();
    let suffix = "pump";
    let found = Arc::new(AtomicBool::new(false));
    
    // Use all available cores
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8);
    
    println!("Searching for seed for suffix '{}' with {} threads...", suffix, threads);
    let start = std::time::Instant::now();

    let mut handles = vec![];

    // Stride for each thread to avoid overlap
    // We divide the u64 space among threads.
    let stride = u64::MAX / (threads as u64);

    for i in 0..threads {
        let found = found.clone();
        let program_id = program_id;
        let suffix = suffix.to_string();
        let mut seed = stride * (i as u64);
        let end_seed = if i == threads - 1 { u64::MAX } else { seed + stride };

        handles.push(thread::spawn(move || {
            let mut local_count = 0;
            // Optimization: Only check bump 255. 
            // If bump 255 is valid, it IS the canonical bump (highest valid bump).
            // If it's not valid, we skip this seed. 
            // This avoids the loop in find_program_address (255..0) and is ~2x faster per seed on average.
            let bump = 255u8; 
            let bump_slice = &[bump];
            
            while seed < end_seed {
                if found.load(Ordering::Relaxed) {
                    return;
                }

                let seed_bytes = seed.to_le_bytes();
                let seeds_with_bump = &[&seed_bytes[..], bump_slice];
                
                // create_program_address checks if the result is a valid PDA (off-curve)
                // This is the most expensive part (hashing + curve check)
                if let Ok(pda) = Pubkey::create_program_address(seeds_with_bump, &program_id) {
                    // Only encode to string if we have a valid PDA
                    let pda_str = pda.to_string();
                    if pda_str.ends_with(&suffix) {
                        // Found it!
                        if !found.swap(true, Ordering::SeqCst) {
                            println!("Found seed: {}", seed);
                            println!("PDA: {}", pda_str);
                            println!("Bump: {}", bump);
                            println!("Time: {:?}", start.elapsed());
                        }
                        return;
                    }
                }

                seed += 1;
                local_count += 1;
                
                // Print progress from first thread only to avoid clutter
                if i == 0 && local_count % 1_000_000 == 0 {
                     println!("Thread 0 checked {} seeds... (Time: {:?})", local_count, start.elapsed());
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
