const DISPLAY_NAME = document.getElementById("swap-text")

DISPLAY_NAME.addEventListener("click", () => {
    const NAMES = ["Kristofers Solo", "Kristiāns Francis Cagulis"]
    if (DISPLAY_NAME.innerHTML == NAMES[0]) {
        DISPLAY_NAME.innerHTML = NAMES[1]
    } else {
        DISPLAY_NAME.innerHTML = NAMES[0]
    }
})
