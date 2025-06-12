use std::sync::LazyLock;

use i18n_embed::fluent::{FluentLanguageLoader, fluent_language_loader};
use i18n_embed::DesktopLanguageRequester;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/text"]
struct Localizations;

#[allow(dead_code)]
pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader = fluent_language_loader!();
    let languages = DesktopLanguageRequester::requested_languages();
    i18n_embed::select(&loader, &Localizations, &languages).unwrap();
    loader.set_use_isolating(false);
    loader
});

#[allow(unused_macros)]
macro_rules! fl {
    ($message_id:literal) => {
        ::i18n_embed_fl::fl!($crate::localize::LANGUAGE_LOADER, $message_id)
    };
    ($message_id:literal, $($args:expr),*) => {
        ::i18n_embed_fl::fl!($crate::localize::LANGUAGE_LOADER, $message_id, $($args),*)
    };
}

#[allow(unused_imports)]
pub(crate) use fl;
