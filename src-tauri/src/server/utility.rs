pub fn verify_required_data<T>(vector: &[Option<T>]) -> bool {
    let mut has_value = true;
    for field in vector {
        if field.is_none() {
            has_value = false;
            break;
        }
    }
    return has_value;
}
