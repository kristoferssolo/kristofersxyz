window.addEventListener("DOMContentLoaded", () => {
    document.getElementById("cards").onmousemove = (e) => {
        for (const CARD of document.getElementsByClassName("card")) {
            const RECT = CARD.getBoundingClientRect(),
                x = e.clientX - RECT.left,
                y = e.clientY - RECT.top
            CARD.style.setProperty("--mouse-x", `${x}px`)
            CARD.style.setProperty("--mouse-y", `${y}px`)
        }
    }
})
