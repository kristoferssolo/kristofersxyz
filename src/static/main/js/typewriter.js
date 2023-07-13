const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))

window.addEventListener("DOMContentLoaded", () => {
    window.addEventListener("load", async () => {
        const TEXT_DISPLAY = document.getElementById("rotating-text")
        const PHRASES = ["Software Developer", "Jedi", "Student"]
        let current_phrase = []
        let is_end = false

        while (1) {
            for (let phrase in PHRASES) {
                if (!is_end) {
                    for (let character in PHRASES[phrase]) {
                        current_phrase.push(PHRASES[phrase][character])
                        TEXT_DISPLAY.innerHTML = current_phrase.join("")
                        await sleep(150)
                    }
                    if (current_phrase.join("") == PHRASES[phrase]) {
                        await sleep(1000)
                        is_end = true
                    }
                } else {
                    while (1) {
                        current_phrase.pop()
                        if (!current_phrase.length) {
                            is_end = false
                            TEXT_DISPLAY.innerHTML = "&nbsp;"
                            await sleep(500)
                            break
                        }
                        TEXT_DISPLAY.innerHTML = current_phrase.join("")
                        await sleep(80)
                    }
                }
            }
        }
    })
})
