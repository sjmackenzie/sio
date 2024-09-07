#![no_std]
//#![no_main]
//#![feature(future_join)]

use smol::Executor;
extern crate alloc;
use alloc::{vec, vec::Vec};
use alloc::sync::Arc;
use crate::alloc::string::ToString;
use sio_core::scheduler::{Process, Operation};
use sio_core::compiler::value::ThreadValue as Value;

// See http://www.info.ucl.ac.be/~pvr/functional-dataflow.pdf for more details.

fn main() {
    // env_logger::builder()
    //     .filter_level(log::LevelFilter::Debug)
    //     .format_timestamp_nanos()
    //     .init();

    let em0: Vec<Operation> = vec![
        Operation::ThreadSpawn(0, vec![  // step 1
            Operation::int(1, 0, 0)]),   // step 3
        Operation::unbound(0, 0),        // step 2
    ];
    let em1: Vec<Operation> = vec![
        Operation::ThreadSpawn(0, vec![  // step 1
            Operation::WaitNeeded(1, 1), // step 4
            Operation::int(1,1,1),       // step 6
            Operation::unbound(1, 2)]),  // step 7
        Operation::ThreadSpawn(0, vec![  // step 2
            Operation::unbound(2,1),     // step 5
            Operation::int(2,2,2),       // step 8
            Operation::int(2,3,3)]),     // step 9 
        Operation::unbound(0, 3),        // step 3
    ];
    let em2: Vec<Operation> = vec![
        Operation::ThreadSpawn(0, vec![  // step 1
            Operation::int(1,1,1),       // step 4
            Operation::WaitNeeded(1,2),  // step 5
            Operation::int(1,2,2)]),     // step 7
        Operation::ThreadSpawn(1, vec![  // step 2
            Operation::unbound(2,2)]),   // step 6
        Operation::int(0,3,3)            // step 3
    ];
    let em3: Vec<Operation> = vec![
        Operation::ThreadSpawn(0, vec![  // step 1
            Operation::unbound(1,1),     // step 5
            Operation::int(1,3,3)]),     // step 11
        Operation::ThreadSpawn(0, vec![  // step 2
            Operation::WaitNeeded(2,2),  // step 6
            Operation::int(2,1,1),       // step 8
            Operation::int(2,2,2)]),     // step 9
        Operation::ThreadSpawn(0, vec![  // step 3
            Operation::unbound(3,2),     // step 7
            Operation::int(3,3,3)]),     // step 10
        Operation::unbound(0,3)          // step 4
    ];
    let em4: Vec<Operation> = vec![                 // step 1
        Operation::ThreadSpawn(0, vec![             // step 3
            Operation::ThreadSpawn(1, vec![         // step 4
                Operation::ThreadSpawn(2, vec![     // step 5
                    Operation::ThreadSpawn(3, vec![ // step 6
                        Operation::int(4,4,4)       // step 7
                    ])
                ])
            ])
        ]), 
        Operation::unbound(0, 4),                   // step 2
    ];
    let em5: Vec<Operation> = vec![
        Operation::int(0,0,0),
        Operation::ThreadSpawn(0, vec![Operation::unbound(1,0)]),
    ];
    static src: &str =
    "
    url public_key : sio79f708c25a23ed367610facc14035adc7ba4b1bfa9252ef55c6c24f1b9b03abd;
    url type : src;
    url name : app_name;
    url app : public_key::type::name;
    corporal app::Corporal {
        pub master :: () {
            let x, y;
            let z;
            thread {
                let assign_x = lazy (x) {
                    x = 0;
                };
                assign_x(x);
            }
            thread {
                assign_y(y);
            }
            if x == y {
                z = 1;
            }
        }
        lazy assign_y :: (y) {
            y = 0;
        }
    }";
    //use sio_core::run_frontend;
    let ex = Arc::new(Executor::new());
    //let (source, module) = run_frontend(src, path)?;
    let mut process = Process::new(ex.clone(), src.to_string(), "/".to_string());
    //smol::block_on(ex.run(process.run()));
}


