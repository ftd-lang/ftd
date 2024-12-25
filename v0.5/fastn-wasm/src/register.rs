impl<STORE: fastn_wasm::StoreExt> fastn_wasm::Store<STORE> {
    pub fn register_functions(&self, linker: &mut wasmtime::Linker<fastn_wasm::Store<STORE>>) {
        // general utility functions
        fastn_wasm::func2!(linker, "env_print", fastn_wasm::env::print);
        fastn_wasm::func0ret!(linker, "env_now", fastn_wasm::env::now);
        fastn_wasm::func2ret!(linker, "env_var", fastn_wasm::env::var);
        fastn_wasm::func0ret!(linker, "env_random", fastn_wasm::env::random);

        // cryptography related stuff
        fastn_wasm::func2ret!(linker, "crypto_encrypt", fastn_wasm::crypto::encrypt);
        fastn_wasm::func2ret!(linker, "crypto_decrypt", fastn_wasm::crypto::decrypt);

        // sqlite
        fastn_wasm::func2ret!(linker, "sqlite_connect", fastn_wasm::sqlite::connect);
        // fastn_wasm::func3ret!(
        //     linker,
        //     "sqlite_query",
        //     fastn_wasm::wasm::exports::sqlite::query
        // );
        // fastn_wasm::func2ret!(
        //     linker,
        //     "sqlite_execute",
        //     fastn_wasm::wasm::exports::sqlite::execute
        // );
        //
        // // pg related stuff
        // fastn_wasm::func2ret!(linker, "pg_connect", fastn_wasm::wasm::exports::pg::connect);
        // fastn_wasm::func3ret!(linker, "pg_query", fastn_wasm::wasm::exports::pg::query);
        // fastn_wasm::func3ret!(linker, "pg_execute", fastn_wasm::wasm::exports::pg::execute);
        // fastn_wasm::func3ret!(
        //     linker,
        //     "pg_batch_execute",
        //     fastn_wasm::wasm::exports::pg::batch_execute
        // );
        //
        // fastn_wasm::func2ret!(
        //     linker,
        //     "sqlite_batch_execute",
        //     fastn_wasm::wasm::exports::sqlite::batch_execute
        // );
        //
        // // request related stuff
        fastn_wasm::func0ret!(linker, "http_get_request", fastn_wasm::http::get_request);
        fastn_wasm::func2ret!(linker, "http_send_request", fastn_wasm::send_request);
        // fastn_wasm::func2!(
        //     linker,
        //     "http_send_response",
        //     fastn_wasm::wasm::exports::http::send_response
        // );

        // document store related
        fastn_wasm::func2ret!(linker, "hostn_tejar_write", fastn_wasm::ds::tejar_write);
        fastn_wasm::func2ret!(linker, "hostn_tejar_read", fastn_wasm::ds::tejar_read);

        // aws
        fastn_wasm::func2ret!(
            linker,
            "hostn_aws_pre_signed_request",
            fastn_wasm::aws::pre_signed_request
        );
    }
}
