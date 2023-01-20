let lightmode = localStorage.getItem("lightmode")
const HTML = document.documentElement
const AUDIO = document.getElementById("flashbang")

const ENABLE_LIGHT_MODE = () => {
    HTML.setAttribute("data-color-mode", "light")
    localStorage.setItem("lightmode", "enabled")
}

const DISABLE_LIGHT_MODE = () => {
    HTML.setAttribute("data-color-mode", "dark")
    localStorage.setItem("lightmode", null)
}

if (lightmode === "enabled") {
    ENABLE_LIGHT_MODE()
}

document.querySelector("#color-mode-toggle").addEventListener("click", () => {
    lightmode = localStorage.getItem("lightmode")
    if (lightmode !== "enabled") {
        ENABLE_LIGHT_MODE()
        AUDIO.play()
    } else {
        DISABLE_LIGHT_MODE()
    }
})