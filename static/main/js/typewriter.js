const TEXT_DISPLAY = document.getElementById("text")
const PHRASES = ["Software Developer", "Jedi"]
let i = 0
let j = 0
let current_phrase = []
let is_deleting = false
let is_end = false

typewriter = () => {
    is_end = false
    TEXT_DISPLAY.innerHTML = current_phrase.join("")

    if (i < PHRASES.length) {
        if (!is_deleting && j <= PHRASES[i].length) {
            current_phrase.push(PHRASES[i][j])
            j++
            TEXT_DISPLAY.innerHTML = current_phrase.join("")
        }
        if (is_deleting && j <= PHRASES[i].length) {
            current_phrase.pop(PHRASES[i][j])
            j--
            TEXT_DISPLAY.innerHTML = current_phrase.join("")
        }

        if (j == PHRASES[i].length) {
            is_end = true
            is_deleting = true
        }

        if (is_deleting && j === 0) {
            current_phrase = []
            is_deleting = false
            i++
            if (i === PHRASES.length) {
                i = 0
            }
        }
    }
    const TYPING_SPEED = Math.random() * 300
    const DELETE_SPEED = Math.random() * 100
    const TIME = is_end ? 2000 : is_deleting ? DELETE_SPEED : TYPING_SPEED
    setTimeout(typewriter, TIME)
}

window.addEventListener("load", typewriter)
