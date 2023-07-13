window.addEventListener("DOMContentLoaded", () => {
    const HTML = document.documentElement
    const AUDIO = document.getElementById("flashbang")
    let lightmode = localStorage.getItem("lightmode")

    const enable_light_mode = () => {
        HTML.setAttribute("data-color-mode", "light")
        localStorage.setItem("lightmode", "enabled")
    }

    const disable_light_mode = () => {
        HTML.setAttribute("data-color-mode", "dark")
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

    document
        .querySelector("#color-mode-toggle")
        .addEventListener("click", () => {
            lightmode = localStorage.getItem("lightmode")
            if (lightmode !== "enabled") {
                enable_light_mode()
                AUDIO.play()
            } else {
                disable_light_mode()
            }
        })
})
