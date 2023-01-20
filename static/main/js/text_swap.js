const DISPLAY_NAME = document.getElementById("swap-text")
const NAMES = [
    "Kristiāns",
    "Francis",
    "Cagulis",
    "Kristiāns Francis Cagulis",
    "KFC",
    "Kristofers",
    "Solo",
    "Kristofers Solo",
    "Salaspils 1",
    "Šis puisis",
]

DISPLAY_NAME.addEventListener("click", () => {
    DISPLAY_NAME.innerHTML = NAMES[Math.floor(Math.random() * NAMES.length)]
})
