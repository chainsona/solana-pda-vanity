use anchor_lang::prelude::Pubkey;
use clap::Parser;
use sha2::{Sha256, Digest};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The suffix to search for (e.g., "pump")
    #[arg(short, long, default_value = "pump")]
    suffix: String,

    /// Number of threads to use (defaults to available cores)
    #[arg(short, long)]
    threads: Option<usize>,

    /// Program ID to derive PDA for
    #[arg(short, long, default_value = "7d4pygUVej17wWKY6uiPdFSVPTDKEEAzR4YMmkc1Bss1")]
    program_id: String,
}

fn main() {
    let args = Args::parse();
    
    let program_id = Pubkey::from_str(&args.program_id).expect("Invalid Program ID");
    let suffix = args.suffix;
    let found = Arc::new(AtomicBool::new(false));
    
    // Use user-specified threads or all available cores
    let threads = args.threads.unwrap_or_else(|| {
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8)
    });
    
    println!("Searching for seed for suffix '{}' with {} threads...", suffix, threads);
    let start = std::time::Instant::now();

    // Pre-compute target indices for the suffix
    // We match from the end of the string, so we reverse the suffix.
    // "pump" -> check last char 'p', then 'm', then 'u', then 'p'.
    let alphabet = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut target_indices = Vec::new();
    for &char_byte in suffix.as_bytes().iter().rev() {
        let idx = alphabet.iter().position(|&x| x == char_byte)
            .expect("Invalid character in suffix") as u8;
        target_indices.push(idx);
    }
    let target_indices = Arc::new(target_indices); // Share across threads

    let mut handles = vec![];

    // Stride for each thread to avoid overlap
    // We divide the u64 space among threads.
    let stride = u64::MAX / (threads as u64);
    let pda_marker = b"ProgramDerivedAddress";

    for i in 0..threads {
        let found = found.clone();
        let program_id = program_id;
        let target_indices = target_indices.clone();
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
                
                // OPTIMIZATION v2: Hash first, check suffix, THEN check curve.
                // This avoids the expensive elliptic curve check for 99.99% of seeds.
                let mut hasher = Sha256::new();
                hasher.update(&seed_bytes);
                hasher.update(bump_slice);
                hasher.update(program_id.as_ref());
                hasher.update(pda_marker);
                let hash = hasher.finalize(); // GenericArray<u8, 32>
                
                // Copy for suffix check
                let mut bytes: [u8; 32] = hash.into();
                let original_bytes = bytes;
                
                let mut is_match = true;
                
                for &target in target_indices.iter() {
                    let mut remainder = 0u64;
                    // Perform division in-place on the byte array (treating it as big-endian number base 256)
                    for j in 0..32 {
                        let val = (remainder << 8) | (bytes[j] as u64);
                        bytes[j] = (val / 58) as u8;
                        remainder = val % 58;
                    }
                    
                    if remainder as u8 != target {
                        is_match = false;
                        break;
                    }
                }

                if is_match {
                    // Suffix matches! Now verify it's a valid PDA (must be OFF curve).
                    let pda = Pubkey::new_from_array(original_bytes);
                    if !pda.is_on_curve() {
                        // Found it!
                        if !found.swap(true, Ordering::SeqCst) {
                            println!("Found seed: {}", seed);
                            println!("PDA: {}", pda.to_string()); 
                            println!("Bump: {}", bump);
                            println!("Time: {:?}", start.elapsed());
                        }
                        return;
                    }
                }

                seed += 1;
                local_count += 1;
                
                // Print progress from first thread only to avoid clutter
                if i == 0 && local_count % 5_000_000 == 0 {
                     println!("Thread 0 checked {} seeds... (Time: {:?})", local_count, start.elapsed());
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
