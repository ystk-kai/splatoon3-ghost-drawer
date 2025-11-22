fn main() {
    // ビルド時刻を環境変数として設定
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", timestamp);

    // ソースファイルが変更されたときのみ再ビルド
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=build.rs");
}
