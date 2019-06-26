#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(warnings)]
/*
use std::{sync::mpsc, thread};

use heapless::{mpmc::Q64, spsc};
use scoped_threadpool::Pool;

#[test]
fn once() {
    static mut RB: spsc::Queue<i32, 4> = spsc::Queue::<i32, 4>::new();

    let rb = unsafe { &mut RB };

    rb.enqueue(0i32).unwrap();

    let (mut p, mut c) = rb.split();

    p.enqueue(1i32).unwrap();

    thread::spawn(move || {
        p.enqueue(1i32).unwrap();
    });

    thread::spawn(move || {
        c.dequeue().unwrap();
    });
}

#[test]
fn twice() {
    static mut RB: spsc::Queue<i32, 4> = spsc::Queue::<i32, 4>::new();

    let rb = unsafe { &mut RB };

    rb.enqueue(0i32).unwrap();
    rb.enqueue(1i32).unwrap();

    let (mut p, mut c) = rb.split();

    thread::spawn(move || {
        p.enqueue(2i32).unwrap();
        p.enqueue(3i32).unwrap();
    });

    thread::spawn(move || {
        c.dequeue().unwrap();
        c.dequeue().unwrap();
    });
}

#[test]
fn scoped() {
    let mut rb = spsc::Queue::<i32, 4>::new();

    rb.enqueue(0i32).unwrap();

    {
        let (mut p, mut c) = rb.split();

        Pool::new(2).scoped(move |scope| {
            scope.execute(move || {
                p.enqueue(1i32).unwrap();
            });

            scope.execute(move || {
                c.dequeue().unwrap();
            });
        });
    }

    rb.dequeue().unwrap();
}

#[test]
fn contention() {
    const N: usize = 1024;

    let mut rb = spsc::Queue::<u8, {N}>::new();

    {
        let (mut p, mut c) = rb.split();

        Pool::new(2).scoped(move |scope| {
            scope.execute(move || {
                let mut sum: u32 = 0;

                for i in 0..(2 * N as u32) {
                    sum = sum.wrapping_add(i);
                    while let Err(_) = p.enqueue(i as u8) {}
                }

                println!("producer: {}", sum);
            });

            scope.execute(move || {
                let mut sum: u32 = 0;

                for _ in 0..(2 * N) {
                    loop {
                        match c.dequeue() {
                            Some(v) => {
                                sum = sum.wrapping_add(v as u32);
                                break;
                            }
                            _ => {}
                        }
                    }
                }

                println!("consumer: {}", sum);
            });
        });
    }

    assert!(rb.is_empty());
}

#[test]
fn mpmc_contention() {
    const N: u32 = 64;

    static Q: Q64<u32> = Q64::new();

    let (s, r) = mpsc::channel();
    Pool::new(2).scoped(|scope| {
        let s1 = s.clone();
        scope.execute(move || {
            let mut sum: u32 = 0;

            for i in 0..(16 * N) {
                sum = sum.wrapping_add(i);
                while let Err(_) = Q.enqueue(i) {}
            }

            s1.send(sum).unwrap();
        });

        let s2 = s.clone();
        scope.execute(move || {
            let mut sum: u32 = 0;

            for _ in 0..(16 * N) {
                loop {
                    match Q.dequeue() {
                        Some(v) => {
                            sum = sum.wrapping_add(v);
                            break;
                        }
                        _ => {}
                    }
                }
            }

            s2.send(sum).unwrap();
        });
    });

    assert_eq!(r.recv().unwrap(), r.recv().unwrap());
}

#[test]
fn unchecked() {
    const N: usize = 1024;

    let mut rb = spsc::Queue::<u8, {N}>::new();

    for _ in 0..N / 2 {
        rb.enqueue(1u8).unwrap();
    }

    {
        let (mut p, mut c) = rb.split();

        Pool::new(2).scoped(move |scope| {
            scope.execute(move || {
                for _ in 0..N / 2 {
                    unsafe {
                        p.enqueue_unchecked(2u8);
                    }
                }
            });

            scope.execute(move || {
                let mut sum: usize = 0;

                for _ in 0..N / 2 {
                    sum = sum.wrapping_add(usize::from(unsafe { c.dequeue_unchecked() }));
                }

                assert_eq!(sum, N / 2);
            });
        });
    }

    assert_eq!(rb.len(), N / 2);
}

#[test]
fn len_properly_wraps() {
    const N: usize = 3;
    let mut rb = spsc::Queue::<u8, {N}>::new();

    rb.enqueue(1u8).unwrap();
    assert_eq!(rb.len(), 1usize);
    rb.dequeue();
    assert_eq!(rb.len(), 0usize);
    rb.enqueue(2u8).unwrap();
    assert_eq!(rb.len(), 1usize);
    rb.enqueue(3u8).unwrap();
    assert_eq!(rb.len(), 2usize);
    rb.enqueue(4u8).unwrap();
    assert_eq!(rb.len(), 3usize);
}

#[test]
fn iterator_properly_wraps() {
    const N: usize = 3;
    let mut rb = spsc::Queue::<u8, {N}>::new();

    rb.enqueue(1u8).unwrap();
    rb.dequeue();
    rb.enqueue(2u8).unwrap();
    rb.enqueue(3u8).unwrap();
    rb.enqueue(4u8).unwrap();
    let expected = [2u8, 3, 4];
    let mut actual = [0, 0, 0];
    for (idx, el) in rb.iter().enumerate() {
        actual[idx] = *el;
    }
    assert_eq!(expected, actual)
}
*/
