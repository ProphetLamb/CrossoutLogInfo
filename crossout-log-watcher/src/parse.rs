use std::{
    fs,
    io::{BufRead, BufReader},
    ops::{Deref, Range},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use chrono::{NaiveDate, NaiveDateTime};
use closure::closure;
use crossbeam::thread;
use crossbeam::{
    channel::{unbounded, Sender},
    queue::SegQueue,
};

use crate::{
    log::{parse_entry, Entry},
    Error,
};

pub fn logs_in_dir(input: PathBuf) -> Result<Vec<(PathBuf, NaiveDateTime)>, Error> {
    let mut log_dirs = Vec::default();
    for dir in input
        .read_dir()?
        .flatten()
        .filter(|sub| sub.file_type().map_or(false, |t| t.is_dir()))
    {
        if let Some(dir_name) = dir.file_name().to_str() && let Ok(date) = NaiveDateTime::parse_from_str(dir_name, "%Y.%m.%d %H.%M.%S") {
            let mut file_name = dir.path();
            file_name.push("combat.log");
            if file_name.exists() {
                log_dirs.push((file_name, date))
            }
        }
    }

    Ok(log_dirs)
}

pub fn parse_logs<
    In: Iterator<Item = (PathBuf, NaiveDate, Range<usize>)>
        + ExactSizeIterator<Item = (PathBuf, NaiveDate, Range<usize>)>,
>(
    logs: In,
) -> (Vec<Entry>, Vec<String>) {
    let entries = Arc::new(SegQueue::new());
    let errors = Arc::new(SegQueue::new());
    io_cpu_upload_bus(
        logs,
        |(log, date, accept_lines), sender| {
            if let Ok(file) = fs::File::open(log) {
                let reader = BufReader::new(file);
                for (pos, line) in reader.lines().flatten().enumerate() {
                    if accept_lines.contains(&pos) {
                        // collect log information for parser
                        _ = sender.send((line, date));
                    }
                }
            }
        },
        |(line, date)| {
            // parse collection information
            if let Ok((_, entry)) = parse_entry::<()>(date)(&line) {
                Ok(Some(entry))
            } else if !line.is_empty() {
                Err(line.clone())
            } else {
                Ok(None)
            }
        },
        500,
        |buf| {
            while let Some(entry) = buf.pop() {
                entries.push(entry);
            }
            Ok(())
        },
        |e| {
            errors.push(e);
        },
    );
    (collect_segq(entries), collect_segq(errors))
}

fn collect_segq<T, Q: Deref<Target = SegQueue<T>>>(q: Q) -> Vec<T> {
    let mut clone = Vec::with_capacity(q.len());
    while let Some(item) = q.pop() {
        clone.push(item);
    }
    clone
}

pub fn io_cpu_upload_bus<
    In: Iterator<Item = T> + ExactSizeIterator<Item = T>,
    T: Send,
    U: Send,
    V: Send,
    E: Send,
    F: Fn(T, Sender<U>) + Send + Copy,
    G: Fn(U) -> Result<Option<V>, E> + Send + Copy,
    H: Fn(Arc<SegQueue<V>>) -> Result<(), E> + Send + Copy,
    I: Fn(E) + Send + Copy,
>(
    input: In,
    io: F,
    cpu: G,
    upload_threshold: usize,
    upload: H,
    error_handler: I,
) {
    let cpu_threads = 1.max(num_cpus::get() - 1); // cpu count - one thread reserved for upload
    let (sender, receiver) = unbounded(); // work queue fed by io, consumed by cpu heavy tasks
    let io_active = Arc::new(AtomicUsize::new(input.len())); // counts active io threads
    let cpu_active = Arc::new(AtomicUsize::new(cpu_threads)); // counts active cpu threads
    let buf = Arc::new(SegQueue::new()); // upload buffer
    thread::scope(|scope| {
        for item in input {
            scope.spawn(closure!(clone io_active, clone sender, |_|{
                io(item, sender);
                io_active.fetch_sub(1, Ordering::SeqCst);
            }));
        }
        for _ in 0..cpu_threads {
            scope.spawn(
                closure!(clone io_active, clone cpu_active, clone buf, clone receiver, |_| {
                    while io_active.load(Ordering::SeqCst) != 0 {
                        while let Ok(item) = receiver.try_recv() {
                            match cpu(item) {
                                Ok(v) => if let Some(v) = v {
                                    buf.push(v);
                                },
                                Err(e) => error_handler(e),
                            }
                        }
                        std::thread::sleep(Duration::new(0,1)); // encourage ctx change on io pipe fail
                    }
                    cpu_active.fetch_sub(1, Ordering::SeqCst);
                }),
            );
        }
        scope.spawn(move |_| {
            while cpu_active.load(Ordering::SeqCst) != 0 {
                if buf.len() >= upload_threshold {
                    // upload when threshold is exceeded
                    match upload(buf.clone()) {
                        Ok(_) => {}
                        Err(e) => error_handler(e),
                    }
                }
                std::thread::sleep(Duration::new(0, 1)); // encourage ctx change
            }
            // all cpu threads are terminated
            if buf.len() != 0 {
                // upload remainder
                match upload(buf.clone()) {
                    Ok(_) => {}
                    Err(e) => error_handler(e),
                }
            }
        });
    })
    .unwrap();
}
