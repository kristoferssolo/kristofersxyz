const COLORS = [
    "#2ff924",
    "#2e67f8",
    "#871efe",
    "#eb212e",
    "#ff8f18",
    "#ffffff",
]
let random_color = COLORS[Math.floor(Math.random() * COLORS.length)]
document.documentElement.style.setProperty("--lightsaber-color", random_color)

let body = document.body
body.addEventListener("click", () => {
    body.hidden = true
    requestAnimationFrame(() => {
        body.hidden = false
    })
})

let style = document.createElement("style")
style.innerHTML =
    " *, *:before, *:after { animation-play-state: paused !important; }"

document.addEventListener("keypress", () => {
    style.parentNode
        ? document.head.removeChild(style)
        : document.head.appendChild(style)
})
