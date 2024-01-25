extern crate sampleworkflowlib;

use neon::prelude::*;
use tokio::runtime::Runtime;
use std::sync::Arc;
use std::thread;
use crate::sampleworkflowlib::network::make_internet_call;
use crate::sampleworkflowlib::asyncruntime::write_to_file_async;
use crate::sampleworkflowlib::childprocess::execute_command;
use crate::sampleworkflowlib::multithreading::start_threads;
use crate::sampleworkflowlib::filesystem::read_file;

fn make_internet_call_wrapper(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let url = cx.argument::<JsString>(0)?.value(&mut cx);

    // To use your make_internet_call function from the sampleworkflowlib library in your Neon wrapper, 
    // you'll need to make some adjustments because your Rust function is async, and Neon's event loop doesn't natively support async/await.

    // You have to use a workaround to execute asynchronous Rust code from a synchronous Neon function.
    // One common approach is to use a thread pool (like tokio::runtime) to handle the async code.
    
    // Create a new Tokio runtime
    let rt = Runtime::new().unwrap();

    // Spawn a new thread to handle the async function
    std::thread::spawn(move || {
        rt.block_on(async {
            match make_internet_call(&url).await {
                Ok(response) => println!("Response: {}", response),
                Err(err) => eprintln!("Error: {:?}", err),
            }
        });
    });

    Ok(cx.undefined())
}


fn write_to_file_wrapper(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value(&mut cx);
    let data = cx.argument::<JsString>(1)?.value(&mut cx);
    let callback = cx.argument::<JsFunction>(2)?.root(&mut cx);
    let channel = cx.channel();

    let path = Arc::new(path);
    let data = Arc::new(data);

    let runtime = Runtime::new().unwrap();
    runtime.spawn(async move {
        let result = write_to_file_async(&path, &data).await;
        channel.send(move |mut cx| {
            let callback = callback.into_inner(&mut cx);
            let this = cx.undefined();
            let args = match result {
                Ok(_) => vec![cx.null().upcast::<JsValue>(), cx.null().upcast()],
                Err(err) => vec![cx.string(err.to_string()).upcast::<JsValue>(), cx.null().upcast()],
            };
            callback.call(&mut cx, this, args)?;
            Ok(())
        });
    });

    Ok(cx.undefined())
}

fn execute_command_wrapper(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let command = cx.argument::<JsString>(0)?.value(&mut cx);
    let command = Arc::new(command);
    let callback = cx.argument::<JsFunction>(1)?.root(&mut cx);
    let channel = cx.channel();

    thread::spawn(move || {
        let result = execute_command(&command.as_ref());

        channel.send(move |mut cx| {
            let callback = callback.into_inner(&mut cx);
            let this = cx.undefined();
            let args = match result {
                Ok(_) => vec![cx.null().upcast::<JsValue>(), cx.null().upcast()],
                Err(err) => vec![cx.string(&err.to_string()).upcast::<JsValue>(), cx.null().upcast()],
            };
            callback.call(&mut cx, this, args)?;
            Ok(())
        });
    });

    Ok(cx.undefined())
}


fn start_threads_wrapper(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let initial_value = cx.argument::<JsNumber>(0)?.value(&mut cx) as i32;
    let callback = cx.argument::<JsFunction>(1)?.root(&mut cx);
    let channel = cx.channel();

    std::thread::spawn(move || {
        let result = start_threads(initial_value);

        channel.send(move |mut cx| {
            let callback = callback.into_inner(&mut cx);
            let this = cx.undefined();
            let args = vec![
                cx.null().upcast::<JsValue>(), 
                cx.number(result as f64).upcast()
            ];
            callback.call(&mut cx, this, args)?;
            Ok(())
        });
    });

    Ok(cx.undefined())
}


fn read_file_wrapper(mut cx: FunctionContext) -> JsResult<JsValue> {
    let path = cx.argument::<JsString>(0)?.value(&mut cx);

    match read_file(&path) {
        Ok(contents) => Ok(cx.string(contents).upcast()),
        Err(err) => cx.throw_error(err.to_string()),
    }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("makeInternetCall", make_internet_call_wrapper)?;
    cx.export_function("writeToFile", write_to_file_wrapper)?;
    cx.export_function("childProcess", execute_command_wrapper)?;
    cx.export_function("multiThreading", start_threads_wrapper)?;
    cx.export_function("readFile", read_file_wrapper)?;
    Ok(())
}

