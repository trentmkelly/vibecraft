use super::*;

impl ClientboundTestInstanceBlockStatus {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_component(writer, &self.status)?;
        write_optional(writer, self.size.as_ref(), |writer, size| {
            write_var_i32(writer, size.x)?;
            write_var_i32(writer, size.y)?;
            write_var_i32(writer, size.z)
        })
    }
}
