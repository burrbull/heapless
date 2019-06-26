//! Compile fail tests
//!
//! # `Send`-ness
//!
//! Collections of `Send`-able things are `Send`
//!
//! ```
//! use heapless::Vec;
//! use heapless::spsc::{Consumer, Queue, Producer};
//!
//! struct IsSend;
//!
//! unsafe impl Send for IsSend {}
//!
//! fn is_send<T>() where T: Send {}
//!
//! is_send::<Consumer<IsSend, 4>>();
//! is_send::<Producer<IsSend, 4>>();
//! is_send::<Queue<IsSend, 4>>();
//! is_send::<Vec<IsSend, 4>>();
//! ```
//!
//! Collections of non-`Send`-able things are *not* `Send`
//!
//! ``` compile_fail
//! use std::marker::PhantomData;
//! use heapless::ring_buffer::Consumer;
//!
//! type NotSend = PhantomData<*const ()>;
//!
//! fn is_send<T>() where T: Send {}
//!
//! is_send::<Consumer<NotSend, 4>>();
//! ```
//!
//! ``` compile_fail
//! use std::marker::PhantomData;
//! use heapless::ring_buffer::Producer;
//!
//! type NotSend = PhantomData<*const ()>;
//!
//! fn is_send<T>() where T: Send {}
//!
//! is_send::<Producer<NotSend, 4>>();
//! ```
//!
//! ``` compile_fail
//! use std::marker::PhantomData;
//! use heapless::spsc::Queue;
//!
//! type NotSend = PhantomData<*const ()>;
//!
//! fn is_send<T>() where T: Send {}
//!
//! is_send::<Queue<NotSend, [NotSend; 4]>>();
//! ```
//!
//! ``` compile_fail
//! use std::marker::PhantomData;
//! use heapless::Vec;
//!
//! type NotSend = PhantomData<*const ()>;
//!
//! fn is_send<T>() where T: Send {}
//!
//! is_send::<Vec<NotSend, [NotSend; 4]>>();
//! ```
//!
//! # Freeze
//!
//! Splitting a `Queue` should invalidate the original reference.
//!
//! ``` compile_fail
//! use heapless::spsc::Queue;
//!
//! let mut rb: Queue<u8, [u8; 4]> = Queue::new();
//!
//! let (p, c) = rb.split();
//! rb.enqueue(0).unwrap();
//! ```
