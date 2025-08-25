// This file provides a shim for the "env" WASM import module expected by Metashrew.
// It defines global functions that correspond to the host functions linked in metashrew-runtime.

console.log("Loading env.js shim...");
console.log("Current global.env:", global.env);

// Ensure global.env exists, or create it if it doesn't
if (typeof global.env === 'undefined') {
    global.env = {};
    console.log("global.env was undefined, created empty object.");
}

// Define the functions on global.env
global.env.__host_len = function() {
    console.log("WASM Host Function: __host_len called");
    return 0; 
};

global.env.__load_input = function(data_start) {
    console.log(`WASM Host Function: __load_input called with data_start: ${data_start}`);
};

global.env.__log = function(data_start) {
    console.log(`WASM Host Function: __log called with data_start: ${data_start}`);
};

global.env.abort = function(message_ptr, file_ptr, line, column) {
    console.error(`WASM Host Function: abort called. Message Ptr: ${message_ptr}, File Ptr: ${file_ptr}, Line: ${line}, Column: ${column}`);
    throw new Error("WASM Aborted!");
};

global.env.__flush = function(encoded) {
    console.log(`WASM Host Function: __flush called with encoded: ${encoded}`);
};

global.env.__get = function(key_ptr, value_ptr) {
    console.log(`WASM Host Function: __get called with key_ptr: ${key_ptr}, value_ptr: ${value_ptr}`);
};

global.env.__get_len = function(key_ptr) {
    console.log(`WASM Host Function: __get_len called with key_ptr: ${key_ptr}`);
    return 0;
};