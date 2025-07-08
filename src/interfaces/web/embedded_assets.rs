use rust_embed::Embed;

/// WebUIの静的アセットを埋め込む
#[derive(Embed)]
#[folder = "web/"]
#[include = "*"]
#[include = "**/*"]
pub struct WebAssets;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_assets_available() {
        // index.htmlが埋め込まれていることを確認
        let index = WebAssets::get("index.html");
        assert!(index.is_some());

        // CSSファイルが埋め込まれていることを確認
        let css = WebAssets::get("css/style.css");
        assert!(css.is_some());

        // JSファイルが埋め込まれていることを確認
        let js = WebAssets::get("js/app.js");
        assert!(js.is_some());
    }
}