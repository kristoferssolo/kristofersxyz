$(document).ready(function () {
    let register_form = $("#register-form")
    let username = $("#username")
    let password = $("#password")
    let confirm_password = $("#confirm-password")
    let check_block = $("#check-block")
    let valid_username = false
    let valid_password = false
    let valid_password_match = false

    const CHECKS = {
        length: "^.{10,}",
        uppercase: "[A-Z]",
        lowercase: "[a-z]",
        numbers: "[0-9]",
        symbol: "[^A-Za-z0-9]",
    }

    const TEXT = {
        length: "10 Characters",
        uppercase: "1 Uppercase Letter",
        lowercase: "1 Lowercase Letter",
        numbers: "1 Digit",
        symbol: "1 Symbol",
    }

    username.change(function () {
        if (username.val() === "") {
            valid_username = false
        } else {
            valid_username = true
        }
        submit()
    })

    password.change(function () {
        valid_password = true
        check_block.empty()
        check_block.append($("<p>").text("Password must consist of:"))
        check_block.append($("<ul>", {id: "check-list"}))
        for (let item in CHECKS) {
            if (!new RegExp(CHECKS[item]).test(password.val())) {
                $("#check-list").append(
                    $("<li>", {id: item, class: "not-valid"}).text(TEXT[item])
                )
                valid_password = false
            } else {
                $("#check-list").append(
                    $("<li>", {id: item, class: "valid"}).text(TEXT[item])
                )
            }
        }
        check_password_match()
        submit()
    })

    confirm_password.change(function () {
        check_password_match()
    })

    function check_password_match() {
        $("#password-match").remove()
        if (password.val() != confirm_password.val()) {
            check_block.append(
                $("<p>", {id: "password-match", class: "not-valid"}).text(
                    "Passwords do not match"
                )
            )
            valid_password_match = false
        } else {
            valid_password_match = true
        }
        submit()
    }

    function submit() {
        $("#submit-btn").remove()
        if (valid_username && valid_password && valid_password_match) {
            register_form.append(
                $("<input>", {
                    type: "submit",
                    value: "Register",
                    id: "submit-btn",
                    class: "btn btn-primary",
                })
            )
        } else {
            $("#submit-btn").remove()
        }
    }
})

$("#register-form").submit(function (e) {
    e.preventDefault()
})
