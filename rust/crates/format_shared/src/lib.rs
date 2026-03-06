use viewer_core::ViewerError;

pub fn parse_shared_part(_part_name: &str) -> Result<(), ViewerError> {
    Err(ViewerError::NotImplemented("shared OOXML part parsing"))
}
