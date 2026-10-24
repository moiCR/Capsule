if hl.plugin.hyprglass then
    local hg = hl.plugin.hyprglass

    hg.config({
        default_theme = "dark",
        default_preset = "apple",
	layers = {enabled = true},
    })

    -- Custom preset example for glass effect
    hg.preset("clear", {
        glass_opacity = 0.85,
        blur_strength = 1.8,
        dark = { brightness = 0.80 },
    })
    
    hg.preset("apple", {
	blur_strength = 2.2,
	blur_iterations = 3,
	refraction_strength = 0.55,
	chromatic_aberration = 0.3,
	fresnel_strength = 0.5,
	specular_strength = 0.75,
	edge_thickness = 0.05,
	lens_disortion = 0.3,
	dark = {brightness = 0.82, contrast = 0.90, saturation = 0.80, vibrancy = 0.15, adaptive_din = 0.4},
	light = {brightness = 1.12, contrast = 0.92, saturation = 0.85, vibrancy = 0.12, adaptive_din = 0.4},
    })
    
    hg.layer(
	"capsule-panel",
	{exclude = true}
    )
    hg.layer(
	"capsule-bar",
	{exclude = true}
    )
    hg.layer(
	"capsule-wallpaper",
	{exclude = true}
    )
    hg.layer(
        "selection",
        {exclude = true}
    )
end



hl.window_rule({
    match = { class = "kitty" },
    tag = "+hyprglass_preset_apple"
})

hl.window_rule({
    match = { class = "zen" },
    tag = "+hyprglass_preset_apple"
})
