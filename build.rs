fn main() -> Result<(), Box<dyn std::error::Error>> {
    // .proto 파일이 변경될 때만 다시 실행
    println!("cargo:rerun-if-changed=proto/engine.proto");
    tonic_build::configure()
        .build_server(false)
        // 이미 prost가 derive하는 부분들임.
        // .type_attribute(".", "#[derive(PartialEq, Clone)]")
        .compile_protos(
            &["proto/engine.proto"],
            &["proto"]
        )?;
    Ok(())
}