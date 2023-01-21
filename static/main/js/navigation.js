window.addEventListener("DOMContentLoaded", () => {
    const PRIMARY_NAV = document.querySelector(".primary-navigation")
    const MENU_BUTTON = document.querySelector(".mobile-nav-toggle")

    MENU_BUTTON.addEventListener("click", () => {
        const VISIBILITY = PRIMARY_NAV.getAttribute("data-visible") === "false"
        if (VISIBILITY) {
            PRIMARY_NAV.setAttribute("data-visible", true)
            MENU_BUTTON.setAttribute("aria-expanded", true)
            MENU_BUTTON.classList.add("open")
        } else {
            PRIMARY_NAV.setAttribute("data-visible", false)
            MENU_BUTTON.setAttribute("aria-expanded", false)
            MENU_BUTTON.classList.remove("open")
        }
    })
})
