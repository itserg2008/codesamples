extern crate postgres;
extern crate rayon;
extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate llvm_sys as llvm;
use std::{thread, time};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

use llvm::core::*;
use llvm::execution_engine::*;
use llvm::target::*;

use std::mem;

use postgres::{Connection, TlsMode};
use rayon::prelude::*;

use futures::{Future, Stream};
use tokio_io::{io, AsyncRead};
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;

struct Person {
    id: i32,
    name: String,
    data: Option<Vec<u8>>
}

// Tuples can be used as function arguments and as return values
fn reverse(pair: (i32, bool)) -> (bool, i32) {
    // `let` can be used to bind the members of a tuple to variables
    let (integer, boolean) = pair;

    (boolean, integer)
}

// The following struct is for the activity.
#[derive(Debug)]
struct Matrix(f32, f32, f32, f32);

fn long_tuples() {
    // A tuple with a bunch of different types
    let long_tuple = (1u8, 2u16, 3u32, 4u64,
                      -1i8, -2i16, -3i32, -4i64,
                      0.1f32, 0.2f64,
                      'a', true);

    // Values can be extracted from the tuple using tuple indexing
    println!("long tuple first value: {}", long_tuple.0);
    println!("long tuple second value: {}", long_tuple.1);

    // Tuples can be tuple members
    let tuple_of_tuples = ((1u8, 2u16, 2u32), (4u64, -1i8), -2i16);

    // Tuples are printable
    println!("tuple of tuples: {:?}", tuple_of_tuples);

    let pair = (1, true);
    println!("pair is {:?}", pair);

    println!("the reversed pair is {:?}", reverse(pair));

    // To create one element tuples, the comma is required to tell them apart
    // from a literal surrounded by parentheses
    println!("one element tuple: {:?}", (5u32,));
    println!("just an integer: {:?}", (5u32));

    //tuples can be destructured to create bindings
    let tuple = (1, "hello", 4.5, true);

    let (a, b, c, d) = tuple;
    println!("{:?}, {:?}, {:?}, {:?}", a, b, c, d);

    let matrix = Matrix(1.1, 1.2, 2.1, 2.2);
    println!("{:?}", matrix)
    

}


fn main() {

    // The `vec!` macro can be used to initialize a vector
    let mut xs = vec![1i32, 2, 3];
    println!("Initial vector: {:?}", xs);

    // Insert new element at the end of the vector
    println!("Push 4 into the vector");
    xs.push(4);
    println!("Vector: {:?}", xs);

    long_tuples();

    unsafe {
        // Set up a context, module and builder in that context.
        let context = LLVMContextCreate();
        let module = LLVMModuleCreateWithNameInContext(b"sum\0".as_ptr() as *const _,
                                                       context);
        let builder = LLVMCreateBuilderInContext(context);

        // get a type for sum function
        let i64t = LLVMInt64TypeInContext(context);
        let mut argts = [i64t, i64t, i64t];
        let function_type = LLVMFunctionType(
            i64t,
            argts.as_mut_ptr(),
            argts.len() as u32,
            0);

        // add it to our module
        let function = LLVMAddFunction(
            module,
            b"sum\0".as_ptr() as *const _,
            function_type);

        // Create a basic block in the function and set our builder to generate
        // code in it.
        let bb = LLVMAppendBasicBlockInContext(
            context,
            function,
            b"entry\0".as_ptr() as *const _);

        LLVMPositionBuilderAtEnd(builder, bb);

        // get the function's arguments
        let x = LLVMGetParam(function, 0);
        let y = LLVMGetParam(function, 1);
        let z = LLVMGetParam(function, 2);

        let sum = LLVMBuildAdd(builder, x, y, b"sum.1\0".as_ptr() as *const _);
        let sum = LLVMBuildAdd(builder, sum, z, b"sum.2\0".as_ptr() as *const _);

        // Emit a `ret void` into the function
        LLVMBuildRet(builder, sum);

        // done building
        LLVMDisposeBuilder(builder);

        // Dump the module as IR to stdout.
        LLVMDumpModule(module);

        // build an execution engine
        let mut ee = mem::uninitialized();
        let mut out = mem::zeroed();

        // robust code should check that these calls complete successfully
        // each of these calls is necessary to setup an execution engine which compiles to native
        // code
        LLVMLinkInMCJIT();
        LLVM_InitializeNativeTarget();
        LLVM_InitializeNativeAsmPrinter();

        // takes ownership of the module
        LLVMCreateExecutionEngineForModule(&mut ee, module, &mut out);

        let addr = LLVMGetFunctionAddress(ee, b"sum\0".as_ptr() as *const _);

        let f: extern "C" fn(u64, u64, u64) -> u64 = mem::transmute(addr);

        let x: u64 = 1;
        let y: u64 = 1;
        let z: u64 = 1;
        let res = f(x, y, z);

        println!("{} + {} + {} = {}", x, y, z, res);

        // Clean up the rest.
        LLVMDisposeExecutionEngine(ee);
        LLVMContextDispose(context);
    }

    let mut arr = [0, 7, 9, 11];

    arr.par_iter_mut().for_each(|p| *p -= 1);

    println!("{:?}", arr);

    let conn = Connection::connect("postgresql://postgres:password@localhost", TlsMode::None)
        .unwrap();

    conn.execute("CREATE TABLE person (
                    id              SERIAL PRIMARY KEY,
                    name            VARCHAR NOT NULL,
                    data            BYTEA
                  )", &[]).unwrap();
    let me = Person {
        id: 0,
        name: "Steven".to_owned(),
        data: None
    };
    conn.execute("INSERT INTO person (name, data) VALUES ($1, $2)",
                 &[&me.name, &me.data]).unwrap();

    for row in &conn.query("SELECT id, name, data FROM person", &[]).unwrap() {
        let person = Person {
            id: row.get(0),
            name: row.get(1),
            data: row.get(2)
        };
        println!("Found person {}", person.name);
    }

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // Bind the server's socket
    let addr = "127.0.0.1:12345".parse().unwrap();
    let tcp = TcpListener::bind(&addr, &handle).unwrap();

    let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();

    // Iterate incoming connections
    let server = tcp.incoming().for_each(|(tcp, _)| {
        println!("Incoming connection");
        // Split up the read and write halves
        let (reader, writer) = tcp.split();

        // Future of the copy
        let bytes_copied = io::copy(reader, writer);

        // ... after which we'll print what happened
        let handle_conn = bytes_copied.map(|(n, _, _)| {
            println!("wrote {} bytes", n)
        }).map_err(|err| {
            println!("IO error {:?}", err)
        });

        // Spawn the future as a concurrent task
        handle.spawn(handle_conn);

        Ok(())
    });


    thread::spawn(move || {            
            loop {
                println!("this is thread signal");
                let ten_millis = time::Duration::from_millis(1000);
                thread::sleep(ten_millis);
            }
        });

    // Spin up the server on the event loop
    core.run(server).unwrap();
}