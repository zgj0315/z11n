fn main() -> anyhow::Result<()> {
    prost_build::Config::new()
        .out_dir("src/")
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(
            &["../client_service/proto/z11n.proto"],
            &["../client_service/proto/"],
        )?;
    Ok(())
}
