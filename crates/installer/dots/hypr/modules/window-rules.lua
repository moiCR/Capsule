local suppressMaximizeRule = hl.window_rule({
    name           = "suppress-maximize-events",
    match          = { class = ".*" },
    suppress_event = "maximize",
})

hl.window_rule({
    name     = "fix-xwayland-drags",
    match    = {
        class      = "^$",
        title      = "^$",
        xwayland   = true,
        float      = true,
        fullscreen = false,
        pin        = false,
    },
    no_focus = true,
})

hl.window_rule({
    name  = "move-hyprland-run",
    match = { class = "hyprland-run" },
    move  = "20 monitor_h-120",
    float = true,
})


hl.window_rule({
    name = "spotify-workspace",
    match = {
        class = "^(Spotify|fastpotify)$",
    },
    workspace = "special:music"
})

hl.window_rule({
    name = "discord-workspace",
    match = {
        class = "discord",
    },
    float = true,
    workspace = "special:discord",
    center = true,
    size = "1000 700",
})

hl.window_rule({
    name = "zapzap-workspace",
    match = {
        class = "com.rtosta.zapzap",
    },
    float = true,
    workspace = "special:zapzap",
    center = true,
    size = "1000 700",
})


hl.window_rule({
    name = "steam-workspace",
    match = {
        class = "steam",
        title = "^Steam$",
    },
    workspace = "special:steam"
})

hl.window_rule({
    name = "steam-floating-popups",
    match = {
        class = "steam"
    },
    float = true,
    center = true,
    size = "1000 700",
})

hl.window_rule({
    name = "1password1-floating",
    match = {
        class = "1[pP]assword"
    },
    float = true,
    center = true,
    size = "1000 700",
})

hl.window_rule({
    name = "steam-main-tile",
    match = {
        class = "steam",
        title = "^Steam$",
    },
    float = false,
})

hl.window_rule({
    name = "portals",
    match = {
        class = "^(xdg-desktop-portal-gtk|xdg-desktop-portal-kde)$"
    },
    float = true,
    center = true,
    size = "1000 700",
})

hl.layer_rule({
    name = "capsule-panel-blur",
    match = {
        namespace = "^capsule-panel$",
    },
    blur = true,
    ignore_alpha = 0.05,
})
