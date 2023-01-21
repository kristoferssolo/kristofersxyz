$(document).ready(function () {
    let lightmode = localStorage.getItem("lightmode")
    const HTML = $("html")
    const AUDIO = $("#flashbang")

    const enable_light_mode = () => {
        HTML.attr("data-color-mode", "light")
        localStorage.setItem("lightmode", "enabled")
    }

    const disable_light_mode = () => {
        HTML.attr("data-color-mode", "dark")
        localStorage.setItem("lightmode", null)
    }

    if (lightmode === "enabled") {
        enable_light_mode()
    }

    window
        .matchMedia("(prefers-color-scheme: dark)")
        .addEventListener("change", (event) => {
            if (event.matches) {
                disable_light_mode()
            } else {
                enable_light_mode()
            }
        })

    if (window.matchMedia) {
        if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
            disable_light_mode()
        } else {
            enable_light_mode()
        }
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
