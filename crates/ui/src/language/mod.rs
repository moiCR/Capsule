pub mod language_manager;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Language {
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub locale: Option<String>,
    #[serde(default)]
    pub language_env: Option<String>,
    pub common: CommonLang,
    pub datetime: DateTimeLang,
    pub power: PowerLang,
    pub dashboard: DashboardLang,
    pub quick_settings: QuickSettingsLang,
    pub clipboard: ClipboardLang,
    pub launcher: LauncherLang,
    pub themes: ThemesLang,
    pub wallpaper: WallpaperLang,
    pub volume: VolumeLang,
    pub tray: TrayLang,
    pub polkit: PolkitLang,
    pub language_section: LanguageLang,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LanguageLang {
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommonLang {
    pub cancel: String,
    pub save: String,
    pub close: String,
    pub ok: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DateTimeLang {
    pub today: String,
    pub days: Vec<String>,
    pub months: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PowerLang {
    pub title: String,
    pub ac_desktop: String,
    pub performance: String,
    pub balanced: String,
    pub power_saver: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DashboardLang {
    pub no_media: String,
    pub no_notifications: String,
    pub notifications_title: String,
    pub clear_all: String,
    pub quick_settings_title: String,
    pub volume: String,
    pub brightness: String,
    pub wifi: String,
    pub bluetooth: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuickSettingsLang {
    pub wifi_networks: String,
    pub no_wifi_found: String,
    pub bt_devices: String,
    pub no_bt_found: String,
    pub ethernet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClipboardLang {
    pub title: String,
    pub search_placeholder: String,
    pub empty_history: String,
    pub empty_item: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LauncherLang {
    pub search_placeholder: String,
    pub no_apps: String,
    pub navigate_hint: String,
    pub open_hint: String,
    pub close_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThemesLang {
    pub select_title: String,
    pub create_title: String,
    pub create_button: String,
    pub customize_colors: String,
    pub default_theme: String,
    pub preview: String,
    pub audio_player: String,
    pub surface: String,
    pub theme_mode: String,
    pub dark: String,
    pub light: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WallpaperLang {
    pub title: String,
    pub no_wallpapers: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VolumeLang {
    pub audio_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrayLang {
    pub no_menu: String,
    pub open_app: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolkitLang {
    pub auth_required: String,
    pub user_prefix: String,
    pub verifying: String,
}

impl Default for Language {
    fn default() -> Self {
        Self::spanish()
    }
}

impl Language {
    pub fn spanish() -> Self {
        Self {
            code: "es".to_string(),
            name: "Español".to_string(),
            locale: Some("es_ES.UTF-8".to_string()),
            language_env: Some("es_ES:es".to_string()),
            common: CommonLang {
                cancel: "Cancelar".to_string(),
                save: "Guardar".to_string(),
                close: "Cerrar".to_string(),
                ok: "Aceptar".to_string(),
            },
            datetime: DateTimeLang {
                today: "Hoy".to_string(),
                days: vec![
                    "Lunes".to_string(),
                    "Martes".to_string(),
                    "Miércoles".to_string(),
                    "Jueves".to_string(),
                    "Viernes".to_string(),
                    "Sábado".to_string(),
                    "Domingo".to_string(),
                ],
                months: vec![
                    "Enero".to_string(),
                    "Febrero".to_string(),
                    "Marzo".to_string(),
                    "Abril".to_string(),
                    "Mayo".to_string(),
                    "Junio".to_string(),
                    "Julio".to_string(),
                    "Agosto".to_string(),
                    "Septiembre".to_string(),
                    "Octubre".to_string(),
                    "Noviembre".to_string(),
                    "Diciembre".to_string(),
                ],
            },
            power: PowerLang {
                title: "POWER PLAN".to_string(),
                ac_desktop: "AC Desktop".to_string(),
                performance: "Performance".to_string(),
                balanced: "Balanced".to_string(),
                power_saver: "Power Saver".to_string(),
            },
            dashboard: DashboardLang {
                no_media: "No hay reproductor activo".to_string(),
                no_notifications: "Sin notificaciones".to_string(),
                notifications_title: "Notificaciones".to_string(),
                clear_all: "Borrar todas".to_string(),
                quick_settings_title: "AJUSTES RÁPIDOS".to_string(),
                volume: "Volumen".to_string(),
                brightness: "Brillo".to_string(),
                wifi: "Wi-Fi".to_string(),
                bluetooth: "Bluetooth".to_string(),
            },
            quick_settings: QuickSettingsLang {
                wifi_networks: "Redes Wi-Fi".to_string(),
                no_wifi_found: "No hay redes Wi-Fi encontradas".to_string(),
                bt_devices: "Dispositivos Bluetooth".to_string(),
                no_bt_found: "No hay dispositivos Bluetooth".to_string(),
                ethernet: "Ethernet".to_string(),
            },
            clipboard: ClipboardLang {
                title: "PORTAPAPELES".to_string(),
                search_placeholder: "Buscar en el historial...".to_string(),
                empty_history: "No hay elementos en el historial".to_string(),
                empty_item: "[Elemento vacío]".to_string(),
            },
            launcher: LauncherLang {
                search_placeholder: "Buscar aplicaciones...".to_string(),
                no_apps: "No se encontraron aplicaciones".to_string(),
                navigate_hint: "↑↓ navegar".to_string(),
                open_hint: "↵ abrir".to_string(),
                close_hint: "esc cerrar".to_string(),
            },
            themes: ThemesLang {
                select_title: "Temas".to_string(),
                create_title: "Crear Tema".to_string(),
                create_button: "+ Crear".to_string(),
                customize_colors: "Personaliza los colores".to_string(),
                default_theme: "Por defecto".to_string(),
                preview: "Vista Previa (Capsule)".to_string(),
                audio_player: "Reproductor de audio".to_string(),
                surface: "Superficie".to_string(),
                theme_mode: "Modo del Tema".to_string(),
                dark: "Oscuro".to_string(),
                light: "Claro".to_string(),
            },
            wallpaper: WallpaperLang {
                title: "FONDOS DE PANTALLA".to_string(),
                no_wallpapers: "No hay imágenes en ~/Wallpapers".to_string(),
            },
            volume: VolumeLang {
                audio_output: "Salida de audio".to_string(),
            },
            tray: TrayLang {
                no_menu: "Sin menú disponible".to_string(),
                open_app: "Abrir aplicación".to_string(),
            },
            polkit: PolkitLang {
                auth_required: "Autenticación Requerida".to_string(),
                user_prefix: "Usuario: ".to_string(),
                verifying: "Verificando contraseña...".to_string(),
            },
            language_section: LanguageLang {
                title: "IDIOMAS".to_string(),
            },
        }
    }

    pub fn english() -> Self {
        Self {
            code: "en".to_string(),
            name: "English".to_string(),
            locale: Some("en_US.UTF-8".to_string()),
            language_env: Some("en_US:en".to_string()),
            common: CommonLang {
                cancel: "Cancel".to_string(),
                save: "Save".to_string(),
                close: "Close".to_string(),
                ok: "OK".to_string(),
            },
            datetime: DateTimeLang {
                today: "Today".to_string(),
                days: vec![
                    "Monday".to_string(),
                    "Tuesday".to_string(),
                    "Wednesday".to_string(),
                    "Thursday".to_string(),
                    "Friday".to_string(),
                    "Saturday".to_string(),
                    "Sunday".to_string(),
                ],
                months: vec![
                    "January".to_string(),
                    "February".to_string(),
                    "March".to_string(),
                    "April".to_string(),
                    "May".to_string(),
                    "June".to_string(),
                    "July".to_string(),
                    "August".to_string(),
                    "September".to_string(),
                    "October".to_string(),
                    "November".to_string(),
                    "December".to_string(),
                ],
            },
            power: PowerLang {
                title: "POWER PLAN".to_string(),
                ac_desktop: "AC Desktop".to_string(),
                performance: "Performance".to_string(),
                balanced: "Balanced".to_string(),
                power_saver: "Power Saver".to_string(),
            },
            dashboard: DashboardLang {
                no_media: "No active media player".to_string(),
                no_notifications: "No notifications".to_string(),
                notifications_title: "Notifications".to_string(),
                clear_all: "Clear all".to_string(),
                quick_settings_title: "QUICK SETTINGS".to_string(),
                volume: "Volume".to_string(),
                brightness: "Brightness".to_string(),
                wifi: "Wi-Fi".to_string(),
                bluetooth: "Bluetooth".to_string(),
            },
            quick_settings: QuickSettingsLang {
                wifi_networks: "Wi-Fi Networks".to_string(),
                no_wifi_found: "No Wi-Fi networks found".to_string(),
                bt_devices: "Bluetooth Devices".to_string(),
                no_bt_found: "No Bluetooth devices found".to_string(),
                ethernet: "Ethernet".to_string(),
            },
            clipboard: ClipboardLang {
                title: "CLIPBOARD".to_string(),
                search_placeholder: "Search history...".to_string(),
                empty_history: "No items in history".to_string(),
                empty_item: "[Empty item]".to_string(),
            },
            launcher: LauncherLang {
                search_placeholder: "Search applications...".to_string(),
                no_apps: "No applications found".to_string(),
                navigate_hint: "↑↓ navigate".to_string(),
                open_hint: "↵ open".to_string(),
                close_hint: "esc close".to_string(),
            },
            themes: ThemesLang {
                select_title: "Themes".to_string(),
                create_title: "Create Theme".to_string(),
                create_button: "+ Create".to_string(),
                customize_colors: "Customize colors".to_string(),
                default_theme: "Default".to_string(),
                preview: "Preview (Capsule)".to_string(),
                audio_player: "Audio player".to_string(),
                surface: "Surface".to_string(),
                theme_mode: "Theme Mode".to_string(),
                dark: "Dark".to_string(),
                light: "Light".to_string(),
            },
            wallpaper: WallpaperLang {
                title: "WALLPAPERS".to_string(),
                no_wallpapers: "No images in ~/Wallpapers".to_string(),
            },
            volume: VolumeLang {
                audio_output: "Audio Output".to_string(),
            },
            tray: TrayLang {
                no_menu: "No menu available".to_string(),
                open_app: "Open application".to_string(),
            },
            polkit: PolkitLang {
                auth_required: "Authentication Required".to_string(),
                user_prefix: "User: ".to_string(),
                verifying: "Verifying password...".to_string(),
            },
            language_section: LanguageLang {
                title: "LANGUAGES".to_string(),
            },
        }
    }
}

impl gpui::Global for Language {}
