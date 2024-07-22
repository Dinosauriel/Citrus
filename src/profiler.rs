use std::fs::File;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::ops::Mul;
use std::thread;
use std::time::{Duration, Instant};

struct Run {
    start: Instant,
    end: Instant,
}

impl Run {
    fn duration(&self) -> Duration {
        self.end.duration_since(self.start)
    }
}

thread_local! {
    static PROCEDURES: RefCell<HashMap<String, Vec<Run>>> = RefCell::new(HashMap::from([]));
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
            let end = Instant::now();

            PROCEDURES.with_borrow_mut(|pmap| {
                pmap.get_mut(&name_s).unwrap().push(Run { start, end });
            });
        }
    });
}

pub fn p_graph(granularity: Duration) {
    PROCEDURES.with_borrow_mut(|procedures| {
        if procedures.len() == 0 {
            return;
        }
        let t0 = procedures.iter().map(|(_, runs)| { runs.first().unwrap().start }).min().unwrap();
        let te = procedures.iter().map(|(_, runs)| { runs.last().unwrap().end }).max().unwrap();
        // number of time steps
        let n = te.duration_since(t0).as_nanos().div_ceil(granularity.as_nanos()) as usize;

        let name_length = procedures.iter().map(|(name, _)| { name.len() }).max().unwrap_or(0);

        let thread = thread::current();
        let mut file = File::create(format!("profiles/profiler-graph-{}-{:?}.txt", thread.name().unwrap(), thread.id())).unwrap();

        file.write(format!("t0 = {:?}\n", t0).as_bytes()).unwrap();
        file.write(format!("te = {:?}\n", te).as_bytes()).unwrap();
        file.write(format!("granularity = {granularity:?}\n").as_bytes()).unwrap();
        file.write(format!("number of timesteps = {n}\n").as_bytes()).unwrap();

        let tick_width: usize = 20;
        file.write(" ".repeat(name_length + 1).as_bytes()).unwrap();
        for i in 0..n.div_ceil(tick_width) {
            let dstring = format!("{:?}", granularity.mul((i * tick_width) as u32));
            file.write(dstring.as_bytes()).unwrap();
            file.write(" ".repeat(tick_width - dstring.len()).as_bytes()).unwrap();
        }
        file.write(b"\n").unwrap();


        file.write(" ".repeat(name_length + 1).as_bytes()).unwrap();
        for i in 0..n {
            if i % tick_width == 0 {
                file.write(b"|").unwrap();
            } else {
                file.write(b" ").unwrap();
            }
        }
        file.write(b"\n\n").unwrap();

        for (name, runs) in procedures {
            file.write(format!("{name}").as_bytes()).unwrap();
            file.write(" ".repeat(name_length + 1 - name.len()).as_bytes()).unwrap();
            let mut t = t0;

            let mut i: usize = 0;
            while t < te {
                let t_next = t + granularity;
                if i >= runs.len() {
                    break;
                }
                
                if runs[i].end < t_next {
                    file.write(b">").unwrap();
                    while i < runs.len() && runs[i].end < t_next {
                        i += 1;
                    }
                } else {
                    if runs[i].start < t {
                        file.write(b"-").unwrap();
                    } else if runs[i].start < t_next {
                        file.write(b"|").unwrap();
                    } else {
                        file.write(b" ").unwrap();
                    }
                }
                t += granularity;
            }
            file.write(b"\n").unwrap();
        }
    });
}

// summarize accumulated state
pub fn p_summary() {
    PROCEDURES.with_borrow_mut(|map| {
        let mut map: Vec<_> = map.iter().collect();
        map.sort_by(|(k1, _), (k2, _)| { k1.cmp(k2) });
        let thread = thread::current();
        let mut file = File::create(format!("profiles/profiler-{}-{:?}.csv", thread.name().unwrap(), thread.id())).unwrap();
        file.write(b"name, n, min, max, avg, p50, p95, total\n").unwrap();
        for (name, runs) in map {
            let n = runs.len();
            let mut durations: Vec<_> = runs.iter().map(Run::duration).collect();
            durations.sort();
            let p50 = durations[n / 2];
            let p95 = durations[(n as f64 * 0.95) as usize];
            let total = durations.iter().sum::<Duration>();
            let avg = total / durations.len() as u32;
            let min = durations.iter().min().unwrap();
            let max = durations.iter().max().unwrap();

            file.write(format!("{}, {}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}\n",
                                    name, n, min, max, avg, p50, p95, total).as_bytes()).unwrap();
        }
    });
}