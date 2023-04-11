pub fn get_table_name() -> String {
    std::env::var("TABLE_NAME").unwrap()
}
