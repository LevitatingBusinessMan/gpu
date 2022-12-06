// Copyright (c) 2021 Via Technology Ltd. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![feature(libc)]
extern crate libc;

use libc::c_void;

use opencl3::command_queue::{CommandQueue, CL_QUEUE_PROFILING_ENABLE};
use opencl3::context::Context;
use opencl3::device::{get_all_devices, Device, CL_DEVICE_TYPE_GPU};
use opencl3::kernel::{ExecuteKernel, Kernel};
use opencl3::memory::*;
use opencl3::program::Program;
use opencl3::types::*;
use opencl3::Result;
use std::ptr;
use std::boxed::Box;

const PROGRAM_SOURCE: &str = r#"
/* Basic MD5 functions */
#define F(x, y, z) ((x & y) | (~x & z))
#define G(x, y, z) ((x & z) | (y & ~z))
#define H(x, y, z) (x ^ y ^ z)
#define I(x, y, z) (y ^ (x | ~z))

/* ROTATE_LEFT rotates x left n bits */
#define ROTATE_LEFT(x, n) (x << n) | (x >> (32 - n))

// message is 512 bits (should be 16 ints)
// padded like message + 1 + many zeros + 64bit-length
// can be improved according to wikipedia
kernel void md5 (global uchar* s, global uint* k, global uint* message, global uint* result) {
	int A, a0;
	A = a0 = 0x67452301;
	int B, b0;
	B = b0 = 0xefcdab89;
	int C, c0;
	C = c0 = 0x98badcfe;
	int D, d0;
	D = d0 = 0x10325476;

	for (int i=0; i < 64; i++) {
		int f, g;
		if (0 <= i <= 15) {
			f = F(B, C, D);
			g = i; 
		}
		else if (16 <= i <= 31) {
			f = G(B, C, D);
			g = (5 * i + 1) % 16;
		}
		else if (32 <= i <= 47) {
			f = H(B, C, D);
			g = (3 * i + 5) % 16;
		}
		else if (48 <= i <= 63) {
			f = I(B, C, D);
			g = (7 * i) % 16;
		}
		f = f + A + k[i] + message[g];
		A = D;
		D = C;
		C = B;
		B = B + ROTATE_LEFT(f, s[i]);
	}


	result[0] = a0 + A;
	result[1] = b0 + B;
	result[2] = c0 + C;
	result[3] = d0 + D;
}

"#;

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

    /////////////////////////////////////////////////////////////////////
    // Compute data

    // The input data
    const ARRAY_SIZE: usize = 1000;
    let ones: [cl_float; ARRAY_SIZE] = [1.0; ARRAY_SIZE];
    let mut sums: [cl_float; ARRAY_SIZE] = [0.0; ARRAY_SIZE];
    for i in 0..ARRAY_SIZE {
        sums[i] = 1.0 + 1.0 * i as cl_float;
    }

    // Create OpenCL device buffers
    let mut x = unsafe {
        Buffer::<cl_float>::create(&context, CL_MEM_READ_ONLY, ARRAY_SIZE, ptr::null_mut())?
    };
    let mut y = unsafe {
        Buffer::<cl_float>::create(&context, CL_MEM_READ_ONLY, ARRAY_SIZE, ptr::null_mut())?
    };
    let z = unsafe {
        Buffer::<cl_float>::create(&context, CL_MEM_WRITE_ONLY, ARRAY_SIZE, ptr::null_mut())?
    };

	let mut s: [cl_uchar; 64] = [ 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21];

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

	let mut message: [cl_uint; 16] = [
		0xb00bb00b, 0xb00bb00b, 0xb00bb00b, 0xb00bb00c,
		0x00000000, 0x00000000, 0x00000000, 0x00000000,
		0x00000000, 0x00000000, 0x00000000, 0x00000000,
		0x00000000, 0x00000000, 0x00000000, 0x00000020,
	];

	// let mut message: [cl_uint; 16] = [
		// 0x0000000, 0x00000000, 0x00000000, 0x00000000,
		// 0x00000000, 0x00000000, 0x00000000, 0x00000000,
		// 0x00000000, 0x00000000, 0x00000000, 0x00000000,
		// 0x00000000, 0x00000000, 0x00000000, 0x00000000,
	// ];

    let messagebuf = unsafe {
        Buffer::<cl_uint>::create(&context, CL_MEM_USE_HOST_PTR, 64, message.as_mut_ptr() as *mut c_void)?
    };

    let mut result: [cl_uint; 4] = [0; 4];

    let resultbuf = unsafe {
        Buffer::<cl_uint>::create(&context, CL_MEM_USE_HOST_PTR, 16, result.as_mut_ptr() as *mut c_void)?
    };

    
    // Blocking write
    let _x_write_event = unsafe { queue.enqueue_write_buffer(&mut x, CL_BLOCKING, 0, &ones, &[])? };

    // Non-blocking write, wait for y_write_event
    let y_write_event =
        unsafe { queue.enqueue_write_buffer(&mut y, CL_NON_BLOCKING, 0, &sums, &[])? };

    // a value for the kernel function
    let a: cl_float = 300.0;

    // Use the ExecuteKernel builder to set the kernel buffer and
    // cl_float value arguments, before setting the one dimensional
    // global_work_size for the call to enqueue_nd_range.
    // Unwraps the Result to get the kernel execution event.
    // let kernel_event = unsafe {
        // ExecuteKernel::new(&kernel)
            // .set_arg(&z)
            // .set_arg(&x)
            // .set_arg(&y)
            // .set_arg(&a)
            // .set_global_work_size(ARRAY_SIZE)
            // .set_wait_event(&y_write_event)
            // .enqueue_nd_range(&queue)?
    // };

    let kernel_event = unsafe {
        ExecuteKernel::new(&kernel)
            .set_arg(&sbuf)
            .set_arg(&kbuf)
            .set_arg(&messagebuf)
            .set_arg(&resultbuf)
            .set_global_work_size(1)
            .enqueue_nd_range(&queue)?
    };

    let mut events: Vec<cl_event> = Vec::default();
    events.push(kernel_event.get());

    // Create a results array to hold the results from the OpenCL device
    // and enqueue a read command to read the device buffer into the array
    // after the kernel event completes.
    let mut results: [cl_float; ARRAY_SIZE] = [0.0; ARRAY_SIZE];
    let read_event =
        unsafe { queue.enqueue_read_buffer(&z, CL_NON_BLOCKING, 0, &mut results, &events)? };

    // Wait for the read_event to complete.
    read_event.wait()?;

    // Output the first and last results
    println!("results front: {}", results[0]);
    println!("results back: {}", results[ARRAY_SIZE - 1]);

    // Calculate the kernel duration, from the kernel_event
    let start_time = kernel_event.profiling_command_start()?;
    let end_time = kernel_event.profiling_command_end()?;
    let duration = end_time - start_time;
    println!("kernel execution duration (ns): {}", duration);

	println!("{:x?}", result);

    Ok(())
}
