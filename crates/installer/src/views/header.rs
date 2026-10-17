use crate::app::{InstallerApp, Step};
use crate::style;
use gpui::{AnyElement, Context, IntoElement, ParentElement, Styled, div, prelude::*, px};
use ui::theme::Theme;

pub fn render_header(app: &InstallerApp, cx: &mut Context<InstallerApp>) -> AnyElement {
    let theme = cx.global::<Theme>().clone();
    let current_step = app.step;
    let is_installing = current_step == Step::Installing;

    let step_pills = [
        (Step::Welcome, "Overview"),
        (Step::Dependencies, "Dependencies"),
        (Step::Compositor, "Compositor"),
        (Step::Installing, "Install"),
        (Step::Finished, "Done"),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (step, label))| {
        let is_active = step == current_step;
        let is_completed = matches!(
            (current_step, step),
            (Step::Dependencies, Step::Welcome)
                | (Step::Compositor, Step::Welcome | Step::Dependencies)
                | (
                    Step::Installing,
                    Step::Welcome | Step::Dependencies | Step::Compositor
                )
                | (Step::Finished, _)
        );

        div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .child(
                div()
                    .size(px(20.0))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(10.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .when(is_completed, |s| {
                        s.bg(theme.accent()).text_color(style::on_accent(&theme))
                    })
                    .when(is_active && !is_completed, |s| {
                        s.bg(style::tint(style::surface(&theme), theme.accent(), 0.3))
                            .text_color(theme.accent())
                            .border_1()
                            .border_color(theme.accent())
                    })
                    .when(!is_active && !is_completed, |s| {
                        s.bg(style::raised(&theme))
                            .text_color(theme.foreground_muted())
                    })
                    .child(if is_completed {
                        "✓"
                    } else {
                        match i {
                            0 => "1",
                            1 => "2",
                            2 => "3",
                            3 => "4",
                            _ => "5",
                        }
                    }),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .font_weight(if is_active {
                        gpui::FontWeight::MEDIUM
                    } else {
                        gpui::FontWeight::NORMAL
                    })
                    .text_color(if is_active {
                        theme.foreground()
                    } else {
                        theme.foreground_muted()
                    })
                    .child(label),
            )
    });

    div()
        .flex()
        .items_center()
        .justify_between()
        .pb(px(14.0))
        .border_b_1()
        .border_color(style::border(&theme))
        // Brand / Title
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    gpui::svg()
                        .path("package.svg")
                        .size(px(18.0))
                        .text_color(theme.accent()),
                )
                .child(
                    div()
                        .text_size(px(14.0))
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme.foreground())
                        .child("Capsule"),
                )
                .child(
                    div()
                        .text_size(px(14.0))
                        .text_color(theme.foreground_muted())
                        .child("/"),
                )
                .child(
                    div()
                        .text_size(px(14.0))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme.foreground_muted())
                        .child("Installer"),
                ),
        )
        // Step progress pills
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(16.0))
                .children(step_pills),
        )
        // Close button
        .child(div().when(!is_installing, |s| {
            s.child(
                style::circle_button("installer-close-btn", "close.svg", 28.0, &theme).on_click(
                    cx.listener(|this, _, _, cx| {
                        this.cancel_and_exit(cx);
                    }),
                ),
            )
        }))
        .into_any_element()
}
