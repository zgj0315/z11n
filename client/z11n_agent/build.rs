fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../../server/client_service/proto/z11n.proto")?;
    Ok(())
}
