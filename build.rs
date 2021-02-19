use std::io;
#[cfg(windows)] use winres::WindowsResource;

fn main() -> io::Result<()> {
    #[cfg(windows)] {
        WindowsResource::new()
            .set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
<security>
    <requestedPrivileges>
        <requestedExecutionLevel level="highestAvailable" uiAccess="false" />
    </requestedPrivileges>
</security>
</trustInfo>
</assembly>
"#)

            .set_icon("src/assets/blackhole.ico")
            .compile()?;
    }
    Ok(())
}