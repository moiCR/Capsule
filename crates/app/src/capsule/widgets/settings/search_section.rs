use gpui::{
    AnyElement, Context, ElementId, FontWeight, IntoElement, ParentElement, Styled, div, px, svg,
};
use services::{AppState, CapsuleStyle, PowerProfile};
use std::sync::Arc;
use ui::theme::Theme;

use super::general_section::{
    render_app_select, render_audio_device_control, render_language_select,
    render_music_players_card, render_power_selector,
};
use super::lockscreen_section::render_text_input;
use super::record_section::{
    render_audio_selector, render_container_selector, render_fps_selector, render_output_selector,
    render_quality_selector, render_resolution_selector,
};
use super::setting_item::{
    render_card_container, render_control_row, render_hero_header, render_int_slider_row,
    render_row_divider, render_slider_row, render_toggle_row,
};
use crate::capsule::modules::settings::{SettingsField, SettingsModule};

pub fn render_search_results(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let query = module.search_query.trim().to_lowercase();

    let (
        search_results_title,
        search_results_sub_tmpl,
        search_no_results_title,
        search_no_results_sub_tmpl,
        concave_title,
        concave_sub,
        idle_title,
        margin_title,
        gap_title,
        round_title,
        sat_title,
        card_title,
        anim_title,
        term_title,
        term_sub,
        browser_title,
        browser_sub,
        editor_title,
        editor_sub,
        file_manager_title,
        file_manager_sub,
        idle_lyrics_title,
        idle_lyrics_sub,
        output_vol_title,
        output_vol_sub,
        input_vol_title,
        input_vol_sub,
        media_title,
        media_sub,
        music_title,
        music_sub,
        music_placeholder,
        music_add,
        music_empty,
        idle_timeout_title,
        show_clock_title,
        show_clock_sub,
        time_title,
        time_sub,
        date_title,
        date_sub,
        power_title,
        power_sub,
        lang_title,
        lang_sub,
    ) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("settings.search_results_title"),
            lang.get("settings.search_results_subtitle"),
            lang.get("settings.search_no_results_title"),
            lang.get("settings.search_no_results_subtitle"),
            lang.get("settings.capsule_style_title"),
            lang.get("settings.capsule_style_subtitle"),
            lang.get("settings.idle_height_title"),
            lang.get("settings.margin_top_title"),
            lang.get("settings.gap_title"),
            lang.get("settings.capsule_round_title"),
            lang.get("settings.satellite_round_title"),
            lang.get("settings.cards_round_title"),
            lang.get("settings.anim_duration_title"),
            lang.get("settings.terminal_title"),
            lang.get("settings.terminal_subtitle"),
            lang.get("settings.browser_title"),
            lang.get("settings.browser_subtitle"),
            lang.get("settings.editor_title"),
            lang.get("settings.editor_subtitle"),
            lang.get("settings.file_manager_title"),
            lang.get("settings.file_manager_subtitle"),
            lang.get("settings.idle_lyrics_title"),
            lang.get("settings.idle_lyrics_subtitle"),
            lang.get("settings.output_volume_title"),
            lang.get("settings.output_volume_subtitle"),
            lang.get("settings.input_volume_title"),
            lang.get("settings.input_volume_subtitle"),
            lang.get("settings.show_media_title"),
            lang.get("settings.show_media_subtitle"),
            lang.get("settings.music_players_title"),
            lang.get("settings.music_players_subtitle"),
            lang.get("settings.music_players_placeholder"),
            lang.get("settings.music_players_add"),
            lang.get("settings.music_players_empty"),
            lang.get("settings.idle_timeout_title"),
            lang.get("settings.show_clock_title"),
            lang.get("settings.show_clock_subtitle"),
            lang.get("settings.time_format_title"),
            lang.get("settings.time_format_subtitle"),
            lang.get("settings.date_format_title"),
            lang.get("settings.date_format_subtitle"),
            lang.get("settings.power_plan_title"),
            lang.get("settings.power_plan_subtitle"),
            lang.get("settings.language_title"),
            lang.get("settings.language_subtitle"),
        )
    } else {
        (
            "Resultados de Búsqueda".to_string(),
            "{count} opciones encontradas para \"{query}\"".to_string(),
            "Sin resultados".to_string(),
            "No se encontraron ajustes para \"{query}\"".to_string(),
            "Modo Cóncavo".to_string(),
            "Cápsula cóncava integrada a la pantalla o barra flotante.".to_string(),
            "Altura Base (Idle)".to_string(),
            "Margen Superior".to_string(),
            "Separación entre Elementos (Gap)".to_string(),
            "Radio de la Cápsula".to_string(),
            "Radio de Satélites".to_string(),
            "Radio de Tarjetas".to_string(),
            "Duración de Animación".to_string(),
            "Terminal".to_string(),
            "Lanzado con 'capsule terminal' o apps en terminal.".to_string(),
            "Navegador Web".to_string(),
            "Lanzado con 'capsule browser' o links del sistema.".to_string(),
            "Editor de Código / Texto".to_string(),
            "Lanzado con 'capsule editor'. Si es CLI, se ejecuta en terminal.".to_string(),
            "Gestor de Archivos".to_string(),
            "Lanzado con 'capsule files' o explorador del sistema.".to_string(),
            "Letras en Reposo".to_string(),
            "Muestra la letra sincronizada de la canción en la cápsula idle.".to_string(),
            "Salida de Audio".to_string(),
            "Control de volumen y selección del dispositivo de reproducción.".to_string(),
            "Entrada de Audio (Micrófono)".to_string(),
            "Control de volumen y selección del dispositivo de captura.".to_string(),
            "Reproductor en Bloqueo".to_string(),
            "Muestra controles MPRIS y letras de canciones en el lockscreen.".to_string(),
            "Reproductores Autorizados".to_string(),
            "Servicios MPRIS permitidos para control multimedia (ej. Spotify, Fastpotify)."
                .to_string(),
            "Nombre de la app (ej. Fastpotify)...".to_string(),
            "Añadir".to_string(),
            "No hay reproductores configurados.".to_string(),
            "Tiempo de Inactividad".to_string(),
            "Mostrar Reloj".to_string(),
            "Muestra la hora y fecha en el centro de la pantalla.".to_string(),
            "Formato de Hora".to_string(),
            "Sintaxis chrono (ej: %H:%M para 24h, %I:%M %p para 12h).".to_string(),
            "Formato de Fecha".to_string(),
            "Sintaxis chrono para la fecha bajo el reloj principal.".to_string(),
            "Plan de Energía".to_string(),
            "Ajusta el consumo y velocidad del procesador vía power-profiles-daemon.".to_string(),
            "Idioma de la Interfaz".to_string(),
            "Cambia el idioma activo de las etiquetas y controles del sistema.".to_string(),
        )
    };

    let (
        rec_fps_title,
        rec_fps_sub,
        rec_res_title,
        rec_res_sub,
        rec_quality_title,
        rec_quality_sub,
        rec_container_title,
        rec_container_sub,
        rec_audio_title,
        rec_audio_sub,
        rec_cursor_title,
        rec_cursor_sub,
        rec_output_title,
        rec_output_sub,
    ) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("settings.record_fps_title"),
            lang.get("settings.record_fps_subtitle"),
            lang.get("settings.record_resolution_title"),
            lang.get("settings.record_resolution_subtitle"),
            lang.get("settings.record_quality_title"),
            lang.get("settings.record_quality_subtitle"),
            lang.get("settings.record_container_title"),
            lang.get("settings.record_container_subtitle"),
            lang.get("settings.record_audio_title"),
            lang.get("settings.record_audio_subtitle"),
            lang.get("settings.record_cursor_title"),
            lang.get("settings.record_cursor_subtitle"),
            lang.get("settings.record_output_title"),
            lang.get("settings.record_output_subtitle"),
        )
    } else {
        (
            "Cuadros por Segundo (FPS)".to_string(),
            "Fluidez de captura de la grabación de pantalla.".to_string(),
            "Resolución".to_string(),
            "Escala de captura del monitor seleccionado.".to_string(),
            "Calidad de Video".to_string(),
            "Tasa de bits y compresión del codificador de video.".to_string(),
            "Formato de Contenedor".to_string(),
            "Formato de archivo final (MP4 o MKV).".to_string(),
            "Fuente de Audio".to_string(),
            "Audio capturado por defecto durante la grabación.".to_string(),
            "Capturar Cursor".to_string(),
            "Incluir el puntero del ratón en la grabación.".to_string(),
            "Salida de Video".to_string(),
            "Monitor o pantalla capturada en la grabación.".to_string(),
        )
    };

    let empty_apps = Vec::new();
    let installed_apps = if cx.has_global::<AppState>() {
        cx.global::<AppState>().launcher.get_apps()
    } else {
        Arc::new(empty_apps)
    };

    let (out_vol, out_muted, sinks, in_vol, in_muted, sources) = if cx.has_global::<AppState>() {
        let status = cx.global::<AppState>().system.get_status();
        (
            status.volume,
            status.is_muted,
            status
                .audio_sinks
                .into_iter()
                .map(|s| (s.name, s.description, s.is_default))
                .collect::<Vec<_>>(),
            status.input_volume,
            status.is_input_muted,
            status
                .audio_sources
                .into_iter()
                .map(|s| (s.name, s.description, s.is_default))
                .collect::<Vec<_>>(),
        )
    } else {
        (50, false, Vec::new(), 50, false, Vec::new())
    };

    let current_profile = if cx.has_global::<AppState>() {
        cx.global::<AppState>().power.get_active_profile()
    } else {
        PowerProfile::Balanced
    };

    let languages = if cx.has_global::<AppState>() {
        cx.global::<AppState>().language.list_languages()
    } else {
        Vec::new()
    };

    let matches_query = |terms: &[&str]| -> bool {
        terms.iter().any(|t| {
            let t_low = t.to_lowercase();
            t_low.contains(&query) || query.contains(&t_low)
        })
    };

    let mut matched_rows: Vec<AnyElement> = Vec::new();

    if matches_query(&[
        &concave_title,
        &concave_sub,
        "concavo",
        "modo concavo",
        "concave",
        "concave mode",
        "estilo",
        "capsule style",
        "capsula concava",
    ]) {
        let is_concave = module.capsule_style == CapsuleStyle::Concave;
        matched_rows.push(
            render_toggle_row(
                ElementId::Name("search-capsule-concave".into()),
                &concave_title,
                Some(&concave_sub),
                is_concave,
                theme,
                cx.listener(|this, _, _, cx| {
                    let next = if this.capsule_style == CapsuleStyle::Concave {
                        CapsuleStyle::Normal
                    } else {
                        CapsuleStyle::Concave
                    };
                    this.set_capsule_style(next, cx);
                }),
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &idle_title,
        "altura base",
        "base height",
        "idle height",
        "altura",
        "idle",
        "reposo",
        "barra",
    ]) {
        matched_rows.push(
            render_slider_row(
                SettingsField::IdleHeight,
                &idle_title,
                &module.idle_height_input,
                "px",
                16.0,
                60.0,
                1.0,
                theme,
                cx,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &margin_title,
        "margen superior",
        "top margin",
        "margen",
        "margin",
        "arriba",
        "offset",
    ]) {
        matched_rows.push(
            render_slider_row(
                SettingsField::MarginTop,
                &margin_title,
                &module.margin_top_input,
                "px",
                0.0,
                40.0,
                1.0,
                theme,
                cx,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &gap_title,
        "separacion entre elementos",
        "element gap",
        "separacion",
        "gap",
        "espaciado",
        "espacio",
    ]) {
        matched_rows.push(
            render_slider_row(
                SettingsField::Gap,
                &gap_title,
                &module.gap_input,
                "px",
                0.0,
                32.0,
                1.0,
                theme,
                cx,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &round_title,
        "radio de la capsula",
        "capsule round",
        "capsule radius",
        "radio capsula",
        "curvatura",
        "redondeo",
        "round",
    ]) {
        matched_rows.push(
            render_slider_row(
                SettingsField::CapsuleRound,
                &round_title,
                &module.capsule_round_input,
                "px",
                0.0,
                24.0,
                1.0,
                theme,
                cx,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &sat_title,
        "radio de satelites",
        "satellite round",
        "satellite radius",
        "satelites",
        "satellites",
    ]) {
        matched_rows.push(
            render_slider_row(
                SettingsField::SatelliteRound,
                &sat_title,
                &module.satellite_round_input,
                "px",
                0.0,
                24.0,
                1.0,
                theme,
                cx,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &card_title,
        "radio de tarjetas",
        "cards round",
        "card radius",
        "tarjetas",
        "cards",
    ]) {
        matched_rows.push(
            render_slider_row(
                SettingsField::CardsRound,
                &card_title,
                &module.cards_round_input,
                "px",
                0.0,
                24.0,
                1.0,
                theme,
                cx,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &anim_title,
        "duracion de animacion",
        "animation duration",
        "duracion",
        "animacion",
        "velocidad",
        "speed",
        "transicion",
    ]) {
        matched_rows.push(
            render_slider_row(
                SettingsField::AnimDuration,
                &anim_title,
                &module.anim_duration_input,
                "ms",
                0.0,
                1000.0,
                25.0,
                theme,
                cx,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &term_title,
        &term_sub,
        "terminal",
        "consola",
        "shell",
        "bash",
        "zsh",
        "alacritty",
        "kitty",
        "foot",
    ]) {
        matched_rows.push(
            render_control_row(
                &term_title,
                Some(&term_sub),
                render_app_select(
                    module,
                    SettingsField::Terminal,
                    &module.terminal_input,
                    module.open_dropdown == Some(SettingsField::Terminal),
                    &installed_apps,
                    "sparkles.svg",
                    theme,
                    cx,
                ),
                theme,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &browser_title,
        &browser_sub,
        "navegador web",
        "web browser",
        "navegador",
        "browser",
        "internet",
        "firefox",
        "chrome",
        "chromium",
        "brave",
    ]) {
        matched_rows.push(
            render_control_row(
                &browser_title,
                Some(&browser_sub),
                render_app_select(
                    module,
                    SettingsField::Browser,
                    &module.browser_input,
                    module.open_dropdown == Some(SettingsField::Browser),
                    &installed_apps,
                    "sparkles.svg",
                    theme,
                    cx,
                ),
                theme,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &editor_title,
        &editor_sub,
        "editor de codigo",
        "text editor",
        "code editor",
        "editor",
        "codigo",
        "code",
        "nvim",
        "neovim",
        "vim",
        "vscodium",
    ]) {
        matched_rows.push(
            render_control_row(
                &editor_title,
                Some(&editor_sub),
                render_app_select(
                    module,
                    SettingsField::Editor,
                    &module.editor_input,
                    module.open_dropdown == Some(SettingsField::Editor),
                    &installed_apps,
                    "sparkles.svg",
                    theme,
                    cx,
                ),
                theme,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &file_manager_title,
        &file_manager_sub,
        "gestor de archivos",
        "file manager",
        "archivos",
        "files",
        "thunar",
        "dolphin",
        "nautilus",
        "nemo",
        "yazi",
        "ranger",
    ]) {
        matched_rows.push(
            render_control_row(
                &file_manager_title,
                Some(&file_manager_sub),
                render_app_select(
                    module,
                    SettingsField::FileManager,
                    &module.file_manager_input,
                    module.open_dropdown == Some(SettingsField::FileManager),
                    &installed_apps,
                    "folder.svg",
                    theme,
                    cx,
                ),
                theme,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &idle_lyrics_title,
        &idle_lyrics_sub,
        "letras en reposo",
        "idle lyrics",
        "letras",
        "lyrics",
        "cancion",
        "musica letras",
    ]) {
        matched_rows.push(
            render_toggle_row(
                ElementId::Name("search-idle-lyrics".into()),
                &idle_lyrics_title,
                Some(&idle_lyrics_sub),
                module.show_lyrics,
                theme,
                cx.listener(|this, _, _, cx| {
                    this.toggle_show_lyrics(cx);
                }),
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &output_vol_title,
        &output_vol_sub,
        "salida de audio",
        "volumen de salida",
        "audio output",
        "output volume",
        "volumen",
        "sonido",
        "audio",
        "altavoces",
        "auriculares",
        "sound",
        "speaker",
        "headphones",
    ]) {
        matched_rows.push(
            render_audio_device_control(
                SettingsField::VolumeOutput,
                &output_vol_title,
                &output_vol_sub,
                out_vol,
                out_muted,
                "volume-2.svg",
                "volume-x.svg",
                module.show_output_devices,
                sinks.clone(),
                module.output_slider_bounds.clone(),
                theme,
                cx,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &input_vol_title,
        &input_vol_sub,
        "entrada de audio",
        "volumen de entrada",
        "audio input",
        "input volume",
        "microfono",
        "mic",
        "microphone",
        "captura de audio",
    ]) {
        matched_rows.push(
            render_audio_device_control(
                SettingsField::VolumeInput,
                &input_vol_title,
                &input_vol_sub,
                in_vol,
                in_muted,
                "mic.svg",
                "mic-off.svg",
                module.show_input_devices,
                sources.clone(),
                module.input_slider_bounds.clone(),
                theme,
                cx,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &media_title,
        &media_sub,
        "reproductor en bloqueo",
        "media player",
        "reproductor",
        "player",
        "mpris",
        "lockscreen player",
    ]) {
        matched_rows.push(
            render_toggle_row(
                ElementId::Name("search-media-show-lockscreen".into()),
                &media_title,
                Some(&media_sub),
                module.show_media_player,
                theme,
                cx.listener(|this, _, _, cx| {
                    this.toggle_show_media_player(cx);
                }),
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &music_title,
        &music_sub,
        "reproductores autorizados",
        "authorized music players",
        "spotify",
        "fastpotify",
        "reproductores",
        "music players",
        "musica",
    ]) {
        matched_rows.push(
            render_control_row(
                &music_title,
                Some(&music_sub),
                render_music_players_card(
                    module,
                    &music_placeholder,
                    &music_add,
                    &music_empty,
                    theme,
                    cx,
                ),
                theme,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &idle_timeout_title,
        "tiempo de inactividad",
        "idle timeout",
        "inactividad",
        "idle",
        "suspension",
        "bloqueo",
        "lock timeout",
    ]) {
        let idle_secs = module.idle_timeout_input.parse::<u64>().unwrap_or(900);
        matched_rows.push(
            render_int_slider_row(
                SettingsField::IdleTimeout,
                &idle_timeout_title,
                idle_secs,
                theme,
                cx,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &show_clock_title,
        &show_clock_sub,
        "mostrar reloj",
        "show clock",
        "reloj",
        "clock",
        "pantalla reloj",
    ]) {
        matched_rows.push(
            render_toggle_row(
                ElementId::Name("search-lock-show-clock".into()),
                &show_clock_title,
                Some(&show_clock_sub),
                module.show_clock,
                theme,
                cx.listener(|this, _, _, cx| {
                    this.toggle_show_clock(cx);
                }),
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &time_title,
        &time_sub,
        "formato de hora",
        "time format",
        "hora",
        "time",
        "chrono",
        "24h",
        "12h",
    ]) {
        matched_rows.push(
            render_control_row(
                &time_title,
                Some(&time_sub),
                render_text_input(
                    SettingsField::TimeFormat,
                    &module.time_format_input,
                    module.active_field == Some(SettingsField::TimeFormat),
                    &["%H:%M", "%I:%M %p"],
                    theme,
                    cx,
                ),
                theme,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &date_title,
        &date_sub,
        "formato de fecha",
        "date format",
        "fecha",
        "date",
        "dia",
        "mes",
    ]) {
        matched_rows.push(
            render_control_row(
                &date_title,
                Some(&date_sub),
                render_text_input(
                    SettingsField::DateFormat,
                    &module.date_format_input,
                    module.active_field == Some(SettingsField::DateFormat),
                    &["%A, %B %e, %Y", "%d/%m/%Y", "%Y-%m-%d"],
                    theme,
                    cx,
                ),
                theme,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &power_title,
        &power_sub,
        "plan de energia",
        "power profile",
        "power",
        "energia",
        "rendimiento",
        "performance",
        "bateria",
        "battery",
    ]) {
        matched_rows.push(
            render_control_row(
                &power_title,
                Some(&power_sub),
                render_power_selector(current_profile, theme, cx),
                theme,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &lang_title,
        &lang_sub,
        "idioma de la interfaz",
        "interface language",
        "idioma",
        "language",
        "lenguaje",
        "espanol",
        "spanish",
        "english",
    ]) {
        matched_rows.push(
            render_control_row(
                &lang_title,
                Some(&lang_sub),
                render_language_select(module, &languages, theme, cx),
                theme,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &rec_output_title,
        &rec_output_sub,
        "salida",
        "output",
        "monitor",
        "pantalla",
        "display",
        "screen",
        "grabacion",
        "record",
    ]) {
        matched_rows.push(
            render_control_row(
                &rec_output_title,
                Some(&rec_output_sub),
                render_output_selector(
                    &module.record_output,
                    &module.record_available_monitors,
                    theme,
                    cx,
                ),
                theme,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &rec_fps_title,
        &rec_fps_sub,
        "fps",
        "cuadros",
        "frame rate",
        "fluidez",
        "grabacion",
        "record",
    ]) {
        matched_rows.push(
            render_control_row(
                &rec_fps_title,
                Some(&rec_fps_sub),
                render_fps_selector(module.record_fps, theme, cx),
                theme,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &rec_res_title,
        &rec_res_sub,
        "resolucion",
        "resolution",
        "escala",
        "1080p",
        "720p",
        "native",
        "grabacion",
        "record",
    ]) {
        matched_rows.push(
            render_control_row(
                &rec_res_title,
                Some(&rec_res_sub),
                render_resolution_selector(&module.record_resolution, theme, cx),
                theme,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &rec_quality_title,
        &rec_quality_sub,
        "calidad",
        "quality",
        "bitrate",
        "video",
        "grabacion",
        "record",
    ]) {
        matched_rows.push(
            render_control_row(
                &rec_quality_title,
                Some(&rec_quality_sub),
                render_quality_selector(&module.record_quality, theme, cx),
                theme,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &rec_container_title,
        &rec_container_sub,
        "contenedor",
        "container",
        "formato",
        "format",
        "mp4",
        "mkv",
        "grabacion",
        "record",
    ]) {
        matched_rows.push(
            render_control_row(
                &rec_container_title,
                Some(&rec_container_sub),
                render_container_selector(&module.record_container, theme, cx),
                theme,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &rec_audio_title,
        &rec_audio_sub,
        "audio",
        "microfono",
        "mic",
        "sonido",
        "sound",
        "grabacion",
        "record",
    ]) {
        matched_rows.push(
            render_control_row(
                &rec_audio_title,
                Some(&rec_audio_sub),
                render_audio_selector(&module.record_audio, theme, cx),
                theme,
            )
            .into_any_element(),
        );
    }

    if matches_query(&[
        &rec_cursor_title,
        &rec_cursor_sub,
        "cursor",
        "puntero",
        "raton",
        "mouse",
        "pointer",
        "grabacion",
        "record",
    ]) {
        matched_rows.push(
            render_toggle_row(
                ElementId::Name("search-record-cursor".into()),
                &rec_cursor_title,
                Some(&rec_cursor_sub),
                module.record_include_cursor,
                theme,
                cx.listener(|this, _, _, cx| {
                    this.toggle_record_include_cursor(cx);
                }),
            )
            .into_any_element(),
        );
    }

    if matched_rows.is_empty() {
        let no_sub = search_no_results_sub_tmpl.replace("{query}", &module.search_query);

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .w_full()
            .min_w_0()
            .py_12()
            .px_4()
            .gap(px(12.0))
            .child(
                div()
                    .w(px(52.0))
                    .h(px(52.0))
                    .rounded_full()
                    .bg(theme.surface().opacity(0.35))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        svg()
                            .path("search.svg")
                            .size(px(24.0))
                            .text_color(theme.foreground_muted().opacity(0.5)),
                    ),
            )
            .child(
                div()
                    .font_weight(FontWeight::BOLD)
                    .text_size(px(16.0))
                    .text_color(theme.foreground())
                    .child(search_no_results_title),
            )
            .child(
                div()
                    .text_size(px(12.5))
                    .text_color(theme.foreground_muted())
                    .text_center()
                    .max_w(px(380.0))
                    .child(no_sub),
            )
            .into_any_element()
    } else {
        let count_str = matched_rows.len().to_string();
        let sub_text = search_results_sub_tmpl
            .replace("{count}", &count_str)
            .replace("{query}", &module.search_query);

        let mut card = render_card_container(theme);
        for (idx, row) in matched_rows.into_iter().enumerate() {
            if idx > 0 {
                card = card.child(render_row_divider(theme));
            }
            card = card.child(row);
        }

        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .w_full()
            .min_w_0()
            .child(render_hero_header(
                "search.svg",
                &search_results_title,
                &sub_text,
                theme,
            ))
            .child(card)
            .into_any_element()
    }
}
