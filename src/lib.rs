extern crate duckdb;
extern crate duckdb_loadable_macros;
extern crate libduckdb_sys;

use duckdb::{
    arrow::datatypes::DataType,
    core::{DataChunkHandle, Inserter, LogicalTypeHandle, LogicalTypeId},
    vscalar::{ArrowFunctionSignature, ScalarFunctionSignature, VScalar},
    vtab::{arrow::WritableVector, BindInfo, InitInfo, TableFunctionInfo, VTab},
    Connection, Result,
};
use duckdb_loadable_macros::duckdb_entrypoint_c_api;
use libduckdb_sys as ffi;
use std::{
    error::Error,
    ffi::CString,
    sync::atomic::{AtomicBool, Ordering},
};

#[repr(C)]
struct HelloBindData {
    name: String,
}

#[repr(C)]
struct HelloInitData {
    done: AtomicBool,
}

struct HelloVTab;

impl VTab for HelloVTab {
    type InitData = HelloInitData;
    type BindData = HelloBindData;

    fn bind(bind: &BindInfo) -> Result<Self::BindData, Box<dyn std::error::Error>> {
        bind.add_result_column("column0", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        let name = bind.get_parameter(0).to_string();
        Ok(HelloBindData { name })
    }

    fn init(_: &InitInfo) -> Result<Self::InitData, Box<dyn std::error::Error>> {
        Ok(HelloInitData {
            done: AtomicBool::new(false),
        })
    }

    fn func(
        func: &TableFunctionInfo<Self>,
        output: &mut DataChunkHandle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let init_data = func.get_init_data();
        let bind_data = func.get_bind_data();
        if init_data.done.swap(true, Ordering::Relaxed) {
            output.set_len(0);
        } else {
            let vector = output.flat_vector(0);
            let result = CString::new(format!("Rusty Quack {} 🐥", bind_data.name))?;
            vector.insert(0, result);
            output.set_len(1);
        }
        Ok(())
    }

    fn parameters() -> Option<Vec<LogicalTypeHandle>> {
        Some(vec![LogicalTypeHandle::from(LogicalTypeId::Varchar)])
    }
}

struct HelloVScalar;

impl VScalar for HelloVScalar {
    type State = ();

    unsafe fn invoke(
        _: &Self::State,
        input: &mut DataChunkHandle,
        output: &mut dyn WritableVector,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let len = input.len();
        let input_vec = input.flat_vector(0);
        let input_values = match input_vec.logical_type().id() {
            LogicalTypeId::Integer => input_vec.as_slice_with_len::<i32>(len),
            id => {
                return Err(format!("Unsupported type: {id:?}").into());
            }
        };

        let mut flat_vec = output.flat_vector();
        flat_vec
            .as_mut_slice_with_len::<i32>(len)
            .iter_mut()
            .zip(input_values.iter())
            .for_each(|(o, i)| {
                *o = 2 * i;
            });
        Ok(())
    }

    fn signatures() -> Vec<duckdb::vscalar::ScalarFunctionSignature> {
        vec![ScalarFunctionSignature::exact(
            vec![LogicalTypeId::Integer.into()],
            LogicalTypeId::Integer.into(),
        )]
    }
}

const TABLE_FUNCITON_NAME: &str = "hello_table";
const SCALAR_FUNCITON_NAME: &str = "hello_scalar";

#[duckdb_entrypoint_c_api]
pub unsafe fn extension_entrypoint(con: Connection) -> Result<(), Box<dyn Error>> {
    con.register_table_function::<HelloVTab>(TABLE_FUNCITON_NAME)
        .expect("Failed to register hello table function");

    con.register_scalar_function::<HelloVScalar>(SCALAR_FUNCITON_NAME)
        .expect("Failed to register hello scalar function");

    Ok(())
}
