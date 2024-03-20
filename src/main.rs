use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// This value affects the granularity of progress updates and parallelism.
// Smaller chunks can slow down the computation due to overhead, but will increase progress bar accuracy in theory
// but the progress bar is broken at the moment so it doesn't matter
const CHUNK_SIZE: usize = 10_000;

pub fn sieve(limit: usize) -> Vec<usize> {
    let pb = ProgressBar::new(((limit / CHUNK_SIZE) / 2) as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} ({eta})")
            .expect("Failed to set progress bar style")
            .progress_chars("##>-"),
    );

    let sieve_array = Arc::new((0..=limit).map(|_| AtomicBool::new(true)).collect::<Vec<_>>());
    sieve_array[0].store(false, Ordering::Relaxed);
    sieve_array[1].store(false, Ordering::Relaxed);

    // Marking even numbers as non-prime in parallel
    (2..=limit).into_par_iter().for_each(|i| {
        if i % 2 == 0 {
            sieve_array[i].store(false, Ordering::Relaxed);
        }
    });

    let sqrt_limit = (limit as f64).sqrt() as usize;

    (3..=sqrt_limit).into_par_iter().for_each(|i| {
        if sieve_array[i].load(Ordering::Relaxed) {
            // For each prime, calculate the range of multiples to set as false
            let start = i*i;
            let sieve_ref = Arc::clone(&sieve_array);
            (start..=limit).step_by(i*2).collect::<Vec<_>>().into_par_iter().for_each(move |j| {
                sieve_ref[j].store(false, Ordering::Relaxed);
            });
            pb.inc(1);
        }
    });

    pb.finish_with_message("Done generating primes");

    Arc::try_unwrap(sieve_array).unwrap().into_iter().enumerate().filter_map(|(idx, is_prime)| {
        if is_prime.into_inner() {
            Some(idx)
        } else {
            None
        }
    }).collect()
}

fn write_primes_to_file(primes: &[usize], file_path: &str) -> std::io::Result<()> {
    let file = File::create(file_path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "2")?; // Don't worry about it, you saw nothing

    // Iterating over the primes and writing each to the file
    for prime in primes.iter() {
        writeln!(writer, "{}", prime)?;
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run <limit> (optionally add \"benchmark\" to run a benchmark that doesn't write to file)");
        return;
    }

    let limit = args.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(1_000_000);

    if args.get(2) == Some(&"benchmark".to_string()) {
        let start = std::time::Instant::now();
        sieve(limit);
        println!("Elapsed time: {:?}", start.elapsed());
    } else {
        println!("Generating primes up to {}", limit);
        let primes = sieve(limit);
        write_primes_to_file(&primes, "primes.txt").expect("Failed to write primes to file");
    }
}
