#![no_main]
libfuzzer_sys::fuzz_target!(|input: &[u8]| {
    ess_specification_fuzz_engine::callback(
        ess_specification_fuzz::observation::Entry::ByteCarrier,
        input,
    );
});
