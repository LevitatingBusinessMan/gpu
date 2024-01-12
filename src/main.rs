#![feature(libc)]
extern crate libc;

use libc::{c_void, c_uint};
use opencl3::command_queue::{CommandQueue, CL_QUEUE_PROFILING_ENABLE};
use opencl3::context::Context;
use opencl3::device::{get_all_devices, Device, CL_DEVICE_TYPE_GPU};
use opencl3::kernel::{ExecuteKernel, Kernel};
use opencl3::memory::*;
use opencl3::program::Program;
use opencl3::types::*;
use opencl3::Result;

const PROGRAM_SOURCE: &str = include_str!("md5.cl");

const KERNEL_NAME: &str = "md5";

fn main() -> Result<()> {
    // Find a usable device for this application
    let device_id = *get_all_devices(CL_DEVICE_TYPE_GPU)?
        .first()
        .expect("no device found in platform");
    let device = Device::new(device_id);

    // Create a Context on an OpenCL device
    let context = Context::from_device(&device).expect("Context::from_device failed");

    // Create a command_queue on the Context's device
    let queue = CommandQueue::create_default(&context, CL_QUEUE_PROFILING_ENABLE)
        .expect("CommandQueue::create_default failed");

    // Build the OpenCL program source and create the kernel.
    let program = Program::create_and_build_from_source(&context, PROGRAM_SOURCE, "")
        .expect("Program::create_and_build_from_source failed");
    let kernel = Kernel::create(&program, KERNEL_NAME).expect("Kernel::create failed");

	let mut s: [cl_uchar; 64] = [
		7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 4,
		11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21
	];

	let mut k: [cl_uint; 64] = [
		0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee,
		0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
		0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be,
		0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
		0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa,
		0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
		0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
		0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
		0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c,
		0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
		0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05,
		0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
		0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039,
		0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
		0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1,
		0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
	];
	

    let sbuf = unsafe {
        Buffer::<cl_uchar>::create(&context, CL_MEM_USE_HOST_PTR, 64, s.as_mut_ptr() as *mut c_void)?
    };

    let kbuf = unsafe {
        Buffer::<cl_uint>::create(&context, CL_MEM_USE_HOST_PTR, 64, k.as_mut_ptr() as *mut c_void)?
    };

	let mut message: Vec<u8> = "The quick brown fox jumps over the lazy dog".to_string().into_bytes();
	let msg_len = message.len();
	message.append(&mut ((0x80) as u64).to_le_bytes().to_vec()); // append a bit
	while message.len() * 8 < 448 { // append 0x00 untill at length 448
		message.push(0x00);
	}

	message.append(&mut ((msg_len * 8) as u64).to_le_bytes().to_vec()); // append the length as a u64

	println!("Message:");
	for bytes in message.chunks(16) {
		for bytes in bytes.chunks(4) {
			for b in bytes.iter().rev() {
				print!("{b:02x?}");
			}
			print!(" ");
		}
		print!("\n");
	}

	// let mut message: [cl_uint; 16] = [
	// 	0xb00bb00b_u32.to_be(), 0xb00bb00b_u32.to_be(), 0xb00bb00b_u32.to_be(), 0xb00bb00b_u32.to_be(),
	// 	0x00000080, 0x00000000, 0x00000000, 0x00000000, // append a bit, meaning append 0x80 byte
	// 	0x00000000, 0x00000000, 0x00000000, 0x00000000,
	// 	0x00000000, 0x00000000, 0x00000080, 0x00000000, // last 64 bits is length in bits
	// ];

    let messagebuf = unsafe {
        Buffer::<cl_uint>::create(&context, CL_MEM_USE_HOST_PTR, 64, message.as_mut_ptr() as *mut c_void)?
    };

    let mut digest: [cl_uint; 4] = [0; 4];

    let digestbuf = unsafe {
        Buffer::<cl_uint>::create(&context, CL_MEM_USE_HOST_PTR, 16, digest.as_mut_ptr() as *mut c_void)?
    };

    let kernel_event = unsafe {
        ExecuteKernel::new(&kernel)
            .set_arg(&sbuf)
            .set_arg(&kbuf)
            .set_arg(&messagebuf)
            .set_arg(&digestbuf)
            .set_global_work_size(1)
            .enqueue_nd_range(&queue)?
    };

    let mut events: Vec<cl_event> = Vec::default();
    events.push(kernel_event.get());

    kernel_event.wait()?;

    // Calculate the kernel duration, from the kernel_event
    let start_time = kernel_event.profiling_command_start()?;
    let end_time = kernel_event.profiling_command_end()?;
    let duration = end_time - start_time;
    println!("kernel execution duration (ns): {}", duration);

	let digest: [cl_uint; 4] = digest.map(|b: u32| b.to_be());

	println!("{:x?}", digest);

    Ok(())
}
