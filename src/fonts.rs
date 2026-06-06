use log;
use ruffle_core::backend::ui::FontDefinition;
use ruffle_core::font::FontFileData;
use ruffle_core::font::DefaultFont;
use ruffle_core::Player;
use std::fs;
use std::sync::Arc;

/// CJK font names commonly referenced as device fonts in Chinese SWF games.
/// These cover the fonts SWFs typically request by name.
const CJK_FONT_NAMES: &[&str] = &[
    "SimSun",
    "宋体",
    "NSimSun",
    "新宋体",
    "SimHei",
    "黑体",
    "Microsoft YaHei",
    "微软雅黑",
    "FangSong",
    "仿宋",
    "KaiTi",
    "楷体",
    "FZShuTi",
    "隶书",
    "LiSu",
    "STLiti",
    "华文隶书",
    "FZXiaoZhuanTi",
    "STXingkai",
    "华文行楷",
    "STCaiyun",
    "华文彩云",
    "MingLiU",
    "細明體",
    "PMingLiU",
    "新細明體",
    "SimKai",
    "DFKai-SB",
    "標楷體",
    "Noto Sans CJK SC",
    "Noto Sans SC",
    "DroidSansFallback",
];

/// Canonical device font name for our CJK fallback.
/// When set as the default font for Sans/Serif/Typewriter,
/// any unknown font lookup eventually falls back to this.
const CJK_CANONICAL_NAME: &str = "DroidSansFallback";

/// Font file paths to try on Android, in order of preference.
const SYSTEM_FONT_PATHS: &[(&str, u32)] = &[
    ("/system/fonts/NotoSansSC-Regular.otf", 0),
    ("/system/fonts/NotoSansCJK-Regular.ttc", 0),
    ("/system/fonts/NotoSansHans-Regular.otf", 0),
    ("/system/fonts/NotoSansCJKsc-Regular.otf", 0),
    ("/system/fonts/DroidSansFallback.ttf", 0),
    ("/system/fonts/DroidSansChinese.ttf", 0),
    ("/system/fonts/FangZheng.ttf", 0),
    ("/system/fonts/Miui-Regular.ttf", 0),
];

fn find_cjk_font() -> Option<(Vec<u8>, u32)> {
    for &(path, index) in SYSTEM_FONT_PATHS {
        match fs::read(path) {
            Ok(data) => {
                log::info!(
                    "Found CJK system font: {} ({} bytes, index {})",
                    path,
                    data.len(),
                    index
                );
                return Some((data, index));
            }
            Err(e) => {
                log::debug!("System font not found at {}: {}", path, e);
            }
        }
    }
    log::warn!("No CJK system font found in /system/fonts/");
    None
}

/// Load Android system CJK fonts and install them as both named device fonts
/// AND as the global default fallback for all text rendering.
pub fn load_android_cjk_fonts(player: &mut Player) {
    let (font_bytes, font_index) = match find_cjk_font() {
        Some(found) => found,
        None => {
            log::warn!("Chinese font support unavailable: no CJK system font found");
            return;
        }
    };

    // Share the same font data across all registrations
    let shared_data: Arc<dyn AsRef<[u8]>> = Arc::new(font_bytes);

    // 1) Register under common Chinese font names for SWFs that request them by name
    for &name in CJK_FONT_NAMES {
        let font_data = FontFileData::new_shared(shared_data.clone());
        player.register_device_font(FontDefinition::FontFile {
            name: name.to_string(),
            is_bold: false,
            is_italic: false,
            data: font_data,
            index: font_index,
        });
    }
    log::info!(
        "Registered CJK font under {} Chinese device font names",
        CJK_FONT_NAMES.len()
    );

    // 2) Replace the default fonts (_sans, _serif, _typewriter) with the CJK font.
    //    This is the critical fallback: when a SWF requests ANY unknown font name
    //    (e.g. "隶书" not in our list above), the resolution chain eventually
    //    falls back to DefaultFont::Sans. By making Sans point to our CJK font
    //    instead of the Latin-only Noto Sans subset, ALL CJK text gets a glyph.
    let cjk_name = CJK_CANONICAL_NAME.to_string();
    player.set_default_font(DefaultFont::Sans, vec![cjk_name.clone()]);
    player.set_default_font(DefaultFont::Serif, vec![cjk_name.clone()]);
    player.set_default_font(DefaultFont::Typewriter, vec![cjk_name.clone()]);
    player.set_default_font(DefaultFont::JapaneseGothicMono, vec![cjk_name.clone()]);
    player.set_default_font(DefaultFont::JapaneseGothic, vec![cjk_name.clone()]);
    player.set_default_font(DefaultFont::JapaneseMincho, vec![cjk_name]);

    log::info!("CJK font installed as global default fallback for all text");
}
