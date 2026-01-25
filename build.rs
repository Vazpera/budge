fn main() {
    let proj_dir = env!("CARGO_MANIFEST_DIR");
    let db_url = format!("sqlite://{}/budge.db", proj_dir);
    println!("cargo:rustc-env=DATABASE_URL={}", db_url);
}
