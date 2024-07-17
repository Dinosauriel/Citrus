use std::fs::File;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

thread_local! {
    static PROCEDURES: RefCell<HashMap<String, Vec<Duration>>> = RefCell::new(HashMap::from([]));
    static START: RefCell<HashMap<String, Instant>> = RefCell::new(HashMap::from([]));
}

pub fn p_start(name: &str) {
    START.with_borrow_mut(|map| {
        map.insert(String::from(name), Instant::now());
    });
}

pub fn p_end(name: &str) {
    let name_s = String::from(name);
    PROCEDURES.with_borrow_mut(|map| {
        if !map.contains_key(&name_s) {
            map.insert(name_s.clone(), vec![]);
        }
    });

    START.with_borrow_mut(|map| {
        if let Some(&start) = map.get(&name_s) {
            map.remove(&name_s);
            let d = Instant::now().duration_since(start);

            PROCEDURES.with_borrow_mut(|pmap| {
                pmap.get_mut(&name_s).unwrap().push(d);
            });
        }
    });
}

// summarize accumulated state
pub fn p_summary() {
    PROCEDURES.with_borrow_mut(|map| {
        let thread = thread::current();
        let mut file = File::create(format!("profiles/profiler-{}-{:?}.csv", thread.name().unwrap(), thread.id())).unwrap();
        file.write(b"name, n, min, max, avg, p50, p95\n").unwrap();
        for (name, durations) in map {
            let n = durations.len();
            let mut d_sorted = durations.clone();
            d_sorted.sort();
            let p50 = d_sorted[n / 2];
            let p95 = d_sorted[(n as f64 * 0.95) as usize];
            let avg = durations.iter().sum::<Duration>() / durations.len() as u32;
            let min = durations.iter().min().unwrap();
            let max = durations.iter().max().unwrap();
            
            file.write(format!("{}, {}, {:?}, {:?}, {:?}, {:?}, {:?}\n", name, n, min, max, avg, p50, p95).as_bytes()).unwrap();
        }
    });
}