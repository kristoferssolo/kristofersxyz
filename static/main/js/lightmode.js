$(document).ready(function() {
    let lightmode = localStorage.getItem("lightmode")
    const HTML = $("html")
    const AUDIO = $("#flashbang")

    const enable_light_mode = () => {
        HTML.attr("data-color-mode", "light")
        $("meta[name='color-scheme']").attr("content", "light")
        localStorage.setItem("lightmode", "enabled")
    }

    const disable_light_mode = () => {
        HTML.attr("data-color-mode", "dark")
        $("meta[name='color-scheme']").attr("content", "dark")
        localStorage.setItem("lightmode", null)
    }

    if (lightmode === "enabled") {
        enable_light_mode()
    }

    $("#color-mode-toggle").click(() => {
        lightmode = localStorage.getItem("lightmode")
        if (lightmode !== "enabled") {
            enable_light_mode()
            AUDIO[0].play()
        } else {
            disable_light_mode()
        }
    })
})
